/// 16-bit cell-addressed memory.
/// Each address refers to a 16-bit cell. A 32-bit word spans two consecutive cells.
pub struct Memory {
    cells: Vec<u16>,
}

impl Memory {
    pub fn new(size_cells: usize) -> Self {
        Self {
            cells: vec![0; size_cells],
        }
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
}
