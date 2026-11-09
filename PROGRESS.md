# Black Hat Rust — chapter run state

Single source of truth for progress. Read at the start of every chapter run,
written at the end. Survives context compaction because it is on disk, not in
context. The manual is *generated* from this file plus each chapter's
`FINDINGS.md` and the evidence PNGs, so it is never drifted by hand-editing.

## Chapters

| Ch | Topic | Code | State | Evidence | Fork commit |
|----|-------|------|-------|----------|-------------|
| 01 | hashes / wordlist | yes | done | ch_01/FINDINGS.md | ba656dd |
| 02 | port scanner | yes | done | — | 0d6f26a |
| 03 | async scanner | yes | done | ch_03_async_tricoder.png | 0d6f26a |
| 04 | tricoder | yes | done | ch_04_modules_tricoder.png | c8c1be8 |
| 05 | crawler (OSINT) | yes | done | ch_05_crawler_osint.png | 2af0877 |
| 05 | crawler (JS/browser) | yes | done | ch_05_js_crawler.png | 3644334, 2da5473 |
| 06 | fuzzing + SVG XSS | yes | done | ch_06_fuzzing_overflow.png, ch_06_svg_xss.png | 9f68414 |
| 07 | CVE-2021-3156 | yes | done, gap noted | ch_07/FINDINGS.md | target commits |
| 08 | shellcode | yes | done, 2 of 3 verified | ch_08_shellcode_executor.png, ch_08_shell_shellcode.png | see log |
| 09 | phishing / DNS squat / evil twin | yes | done, 2 gaps noted | ch_09/FINDINGS.md, ch_09_phishing.png | see log |
| 10 | RAT | yes | done, C2 loop verified | ch_10/FINDINGS.md, ch_10_rat.png | see log |
| 11 | RAT + TLS (no TLS: E2E only) | yes | done, 2 defects noted | ch_11/FINDINGS.md, ch_11_encrypted_rat.png | see log |
| 12 | shellcode cross-arch | yes | x86_64 done; aarch64/armv7 blocked | — | see log |
| 13 | self-propagating worm | yes | todo | — | — |
| 14 | report | no code | todo | — | — |

Done: 12 of 14. Remaining: 2 with code (13, 12-cross-arch) plus ch14 report.

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

- ba656dd — ch_01 hash cracking + FINDINGS
- 0d6f26a — ch_02/ch_03 scanner fixes, real scanme.nmap.org scan
- c8c1be8 — ch_04 tricoder fixes, three disclosures fired
- 2af0877 — ch_05 crawler + 207-item fixture
- 3644334 — ch_05 overridable quotes spider
- 2da5473 — ch_05 evidence: JS-rendered crawl
- 9f68414 — ch_06 evidence (Ch1–6 complete)
- (ch_07) — isolated victim image, pristine upstream sudo, FINDINGS.md
- (ch_08) — `. = 0x400000` mmap_min_addr fix
- (ch_08) — `-z noseparate-code` + `*(.rodata .rodata.*)` executor blob fix
- (ch_08) — ch_12 `mod syscalls` asm import
- (ch_08) — shell shellcode verified, isolated container target
