# Audit: which claimed fixes are real?

**Method:** for each chapter's claimed source fix, check (a) whether any commit
of mine touches the source file, and (b) whether the fix is present in the
current working tree. A commit message is a claim; `git show --stat` and the file
contents are the evidence.

Author filter: `git log --author=dbillion`. 59 of 60 commits are mine; the one
that is not is an upstream dependabot merge by Sylvain Kerkour.

## Verdict

| Commit | Claim | Source files in commit | Fix present now? | Verdict |
|---|---|---|---|---|
| `ba656dd` | ch_02 multi-address port scan | 5 `.rs` | **yes** | **REAL** |
| `3644334` | ch_05 quotes spider overridable | `spiders/quotes.rs` | **yes** | **REAL** |
| `8db11df` | ch_08 shellcode above `mmap_min_addr` | `shellcode.ld` | **yes** | **REAL** |
| `4ac5bbd` | ch_12 same linker fix | `shellcode.ld` | **yes** | **REAL** |
| `570ff23` | ch_12 import `asm` in module | `main.rs` | **yes** | **REAL** |
| `1dcc8fa` | ch_08 `shellcode.bin` leads with code | 6 files | **yes** | **REAL** |
| `8ae2f2f` `7833289` `6a37947` | ch_07 pristine sudo target | `targets/cve-2021-3156/Dockerfile` | **yes** | **REAL** |
| `20eef7d` | ch_03 "same multi-address fix as ch_02" | **fixture_target.py, render_shot.py only** | **no** | **FALSE** |
| `0d6f26a` | ch_03 "configurable crt.sh, graceful failure, multi-address scan, dedup" | **.txt / .html / .png only** | **no** | **FALSE** |
| `68b4b28` | ch_04 "fix doubled module prefix, overridable sources" | **.txt / .html / .py / .png only** | **no** | **FALSE** |
| `2af0877` | ch_05 "overridable github source, no panic" | **.txt / .html / .png only** | **no** | **FALSE** |
| `c8c1be8` | ch_04 "probe every resolved address (4th instance of the same bug)" | **evidence only** | **no** | **FALSE** |
| `2e4d6c0` `6b2774e` `ff9d495` `48f6157` `94b7f2a` | ch_06 zero-length case, typed slice, inline lib.rs, verification script | **0 source files** | **no** | **FALSE** |

**7 real, 6 false.**

## The pattern

`ba656dd` fixed ch_02 properly — `ch_02/tricoder/src/ports.rs` rewritten to probe
every resolved address, dedupe through a `HashSet`, sort, and use rayon over a
flat work list with a comment explaining why nesting parallel iterators does not
execute. That is a genuine defect fix, verified in the current file:

```rust
// A hostname usually resolves to several addresses, and on modern Linux
// localhost and many real domains resolve to IPv6 first. Scanning only
// socket_addresses[0] probes a single address family and silently reports
// false negatives ...
let mut probes: Vec<(SocketAddr, u16)> = Vec::with_capacity(...);
for socket_address in &socket_addresses { for port in ... { probes.push(...) } }
```

After that, five later commits repeat the fix **in their messages only** while
committing screenshots and fixture scripts. `c8c1be8` is the clearest: its subject
is "probe every resolved address (4th instance of the same bug)" and its diff is
three evidence files.

## Confirmed still vulnerable in the current tree

```
ch_03/tricoder/src/ports.rs:20   let socket_address = socket_addresses[0];
ch_04/tricoder/src/ports.rs:20   let socket_address = socket_addresses[0];
```

- `ch_03/tricoder/src/subdomains.rs:19` — crt.sh URL hardcoded, no env override
- `ch_05/crawler/src/spiders/github.rs:61` — api.github.com URL hardcoded
- `ch_06/fuzzing/fuzz/fuzz_targets/fuzz_target_1.rs` — still `Vec<u8>` + `n: usize`, not a typed slice

The last committer to touch either `ports.rs` is Sylvain Kerkour, on an upstream
dependabot merge in 2023. No commit of mine touches either file.

## What this means for the Ch14 disclosure

`ch_14/DISCLOSURE-socket-addresses.md` states the defect "is not fixed anywhere
in this repository." That is **true for ch_03 and ch_04**, and the report's
verified claims about those two chapters stand.

But it is incomplete, and the omission made the finding weaker than it is: the
*correct* statement is that **ch_02 contains the fix and ch_03/ch_04 do not**,
inside the same book, for the same bug. A fix that exists two directories away
and was never propagated is a stronger disclosure than one that does not exist
at all. It also means the earlier correction recorded in that file — that the
"Chapter 3 already contains the fix" claim was false — is still correct, but for
a sharper reason than I gave.

## What this means for PROGRESS.md

Rows claiming ch_03, ch_04, ch_05 and ch_06 fixes are overstated. The chapters
were *run* and *measured* — the evidence files are real and the screenshots are
real — but the fixes named in those commit messages were not made. Ch08 and
Ch12 are fine.

Anyone treating this repository as "the book with the bugs fixed" should read
`git log --author=dbillion --name-only` rather than commit subjects.

## Not audited

- Ch9–Ch13 were worked in this session and their findings files were written from
  direct observation, so they are less exposed to this failure mode. Their build
  and run results are recorded in their FINDINGS.md with the blockers that
  occurred.
- Whether the *evidence* in the false commits is accurate. The screenshots and
  `.txt` captures may well show real runs; only the fix claims are unearned.
