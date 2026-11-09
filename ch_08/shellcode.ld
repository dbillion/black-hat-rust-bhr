ENTRY(_start);

# Base the image at a nonzero address. `. = ALIGN(16)` starts the location
# counter at 0, which places the first LOAD segment at virtual address 0. The
# kernel refuses to map anything below vm.mmap_min_addr (65536 on x86_64), so
# execve() fails with EPERM before _start runs. Any base above mmap_min_addr
# works; 0x400000 is the conventional choice.
SECTIONS
{
	. = 0x400000;
	.text :
	{
		*(.text.prologue)
		*(.text)
		*(.rodata)
	}
	.data :
	{
		*(.data)
	}

	/DISCARD/ :
	{
		*(.interp)
		*(.comment)
		*(.debug_frame)
	}
}