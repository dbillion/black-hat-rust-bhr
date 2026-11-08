"""Chapter 6 verification: fuzzing overflow + SVG XSS.

Part 1  vulnerable_memcopy() is a C-style overflow expressed in safe Rust. The
        point of the chapter is that Rust's bounds check converts silent heap
        corruption into a controlled panic. Each case below compiles the shipped
        lib.rs against a fresh main() and checks the actual outcome.

Part 2  xss_svg/image.svg carries an inline <script>. The payload is only real
        if a browser executes it, so it is loaded in a real browser and the
        dialog it raises is caught.

Nothing here is self-propagating: no shellcode, no listener, no network client.
The fuzz target is in-memory arithmetic; the SVG is a local file in a local
browser.
"""

import os
import subprocess
import sys
import tempfile
from pathlib import Path

HOME = Path("/home/codespace")
LIBS = [
    Path("ch_06/fuzzing/src/lib.rs"),
]
SVG = Path("ch_06/xss_svg/image.svg")
WORKDIR = Path("/workspaces/black-hat-rust-bhr")

# rustc lives under ~/.cargo/bin in the codespace; an un-sourced shell does not
# have it on PATH, which makes every compile case fail as "compile error".
CARGO_BIN = HOME / ".cargo/bin"
if (CARGO_BIN / "rustc").exists():
    ENV = dict(os.environ)
    ENV["PATH"] = str(CARGO_BIN) + ":" + ENV.get("PATH", "")
else:
    ENV = dict(os.environ)

# playwright is importable only from the interpreter it was installed for.
PY_BIN = "python3"

CASES = {
    # name: (n, src_len, dest_len, must_panic)
    "reads past src": (5, 3, 10, True),
    "writes past dest": (10, 10, 4, True),
    "copy exactly fits": (4, 4, 4, False),
    "zero length is a noop": (0, 1, 4, False),
}


def build_and_run(name, n, src_len, dest_len):
    # always pass a real &[u8]; an empty one for the zero-length case so types
    # match the signature (a raw &[] has no known element type).
    src_names = "src"
    body = f"""
fn main() {{
    let src = [{", ".join(str(i % 256) for i in range(src_len))}u8];
    let mut dest = [0u8; {dest_len}];
    vulnerable_memcopy(&mut dest, &src, {n});
    println!("REACHED_END");
}}
"""
    lib = (WORKDIR / LIBS[0]).read_text()
    with tempfile.TemporaryDirectory() as d:
        probe = Path(d) / "probe.rs"
        out = Path(d) / "probe"
        # lib.rs is *inlined* (not depended on as a crate), so the free function
        # is in scope directly; the book's fuzz target calls it as
        # fuzzing::vulnerable_memcopy because there it is a separate crate.
        probe.write_text(lib + body)
        compile_run = subprocess.run(
            ["rustc", "--edition", "2021", "-O", str(probe), "-o", str(out)],
            capture_output=True, text=True, env=ENV,
        )
        if compile_run.returncode != 0:
            return None, compile_run
        return subprocess.run([str(out)], capture_output=True, text=True, env=ENV), None


def main():
    failures = []
    print("== part 1: vulnerable_memcopy ==")
    for name, (n, sl, dl, must_panic) in CASES.items():
        proc, compile_err = build_and_run(name, n, sl, dl)
        if compile_err is not None:
            failures.append(f"{name}: compile failed\n{compile_err.stderr[-500:]}")
            print(f"  [FAIL] {name}: compile error")
            continue

        reached = "REACHED_END" in proc.stdout
        kind = "panic" if must_panic else "clean"
        if must_panic:
            bounded = (not reached) and proc.returncode != 0
            detail = "index out of bounds" if "index out of bounds" in proc.stderr else \
                     ("subtract overflow" if "subtract with overflow" in proc.stderr else proc.stderr.strip()[-120:])
            ok = bounded
        else:
            ok = reached and proc.returncode == 0
            detail = "copied and returned" if ok else proc.stderr.strip()[-120:]

        print(f"  [{'ok' if ok else 'FAIL'}] {name}: expected {kind}, got {detail}")
        if not ok:
            failures.append(f"{name}: expected {must_panic=}, reached_end={reached}, rc={proc.returncode}, stderr={proc.stderr[-300:]}")

    print("\n== part 2: SVG XSS ==")
    if not SVG.exists():
        failures.append("image.svg missing")
        print("  [FAIL] ch_06/xss_svg/image.svg not found")
    else:
        text = (WORKDIR / SVG).read_text()
        has_script = "<script" in text.lower()
        has_alert = "alert(" in text
        print(f"  [{'ok' if has_script else 'FAIL'}] inline <script> present in the payload")
        print(f"  [{'ok' if has_alert else 'FAIL'}] alert() call present")
        if not (has_script and has_alert):
            failures.append("payload missing script/alert")

        fired = []
        from playwright.sync_api import sync_playwright
        with sync_playwright() as p:
            browser = p.chromium.launch()
            page = browser.new_page()
            page.on("dialog", lambda dlg: (fired.append(dlg.message), dlg.dismiss()))
            page.goto((WORKDIR / SVG).resolve().as_uri(), wait_until="load")
            page.wait_for_timeout(700)
            globals_before = page.evaluate("() => Object.getOwnPropertyNames(window).length")
            browser.close()
        executed = bool(fired)
        print(f"  [{'ok' if executed else 'FAIL'}] script executed in the browser (dialog: {fired})")
        if executed:
            print(f"        the payload's alert() fired with message: {fired[0]!r}")
        else:
            failures.append("SVG script did not execute in a real browser")

    print("\n== summary ==")
    if failures:
        print(f"{len(failures)} check(s) failed:")
        for f in failures:
            print("  - " + f)
        return 1
    print("all chapter 6 checks passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
