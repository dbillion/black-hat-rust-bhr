# Chapter 8 — shellcode: one fix confirmed, one defect still open

## Fixed: `execve()` EPERM → `hello world` prints, exit 0

**Root cause.** `shellcode.ld` opened with `. = ALIGN(16)`, which starts the
location counter at zero, placing the read-only LOAD segment at virtual address
0. The kernel refuses to map below `vm.mmap_min_addr` (65536 on x86_64), so
`execve()` returned **EPERM before `_start` ran**. Confirmed three ways:
`strace` (`execve(...) = -1 EPERM` then SIGSEGV), `Seccomp: 0`, AppArmor
`unconfined`, and a control gcc binary running fine in the same directories.

**Fix.** `. = 0x400000;` — any base above `mmap_min_addr` works.

**Verified end-to-end in `ch_08/` itself:**

```
$ make hello_world
$ ./hello_world/target/release/hello_world
hello world
EXIT=0
$ readelf -h hello_world | grep Entry
  Entry point address: 0x400064
```

Applied to both copies of the script (`ch_08/shellcode.ld` and
`ch_12/shellcode/hello_world/shellcode.ld`), which are byte-identical
duplicates — the same defect-propagation pattern as `socket_addresses[0]` in the
earlier chapters.

Note: cargo does not track linker-script changes in its fingerprint. Editing
`shellcode.ld` alone leaves a stale binary; `touch src/main.rs` or delete
`target/` before rebuilding.

## Open: the executor still segfaults, and I know why

`objcopy -O binary` dumps from the lowest VMA (0x400000, the read-only data)
to the highest (end of code), so `shellcode.bin` is ordered:

```
00000000: 6865 6c6c 6f20 776f 726c 640a  hello world\n   <- data first
0000000c: ...eh_frame...
00000064: 488d 3595 ffff ff6a 0158 ...  <- the code starts here
```

`executor` embeds this whole blob and jumps to its **first byte**, so it begins
executing the ASCII string `68 65 6c 6c 6f` as instructions. `01 1b` decodes to
`add [rbx], ebx` with `rbx` unset — hence the core dump.

The extracted blob must start with the code. Two attempts at reordering the
linker script did not achieve it: `*(.rodata)` never matched the actual section
name (`.rodata..Lanon.<hash>.0`), and after widening the glob the `.eh_frame`
orphans still landed ahead of `.text._start`. Both are recorded so the next
attempt does not repeat them.

This is **not claimed fixed**. The confirmed, measured result is the `execve`
fix above; the executor defect is identified and stated as open.

## Still unverified

- `make execute` (executor loading `shellcode.bin`) — **fails, cause identified**
- `ch_08/shell` — not built
- `ch_08/reverse_tcp` — **deliberately not run**: it is a reverse shell
