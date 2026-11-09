ENTRY(_start);

# Shellcode image. A nonzero base is required: `. = ALIGN(16)` starts the
# location counter at 0, placing the first LOAD segment at virtual address 0,
# which the kernel refuses to map (vm.mmap_min_addr = 65536 on x86_64) --
# execve() then fails with EPERM before _start runs. 0x400000 is conventional.
#
# Code must LEAD the image, because objcopy -O binary dumps from the lowest VMA
# and the executor jumps to byte 0 of the extracted blob. ld's default
# -z separate-code puts read-only data in a separate, lower-addressed segment,
# which silently reorders the blob to data-first. Two things stop that:
# `-z noseparate-code` (in .cargo/config.toml) keeps one segment, and .rodata is
# explicitly collected *after* the code inside the same .text output section.
# `*(.rodata)` alone never matches Rust's real section name, which is
# `.rodata..Lanon.<hash>.0`, hence the `.rodata.*` glob.
#
# .eh_frame* are useless without an unwinder and would widen the blob's data
# prefix, so they are discarded.
SECTIONS
{
	. = 0x400000;
	.text :
	{
		*(.text.prologue)
		*(.text)
		*(.text._start)
		*(.rodata .rodata.*)
	}
	.data :
	{
		*(.data)
	}
	/DISCARD/ :
	{
		*(.eh_frame .eh_frame_hdr)
		*(.interp)
		*(.comment)
		*(.debug_frame)
		*(.note .note.*)
	}
}
