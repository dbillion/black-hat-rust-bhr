ENTRY(_start);

# A nonzero base is required: `. = ALIGN(16)` starts the location counter at 0,
# placing the first LOAD segment at virtual address 0, which the kernel refuses
# to map (vm.mmap_min_addr = 65536) — execve() then fails with EPERM.
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