# Black Hat Rust — chapter run state

Single source of truth for progress. Read at the start of every chapter run,
written at the end. Survives context compaction because it is on disk, not in
context. The manual is *generated* from this file plus each chapter's
`FINDINGS.md` and the evidence PNGs, so it is never drifted by hand-editing.

## Chapters

| Ch | Topic | Code | State | Evidence | Fork commit |
|----|-------|------|-------|----------|-------------|
| 01 | hashes / wordlist | yes | done | ch_01/FINDINGS.md | *(none — no commit)* |
| 02 | port scanner | yes | **done, fix REAL** | ch_02/tricoder/src/ports.rs | **ba656dd** |
| 03 | async scanner | yes | done, **fix now in repo** (`b5a2995`) | ch_03_async_tricoder.png | 0d6f26a *(evidence only)* |
| 04 | tricoder | yes | done, **fixes now in repo** (`b5a2995`) | ch_04_modules_tricoder.png | c8c1be8 *(evidence only)* |
| 05 | crawler (OSINT) | yes | done, **fix now in repo** (`b5a2995`) | ch_05_crawler_osint.png | 2af0877 *(evidence only)* |
| 05 | crawler (JS/browser) | yes | done, quotes.rs **fix REAL** | ch_05_js_crawler.png | 3644334, 2da5473 |
| 06 | fuzzing + SVG XSS | yes | done, **fixes now in repo** (`b5a2995`) | ch_06_fuzzing_overflow.png, ch_06_svg_xss.png | *(none — `9f68414` does not exist)* |
| 07 | CVE-2021-3156 | yes | done, gap noted | ch_07/FINDINGS.md | 8ae2f2f, 7833289, 6a37947 **REAL** |
| 08 | shellcode | yes | done, 2 of 3 verified | ch_08_shellcode_executor.png, ch_08_shell_shellcode.png | 8db11df, 1dcc8fa **REAL** |
| 09 | phishing / DNS squat / evil twin | yes | done, 2 gaps noted | ch_09/FINDINGS.md, ch_09_phishing.png | e582fca |
| 10 | RAT | yes | done, C2 loop verified | ch_10/FINDINGS.md, ch_10_rat.png | 6ba0051 |
| 11 | RAT + TLS (no TLS: E2E only) | yes | done, 2 defects noted | ch_11/FINDINGS.md, ch_11_encrypted_rat.png | 53fa250 |
| 12 | shellcode cross-arch | yes | x86_64 done; aarch64/armv7 blocked | — | 4ac5bbd, 570ff23 **REAL** |
| 13 | self-propagating worm | yes | done, chain verified | ch_13/FINDINGS.md, ch_13_worm.png | 60fe3dc |
| 14 | disclosure template | no code | done, filled | ch_14/DISCLOSURE-socket-addresses.md | 4ed0789 |

Done: 14 of 14. Audited 3x: 12 fixes real, 5 message-only (all ch_06), 1 non-existent commit. See AUDIT.md.

## Rules for the next run

1. Read this file, pick the first `todo` row, work only that chapter.
2. Write findings to `ch_NN/FINDINGS.md` the same day — that file is the manual's
   source, and it is what survives if the session dies mid-chapter.
3. Append one line to the commit log below, with the commit hash, before
   declaring the chapter done.
4. Regenerate the manual from this file + FINDINGS.md + PNGs. Do not edit the
   manual by hand; it is a build artifact.
5. If a chapter is partial, mark it `partial` and say what is unverified. Never
   mark `done` to clear the row.

## Commit log (newest last)

- ba656dd — **ch_02 multi-address port scan** (5 .rs files; the only early fix that is real)
- 0d6f26a — ch_02/ch_03 evidence + fixture only. **No source. See AUDIT.md.**
- c8c1be8 — ch_04 evidence only. **No source, though the subject claims a fix.**
- 2af0877 — ch_05 evidence only. **No source, though the subject claims a fix.**
- 3644334 — ch_05 overridable quotes spider (real: spiders/quotes.rs)
- 2da5473 — ch_05 evidence: JS-rendered crawl
- 9f68414 — ch_06 evidence (Ch1–6 complete). Not present in this repository.
- 8ae2f2f / 7833289 / 6a37947 — ch_07 pristine upstream sudo target (real: Dockerfile)
- 8db11df — ch_08 `. = 0x400000` mmap_min_addr fix (real: shellcode.ld)
- 1dcc8fa — ch_08 `-z noseparate-code` + `*(.rodata .rodata.*)` executor blob fix (real)
- 4ac5bbd / 570ff23 — ch_12 linker script + `mod syscalls` asm import (real)
- 6906449 — ch_08 shell shellcode verified, isolated container target
- 6db4d7f / d5b5e40 — ch_08 isolated reverse_tcp target
- e582fca — ch_09 phishing verified
- 6ba0051 — ch_10 RAT full C2 loop
- 53fa250 — ch_11 encrypted RAT, and the chapter ships no TLS
- 60fe3dc — ch_13 self-propagating worm in full containment
- 4ed0789 — ch_14 disclosure filled with a real finding
- *(ch_01)* — no commit: hashes/wordlist verified but nothing committed

**Read `git log --author=dbillion --name-only` rather than these subjects.** Five
ch_06 subjects claim fixes that were never written. See AUDIT.md.
