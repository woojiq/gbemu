mod registers;

use registers::Registers;

#[allow(clippy::upper_case_acronyms)]
pub struct CPU {
    registers: Registers,
    /// Program counter.
    pc: usize,
    /// Stack pointer.
    sp: usize,
}

impl CPU {
    pub fn new() -> Self {
        Self {
            pc: 0,
            sp: 0,
            registers: Registers::new(),
        }
    }
}
