# Page Tables in Moss

Moss implements AArch64 page tables using a type-safe Rust abstraction that makes it difficult to misconfigure the hardware.

## The `PgTableArray` Type

Rather than working with raw arrays of 64-bit integers, Moss wraps each level of the page table in a typed container:

```rust
pub struct PgTableArray<L: Level> {
    entries: [PageTableEntry; 512],
    _phantom: PhantomData<L>,
}
```

The `Level` type parameter (`L0Table`, `L1Table`, `L2Table`, `L3Table`) encodes which level the array is at. This prevents accidentally passing an L2 table where an L3 table is expected — such a mistake would be a compile error, not a silent bug.

## Page Table Entry Types

A `PageTableEntry` is a 64-bit value with bit-field accessors:

```rust
pub struct PageTableEntry(u64);

impl PageTableEntry {
    pub fn is_valid(&self) -> bool { self.0 & 1 != 0 }
    pub fn is_table(&self) -> bool { self.0 & 0b10 != 0 }
    pub fn physical_address(&self) -> PA {
        PA((self.0 & 0x0000_ffff_ffff_f000) as usize)
    }
    // ... attribute getters/setters
}
```

## The `MapAttributes` Struct

All mappings are described with a `MapAttributes` value:

```rust
pub struct MapAttributes {
    pub phys: PA,          // Physical address to map to
    pub virt: VA,          // Virtual address to map
    pub size: usize,       // Size of mapping
    pub prot: Prot,        // Read, Write, Execute permissions
    pub cache: CacheType,  // Normal, Device, etc.
    pub user: bool,        // Accessible from EL0?
}
```

This struct is passed to `map_range()` which walks the page table, allocating new table pages as needed, and installs the leaf entries.

## Process Address Spaces

Each process (thread group) has an `Arm64ProcessAddressSpace`:

```rust
pub struct Arm64ProcessAddressSpace {
    l0_table: Box<PgTableArray<L0Table>>,
}

impl Arm64ProcessAddressSpace {
    pub fn activate(&self) {
        // Load this process's L0 table into TTBR0_EL1
        let pa = self.l0_table.physical_address();
        unsafe { write_sysreg!(ttbr0_el1, pa.0 as u64) };
        // Flush TLB entries for the old process
        unsafe { asm!("tlbi vmalle1is; dsb ish; isb") };
    }
}
```

On a context switch, `activate()` is called for the new process, which:
1. Updates TTBR0_EL1 to point to the new page tables
2. Issues a TLB invalidation for user-space entries
3. Issues memory barriers to ensure the CPU sees the change

## Kernel Address Space

The kernel address space (TTBR1) is shared across all processes. It is set up once during boot and never changes per-process. The kernel's L0 table is a static array allocated in Smalloc:

```rust
static KERNEL_L0: PgTableArray<L0Table> = PgTableArray::new_zeroed();
```

The L0 entries for user space are zeroed (invalid). Only kernel-space entries are populated. When a context switch occurs and TTBR0 changes, TTBR1 stays the same.

## Lazy Table Allocation

Moss allocates page table pages lazily — only when a new mapping falls in a region that doesn't yet have a table. The `ensure_table()` function allocates a new page and installs it as a table entry if the current entry is invalid:

```rust
fn ensure_table(entry: &mut PageTableEntry, allocator: &mut FrameAllocator)
    -> &mut PgTableArray<_>
{
    if !entry.is_valid() {
        let new_page = allocator.alloc_one_page();
        zero_page(new_page);
        *entry = PageTableEntry::new_table(new_page);
    }
    entry.as_table_mut()
}
```

This is important for process address spaces: most of the 256 TiB user space is never mapped, so the kernel only allocates table pages for the regions a process actually uses.

## Source Reference

- `src/arch/arm64/memory/address_space.rs` — Process address space
- `src/arch/arm64/memory/mmu.rs` — Kernel page tables, MMU setup
- `libkernel/src/memory/` — Architecture-agnostic page table types

## Exercises

1. When a process forks, does the child get the same L0 table or a new one? What about the L1, L2, and L3 tables?

2. Why is it important to issue a TLB flush after updating TTBR0? What could go wrong without it?

3. If a process maps 10 different 4 KiB regions spread across 10 different 2 MiB regions, how many page table pages does it need? Draw the table hierarchy.
