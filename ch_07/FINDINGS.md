# Chapter 7 — CVE-2021-3156 (Baron Samedit): what ran, what did not

## Target built

`targets/cve-2021-3156/Dockerfile` — an isolated victim container:

| Property | Value |
|---|---|
| Base | `ubuntu:20.04` |
| sudo | **pristine upstream 1.8.31** (built from tarball, no distro patches) |
| Victim user | `victim` (uid 1000) |
| sudoers | `victim ALL=(ALL) NOPASSWD: /usr/bin/sudoedit` |
| Exploit payload | `/libnss_x/x.so.2` from the book's `shellcode.c` |
| Run flags | `--network none`, no `--privileged`, no host mounts |

## Why the distro sudo was not used

`ubuntu:20.04` ships a sudo whose `--version` prints **1.8.31**, which is inside
the vulnerable range — and that string is misleading. `dpkg -s sudo` reports
`1.8.31-1ubuntu1.5`, a Debian revision that carries the CVE-2021-3156 fix
**backported**. The book's own Python reference exploit refused to continue
against it, with `assert check_is_vuln(), "target is patched"` — which is the
correct answer, not a defect in the reference.

So the version string cannot be trusted; the package revision must be read. The
pristine upstream 1.8.31 tarball predates the January 2021 fix and carries no
distro patches, so it is the honest vulnerable target.

## Result: real, but not a clean escalation

| Attempt | Outcome |
|---|---|
| Rust exploit (`/exploit`) | **SIGSEGV, exit 139, core dumped** — reproducible across runs |
| Reference Python exploit | Parses argv, reaches the editor path (`/usr/bin/vi`), then stops without spawning the NSS-hijacked shell |
| `sudoedit` baseline, no exploit | exits 1, no escalation — correct, control case behaves |

The SIGSEGV is genuine evidence that the heap overflow lands: the crafted argv
and the `LC_*` environment spray do corrupt sudo's memory badly enough to take
the process down. It is **not** the intended outcome, and the payload's
`[+] We are root!` never printed.

## Why it stops here, stated plainly

Getting a clean escalation on this CVE depends on glibc malloc arena layout,
which differs between the codespace's kernel/glibc build and the author's
machine. The exploit's heap-grooming constants are tuned for a specific target
family. I rebuilt the target to be genuinely vulnerable and confirmed the
process crashes on the overflow, but I did not land the controlled overwrite.

**Do not read this as a working exploit.** It is a working *victim*, and a
confirmed memory-safety violation.

## Still unverified

- Clean privilege escalation to `root` from `victim` via this exploit
- `cve_2019_11229` (Gitea RCE) — not attempted
- `cve_2019_89242` (WordPress image RCE) — not attempted
