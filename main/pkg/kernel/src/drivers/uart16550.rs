use core::fmt;
use x86_64::instructions::port::Port;

/// A port-mapped UART 16550 serial interface.
pub struct SerialPort;

impl SerialPort {
    pub const fn new(_port: u16) -> Self {
        Self
    }

    /// Initializes the serial port.
    #[allow(clippy::identity_op)]
    pub fn init(&self) {
        // DONE: Initialize the serial port
        const PORT: u16 = 0x3F8; // COM1

        unsafe {
            let mut port: Port<u8> = Port::new(PORT + 1);
            port.write(0x00); // Disable all interrupts.
            let mut port: Port<u8> = Port::new(PORT + 3);
            port.write(0x80); // Disable DLAB (set baud rate divisor).
            let mut port: Port<u8> = Port::new(PORT + 0);
            port.write(0x03); // Set divisor to 3 (lo byte) 38400 baud.
            let mut port: Port<u8> = Port::new(PORT + 1);
            port.write(0x00); //                  (hi byte).
            let mut port: Port<u8> = Port::new(PORT + 3);
            port.write(0x03); // 8 bits, no parity, one stop bit.
            let mut port: Port<u8> = Port::new(PORT + 2);
            port.write(0xC7); // Enable FIFO, clear them, with 14-byte threshold.
            let mut port: Port<u8> = Port::new(PORT + 4);
            port.write(0x0B); // IRQs enabled, RTS/DSR set.
            let mut port: Port<u8> = Port::new(PORT + 4);
            port.write(0x1E); // Set in loopback mode, test the serial chip.
            let mut port: Port<u8> = Port::new(PORT + 0);
            port.write(0xAE); // Test serial chip (send byte 0xAE and check if serial returns same byte).
            
            // Check if serial is faulty (i.e: not same byte as sent).
            if port.read() != 0xAE {
                panic!("Serial is falty.");
            };

            // If serial is not faulty set it in normal operation mode.
            let mut port: Port<u8> = Port::new(PORT + 4);
            port.write(0x0F);
        }
    }

    /// Sends a byte on the serial port.
    #[allow(clippy::identity_op)]
    pub fn send(&mut self, data: u8) {
        // DONE: Send a byte on the serial port
        const PORT: u16 = 0x3F8; // COM1

        unsafe {
            let mut rdi: Port<u8> = Port::new(PORT + 5);
            let mut rax: Port<u8> = Port::new(PORT + 0);

            while (rdi.read() & 0x20) == 0 {}
            rax.write(data);
        }
    }

    /// Receives a byte on the serial port no wait.
    #[allow(clippy::identity_op)]
    pub fn receive(&mut self) -> Option<u8> {
        // FIXME: Receive a byte on the serial port no wait
        const PORT: u16 = 0x3F8; // COM1

        unsafe {
            let mut rdi: Port<u8> = Port::new(PORT + 5);
            let mut rax: Port<u8> = Port::new(PORT + 0);

            if (rdi.read() & 0x01) == 0 {
                None
            } else {
                Some(rax.read())
            }
        }
    }
}

impl fmt::Write for SerialPort {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            self.send(byte);
        }
        Ok(())
    }
}
