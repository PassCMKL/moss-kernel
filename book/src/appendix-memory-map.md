# Appendix B: Memory Map Reference

## Kernel Virtual Address Space

```
Virtual Address Range                      Size        Region
──────────────────────────────────────────────────────────────────
0xffff_0000_0000_0000 - 0xffff_7fff_ffff_ffff  128 TiB  Logical Map
                                                          (physical PA maps to
                                                           VA = PA + 0xffff_0000_0000_0000)

0xffff_8000_0000_0000 - 0xffff_8000_1fff_ffff  512 MiB  Kernel Image
                                                          .text, .rodata, .data, .bss

0xffff_8100_0000_0000 - 0xffff_8100_0000_0fff    4 KiB  VDSO
                                                          Mapped into every process

0xffff_9000_0000_0000 - 0xffff_9000_0020_1fff  ~2 MiB  Fixmap Region
                                                          Temporary kernel mappings

0xffff_b800_0000_0000 - 0xffff_b800_0000_7fff   32 KiB  CPU 0 Kernel Stack
0xffff_b800_0001_0000 - 0xffff_b800_0001_7fff   32 KiB  CPU 1 Kernel Stack
0xffff_b800_0002_0000 - 0xffff_b800_0002_7fff   32 KiB  CPU 2 Kernel Stack
...

0xffff_d000_0000_0000 - 0xffff_d000_ffff_ffff  256 GiB  MMIO Remap Region
                                                          Device registers mapped here

0xffff_e000_0000_0000 - 0xffff_e000_0000_0fff    4 KiB  Exception Vector Table
```

## User Virtual Address Space (AArch64)

```
Virtual Address Range                      Region
────────────────────────────────────────────────────────────────
0x0000_0000_0000_0000                      NULL page (unmapped, guard page)

0x0000_0000_0001_0000 - 0x0000_4fff_ffff_ffff  User program + stack
                                                (varies with ASLR)

0x0000_5000_0000_0000                      Dynamic linker load bias
                                            (main program loaded here)

0x0000_7000_0000_0000                      Library bias
                                            (libc, libm, etc. loaded here)

0x0000_7fff_f000_0000 - 0x0000_7fff_ffff_efff  User stack (grows downward)

0x0000_7fff_ffff_f000 - 0x0000_7fff_ffff_ffff    4 KiB  vDSO
```

## Physical Memory (QEMU virt machine)

```
Physical Address Range                     Region
────────────────────────────────────────────────────────────────
0x0000_0000 - 0x0000_0fff                  Reserved / boot ROM

0x0800_0000 - 0x0800_ffff    64 KiB        GICv2 Distributor

0x0801_0000 - 0x0801_ffff    64 KiB        GICv2 CPU Interface

0x0900_0000 - 0x0900_0fff     4 KiB        PL011 UART

0x0A00_0000 - 0x0A00_0fff     4 KiB        RTC (Real Time Clock)

0x1000_0000 - 0x1000_0fff     4 KiB        VirtIO Block Device

0x4000_0000 - ...              varies       RAM (reported by DTB)
                                            Default: 512 MiB
                                            (0x4000_0000 to 0x5fff_ffff)
```

## Stack Layout (Kernel Stack, per CPU)

```
High address (stack base)
┌─────────────────────────────┐ ← Initial SP
│         Stack               │
│  (grows downward)           │
│                             │
│  Exception frames           │
│  (ExceptionState structs)   │
│  ...                        │
│                             │
└─────────────────────────────┘ ← Stack limit (guard page below)
Low address (guard page — unmapped, catches stack overflow)
```

## ELF Process Layout After `exec`

```
High address
┌───────────────────────────┐ 0x0000_7fff_ffff_f000
│ vDSO                      │ (kernel-provided)
├───────────────────────────┤
│ Environment strings       │
│ Argument strings          │
│ argv[], envp[] pointers   │
│ Auxiliary vector          │
│ argc                      │
├───────────────────────────┤ ← Initial stack pointer
│ Stack (grows down)        │
│                           │
│                           │
│ ...                       │
│                           │
├───────────────────────────┤
│ Dynamic libraries         │ (mapped from 0x7000_0000_0000)
│ (libc.so, libm.so, etc.)  │
├───────────────────────────┤
│ Main program              │ (mapped from 0x5000_0000_0000)
│ .text (code)              │
│ .rodata (read-only data)  │
│ .data (initialized data)  │
│ .bss (zero-init data)     │
├───────────────────────────┤
│ Heap (grows up)           │ (via brk/mmap)
├───────────────────────────┤
│ NULL guard page           │
Low address
```
