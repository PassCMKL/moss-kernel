# Character Devices

**Character devices** transfer data as a stream of bytes, one character at a time. The most fundamental example is a **UART** (Universal Asynchronous Receiver/Transmitter) — the hardware that provides the serial console.

## Memory-Mapped I/O

Modern devices are controlled through **Memory-Mapped I/O (MMIO)**. The device's control registers appear at specific physical addresses. Reading or writing these addresses sends commands to or reads status from the device.

For example, the ARM PL011 UART has registers at fixed offsets from its base address:

```
Base + 0x00: DR   — Data Register (read: received byte, write: transmit byte)
Base + 0x04: RSR  — Receive Status Register
Base + 0x18: FR   — Flag Register (bit 5: TX FIFO full, bit 4: RX FIFO empty)
Base + 0x24: IBRD — Integer Baud Rate Divisor
Base + 0x28: FBRD — Fractional Baud Rate Divisor
Base + 0x2C: LCR  — Line Control Register
Base + 0x30: CR   — Control Register
Base + 0x38: IMSC — Interrupt Mask Set/Clear Register
```

## The PL011 UART Driver

Moss includes a driver for the ARM PL011 UART (`src/drivers/uart/pl011.rs`). The kernel uses this for console output (`kprintln!`).

### Transmitting a Byte

```rust
pub fn write_byte(&self, byte: u8) {
    unsafe {
        // Wait until the TX FIFO has space
        while self.fr().read() & FR_TXFF != 0 {
            // FR_TXFF = TX FIFO Full bit
            core::hint::spin_loop();
        }
        // Write the byte to the Data Register
        self.dr().write(byte as u32);
    }
}

pub fn write_str(&self, s: &str) {
    for byte in s.bytes() {
        if byte == b'\n' {
            self.write_byte(b'\r');  // Most terminals need CR+LF
        }
        self.write_byte(byte);
    }
}
```

### Receiving a Byte (Interrupt-Driven)

For receiving, the driver registers an interrupt handler:

```rust
impl InterruptHandler for Pl011Driver {
    fn handle(&self, _irq: InterruptId) {
        unsafe {
            // Read all available bytes from the RX FIFO
            while self.fr().read() & FR_RXFE == 0 {
                // FR_RXFE = RX FIFO Empty
                let byte = (self.dr().read() & 0xFF) as u8;
                self.rx_buffer.push(byte);
                // Wake any readers waiting for data
                self.reader_waker.wake();
            }
        }
    }
}
```

### Why `volatile`?

MMIO reads and writes must use **volatile** access:

```rust
// Non-volatile: compiler may optimize this away or reorder it
*(0xffff_d000_1234 as *mut u32) = 0x1;

// Volatile: compiler guarantees the write happens in program order
ptr::write_volatile(0xffff_d000_1234 as *mut u32, 0x1);
```

Without volatile, the compiler might:
- Cache the value and never write to the register
- Reorder writes, breaking hardware state machines
- Eliminate "redundant" reads that check hardware status flags

## Other UART Drivers

Moss includes additional UART drivers for different hardware:

- **`imx_lp.rs`**: i.MX Low-Power UART (used in NXP i.MX SoCs)
- **`bcm2835_aux.rs`**: Raspberry Pi's auxiliary UART (in bcm2835/bcm2711 SoCs)

All three implement the same internal interface, so the rest of the kernel does not need to know which hardware is present.

## Driver Abstraction

The console abstraction `src/console/` wraps the UART driver and provides the `kprint!` / `kprintln!` macros used throughout the kernel:

```rust
pub static CONSOLE: OnceCell<SpinLock<&'static dyn Console>> = OnceCell::new();

#[macro_export]
macro_rules! kprintln {
    ($fmt:literal $(, $args:expr)*) => {
        if let Some(console) = crate::console::CONSOLE.get() {
            write!(console.lock(), concat!($fmt, "\n") $(, $args)*).ok();
        }
    }
}
```

## Exercises

1. Why must MMIO accesses be marked volatile? What kind of compiler optimization would break a UART driver that didn't use volatile?

2. The PL011 driver busy-waits when the TX FIFO is full. For a high-throughput use case, would an interrupt-driven approach be better? What would the code look like?

3. Design a simple ring buffer for the UART receive path: what happens when the buffer is full and new data arrives from the hardware?
