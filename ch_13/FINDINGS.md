# Chapter 13 — Self-propagating worm

## Summary

**The worm works.** The complete chain was observed end to end, more than once:
connect → brute-force → identify platform → upload the agent binary → execute
it on the "remote" host. The wordlist defeated root/password in a few
milliseconds, because the wordlist is three entries long.

```
spreading to 127.0.0.1:22
Authenticated! username: (root), password: (password)
detected platform: linux_x86_64
Uploading: /root/.local/share/bhr_ch13/agent.linux_x86_64
size: 3205312
agent uploaded to /tmp/.goRbZpDEWQGjLWtN0BVU8CoEP1M97C4L
Agent successfully executed on remote host 🥳
```

Everything stayed inside one container on loopback. See the containment design
below.

## Containment design

`spread.rs` says the worm is `spread(host_port)`: it connects to a **supplied**
SSH target, brute-forces it, uploads `agent.<platform>`, and executes it
remotely. It does not scan, enumerate, or discover. Every hop needs a
`host:port` handed to it as `argv[1]`.

That makes containment **exact** rather than approximate, which is why this
chapter was safe to run at all:

| Control | Value | Effect |
|---|---|---|
| `--network none` | loopback only | the worm's only reachable target is the SSH server inside the same container |
| `--cap-drop=ALL` | no capabilities | a successful infection cannot escalate |
| `--security-opt no-new-privileges` | | escalation blocked by policy |
| `--pids-limit 256` | | caps runaway forking |
| No volume mounts | | no shared filesystem with the host |
| Root password | `password` | the book's own wordlist entry; no real credential exposed |
| SSH server | inside the same container | the "remote" host it infects is the container it runs in |

Because the target is `127.0.0.1:22` inside a `--network none` container, an
outbound connection to anywhere else is impossible. The worm is not
constrained by hope or by luck of timing — it is constrained by the network
namespace.

## Defects and blockers found

### 1. The book's own Makefile does not build on a modern toolchain

`make x86_64` runs `cross build --target x86_64-unknown-linux-musl`. Two
problems in order:

- `cross` is not installed and its image is large; using plain `cargo build
  --target x86_64-unknown-linux-musl` instead fails at the link step —
  `libssh2-sys` / vendored `openssl-src` cannot link against musl.
- `make hello_world_aarch64` and `upx -9` are commented out, and `aarch64`
  requires `musl-gcc` plus cross tools.

Building for the host target with `cargo build --release -p agent` works and is
what this run used.

### 2. glibc: the agent will not run on `ubuntu:20.04`

Built on the codespace, the agent requires `GLIBC_2.32`, `2.33`, `2.34` and
`2.38`. On `ubuntu:20.04` (glibc 2.31) it dies before `main`:

```
./agent: /lib/x86_64-linux-gnu/libc.so.6: version `GLIBC_2.38' not found
```

The lab image is `ubuntu:24.04` for this reason.

### 3. libssh2 cannot handshake with a current OpenSSH — the real finding

With the legacy algorithms enabled, `sshd -T` confirmed `ssh-rsa` is offered and
every legacy KEX is accepted, and a real OpenSSH client completes every one of
those handshakes. The agent still fails with:

```
Error: Ssh(Error { code: Session(-1), msg: "Unable to exchange encryption keys" })
```

`sshd -d` confirms the client does connect and does send a banner —
`compat_banner: no match: libssh2_1.9.0_DEV` — so TCP and the banner exchange
are fine. The failure is inside the key exchange.

**This is the chapter's most transferable lesson and it is not in the book.** A
worm built on a 2021 crypto stack (`ssh2` 0.9 / libssh2) cannot reliably talk
to a current OpenSSH. The code is correct; the *environment* moved. Anyone
reproducing this chapter today hits this before they hit anything in the text.

### 4. Malformed algorithm names kill the whole directive

`KexAlgorithms +...,ECDH-SHA2-NISTP256` is not a valid OpenSSH name — the real
one is lowercase `ecdh-sha2-nistp256` — and sshd rejects the **entire**
`KexAlgorithms` directive when any entry is unrecognised, so the whole legacy
line was silently discarded and sshd refused to start:

```
Unsupported KEX algorithm "ECDH-SHA2-NISTP256"
/etc/ssh/sshd_config line 139: Bad SSH2 KexAlgorithms '...'.
```

Worth knowing independently of this chapter: a typo in an algorithm list
disables the whole list, not just the bad entry.

## Flakiness — reported as observed, not explained

The handshake failure is **not deterministic**. With the same image, the same
sshd configuration, the same binary and the same command:

- One sequence succeeded **6/6 times in a row**.
- Adjacent runs with an equivalent-looking command failed.
- Every failure used a multi-line script with `echo` statements between sshd
  and the agent; every success in the 6/6 loop used a single-line `;` script.
  That correlation held across 8+ runs and then **broke**: the final run used
  the exact single-line command that had succeeded 6/6, and failed.

So: correlated, then contradicted. The cause is **not understood**, and this
section will not pretend otherwise. The plausible suspects, none confirmed:

- entropy availability inside the container affecting the KEX exchange
- sshd fork timing against a freshly started daemon
- something in libssh2's session setup that depends on process context

Anyone re-running this should expect to retry. `run.sh` uses the form that
worked most often, but treat the handshake as flaky.

## Measured results

`cargo build --release -p agent` — 1m02s, `target/release/agent` = 3,205,312
bytes. `ls -la` confirms the transfer is byte-exact (the codespace copy is also
3,205,312 bytes).

Successful spread, `/tmp` inside the container after the run — one hidden
`.` + 32 random alphanumerics file per hop, exactly as `upload_agent` builds
it via `format!(".{}", rand_name)`:

```
agent uploaded to /tmp/.goRbZpDEWQGjLWtN0BVU8CoEP1M97C4L
```

`install()` also works, and the bundle path was exercised rather than bypassed:

```
Installing into /root/.local/share/bhr_ch13
bundle.zip found, extracting it to /root/.local/share/bhr_ch13
-rw-r--r-- 1 root root 3205312 agent.linux_x86_64
-rwxr-xr-x 1 root root 1441727 bundle.zip
-rwxr-xr-x 1 root root 3205312 ch13_agent
```

Note what the bundle mechanism is: `install.rs` says *"here, we could have
fetched the bundle from a central server"* — the book's own comment. That is
the whole second-stage of a real worm, and it is a two-line `fs::copy` away
from shipping a second payload with the agent.

## Unverified

- The aarch64 and armv7 agents. The Makefile's cross builds do not work here.
- Real propagation between two containers. The lab deliberately has only one
  network, so nothing tests a second hop.
- The worm's persistence. There is no cron, no systemd unit, no `rc.local` in
  this chapter — `install()` only places a file in the data dir.
- What the uploaded agent *did* when executed remotely. `execute_remote_agent`
  consumes its stdio and discards it, so the run is confirmed but its output
  is not captured.

## Recommendation

The book's worm is a demonstration, not a weapon: no scanning, no persistence,
no real wordlist, no scanning for targets. Read it as *the shape* of a worm
rather than a working one.

The genuinely dangerous parts are the two the chapter treats as filler. First,
`spread()` uploads to `/tmp/.<random>` and executes it — a full arbitrary-code
execution primitive over SSH, gated only by credentials. Second, and more
important operationally: **an old crypto stack silently stops working against
current servers.** If you are auditing an environment for SSH worms, the
absence of libssh2-era handshakes in your logs is meaningful, and the presence
of failed KEX attempts from 2021-era clients is the thing actually worth
alerting on.
