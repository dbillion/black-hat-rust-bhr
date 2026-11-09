# Chapter 8 — shellcode: build works, exec does not

## What was built

`ch_08/hello_world` compiles clean on Rust 1.99.1 in 21.23s. The source is
correct: `#![no_std]`, `#![no_main]`, two raw syscalls via `asm!("syscall")` —
`write(1, msg, 12)` then `exit(0)`.

## What fails, reproducibly

The built binary segfaults on exec — **SIGSEGV, exit 139, core dumped** — in the
codespace, in `/tmp`, in `$HOME`, and inside a dedicated Docker container
(`targets/ch_08-shellcode/Dockerfile`, `--network none`). A plain C binary built
with the same gcc in the same directories runs fine, so this is not filesystem
permissions, not seccomp, not the mount.

`strace` on the failing binary:

```
execve("./hw", ["./hw"], 0x7ffdfaa87d00 /* 53 vars */) = -1 EPERM (Operation not permitted)
+++ killed by SIGSEGV +++
```

The **kernel** rejects the binary at `execve`, before a single instruction of the
shellcode runs. So this is an ELF layout problem, not a shellcode bug.

## The defect, precisely

`objcopy -O binary` output (first 12 bytes):

```
00000000: 6865 6c6c 6f20 776f 726c 640a  hello world\n
```

`readelf -h` on the linked binary:

```
Type:    EXEC
Entry:   0x64
LOAD     0x1000  vaddr 0x0        filesz 0x64  R
LOAD     0x1064  vaddr 0x64       filesz 0x1a  R E
```

The **first LOAD segment sits at virtual address 0**. The custom
`shellcode.ld` opens with `. = ALIGN(16)`, which starts the address counter at
zero, so the read-only segment (message string + `.eh_frame`) is placed at
vaddr 0. Linux maps LOAD segments at their stated vaddr; a segment at vaddr 0
is not a mappable target, and the kernel refuses the image.

The `_start` code is at 0x64 and its `lea` computes a string address around
0x35 — inside the unmappable first segment, which is why the process dies
rather than printing garbage.

## Fix (not applied to the fork yet)

Give the linker script a nonzero base:

```
. = 0x400000;   /* or any conventional base */
```

`objcopy -O binary` still works: it dumps from the lowest LMA, and the shellcode
bytes remain position-independent because `_start` reaches the string through
`%rip`-relative addressing. A far larger base just widens the zero gap before
the code, so the extracted `shellcode.bin` needs its own trimming or the
executor's entry offset updated to match.

I have not applied this, because the correct base depends on what the `executor`
crate expects (`include_bytes!("../../shellcode.bin")` with a fixed offset), and
changing one without the other trades a crash for a silent wrong result — worse
than an honest failure.

## Still unverified

- Actually executing the shellcode end-to-end
- `ch_08/shell` (interactive shell shellcode)
- `ch_08/reverse_tcp` (needs a listener — deliberately not run)
- `ch_08/executor` (loads `shellcode.bin`)

Note: `reverse_tcp` is a reverse shell. It was **not** run, and the target image
does not start a listener.
