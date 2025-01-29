/**
List of abbreviations used in this document:
<https://rgbds.gbdev.io/docs/v0.9.0/gbz80.7#LEGEND>
*/
pub enum Instruction {
    Load(LoadInstruction),
    Bit8(Bit8Instruction),
    Bit16(Bit16Instruction),
    BitwiseLogic(BitwiseLogicInstruction),
    BitFlag(BitFlagInstruction),
    BitShift(BitShiftInstruction),
    JumpsAndSubroutine(JumpsAndSubroutineInstruction),
    CarryFlag(CarryFlagInstruction),
    StackManipulation(StackManipulationInstruction),
    InterruptRelated(InterruptRelatedInstruction),
    Miscellaneous(MiscellaneousInstruction),
}

pub enum LoadInstruction {
    LD_R8_U8(R8),
    LD_R8_R8(R8, R8),
    LD_R8_HLP(R8),
    LD_HLP_R8(R8),
    LD_HLP_U8,
    LD_R16_U16(R16),
    LD_U16_A,
    LD_A_U16,

    LD_A_R16P(R16P),
    LD_R16P_A(R16P),

    LDH_A_U8,
    LDH_U8_A,
    LD_C_A,
    LD_A_C,
}

pub enum Bit8Instruction {
    ADD_A_R8(R8),
    ADD_A_HLP,
    ADD_A_U8,

    ADC_A_R8(R8),
    ADC_A_HLP,
    ADC_A_U8,

    SUB_A_R8(R8),
    SUB_A_HLP,
    SUB_A_U8,

    SBC_A_R8(R8),
    SBC_A_HLP,
    SBC_A_U8,

    CP_A_R8(R8),
    CP_A_HLP,
    CP_A_U8,

    INC_R8(R8),
    INC_HLP,

    DEC_R8(R8),
    DEC_HLP,
}

pub enum Bit16Instruction {
    ADD_HL_R16(R16),

    INC_R16(R16),
    DEC_R16(R16),
}

pub enum BitwiseLogicInstruction {
    CPL,

    AND_A_R8(R8),
    AND_A_HLP,
    AND_A_U8,

    XOR_A_R8(R8),
    XOR_A_HLP,
    XOR_A_U8,

    OR_A_R8(R8),
    OR_A_HLP,
    OR_A_U8,
}

pub enum BitFlagInstruction {
    BIT_U3_R8E(U3, R8E),
    RES_U3_R8E(U3, R8E),
    SET_U3_R8E(U3, R8E),
}

pub enum BitShiftInstruction {
    RLA,
    RL_R8(R8),
    RL_HLP,
    RLC_R8(R8),
    RLC_HLP,
    RLCA,
    RRA,
    RR_R8(R8),
    RR_HLP,
    RRCA,
    RRC_R8(R8),
    RRC_HLP,
    SLA_R8(R8),
    SLA_HLP,
    SRA_R8(R8),
    SRA_HLP,
    SRL_R8(R8),
    SRL_HLP,
    SWAP_R8(R8),
    SWAP_HLP,
}

pub enum JumpsAndSubroutineInstruction {
    JR_CC_I8(CC),
    JR_I8,
    JP_CC_U16(CC),
    JP_U16,
    JP_HL,

    CALL_CC_U16(CC),
    CALL_U16,

    RET_CC(CC),
    RET,
    RETI,

    RST(VEC),
}

pub enum CarryFlagInstruction {
    SCF,
    CCF,
}

pub enum StackManipulationInstruction {
    ADD_HL_SP,
    ADD_SP_I8,

    LD_SP_U16,
    LD_SP_HL,
    LD_U16_SP,
    LD_HL_SP_PLUS_I8,

    POP_AF,
    POP_R16(R16),
    PUSH_AF,
    PUSH_R16(R16),

    INC_SP,
    DEC_SP,
}

pub enum InterruptRelatedInstruction {
    DI,
    EI,
    HALT,
}

pub enum MiscellaneousInstruction {
    DAA,
    NOP,
    STOP,
}

/// Any of the 8-bit registers (A, B, C, D, E, H, L).
pub enum R8 {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
}

/// Extended [`R8`]
pub enum R8E {
    R8(R8),
    HLP,
}

/// Any of the general-purpose 16-bit registers (BC, DE, HL).
pub enum R16 {
    BC,
    DE,
    HL,
}

pub enum R16P {
    BC,
    DE,
    HLI,
    HLD,
}

/// An RST vector (0x00, 0x08, 0x10, 0x18, 0x20, 0x28, 0x30, and 0x38).
pub enum VEC {
    X00,
    X08,
    X10,
    X18,
    X20,
    X28,
    X30,
    X38,
}

/// A condition code
pub enum CC {
    /// Execute if Z is set.
    Z,
    /// Execute if Z is not set.
    NZ,
    /// Execute if C is set.
    C,
    /// Execute if C is not set.
    NC,
}

/// 3-bit unsigned bit index (0 to 7).
pub enum U3 {
    B0,
    B1,
    B2,
    B3,
    B4,
    B5,
    B6,
    B7,
}

impl Instruction {
    pub fn from_byte(byte: u8, prefixed: bool) -> Option<Self> {
        if prefixed {
            Self::from_byte_prefixed(byte)
        } else {
            Self::from_byte_not_prefixed(byte)
        }
    }

    fn from_byte_not_prefixed(byte: u8) -> Option<Self> {
        match byte {
            0x00 => Some(Instruction::Miscellaneous(MiscellaneousInstruction::NOP)),
            0x01 => Some(Instruction::Load(LoadInstruction::LD_R16_U16(R16::BC))),
            0x02 => Some(Instruction::Load(LoadInstruction::LD_R16P_A(R16P::BC))),
            0x03 => Some(Instruction::Bit16(Bit16Instruction::INC_R16(R16::BC))),
            0x04 => Some(Instruction::Bit8(Bit8Instruction::INC_R8(R8::B))),
            0x05 => Some(Instruction::Bit8(Bit8Instruction::DEC_R8(R8::B))),
            0x06 => Some(Instruction::Load(LoadInstruction::LD_R8_U8(R8::B))),
            0x07 => Some(Instruction::BitShift(BitShiftInstruction::RLCA)),
            0x08 => Some(Instruction::StackManipulation(
                StackManipulationInstruction::LD_U16_SP,
            )),
            0x09 => Some(Instruction::Bit16(Bit16Instruction::ADD_HL_R16(R16::BC))),
            0x0a => Some(Instruction::Load(LoadInstruction::LD_A_R16P(R16P::BC))),
            0x0b => Some(Instruction::Bit16(Bit16Instruction::DEC_R16(R16::BC))),
            0x0c => Some(Instruction::Bit8(Bit8Instruction::INC_R8(R8::C))),
            0x0d => Some(Instruction::Bit8(Bit8Instruction::DEC_R8(R8::C))),
            0x0e => Some(Instruction::Load(LoadInstruction::LD_R8_U8(R8::C))),
            0x0f => Some(Instruction::BitShift(BitShiftInstruction::RRCA)),

            0x10 => Some(Instruction::Miscellaneous(MiscellaneousInstruction::STOP)),
            0x11 => Some(Instruction::Load(LoadInstruction::LD_R16_U16(R16::DE))),
            0x12 => Some(Instruction::Load(LoadInstruction::LD_R16P_A(R16P::DE))),
            0x13 => Some(Instruction::Bit16(Bit16Instruction::INC_R16(R16::DE))),
            0x14 => Some(Instruction::Bit8(Bit8Instruction::INC_R8(R8::D))),
            0x15 => Some(Instruction::Bit8(Bit8Instruction::DEC_R8(R8::D))),
            0x16 => Some(Instruction::Load(LoadInstruction::LD_R8_U8(R8::D))),
            0x17 => Some(Instruction::BitShift(BitShiftInstruction::RLA)),
            0x18 => Some(Instruction::JumpsAndSubroutine(
                JumpsAndSubroutineInstruction::JR_I8,
            )),
            0x19 => Some(Instruction::Bit16(Bit16Instruction::ADD_HL_R16(R16::DE))),
            0x1a => Some(Instruction::Load(LoadInstruction::LD_A_R16P(R16P::DE))),
            0x1b => Some(Instruction::Bit16(Bit16Instruction::DEC_R16(R16::DE))),
            0x1c => Some(Instruction::Bit8(Bit8Instruction::INC_R8(R8::E))),
            0x1d => Some(Instruction::Bit8(Bit8Instruction::DEC_R8(R8::E))),
            0x1e => Some(Instruction::Load(LoadInstruction::LD_R8_U8(R8::E))),
            0x1f => Some(Instruction::BitShift(BitShiftInstruction::RRA)),

            0x20 => Some(Instruction::JumpsAndSubroutine(
                JumpsAndSubroutineInstruction::JR_CC_I8(CC::NZ),
            )),
            0x21 => Some(Instruction::Load(LoadInstruction::LD_R16_U16(R16::HL))),
            0x22 => Some(Instruction::Load(LoadInstruction::LD_R16P_A(R16P::HLI))),
            0x23 => Some(Instruction::Bit16(Bit16Instruction::INC_R16(R16::HL))),
            0x24 => Some(Instruction::Bit8(Bit8Instruction::INC_R8(R8::H))),
            0x25 => Some(Instruction::Bit8(Bit8Instruction::DEC_R8(R8::H))),
            0x26 => Some(Instruction::Load(LoadInstruction::LD_R8_U8(R8::H))),
            0x27 => Some(Instruction::Miscellaneous(MiscellaneousInstruction::DAA)),
            0x28 => Some(Instruction::JumpsAndSubroutine(
                JumpsAndSubroutineInstruction::JR_CC_I8(CC::Z),
            )),
            0x29 => Some(Instruction::Bit16(Bit16Instruction::ADD_HL_R16(R16::HL))),
            0x2a => Some(Instruction::Load(LoadInstruction::LD_A_R16P(R16P::HLI))),
            0x2b => Some(Instruction::Bit16(Bit16Instruction::DEC_R16(R16::HL))),
            0x2c => Some(Instruction::Bit8(Bit8Instruction::INC_R8(R8::L))),
            0x2d => Some(Instruction::Bit8(Bit8Instruction::DEC_R8(R8::L))),
            0x2e => Some(Instruction::Load(LoadInstruction::LD_R8_U8(R8::L))),
            0x2f => Some(Instruction::BitwiseLogic(BitwiseLogicInstruction::CPL)),

            0x30 => Some(Instruction::JumpsAndSubroutine(
                JumpsAndSubroutineInstruction::JR_CC_I8(CC::NC),
            )),
            0x31 => Some(Instruction::StackManipulation(
                StackManipulationInstruction::LD_SP_U16,
            )),
            0x32 => Some(Instruction::Load(LoadInstruction::LD_R16P_A(R16P::HLD))),
            0x33 => Some(Instruction::StackManipulation(
                StackManipulationInstruction::INC_SP,
            )),
            0x34 => Some(Instruction::Bit8(Bit8Instruction::INC_HLP)),
            0x35 => Some(Instruction::Bit8(Bit8Instruction::DEC_HLP)),
            0x36 => Some(Instruction::Load(LoadInstruction::LD_HLP_U8)),
            0x37 => Some(Instruction::CarryFlag(CarryFlagInstruction::SCF)),
            0x38 => Some(Instruction::JumpsAndSubroutine(
                JumpsAndSubroutineInstruction::JR_CC_I8(CC::C),
            )),
            0x39 => Some(Instruction::StackManipulation(
                StackManipulationInstruction::ADD_HL_SP,
            )),
            0x3a => Some(Instruction::Load(LoadInstruction::LD_A_R16P(R16P::HLD))),
            0x3b => Some(Instruction::StackManipulation(
                StackManipulationInstruction::DEC_SP,
            )),
            0x3c => Some(Instruction::Bit8(Bit8Instruction::INC_R8(R8::A))),
            0x3d => Some(Instruction::Bit8(Bit8Instruction::DEC_R8(R8::A))),
            0x3e => Some(Instruction::Load(LoadInstruction::LD_R8_U8(R8::A))),
            0x3f => Some(Instruction::CarryFlag(CarryFlagInstruction::CCF)),

            0x40 => Some(Instruction::Load(LoadInstruction::LD_R8_R8(R8::B, R8::B))),
            0x41 => Some(Instruction::Load(LoadInstruction::LD_R8_R8(R8::B, R8::C))),
            0x42 => Some(Instruction::Load(LoadInstruction::LD_R8_R8(R8::B, R8::D))),
            0x43 => Some(Instruction::Load(LoadInstruction::LD_R8_R8(R8::B, R8::E))),
            0x44 => Some(Instruction::Load(LoadInstruction::LD_R8_R8(R8::B, R8::H))),
            0x45 => Some(Instruction::Load(LoadInstruction::LD_R8_R8(R8::B, R8::L))),
            0x46 => Some(Instruction::Load(LoadInstruction::LD_R8_HLP(R8::B))),
            0x47 => Some(Instruction::Load(LoadInstruction::LD_R8_R8(R8::B, R8::A))),
            0x48 => Some(Instruction::Load(LoadInstruction::LD_R8_R8(R8::C, R8::B))),
            0x49 => Some(Instruction::Load(LoadInstruction::LD_R8_R8(R8::C, R8::C))),
            0x4a => Some(Instruction::Load(LoadInstruction::LD_R8_R8(R8::C, R8::D))),
            0x4b => Some(Instruction::Load(LoadInstruction::LD_R8_R8(R8::C, R8::E))),
            0x4c => Some(Instruction::Load(LoadInstruction::LD_R8_R8(R8::C, R8::H))),
            0x4d => Some(Instruction::Load(LoadInstruction::LD_R8_R8(R8::C, R8::L))),
            0x4e => Some(Instruction::Load(LoadInstruction::LD_R8_HLP(R8::C))),
            0x4f => Some(Instruction::Load(LoadInstruction::LD_R8_R8(R8::C, R8::A))),

            0x50 => Some(Instruction::Load(LoadInstruction::LD_R8_R8(R8::D, R8::B))),
            0x51 => Some(Instruction::Load(LoadInstruction::LD_R8_R8(R8::D, R8::C))),
            0x52 => Some(Instruction::Load(LoadInstruction::LD_R8_R8(R8::D, R8::D))),
            0x53 => Some(Instruction::Load(LoadInstruction::LD_R8_R8(R8::D, R8::E))),
            0x54 => Some(Instruction::Load(LoadInstruction::LD_R8_R8(R8::D, R8::H))),
            0x55 => Some(Instruction::Load(LoadInstruction::LD_R8_R8(R8::D, R8::L))),
            0x56 => Some(Instruction::Load(LoadInstruction::LD_R8_HLP(R8::D))),
            0x57 => Some(Instruction::Load(LoadInstruction::LD_R8_R8(R8::D, R8::A))),
            0x58 => Some(Instruction::Load(LoadInstruction::LD_R8_R8(R8::E, R8::B))),
            0x59 => Some(Instruction::Load(LoadInstruction::LD_R8_R8(R8::E, R8::C))),
            0x5a => Some(Instruction::Load(LoadInstruction::LD_R8_R8(R8::E, R8::D))),
            0x5b => Some(Instruction::Load(LoadInstruction::LD_R8_R8(R8::E, R8::E))),
            0x5c => Some(Instruction::Load(LoadInstruction::LD_R8_R8(R8::E, R8::H))),
            0x5d => Some(Instruction::Load(LoadInstruction::LD_R8_R8(R8::E, R8::L))),
            0x5e => Some(Instruction::Load(LoadInstruction::LD_R8_HLP(R8::E))),
            0x5f => Some(Instruction::Load(LoadInstruction::LD_R8_R8(R8::E, R8::A))),

            0x60 => Some(Instruction::Load(LoadInstruction::LD_R8_R8(R8::H, R8::B))),
            0x61 => Some(Instruction::Load(LoadInstruction::LD_R8_R8(R8::H, R8::C))),
            0x62 => Some(Instruction::Load(LoadInstruction::LD_R8_R8(R8::H, R8::D))),
            0x63 => Some(Instruction::Load(LoadInstruction::LD_R8_R8(R8::H, R8::E))),
            0x64 => Some(Instruction::Load(LoadInstruction::LD_R8_R8(R8::H, R8::H))),
            0x65 => Some(Instruction::Load(LoadInstruction::LD_R8_R8(R8::H, R8::L))),
            0x66 => Some(Instruction::Load(LoadInstruction::LD_R8_HLP(R8::H))),
            0x67 => Some(Instruction::Load(LoadInstruction::LD_R8_R8(R8::H, R8::A))),
            0x68 => Some(Instruction::Load(LoadInstruction::LD_R8_R8(R8::L, R8::B))),
            0x69 => Some(Instruction::Load(LoadInstruction::LD_R8_R8(R8::L, R8::C))),
            0x6a => Some(Instruction::Load(LoadInstruction::LD_R8_R8(R8::L, R8::D))),
            0x6b => Some(Instruction::Load(LoadInstruction::LD_R8_R8(R8::L, R8::E))),
            0x6c => Some(Instruction::Load(LoadInstruction::LD_R8_R8(R8::L, R8::H))),
            0x6d => Some(Instruction::Load(LoadInstruction::LD_R8_R8(R8::L, R8::L))),
            0x6e => Some(Instruction::Load(LoadInstruction::LD_R8_HLP(R8::L))),
            0x6f => Some(Instruction::Load(LoadInstruction::LD_R8_R8(R8::L, R8::A))),

            0x70 => Some(Instruction::Load(LoadInstruction::LD_HLP_R8(R8::B))),
            0x71 => Some(Instruction::Load(LoadInstruction::LD_HLP_R8(R8::C))),
            0x72 => Some(Instruction::Load(LoadInstruction::LD_HLP_R8(R8::D))),
            0x73 => Some(Instruction::Load(LoadInstruction::LD_HLP_R8(R8::E))),
            0x74 => Some(Instruction::Load(LoadInstruction::LD_HLP_R8(R8::H))),
            0x75 => Some(Instruction::Load(LoadInstruction::LD_HLP_R8(R8::L))),
            0x76 => Some(Instruction::InterruptRelated(
                InterruptRelatedInstruction::HALT,
            )),
            0x77 => Some(Instruction::Load(LoadInstruction::LD_HLP_R8(R8::A))),
            0x78 => Some(Instruction::Load(LoadInstruction::LD_R8_R8(R8::A, R8::B))),
            0x79 => Some(Instruction::Load(LoadInstruction::LD_R8_R8(R8::A, R8::C))),
            0x7a => Some(Instruction::Load(LoadInstruction::LD_R8_R8(R8::A, R8::D))),
            0x7b => Some(Instruction::Load(LoadInstruction::LD_R8_R8(R8::A, R8::E))),
            0x7c => Some(Instruction::Load(LoadInstruction::LD_R8_R8(R8::A, R8::H))),
            0x7d => Some(Instruction::Load(LoadInstruction::LD_R8_R8(R8::A, R8::L))),
            0x7e => Some(Instruction::Load(LoadInstruction::LD_R8_HLP(R8::A))),
            0x7f => Some(Instruction::Load(LoadInstruction::LD_R8_R8(R8::A, R8::A))),

            0x80 => Some(Instruction::Bit8(Bit8Instruction::ADD_A_R8(R8::B))),
            0x81 => Some(Instruction::Bit8(Bit8Instruction::ADD_A_R8(R8::C))),
            0x82 => Some(Instruction::Bit8(Bit8Instruction::ADD_A_R8(R8::D))),
            0x83 => Some(Instruction::Bit8(Bit8Instruction::ADD_A_R8(R8::E))),
            0x84 => Some(Instruction::Bit8(Bit8Instruction::ADD_A_R8(R8::H))),
            0x85 => Some(Instruction::Bit8(Bit8Instruction::ADD_A_R8(R8::L))),
            0x86 => Some(Instruction::Bit8(Bit8Instruction::ADD_A_HLP)),
            0x87 => Some(Instruction::Bit8(Bit8Instruction::ADD_A_R8(R8::A))),
            0x88 => Some(Instruction::Bit8(Bit8Instruction::ADC_A_R8(R8::B))),
            0x89 => Some(Instruction::Bit8(Bit8Instruction::ADC_A_R8(R8::C))),
            0x8a => Some(Instruction::Bit8(Bit8Instruction::ADC_A_R8(R8::D))),
            0x8b => Some(Instruction::Bit8(Bit8Instruction::ADC_A_R8(R8::E))),
            0x8c => Some(Instruction::Bit8(Bit8Instruction::ADC_A_R8(R8::H))),
            0x8d => Some(Instruction::Bit8(Bit8Instruction::ADC_A_R8(R8::L))),
            0x8e => Some(Instruction::Bit8(Bit8Instruction::ADC_A_HLP)),
            0x8f => Some(Instruction::Bit8(Bit8Instruction::ADC_A_R8(R8::A))),

            0x90 => Some(Instruction::Bit8(Bit8Instruction::SUB_A_R8(R8::B))),
            0x91 => Some(Instruction::Bit8(Bit8Instruction::SUB_A_R8(R8::C))),
            0x92 => Some(Instruction::Bit8(Bit8Instruction::SUB_A_R8(R8::D))),
            0x93 => Some(Instruction::Bit8(Bit8Instruction::SUB_A_R8(R8::E))),
            0x94 => Some(Instruction::Bit8(Bit8Instruction::SUB_A_R8(R8::H))),
            0x95 => Some(Instruction::Bit8(Bit8Instruction::SUB_A_R8(R8::L))),
            0x96 => Some(Instruction::Bit8(Bit8Instruction::SUB_A_HLP)),
            0x97 => Some(Instruction::Bit8(Bit8Instruction::SUB_A_R8(R8::A))),
            0x98 => Some(Instruction::Bit8(Bit8Instruction::SBC_A_R8(R8::B))),
            0x99 => Some(Instruction::Bit8(Bit8Instruction::SBC_A_R8(R8::C))),
            0x9a => Some(Instruction::Bit8(Bit8Instruction::SBC_A_R8(R8::D))),
            0x9b => Some(Instruction::Bit8(Bit8Instruction::SBC_A_R8(R8::E))),
            0x9c => Some(Instruction::Bit8(Bit8Instruction::SBC_A_R8(R8::H))),
            0x9d => Some(Instruction::Bit8(Bit8Instruction::SBC_A_R8(R8::L))),
            0x9e => Some(Instruction::Bit8(Bit8Instruction::SBC_A_HLP)),
            0x9f => Some(Instruction::Bit8(Bit8Instruction::SBC_A_R8(R8::A))),

            0xa0 => Some(Instruction::BitwiseLogic(
                BitwiseLogicInstruction::AND_A_R8(R8::B),
            )),
            0xa1 => Some(Instruction::BitwiseLogic(
                BitwiseLogicInstruction::AND_A_R8(R8::C),
            )),
            0xa2 => Some(Instruction::BitwiseLogic(
                BitwiseLogicInstruction::AND_A_R8(R8::D),
            )),
            0xa3 => Some(Instruction::BitwiseLogic(
                BitwiseLogicInstruction::AND_A_R8(R8::E),
            )),
            0xa4 => Some(Instruction::BitwiseLogic(
                BitwiseLogicInstruction::AND_A_R8(R8::H),
            )),
            0xa5 => Some(Instruction::BitwiseLogic(
                BitwiseLogicInstruction::AND_A_R8(R8::L),
            )),
            0xa6 => Some(Instruction::BitwiseLogic(
                BitwiseLogicInstruction::AND_A_HLP,
            )),
            0xa7 => Some(Instruction::BitwiseLogic(
                BitwiseLogicInstruction::AND_A_R8(R8::A),
            )),
            0xa8 => Some(Instruction::BitwiseLogic(
                BitwiseLogicInstruction::XOR_A_R8(R8::B),
            )),
            0xa9 => Some(Instruction::BitwiseLogic(
                BitwiseLogicInstruction::XOR_A_R8(R8::C),
            )),
            0xaa => Some(Instruction::BitwiseLogic(
                BitwiseLogicInstruction::XOR_A_R8(R8::D),
            )),
            0xab => Some(Instruction::BitwiseLogic(
                BitwiseLogicInstruction::XOR_A_R8(R8::E),
            )),
            0xac => Some(Instruction::BitwiseLogic(
                BitwiseLogicInstruction::XOR_A_R8(R8::H),
            )),
            0xad => Some(Instruction::BitwiseLogic(
                BitwiseLogicInstruction::XOR_A_R8(R8::L),
            )),
            0xae => Some(Instruction::BitwiseLogic(
                BitwiseLogicInstruction::XOR_A_HLP,
            )),
            0xaf => Some(Instruction::BitwiseLogic(
                BitwiseLogicInstruction::XOR_A_R8(R8::A),
            )),

            0xb0 => Some(Instruction::BitwiseLogic(BitwiseLogicInstruction::OR_A_R8(
                R8::B,
            ))),
            0xb1 => Some(Instruction::BitwiseLogic(BitwiseLogicInstruction::OR_A_R8(
                R8::C,
            ))),
            0xb2 => Some(Instruction::BitwiseLogic(BitwiseLogicInstruction::OR_A_R8(
                R8::D,
            ))),
            0xb3 => Some(Instruction::BitwiseLogic(BitwiseLogicInstruction::OR_A_R8(
                R8::E,
            ))),
            0xb4 => Some(Instruction::BitwiseLogic(BitwiseLogicInstruction::OR_A_R8(
                R8::H,
            ))),
            0xb5 => Some(Instruction::BitwiseLogic(BitwiseLogicInstruction::OR_A_R8(
                R8::L,
            ))),
            0xb6 => Some(Instruction::BitwiseLogic(BitwiseLogicInstruction::OR_A_HLP)),
            0xb7 => Some(Instruction::BitwiseLogic(BitwiseLogicInstruction::OR_A_R8(
                R8::A,
            ))),
            0xb8 => Some(Instruction::Bit8(Bit8Instruction::CP_A_R8(R8::B))),
            0xb9 => Some(Instruction::Bit8(Bit8Instruction::CP_A_R8(R8::C))),
            0xba => Some(Instruction::Bit8(Bit8Instruction::CP_A_R8(R8::D))),
            0xbb => Some(Instruction::Bit8(Bit8Instruction::CP_A_R8(R8::E))),
            0xbc => Some(Instruction::Bit8(Bit8Instruction::CP_A_R8(R8::H))),
            0xbd => Some(Instruction::Bit8(Bit8Instruction::CP_A_R8(R8::L))),
            0xbe => Some(Instruction::Bit8(Bit8Instruction::CP_A_HLP)),
            0xbf => Some(Instruction::Bit8(Bit8Instruction::CP_A_R8(R8::A))),

            0xc0 => Some(Instruction::JumpsAndSubroutine(
                JumpsAndSubroutineInstruction::RET_CC(CC::NZ),
            )),
            0xc1 => Some(Instruction::StackManipulation(
                StackManipulationInstruction::POP_R16(R16::BC),
            )),
            0xc2 => Some(Instruction::JumpsAndSubroutine(
                JumpsAndSubroutineInstruction::JP_CC_U16(CC::NZ),
            )),
            0xc3 => Some(Instruction::JumpsAndSubroutine(
                JumpsAndSubroutineInstruction::JP_U16,
            )),
            0xc4 => Some(Instruction::JumpsAndSubroutine(
                JumpsAndSubroutineInstruction::CALL_CC_U16(CC::NZ),
            )),
            0xc5 => Some(Instruction::StackManipulation(
                StackManipulationInstruction::PUSH_R16(R16::BC),
            )),
            0xc6 => Some(Instruction::Bit8(Bit8Instruction::ADD_A_U8)),
            0xc7 => Some(Instruction::JumpsAndSubroutine(
                JumpsAndSubroutineInstruction::RST(VEC::X00),
            )),
            0xc8 => Some(Instruction::JumpsAndSubroutine(
                JumpsAndSubroutineInstruction::RET_CC(CC::Z),
            )),
            0xc9 => Some(Instruction::JumpsAndSubroutine(
                JumpsAndSubroutineInstruction::RET,
            )),
            0xca => Some(Instruction::JumpsAndSubroutine(
                JumpsAndSubroutineInstruction::JP_CC_U16(CC::Z),
            )),
            0xcb => unimplemented!(), // Prefix CB
            0xcc => Some(Instruction::JumpsAndSubroutine(
                JumpsAndSubroutineInstruction::CALL_CC_U16(CC::Z),
            )),
            0xcd => Some(Instruction::JumpsAndSubroutine(
                JumpsAndSubroutineInstruction::CALL_U16,
            )),
            0xce => Some(Instruction::Bit8(Bit8Instruction::ADC_A_U8)),
            0xcf => Some(Instruction::JumpsAndSubroutine(
                JumpsAndSubroutineInstruction::RST(VEC::X08),
            )),

            0xd0 => Some(Instruction::JumpsAndSubroutine(
                JumpsAndSubroutineInstruction::RET_CC(CC::NC),
            )),
            0xd1 => Some(Instruction::StackManipulation(
                StackManipulationInstruction::POP_R16(R16::DE),
            )),
            0xd2 => Some(Instruction::JumpsAndSubroutine(
                JumpsAndSubroutineInstruction::JP_CC_U16(CC::NC),
            )),
            0xd4 => Some(Instruction::JumpsAndSubroutine(
                JumpsAndSubroutineInstruction::CALL_CC_U16(CC::NC),
            )),
            0xd5 => Some(Instruction::StackManipulation(
                StackManipulationInstruction::PUSH_R16(R16::DE),
            )),
            0xd6 => Some(Instruction::Bit8(Bit8Instruction::SUB_A_U8)),
            0xd7 => Some(Instruction::JumpsAndSubroutine(
                JumpsAndSubroutineInstruction::RST(VEC::X10),
            )),
            0xd8 => Some(Instruction::JumpsAndSubroutine(
                JumpsAndSubroutineInstruction::RET_CC(CC::C),
            )),
            0xd9 => Some(Instruction::JumpsAndSubroutine(
                JumpsAndSubroutineInstruction::RETI,
            )),
            0xda => Some(Instruction::JumpsAndSubroutine(
                JumpsAndSubroutineInstruction::JP_CC_U16(CC::C),
            )),
            0xdc => Some(Instruction::JumpsAndSubroutine(
                JumpsAndSubroutineInstruction::CALL_CC_U16(CC::C),
            )),
            0xde => Some(Instruction::Bit8(Bit8Instruction::SBC_A_U8)),
            0xdf => Some(Instruction::JumpsAndSubroutine(
                JumpsAndSubroutineInstruction::RST(VEC::X18),
            )),

            0xe0 => Some(Instruction::Load(LoadInstruction::LDH_U8_A)),
            0xe1 => Some(Instruction::StackManipulation(
                StackManipulationInstruction::POP_R16(R16::HL),
            )),
            0xe2 => Some(Instruction::Load(LoadInstruction::LD_C_A)),
            0xe5 => Some(Instruction::StackManipulation(
                StackManipulationInstruction::PUSH_R16(R16::HL),
            )),
            0xe6 => Some(Instruction::BitwiseLogic(BitwiseLogicInstruction::AND_A_U8)),
            0xe7 => Some(Instruction::JumpsAndSubroutine(
                JumpsAndSubroutineInstruction::RST(VEC::X20),
            )),
            0xe8 => Some(Instruction::StackManipulation(
                StackManipulationInstruction::ADD_SP_I8,
            )),
            0xe9 => Some(Instruction::JumpsAndSubroutine(
                JumpsAndSubroutineInstruction::JP_HL,
            )),
            0xea => Some(Instruction::Load(LoadInstruction::LD_U16_A)),
            0xee => Some(Instruction::BitwiseLogic(BitwiseLogicInstruction::XOR_A_U8)),
            0xef => Some(Instruction::JumpsAndSubroutine(
                JumpsAndSubroutineInstruction::RST(VEC::X28),
            )),

            0xf0 => Some(Instruction::Load(LoadInstruction::LDH_A_U8)),
            0xf1 => Some(Instruction::StackManipulation(
                StackManipulationInstruction::POP_AF,
            )),
            0xf2 => Some(Instruction::Load(LoadInstruction::LD_A_C)),
            0xf3 => Some(Instruction::InterruptRelated(
                InterruptRelatedInstruction::DI,
            )),
            0xf5 => Some(Instruction::StackManipulation(
                StackManipulationInstruction::PUSH_AF,
            )),
            0xf6 => Some(Instruction::BitwiseLogic(BitwiseLogicInstruction::OR_A_U8)),
            0xf7 => Some(Instruction::JumpsAndSubroutine(
                JumpsAndSubroutineInstruction::RST(VEC::X30),
            )),
            0xf8 => Some(Instruction::StackManipulation(
                StackManipulationInstruction::LD_HL_SP_PLUS_I8,
            )),
            0xf9 => Some(Instruction::StackManipulation(
                StackManipulationInstruction::LD_SP_HL,
            )),
            0xfa => Some(Instruction::Load(LoadInstruction::LD_A_U16)),
            0xfb => Some(Instruction::InterruptRelated(
                InterruptRelatedInstruction::EI,
            )),
            0xfe => Some(Instruction::Bit8(Bit8Instruction::CP_A_U8)),
            0xff => Some(Instruction::JumpsAndSubroutine(
                JumpsAndSubroutineInstruction::RST(VEC::X38),
            )),

            _ => None,
        }
    }

    fn from_byte_prefixed(byte: u8) -> Option<Self> {
        match byte {
            0x00 => Some(Instruction::BitShift(BitShiftInstruction::RLC_R8(R8::B))),
            0x01 => Some(Instruction::BitShift(BitShiftInstruction::RLC_R8(R8::C))),
            0x02 => Some(Instruction::BitShift(BitShiftInstruction::RLC_R8(R8::D))),
            0x03 => Some(Instruction::BitShift(BitShiftInstruction::RLC_R8(R8::E))),
            0x04 => Some(Instruction::BitShift(BitShiftInstruction::RLC_R8(R8::H))),
            0x05 => Some(Instruction::BitShift(BitShiftInstruction::RLC_R8(R8::L))),
            0x06 => Some(Instruction::BitShift(BitShiftInstruction::RLC_HLP)),
            0x07 => Some(Instruction::BitShift(BitShiftInstruction::RLC_R8(R8::A))),
            0x08 => Some(Instruction::BitShift(BitShiftInstruction::RRC_R8(R8::B))),
            0x09 => Some(Instruction::BitShift(BitShiftInstruction::RRC_R8(R8::C))),
            0x0a => Some(Instruction::BitShift(BitShiftInstruction::RRC_R8(R8::D))),
            0x0b => Some(Instruction::BitShift(BitShiftInstruction::RRC_R8(R8::E))),
            0x0c => Some(Instruction::BitShift(BitShiftInstruction::RRC_R8(R8::H))),
            0x0d => Some(Instruction::BitShift(BitShiftInstruction::RRC_R8(R8::L))),
            0x0e => Some(Instruction::BitShift(BitShiftInstruction::RRC_HLP)),
            0x0f => Some(Instruction::BitShift(BitShiftInstruction::RRC_R8(R8::A))),

            0x10 => Some(Instruction::BitShift(BitShiftInstruction::RL_R8(R8::B))),
            0x11 => Some(Instruction::BitShift(BitShiftInstruction::RL_R8(R8::C))),
            0x12 => Some(Instruction::BitShift(BitShiftInstruction::RL_R8(R8::D))),
            0x13 => Some(Instruction::BitShift(BitShiftInstruction::RL_R8(R8::E))),
            0x14 => Some(Instruction::BitShift(BitShiftInstruction::RL_R8(R8::H))),
            0x15 => Some(Instruction::BitShift(BitShiftInstruction::RL_R8(R8::L))),
            0x16 => Some(Instruction::BitShift(BitShiftInstruction::RL_HLP)),
            0x17 => Some(Instruction::BitShift(BitShiftInstruction::RL_R8(R8::A))),
            0x18 => Some(Instruction::BitShift(BitShiftInstruction::RR_R8(R8::B))),
            0x19 => Some(Instruction::BitShift(BitShiftInstruction::RR_R8(R8::C))),
            0x1a => Some(Instruction::BitShift(BitShiftInstruction::RR_R8(R8::D))),
            0x1b => Some(Instruction::BitShift(BitShiftInstruction::RR_R8(R8::E))),
            0x1c => Some(Instruction::BitShift(BitShiftInstruction::RR_R8(R8::H))),
            0x1d => Some(Instruction::BitShift(BitShiftInstruction::RR_R8(R8::L))),
            0x1e => Some(Instruction::BitShift(BitShiftInstruction::RR_HLP)),
            0x1f => Some(Instruction::BitShift(BitShiftInstruction::RR_R8(R8::A))),

            0x20 => Some(Instruction::BitShift(BitShiftInstruction::SLA_R8(R8::B))),
            0x21 => Some(Instruction::BitShift(BitShiftInstruction::SLA_R8(R8::C))),
            0x22 => Some(Instruction::BitShift(BitShiftInstruction::SLA_R8(R8::D))),
            0x23 => Some(Instruction::BitShift(BitShiftInstruction::SLA_R8(R8::E))),
            0x24 => Some(Instruction::BitShift(BitShiftInstruction::SLA_R8(R8::H))),
            0x25 => Some(Instruction::BitShift(BitShiftInstruction::SLA_R8(R8::L))),
            0x26 => Some(Instruction::BitShift(BitShiftInstruction::SLA_HLP)),
            0x27 => Some(Instruction::BitShift(BitShiftInstruction::SLA_R8(R8::A))),
            0x28 => Some(Instruction::BitShift(BitShiftInstruction::SRA_R8(R8::B))),
            0x29 => Some(Instruction::BitShift(BitShiftInstruction::SRA_R8(R8::C))),
            0x2a => Some(Instruction::BitShift(BitShiftInstruction::SRA_R8(R8::D))),
            0x2b => Some(Instruction::BitShift(BitShiftInstruction::SRA_R8(R8::E))),
            0x2c => Some(Instruction::BitShift(BitShiftInstruction::SRA_R8(R8::H))),
            0x2d => Some(Instruction::BitShift(BitShiftInstruction::SRA_R8(R8::L))),
            0x2e => Some(Instruction::BitShift(BitShiftInstruction::SRA_HLP)),
            0x2f => Some(Instruction::BitShift(BitShiftInstruction::SRA_R8(R8::A))),

            0x30 => Some(Instruction::BitShift(BitShiftInstruction::SWAP_R8(R8::B))),
            0x31 => Some(Instruction::BitShift(BitShiftInstruction::SWAP_R8(R8::C))),
            0x32 => Some(Instruction::BitShift(BitShiftInstruction::SWAP_R8(R8::D))),
            0x33 => Some(Instruction::BitShift(BitShiftInstruction::SWAP_R8(R8::E))),
            0x34 => Some(Instruction::BitShift(BitShiftInstruction::SWAP_R8(R8::H))),
            0x35 => Some(Instruction::BitShift(BitShiftInstruction::SWAP_R8(R8::L))),
            0x36 => Some(Instruction::BitShift(BitShiftInstruction::SWAP_HLP)),
            0x37 => Some(Instruction::BitShift(BitShiftInstruction::SWAP_R8(R8::A))),
            0x38 => Some(Instruction::BitShift(BitShiftInstruction::SRL_R8(R8::B))),
            0x39 => Some(Instruction::BitShift(BitShiftInstruction::SRL_R8(R8::C))),
            0x3a => Some(Instruction::BitShift(BitShiftInstruction::SRL_R8(R8::D))),
            0x3b => Some(Instruction::BitShift(BitShiftInstruction::SRL_R8(R8::E))),
            0x3c => Some(Instruction::BitShift(BitShiftInstruction::SRL_R8(R8::H))),
            0x3d => Some(Instruction::BitShift(BitShiftInstruction::SRL_R8(R8::L))),
            0x3e => Some(Instruction::BitShift(BitShiftInstruction::SRL_HLP)),
            0x3f => Some(Instruction::BitShift(BitShiftInstruction::SRL_R8(R8::A))),

            0x40 => Some(Instruction::BitFlag(BitFlagInstruction::BIT_U3_R8E(
                U3::B0,
                R8E::R8(R8::B),
            ))),
            0x41 => Some(Instruction::BitFlag(BitFlagInstruction::BIT_U3_R8E(
                U3::B0,
                R8E::R8(R8::C),
            ))),
            0x42 => Some(Instruction::BitFlag(BitFlagInstruction::BIT_U3_R8E(
                U3::B0,
                R8E::R8(R8::D),
            ))),
            0x43 => Some(Instruction::BitFlag(BitFlagInstruction::BIT_U3_R8E(
                U3::B0,
                R8E::R8(R8::E),
            ))),
            0x44 => Some(Instruction::BitFlag(BitFlagInstruction::BIT_U3_R8E(
                U3::B0,
                R8E::R8(R8::H),
            ))),
            0x45 => Some(Instruction::BitFlag(BitFlagInstruction::BIT_U3_R8E(
                U3::B0,
                R8E::R8(R8::L),
            ))),
            0x46 => Some(Instruction::BitFlag(BitFlagInstruction::BIT_U3_R8E(
                U3::B0,
                R8E::HLP,
            ))),
            0x47 => Some(Instruction::BitFlag(BitFlagInstruction::BIT_U3_R8E(
                U3::B0,
                R8E::R8(R8::A),
            ))),
            0x48 => Some(Instruction::BitFlag(BitFlagInstruction::BIT_U3_R8E(
                U3::B1,
                R8E::R8(R8::B),
            ))),
            0x49 => Some(Instruction::BitFlag(BitFlagInstruction::BIT_U3_R8E(
                U3::B1,
                R8E::R8(R8::C),
            ))),
            0x4a => Some(Instruction::BitFlag(BitFlagInstruction::BIT_U3_R8E(
                U3::B1,
                R8E::R8(R8::D),
            ))),
            0x4b => Some(Instruction::BitFlag(BitFlagInstruction::BIT_U3_R8E(
                U3::B1,
                R8E::R8(R8::E),
            ))),
            0x4c => Some(Instruction::BitFlag(BitFlagInstruction::BIT_U3_R8E(
                U3::B1,
                R8E::R8(R8::H),
            ))),
            0x4d => Some(Instruction::BitFlag(BitFlagInstruction::BIT_U3_R8E(
                U3::B1,
                R8E::R8(R8::L),
            ))),
            0x4e => Some(Instruction::BitFlag(BitFlagInstruction::BIT_U3_R8E(
                U3::B1,
                R8E::HLP,
            ))),
            0x4f => Some(Instruction::BitFlag(BitFlagInstruction::BIT_U3_R8E(
                U3::B1,
                R8E::R8(R8::A),
            ))),

            0x50 => Some(Instruction::BitFlag(BitFlagInstruction::BIT_U3_R8E(
                U3::B2,
                R8E::R8(R8::B),
            ))),
            0x51 => Some(Instruction::BitFlag(BitFlagInstruction::BIT_U3_R8E(
                U3::B2,
                R8E::R8(R8::C),
            ))),
            0x52 => Some(Instruction::BitFlag(BitFlagInstruction::BIT_U3_R8E(
                U3::B2,
                R8E::R8(R8::D),
            ))),
            0x53 => Some(Instruction::BitFlag(BitFlagInstruction::BIT_U3_R8E(
                U3::B2,
                R8E::R8(R8::E),
            ))),
            0x54 => Some(Instruction::BitFlag(BitFlagInstruction::BIT_U3_R8E(
                U3::B2,
                R8E::R8(R8::H),
            ))),
            0x55 => Some(Instruction::BitFlag(BitFlagInstruction::BIT_U3_R8E(
                U3::B2,
                R8E::R8(R8::L),
            ))),
            0x56 => Some(Instruction::BitFlag(BitFlagInstruction::BIT_U3_R8E(
                U3::B2,
                R8E::HLP,
            ))),
            0x57 => Some(Instruction::BitFlag(BitFlagInstruction::BIT_U3_R8E(
                U3::B2,
                R8E::R8(R8::A),
            ))),
            0x58 => Some(Instruction::BitFlag(BitFlagInstruction::BIT_U3_R8E(
                U3::B3,
                R8E::R8(R8::B),
            ))),
            0x59 => Some(Instruction::BitFlag(BitFlagInstruction::BIT_U3_R8E(
                U3::B3,
                R8E::R8(R8::C),
            ))),
            0x5a => Some(Instruction::BitFlag(BitFlagInstruction::BIT_U3_R8E(
                U3::B3,
                R8E::R8(R8::D),
            ))),
            0x5b => Some(Instruction::BitFlag(BitFlagInstruction::BIT_U3_R8E(
                U3::B3,
                R8E::R8(R8::E),
            ))),
            0x5c => Some(Instruction::BitFlag(BitFlagInstruction::BIT_U3_R8E(
                U3::B3,
                R8E::R8(R8::H),
            ))),
            0x5d => Some(Instruction::BitFlag(BitFlagInstruction::BIT_U3_R8E(
                U3::B3,
                R8E::R8(R8::L),
            ))),
            0x5e => Some(Instruction::BitFlag(BitFlagInstruction::BIT_U3_R8E(
                U3::B3,
                R8E::HLP,
            ))),
            0x5f => Some(Instruction::BitFlag(BitFlagInstruction::BIT_U3_R8E(
                U3::B3,
                R8E::R8(R8::A),
            ))),

            0x60 => Some(Instruction::BitFlag(BitFlagInstruction::BIT_U3_R8E(
                U3::B4,
                R8E::R8(R8::B),
            ))),
            0x61 => Some(Instruction::BitFlag(BitFlagInstruction::BIT_U3_R8E(
                U3::B4,
                R8E::R8(R8::C),
            ))),
            0x62 => Some(Instruction::BitFlag(BitFlagInstruction::BIT_U3_R8E(
                U3::B4,
                R8E::R8(R8::D),
            ))),
            0x63 => Some(Instruction::BitFlag(BitFlagInstruction::BIT_U3_R8E(
                U3::B4,
                R8E::R8(R8::E),
            ))),
            0x64 => Some(Instruction::BitFlag(BitFlagInstruction::BIT_U3_R8E(
                U3::B4,
                R8E::R8(R8::H),
            ))),
            0x65 => Some(Instruction::BitFlag(BitFlagInstruction::BIT_U3_R8E(
                U3::B4,
                R8E::R8(R8::L),
            ))),
            0x66 => Some(Instruction::BitFlag(BitFlagInstruction::BIT_U3_R8E(
                U3::B4,
                R8E::HLP,
            ))),
            0x67 => Some(Instruction::BitFlag(BitFlagInstruction::BIT_U3_R8E(
                U3::B4,
                R8E::R8(R8::A),
            ))),
            0x68 => Some(Instruction::BitFlag(BitFlagInstruction::BIT_U3_R8E(
                U3::B5,
                R8E::R8(R8::B),
            ))),
            0x69 => Some(Instruction::BitFlag(BitFlagInstruction::BIT_U3_R8E(
                U3::B5,
                R8E::R8(R8::C),
            ))),
            0x6a => Some(Instruction::BitFlag(BitFlagInstruction::BIT_U3_R8E(
                U3::B5,
                R8E::R8(R8::D),
            ))),
            0x6b => Some(Instruction::BitFlag(BitFlagInstruction::BIT_U3_R8E(
                U3::B5,
                R8E::R8(R8::E),
            ))),
            0x6c => Some(Instruction::BitFlag(BitFlagInstruction::BIT_U3_R8E(
                U3::B5,
                R8E::R8(R8::H),
            ))),
            0x6d => Some(Instruction::BitFlag(BitFlagInstruction::BIT_U3_R8E(
                U3::B5,
                R8E::R8(R8::L),
            ))),
            0x6e => Some(Instruction::BitFlag(BitFlagInstruction::BIT_U3_R8E(
                U3::B5,
                R8E::HLP,
            ))),
            0x6f => Some(Instruction::BitFlag(BitFlagInstruction::BIT_U3_R8E(
                U3::B5,
                R8E::R8(R8::A),
            ))),

            0x70 => Some(Instruction::BitFlag(BitFlagInstruction::BIT_U3_R8E(
                U3::B6,
                R8E::R8(R8::B),
            ))),
            0x71 => Some(Instruction::BitFlag(BitFlagInstruction::BIT_U3_R8E(
                U3::B6,
                R8E::R8(R8::C),
            ))),
            0x72 => Some(Instruction::BitFlag(BitFlagInstruction::BIT_U3_R8E(
                U3::B6,
                R8E::R8(R8::D),
            ))),
            0x73 => Some(Instruction::BitFlag(BitFlagInstruction::BIT_U3_R8E(
                U3::B6,
                R8E::R8(R8::E),
            ))),
            0x74 => Some(Instruction::BitFlag(BitFlagInstruction::BIT_U3_R8E(
                U3::B6,
                R8E::R8(R8::H),
            ))),
            0x75 => Some(Instruction::BitFlag(BitFlagInstruction::BIT_U3_R8E(
                U3::B6,
                R8E::R8(R8::L),
            ))),
            0x76 => Some(Instruction::BitFlag(BitFlagInstruction::BIT_U3_R8E(
                U3::B6,
                R8E::HLP,
            ))),
            0x77 => Some(Instruction::BitFlag(BitFlagInstruction::BIT_U3_R8E(
                U3::B6,
                R8E::R8(R8::A),
            ))),
            0x78 => Some(Instruction::BitFlag(BitFlagInstruction::BIT_U3_R8E(
                U3::B7,
                R8E::R8(R8::B),
            ))),
            0x79 => Some(Instruction::BitFlag(BitFlagInstruction::BIT_U3_R8E(
                U3::B7,
                R8E::R8(R8::C),
            ))),
            0x7a => Some(Instruction::BitFlag(BitFlagInstruction::BIT_U3_R8E(
                U3::B7,
                R8E::R8(R8::D),
            ))),
            0x7b => Some(Instruction::BitFlag(BitFlagInstruction::BIT_U3_R8E(
                U3::B7,
                R8E::R8(R8::E),
            ))),
            0x7c => Some(Instruction::BitFlag(BitFlagInstruction::BIT_U3_R8E(
                U3::B7,
                R8E::R8(R8::H),
            ))),
            0x7d => Some(Instruction::BitFlag(BitFlagInstruction::BIT_U3_R8E(
                U3::B7,
                R8E::R8(R8::L),
            ))),
            0x7e => Some(Instruction::BitFlag(BitFlagInstruction::BIT_U3_R8E(
                U3::B7,
                R8E::HLP,
            ))),
            0x7f => Some(Instruction::BitFlag(BitFlagInstruction::BIT_U3_R8E(
                U3::B7,
                R8E::R8(R8::A),
            ))),

            0x80 => Some(Instruction::BitFlag(BitFlagInstruction::RES_U3_R8E(
                U3::B0,
                R8E::R8(R8::B),
            ))),
            0x81 => Some(Instruction::BitFlag(BitFlagInstruction::RES_U3_R8E(
                U3::B0,
                R8E::R8(R8::C),
            ))),
            0x82 => Some(Instruction::BitFlag(BitFlagInstruction::RES_U3_R8E(
                U3::B0,
                R8E::R8(R8::D),
            ))),
            0x83 => Some(Instruction::BitFlag(BitFlagInstruction::RES_U3_R8E(
                U3::B0,
                R8E::R8(R8::E),
            ))),
            0x84 => Some(Instruction::BitFlag(BitFlagInstruction::RES_U3_R8E(
                U3::B0,
                R8E::R8(R8::H),
            ))),
            0x85 => Some(Instruction::BitFlag(BitFlagInstruction::RES_U3_R8E(
                U3::B0,
                R8E::R8(R8::L),
            ))),
            0x86 => Some(Instruction::BitFlag(BitFlagInstruction::RES_U3_R8E(
                U3::B0,
                R8E::HLP,
            ))),
            0x87 => Some(Instruction::BitFlag(BitFlagInstruction::RES_U3_R8E(
                U3::B0,
                R8E::R8(R8::A),
            ))),
            0x88 => Some(Instruction::BitFlag(BitFlagInstruction::RES_U3_R8E(
                U3::B1,
                R8E::R8(R8::B),
            ))),
            0x89 => Some(Instruction::BitFlag(BitFlagInstruction::RES_U3_R8E(
                U3::B1,
                R8E::R8(R8::C),
            ))),
            0x8a => Some(Instruction::BitFlag(BitFlagInstruction::RES_U3_R8E(
                U3::B1,
                R8E::R8(R8::D),
            ))),
            0x8b => Some(Instruction::BitFlag(BitFlagInstruction::RES_U3_R8E(
                U3::B1,
                R8E::R8(R8::E),
            ))),
            0x8c => Some(Instruction::BitFlag(BitFlagInstruction::RES_U3_R8E(
                U3::B1,
                R8E::R8(R8::H),
            ))),
            0x8d => Some(Instruction::BitFlag(BitFlagInstruction::RES_U3_R8E(
                U3::B1,
                R8E::R8(R8::L),
            ))),
            0x8e => Some(Instruction::BitFlag(BitFlagInstruction::RES_U3_R8E(
                U3::B1,
                R8E::HLP,
            ))),
            0x8f => Some(Instruction::BitFlag(BitFlagInstruction::RES_U3_R8E(
                U3::B1,
                R8E::R8(R8::A),
            ))),

            0x90 => Some(Instruction::BitFlag(BitFlagInstruction::RES_U3_R8E(
                U3::B2,
                R8E::R8(R8::B),
            ))),
            0x91 => Some(Instruction::BitFlag(BitFlagInstruction::RES_U3_R8E(
                U3::B2,
                R8E::R8(R8::C),
            ))),
            0x92 => Some(Instruction::BitFlag(BitFlagInstruction::RES_U3_R8E(
                U3::B2,
                R8E::R8(R8::D),
            ))),
            0x93 => Some(Instruction::BitFlag(BitFlagInstruction::RES_U3_R8E(
                U3::B2,
                R8E::R8(R8::E),
            ))),
            0x94 => Some(Instruction::BitFlag(BitFlagInstruction::RES_U3_R8E(
                U3::B2,
                R8E::R8(R8::H),
            ))),
            0x95 => Some(Instruction::BitFlag(BitFlagInstruction::RES_U3_R8E(
                U3::B2,
                R8E::R8(R8::L),
            ))),
            0x96 => Some(Instruction::BitFlag(BitFlagInstruction::RES_U3_R8E(
                U3::B2,
                R8E::HLP,
            ))),
            0x97 => Some(Instruction::BitFlag(BitFlagInstruction::RES_U3_R8E(
                U3::B2,
                R8E::R8(R8::A),
            ))),
            0x98 => Some(Instruction::BitFlag(BitFlagInstruction::RES_U3_R8E(
                U3::B3,
                R8E::R8(R8::B),
            ))),
            0x99 => Some(Instruction::BitFlag(BitFlagInstruction::RES_U3_R8E(
                U3::B3,
                R8E::R8(R8::C),
            ))),
            0x9a => Some(Instruction::BitFlag(BitFlagInstruction::RES_U3_R8E(
                U3::B3,
                R8E::R8(R8::D),
            ))),
            0x9b => Some(Instruction::BitFlag(BitFlagInstruction::RES_U3_R8E(
                U3::B3,
                R8E::R8(R8::E),
            ))),
            0x9c => Some(Instruction::BitFlag(BitFlagInstruction::RES_U3_R8E(
                U3::B3,
                R8E::R8(R8::H),
            ))),
            0x9d => Some(Instruction::BitFlag(BitFlagInstruction::RES_U3_R8E(
                U3::B3,
                R8E::R8(R8::L),
            ))),
            0x9e => Some(Instruction::BitFlag(BitFlagInstruction::RES_U3_R8E(
                U3::B3,
                R8E::HLP,
            ))),
            0x9f => Some(Instruction::BitFlag(BitFlagInstruction::RES_U3_R8E(
                U3::B3,
                R8E::R8(R8::A),
            ))),

            0xa0 => Some(Instruction::BitFlag(BitFlagInstruction::RES_U3_R8E(
                U3::B4,
                R8E::R8(R8::B),
            ))),
            0xa1 => Some(Instruction::BitFlag(BitFlagInstruction::RES_U3_R8E(
                U3::B4,
                R8E::R8(R8::C),
            ))),
            0xa2 => Some(Instruction::BitFlag(BitFlagInstruction::RES_U3_R8E(
                U3::B4,
                R8E::R8(R8::D),
            ))),
            0xa3 => Some(Instruction::BitFlag(BitFlagInstruction::RES_U3_R8E(
                U3::B4,
                R8E::R8(R8::E),
            ))),
            0xa4 => Some(Instruction::BitFlag(BitFlagInstruction::RES_U3_R8E(
                U3::B4,
                R8E::R8(R8::H),
            ))),
            0xa5 => Some(Instruction::BitFlag(BitFlagInstruction::RES_U3_R8E(
                U3::B4,
                R8E::R8(R8::L),
            ))),
            0xa6 => Some(Instruction::BitFlag(BitFlagInstruction::RES_U3_R8E(
                U3::B4,
                R8E::HLP,
            ))),
            0xa7 => Some(Instruction::BitFlag(BitFlagInstruction::RES_U3_R8E(
                U3::B4,
                R8E::R8(R8::A),
            ))),
            0xa8 => Some(Instruction::BitFlag(BitFlagInstruction::RES_U3_R8E(
                U3::B5,
                R8E::R8(R8::B),
            ))),
            0xa9 => Some(Instruction::BitFlag(BitFlagInstruction::RES_U3_R8E(
                U3::B5,
                R8E::R8(R8::C),
            ))),
            0xaa => Some(Instruction::BitFlag(BitFlagInstruction::RES_U3_R8E(
                U3::B5,
                R8E::R8(R8::D),
            ))),
            0xab => Some(Instruction::BitFlag(BitFlagInstruction::RES_U3_R8E(
                U3::B5,
                R8E::R8(R8::E),
            ))),
            0xac => Some(Instruction::BitFlag(BitFlagInstruction::RES_U3_R8E(
                U3::B5,
                R8E::R8(R8::H),
            ))),
            0xad => Some(Instruction::BitFlag(BitFlagInstruction::RES_U3_R8E(
                U3::B5,
                R8E::R8(R8::L),
            ))),
            0xae => Some(Instruction::BitFlag(BitFlagInstruction::RES_U3_R8E(
                U3::B5,
                R8E::HLP,
            ))),
            0xaf => Some(Instruction::BitFlag(BitFlagInstruction::RES_U3_R8E(
                U3::B5,
                R8E::R8(R8::A),
            ))),

            0xb0 => Some(Instruction::BitFlag(BitFlagInstruction::RES_U3_R8E(
                U3::B6,
                R8E::R8(R8::B),
            ))),
            0xb1 => Some(Instruction::BitFlag(BitFlagInstruction::RES_U3_R8E(
                U3::B6,
                R8E::R8(R8::C),
            ))),
            0xb2 => Some(Instruction::BitFlag(BitFlagInstruction::RES_U3_R8E(
                U3::B6,
                R8E::R8(R8::D),
            ))),
            0xb3 => Some(Instruction::BitFlag(BitFlagInstruction::RES_U3_R8E(
                U3::B6,
                R8E::R8(R8::E),
            ))),
            0xb4 => Some(Instruction::BitFlag(BitFlagInstruction::RES_U3_R8E(
                U3::B6,
                R8E::R8(R8::H),
            ))),
            0xb5 => Some(Instruction::BitFlag(BitFlagInstruction::RES_U3_R8E(
                U3::B6,
                R8E::R8(R8::L),
            ))),
            0xb6 => Some(Instruction::BitFlag(BitFlagInstruction::RES_U3_R8E(
                U3::B6,
                R8E::HLP,
            ))),
            0xb7 => Some(Instruction::BitFlag(BitFlagInstruction::RES_U3_R8E(
                U3::B6,
                R8E::R8(R8::A),
            ))),
            0xb8 => Some(Instruction::BitFlag(BitFlagInstruction::RES_U3_R8E(
                U3::B7,
                R8E::R8(R8::B),
            ))),
            0xb9 => Some(Instruction::BitFlag(BitFlagInstruction::RES_U3_R8E(
                U3::B7,
                R8E::R8(R8::C),
            ))),
            0xba => Some(Instruction::BitFlag(BitFlagInstruction::RES_U3_R8E(
                U3::B7,
                R8E::R8(R8::D),
            ))),
            0xbb => Some(Instruction::BitFlag(BitFlagInstruction::RES_U3_R8E(
                U3::B7,
                R8E::R8(R8::E),
            ))),
            0xbc => Some(Instruction::BitFlag(BitFlagInstruction::RES_U3_R8E(
                U3::B7,
                R8E::R8(R8::H),
            ))),
            0xbd => Some(Instruction::BitFlag(BitFlagInstruction::RES_U3_R8E(
                U3::B7,
                R8E::R8(R8::L),
            ))),
            0xbe => Some(Instruction::BitFlag(BitFlagInstruction::RES_U3_R8E(
                U3::B7,
                R8E::HLP,
            ))),
            0xbf => Some(Instruction::BitFlag(BitFlagInstruction::RES_U3_R8E(
                U3::B7,
                R8E::R8(R8::A),
            ))),

            0xc0 => Some(Instruction::BitFlag(BitFlagInstruction::SET_U3_R8E(
                U3::B0,
                R8E::R8(R8::B),
            ))),
            0xc1 => Some(Instruction::BitFlag(BitFlagInstruction::SET_U3_R8E(
                U3::B0,
                R8E::R8(R8::C),
            ))),
            0xc2 => Some(Instruction::BitFlag(BitFlagInstruction::SET_U3_R8E(
                U3::B0,
                R8E::R8(R8::D),
            ))),
            0xc3 => Some(Instruction::BitFlag(BitFlagInstruction::SET_U3_R8E(
                U3::B0,
                R8E::R8(R8::E),
            ))),
            0xc4 => Some(Instruction::BitFlag(BitFlagInstruction::SET_U3_R8E(
                U3::B0,
                R8E::R8(R8::H),
            ))),
            0xc5 => Some(Instruction::BitFlag(BitFlagInstruction::SET_U3_R8E(
                U3::B0,
                R8E::R8(R8::L),
            ))),
            0xc6 => Some(Instruction::BitFlag(BitFlagInstruction::SET_U3_R8E(
                U3::B0,
                R8E::HLP,
            ))),
            0xc7 => Some(Instruction::BitFlag(BitFlagInstruction::SET_U3_R8E(
                U3::B0,
                R8E::R8(R8::A),
            ))),
            0xc8 => Some(Instruction::BitFlag(BitFlagInstruction::SET_U3_R8E(
                U3::B1,
                R8E::R8(R8::B),
            ))),
            0xc9 => Some(Instruction::BitFlag(BitFlagInstruction::SET_U3_R8E(
                U3::B1,
                R8E::R8(R8::C),
            ))),
            0xca => Some(Instruction::BitFlag(BitFlagInstruction::SET_U3_R8E(
                U3::B1,
                R8E::R8(R8::D),
            ))),
            0xcb => Some(Instruction::BitFlag(BitFlagInstruction::SET_U3_R8E(
                U3::B1,
                R8E::R8(R8::E),
            ))),
            0xcc => Some(Instruction::BitFlag(BitFlagInstruction::SET_U3_R8E(
                U3::B1,
                R8E::R8(R8::H),
            ))),
            0xcd => Some(Instruction::BitFlag(BitFlagInstruction::SET_U3_R8E(
                U3::B1,
                R8E::R8(R8::L),
            ))),
            0xce => Some(Instruction::BitFlag(BitFlagInstruction::SET_U3_R8E(
                U3::B1,
                R8E::HLP,
            ))),
            0xcf => Some(Instruction::BitFlag(BitFlagInstruction::SET_U3_R8E(
                U3::B1,
                R8E::R8(R8::A),
            ))),

            0xd0 => Some(Instruction::BitFlag(BitFlagInstruction::SET_U3_R8E(
                U3::B2,
                R8E::R8(R8::B),
            ))),
            0xd1 => Some(Instruction::BitFlag(BitFlagInstruction::SET_U3_R8E(
                U3::B2,
                R8E::R8(R8::C),
            ))),
            0xd2 => Some(Instruction::BitFlag(BitFlagInstruction::SET_U3_R8E(
                U3::B2,
                R8E::R8(R8::D),
            ))),
            0xd3 => Some(Instruction::BitFlag(BitFlagInstruction::SET_U3_R8E(
                U3::B2,
                R8E::R8(R8::E),
            ))),
            0xd4 => Some(Instruction::BitFlag(BitFlagInstruction::SET_U3_R8E(
                U3::B2,
                R8E::R8(R8::H),
            ))),
            0xd5 => Some(Instruction::BitFlag(BitFlagInstruction::SET_U3_R8E(
                U3::B2,
                R8E::R8(R8::L),
            ))),
            0xd6 => Some(Instruction::BitFlag(BitFlagInstruction::SET_U3_R8E(
                U3::B2,
                R8E::HLP,
            ))),
            0xd7 => Some(Instruction::BitFlag(BitFlagInstruction::SET_U3_R8E(
                U3::B2,
                R8E::R8(R8::A),
            ))),
            0xd8 => Some(Instruction::BitFlag(BitFlagInstruction::SET_U3_R8E(
                U3::B3,
                R8E::R8(R8::B),
            ))),
            0xd9 => Some(Instruction::BitFlag(BitFlagInstruction::SET_U3_R8E(
                U3::B3,
                R8E::R8(R8::C),
            ))),
            0xda => Some(Instruction::BitFlag(BitFlagInstruction::SET_U3_R8E(
                U3::B3,
                R8E::R8(R8::D),
            ))),
            0xdb => Some(Instruction::BitFlag(BitFlagInstruction::SET_U3_R8E(
                U3::B3,
                R8E::R8(R8::E),
            ))),
            0xdc => Some(Instruction::BitFlag(BitFlagInstruction::SET_U3_R8E(
                U3::B3,
                R8E::R8(R8::H),
            ))),
            0xdd => Some(Instruction::BitFlag(BitFlagInstruction::SET_U3_R8E(
                U3::B3,
                R8E::R8(R8::L),
            ))),
            0xde => Some(Instruction::BitFlag(BitFlagInstruction::SET_U3_R8E(
                U3::B3,
                R8E::HLP,
            ))),
            0xdf => Some(Instruction::BitFlag(BitFlagInstruction::SET_U3_R8E(
                U3::B3,
                R8E::R8(R8::A),
            ))),

            0xe0 => Some(Instruction::BitFlag(BitFlagInstruction::SET_U3_R8E(
                U3::B4,
                R8E::R8(R8::B),
            ))),
            0xe1 => Some(Instruction::BitFlag(BitFlagInstruction::SET_U3_R8E(
                U3::B4,
                R8E::R8(R8::C),
            ))),
            0xe2 => Some(Instruction::BitFlag(BitFlagInstruction::SET_U3_R8E(
                U3::B4,
                R8E::R8(R8::D),
            ))),
            0xe3 => Some(Instruction::BitFlag(BitFlagInstruction::SET_U3_R8E(
                U3::B4,
                R8E::R8(R8::E),
            ))),
            0xe4 => Some(Instruction::BitFlag(BitFlagInstruction::SET_U3_R8E(
                U3::B4,
                R8E::R8(R8::H),
            ))),
            0xe5 => Some(Instruction::BitFlag(BitFlagInstruction::SET_U3_R8E(
                U3::B4,
                R8E::R8(R8::L),
            ))),
            0xe6 => Some(Instruction::BitFlag(BitFlagInstruction::SET_U3_R8E(
                U3::B4,
                R8E::HLP,
            ))),
            0xe7 => Some(Instruction::BitFlag(BitFlagInstruction::SET_U3_R8E(
                U3::B4,
                R8E::R8(R8::A),
            ))),
            0xe8 => Some(Instruction::BitFlag(BitFlagInstruction::SET_U3_R8E(
                U3::B5,
                R8E::R8(R8::B),
            ))),
            0xe9 => Some(Instruction::BitFlag(BitFlagInstruction::SET_U3_R8E(
                U3::B5,
                R8E::R8(R8::C),
            ))),
            0xea => Some(Instruction::BitFlag(BitFlagInstruction::SET_U3_R8E(
                U3::B5,
                R8E::R8(R8::D),
            ))),
            0xeb => Some(Instruction::BitFlag(BitFlagInstruction::SET_U3_R8E(
                U3::B5,
                R8E::R8(R8::E),
            ))),
            0xec => Some(Instruction::BitFlag(BitFlagInstruction::SET_U3_R8E(
                U3::B5,
                R8E::R8(R8::H),
            ))),
            0xed => Some(Instruction::BitFlag(BitFlagInstruction::SET_U3_R8E(
                U3::B5,
                R8E::R8(R8::L),
            ))),
            0xee => Some(Instruction::BitFlag(BitFlagInstruction::SET_U3_R8E(
                U3::B5,
                R8E::HLP,
            ))),
            0xef => Some(Instruction::BitFlag(BitFlagInstruction::SET_U3_R8E(
                U3::B5,
                R8E::R8(R8::A),
            ))),

            0xf0 => Some(Instruction::BitFlag(BitFlagInstruction::SET_U3_R8E(
                U3::B6,
                R8E::R8(R8::B),
            ))),
            0xf1 => Some(Instruction::BitFlag(BitFlagInstruction::SET_U3_R8E(
                U3::B6,
                R8E::R8(R8::C),
            ))),
            0xf2 => Some(Instruction::BitFlag(BitFlagInstruction::SET_U3_R8E(
                U3::B6,
                R8E::R8(R8::D),
            ))),
            0xf3 => Some(Instruction::BitFlag(BitFlagInstruction::SET_U3_R8E(
                U3::B6,
                R8E::R8(R8::E),
            ))),
            0xf4 => Some(Instruction::BitFlag(BitFlagInstruction::SET_U3_R8E(
                U3::B6,
                R8E::R8(R8::H),
            ))),
            0xf5 => Some(Instruction::BitFlag(BitFlagInstruction::SET_U3_R8E(
                U3::B6,
                R8E::R8(R8::L),
            ))),
            0xf6 => Some(Instruction::BitFlag(BitFlagInstruction::SET_U3_R8E(
                U3::B6,
                R8E::HLP,
            ))),
            0xf7 => Some(Instruction::BitFlag(BitFlagInstruction::SET_U3_R8E(
                U3::B6,
                R8E::R8(R8::A),
            ))),
            0xf8 => Some(Instruction::BitFlag(BitFlagInstruction::SET_U3_R8E(
                U3::B7,
                R8E::R8(R8::B),
            ))),
            0xf9 => Some(Instruction::BitFlag(BitFlagInstruction::SET_U3_R8E(
                U3::B7,
                R8E::R8(R8::C),
            ))),
            0xfa => Some(Instruction::BitFlag(BitFlagInstruction::SET_U3_R8E(
                U3::B7,
                R8E::R8(R8::D),
            ))),
            0xfb => Some(Instruction::BitFlag(BitFlagInstruction::SET_U3_R8E(
                U3::B7,
                R8E::R8(R8::E),
            ))),
            0xfc => Some(Instruction::BitFlag(BitFlagInstruction::SET_U3_R8E(
                U3::B7,
                R8E::R8(R8::H),
            ))),
            0xfd => Some(Instruction::BitFlag(BitFlagInstruction::SET_U3_R8E(
                U3::B7,
                R8E::R8(R8::L),
            ))),
            0xfe => Some(Instruction::BitFlag(BitFlagInstruction::SET_U3_R8E(
                U3::B7,
                R8E::HLP,
            ))),
            0xff => Some(Instruction::BitFlag(BitFlagInstruction::SET_U3_R8E(
                U3::B7,
                R8E::R8(R8::A),
            ))),
        }
    }
}
