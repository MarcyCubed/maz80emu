//! Useful tools to manage access to memory

/// Unified interface for memory
pub trait Memory {
    /// Load a byte from memory
    fn load(&self, address: u16) -> u8;

    /// Store a byte to memory
    fn store(&mut self, address: u16, data: u8);

    /// Check if the address is within the memory bounds
    fn contains(&self, address: u16) -> bool;
}

impl<const N: usize> Memory for [u8; N] {
    fn load(&self, address: u16) -> u8 {
        self[address as usize]
    }

    fn store(&mut self, address: u16, data: u8) {
        self[address as usize] = data;
    }

    fn contains(&self, address: u16) -> bool {
        (address as usize) < N
    }
}

impl Memory for [u8] {
    fn load(&self, address: u16) -> u8 {
        self[address as usize]
    }

    fn store(&mut self, address: u16, data: u8) {
        self[address as usize] = data;
    }

    fn contains(&self, address: u16) -> bool {
        (address as usize) < self.len()
    }
}

/// Read Only Memory
impl Memory for &[u8] {
    fn load(&self, address: u16) -> u8 {
        self[address as usize]
    }

    fn store(&mut self, _address: u16, _data: u8) {
        // Just ignore it
    }

    fn contains(&self, address: u16) -> bool {
        (address as usize) < self.len()
    }
}
