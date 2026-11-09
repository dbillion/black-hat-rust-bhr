# Audit: which claimed fixes are real?

**Method:** for each chapter's claimed source fix, check (a) whether any commit
of mine touches the source file, and (b) whether the fix is present in the
current working tree. A commit message is a claim; `git show --stat` and the file
contents are the evidence.

Author filter: `git log --author=dbillion`. 59 of 60 commits are mine; the one
that is not is an upstream dependabot merge by Sylvain Kerkour.

## Verdict — third and final revision

The first revision said the ch_03/ch_05 "configurable source" fixes were never
made. The saved command lines disproved that: the runs used
`TRICODER_CRTSH_URL` and `CRAWLER_GITHUB_BASE`, which only work if the source
reads them.

The second revision concluded "made, worked, and never committed."

**The third revision is the correct one, and it is better news.** When I went to
delete the codespace, `gh api /user/codespaces` reported
`has_uncommitted_changes: true`. Checking before deleting found **13 modified
source files sitting uncommitted in the codespace working tree** — the fixes
themselves. `ch_03/tricoder/src/ports.rs` (40 lines), `subdomains.rs` (39
lines), `ch_04/tricoder/src/ports.rs` (29), `github.rs` (15). Their content is
genuine and even references the ch_02 fix:

```rust
// Chapter 2 fixed this in the blocking scanner and Chapter 3 inherited the
// bug unchanged: scanning only socket_addresses[0] probes a single address
// family. ... Probe every resolved address instead
```

They are now committed as `b5a2995` and pushed, along with the two fixtures in
`417a8b3`.

**What this means: had I deleted the codespace on the day the manual was
delivered — which the `book-code-execution` skill told me to do — those fixes
would be gone, and the audit would have recorded them as phantom.**

| Commits | Verdict |
|---|---|
| `ba656dd`, `3644334`, `8db11df`, `4ac5bbd`, `570ff23`, `1dcc8fa`, `8ae2f2f`/`7833289`/`6a37947` | **REAL** |
| `20eef7d`, `0d6f26a`, `68b4b28`, `2af0877`, `c8c1be8` | **NOW REAL** — fix uncommitted at audit time, committed as `b5a2995` |
| `2e4d6c0`, `6b2774e`, `ff9d495`, `48f6157`, `94b7f2a` (ch_06) | **MESSAGE-ONLY — and this one is confirmed** |

**Ch06 is the one genuinely false claim, and it is now proven rather than
suspected.** In the codespace, `ch_06` has *no* uncommitted changes and
`git diff HEAD -- ch_06/` is empty. Its source was last touched by `26e0233
update Rust edition = '2021'` — an upstream commit, not mine. So the five
ch_06 commits naming a "zero-length case", a "typed slice" and an "inlined
lib.rs" described fixes that never existed anywhere: not committed, not
uncommitted, not applied.

And a second false claim fell out of the same check: `9f68414`, which
`PROGRESS.md` listed as the ch_06 evidence commit, **does not exist in the
repository at all** — `git log 9f68414` returns `unknown revision`. The Ch1–6
evidence files exist; the commit attributed to them does not.

**Final tally: 12 real, 5 message-only (all ch_06), 1 non-existent commit.**

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

## Confirmed still vulnerable, then recovered

At audit time these were vulnerable. They are **fixed now** — commit `b5a2995`,
recovered from the codespace working tree where they had been hiding
uncommitted:

```
ch_03/tricoder/src/ports.rs:20   was: let socket_address = socket_addresses[0];
ch_04/tricoder/src/ports.rs:20   was: let socket_address = socket_addresses[0];
ch_03/tricoder/src/subdomains.rs  was: crt.sh URL hardcoded
ch_05/crawler/src/spiders/github.rs  was: api.github.com URL hardcoded
```

Verified in the tree after the commit:

```
$ grep -c 'is_empty()'              ch_03/tricoder/src/ports.rs         -> 1
$ grep -c 'TRICODER_CRTSH_URL'      ch_03/tricoder/src/subdomains.rs    -> 1
$ grep -c 'CRAWLER_GITHUB_BASE'     ch_05/crawler/src/spiders/github.rs -> 1
```

**The one still unfixed is ch_06**, and it never had a fix at all:
`ch_06/fuzzing/fuzz/fuzz_targets/fuzz_target_1.rs` is still `Vec<u8>` +
`n: usize`, never a typed slice. No uncommitted change exists for it in the
codespace either. Whatever the ch_06 commits named, it was never written.

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
