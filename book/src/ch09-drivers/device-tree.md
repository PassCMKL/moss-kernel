# Device Tree

On x86 systems, hardware announces itself through mechanisms like PCI configuration space and ACPI tables. On ARM systems, there is no such self-describing mechanism — a **Device Tree Blob (DTB)** must be provided to the kernel by the bootloader.

## What Is a Device Tree?

A device tree is a hierarchical description of hardware. It describes:
- Memory regions (base address, size)
- CPU topology (number of CPUs, features)
- Peripheral devices (base address, interrupt line, clock frequency)
- Bus hierarchy (which devices are connected to which bus)

The device tree source format (DTS) is human-readable. The compiled binary form (DTB) is what the bootloader passes to the kernel.

### Example DTS Fragment

```dts
// Simplified QEMU virt machine device tree
/ {
    memory@40000000 {
        device_type = "memory";
        reg = <0x0 0x40000000 0x0 0x40000000>;  // 1 GiB at 0x40000000
    };

    pl011@9000000 {
        compatible = "arm,pl011", "arm,primecell";
        reg = <0x0 0x9000000 0x0 0x1000>;       // Registers at 0x9000000
        interrupts = <0x0 0x1 0x4>;              // IRQ #33
        clock-frequency = <24000000>;            // 24 MHz clock
    };

    intc@8000000 {
        compatible = "arm,gic-v2";
        reg = <0x0 0x8000000 0x0 0x10000>,       // Distributor
              <0x0 0x8010000 0x0 0x10000>;        // CPU interface
    };
};
```

The `compatible` string (e.g., `"arm,pl011"`) identifies the device type. The kernel uses these strings to find the right driver.

## How Moss Uses the Device Tree

Moss parses the DTB at two points:

### Stage 1: Memory Discovery

During Stage 1, the kernel reads the `memory` nodes to discover the available physical memory:

```rust
pub fn parse_dtb_memory(dtb: &Dtb) -> Vec<MemRegion> {
    let mut regions = Vec::new();

    for node in dtb.find_nodes_by_type("memory") {
        let reg = node.property("reg").unwrap();
        // reg = [(base_addr, size), ...]
        for (base, size) in parse_reg_property(reg) {
            regions.push(MemRegion { base: PA(base), size });
        }
    }

    regions
}
```

This is how the frame allocator knows what physical memory exists.

### Stage 2: Device Discovery

During Stage 2, the kernel walks the device tree looking for compatible devices:

```rust
pub fn probe_devices(dtb: &Dtb) {
    for node in dtb.all_nodes() {
        let compatible = match node.property("compatible") {
            Some(s) => s,
            None => continue,
        };

        // Find a driver that matches any of the compatible strings
        if let Some(driver) = find_driver(compatible) {
            let base_pa = parse_reg_base(node.property("reg").unwrap());
            let irq = parse_irq(node.property("interrupts"));

            // Map the device's MMIO region
            let base_va = mmio_remap(base_pa, PAGE_SIZE);

            // Initialize the driver
            driver.probe(base_va, irq);
        }
    }
}
```

The device driver table maps compatible strings to probe functions:

```rust
static DRIVER_TABLE: &[(&str, fn(VA, Option<IrqId>))] = &[
    ("arm,pl011",    pl011::probe),
    ("arm,gic-v2",   gic_v2::probe),
    ("arm,gic-v3",   gic_v3::probe),
    ("arm,armv8-timer", armv8_timer::probe),
];
```

## DTB Format

The DTB is a binary format with:
- A header (magic number, version, structure block offsets)
- A strings block (property name strings)
- A structure block (nodes and properties as a flat serialized tree)

Moss includes a minimal DTB parser that can:
- Locate nodes by name or path
- Read property values (u32, u64, strings, byte arrays)
- Iterate over child nodes

## Exercises

1. What would a kernel have to do on x86 to discover the UART base address? How does this compare to using a device tree?

2. What happens if the bootloader provides a device tree that lists a device the kernel doesn't have a driver for? Should this be an error?

3. The device tree `reg` property encodes addresses and sizes. Why is the format `<parent-address-cells child-address-cells>` and not simply a 64-bit integer?
