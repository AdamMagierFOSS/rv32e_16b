pub struct Memory {
    cells: Vec<u16>,
}

impl Memory {
    pub fn new(size_cells: usize) -> Self {
        Self {
            cells: vec![0; size_cells],
        }
    }

    /// Load an 8-bit byte from a byte address.
    /// Bit 0 of the byte address selects the low or high byte within a cell.
    pub fn load8(&self, byte_addr: u32) -> u8 {
        let cell = self.cells[(byte_addr >> 1) as usize];
        if byte_addr & 1 == 0 { cell as u8 } else { (cell >> 8) as u8 }
    }

    /// Store an 8-bit byte at a byte address.
    pub fn store8(&mut self, byte_addr: u32, val: u8) {
        let idx = (byte_addr >> 1) as usize;
        let cell = self.cells[idx];
        self.cells[idx] = if byte_addr & 1 == 0 {
            (cell & 0xFF00) | val as u16
        } else {
            (cell & 0x00FF) | ((val as u16) << 8)
        };
    }

    pub fn load16(&self, addr: u32) -> u16 {
        self.cells[addr as usize]
    }

    pub fn store16(&mut self, addr: u32, val: u16) {
        self.cells[addr as usize] = val;
    }

    pub fn load32(&self, addr: u32) -> u32 {
        let lo = self.cells[addr as usize] as u32;
        let hi = self.cells[addr as usize + 1] as u32;
        lo | (hi << 16)
    }

    pub fn store32(&mut self, addr: u32, val: u32) {
        self.cells[addr as usize] = val as u16;
        self.cells[addr as usize + 1] = (val >> 16) as u16;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_16() {
        let mut mem = Memory::new(16);
        mem.store16(3, 0xABCD);
        assert_eq!(mem.load16(3), 0xABCD);
    }

    #[test]
    fn round_trip_32() {
        let mut mem = Memory::new(16);
        mem.store32(4, 0xDEAD_BEEF);
        assert_eq!(mem.load32(4), 0xDEAD_BEEF);
        assert_eq!(mem.load16(4), 0xBEEF);
        assert_eq!(mem.load16(5), 0xDEAD);
    }

    #[test]
    fn sub_cell_byte_access() {
        let mut mem = Memory::new(16);
        mem.store16(0, 0xBEEF);
        // byte_addr 0 = low byte of cell 0
        assert_eq!(mem.load8(0), 0xEF);
        // byte_addr 1 = high byte of cell 0
        assert_eq!(mem.load8(1), 0xBE);
        // write individual bytes
        mem.store8(4, 0x41); // 'A' into low byte of cell 2
        mem.store8(5, 0x42); // 'B' into high byte of cell 2
        assert_eq!(mem.load16(2), 0x4241);
        assert_eq!(mem.load8(4), 0x41);
        assert_eq!(mem.load8(5), 0x42);
    }
}
