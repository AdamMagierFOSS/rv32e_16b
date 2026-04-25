pub const NUM_REGS: usize = 16;

pub struct RegisterFile {
    regs: [u32; NUM_REGS],
}

impl RegisterFile {
    pub fn new() -> Self {
        Self {
            regs: [0; NUM_REGS],
        }
    }

    pub fn read(&self, idx: u8) -> u32 {
        self.regs[idx as usize]
    }

    pub fn write(&mut self, idx: u8, val: u32) {
        if idx != 0 {
            self.regs[idx as usize] = val;
        }
    }

    pub fn dump(&self) -> &[u32; NUM_REGS] {
        &self.regs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn x0_always_zero() {
        let mut rf = RegisterFile::new();
        rf.write(0, 42);
        assert_eq!(rf.read(0), 0);
    }

    #[test]
    fn read_write() {
        let mut rf = RegisterFile::new();
        rf.write(5, 0xCAFE);
        assert_eq!(rf.read(5), 0xCAFE);
    }
}
