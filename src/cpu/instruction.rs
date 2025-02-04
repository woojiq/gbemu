/**
List of abbreviations used in this document:
<https://rgbds.gbdev.io/docs/v0.9.0/gbz80.7#LEGEND>

Inspired by <https://github.com/rylev/DMG-01/blob/00bed9baedab5548d63d646f60acb7af4b3e3658/lib-dmg-01/src/cpu/instruction.rs>
*/
pub enum Instruction {
    // Arithmetic instruction
    /// Add the `ArithmeticTarget` value to register A.
    ADD(ArithmeticTarget),
    ADC(ArithmeticTarget),
    SUB(ArithmeticTarget),
    SBC(ArithmeticTarget),
    CP(ArithmeticTarget),

    ADDHL(ADDHLTarget),

    INC(IncDecTarget),
    DEC(IncDecTarget),

    // Bit flag instruction
    BIT(BitPosition, PrefixTarget),
    RES(BitPosition, PrefixTarget),
    SET(BitPosition, PrefixTarget),

    // Bitwise logic instructions
    CPL,
    AND(ArithmeticTarget),
    XOR(ArithmeticTarget),
    OR(ArithmeticTarget),

    // Carry flag instructions
    SCF,
    CCF,

    // Load instructions
    Load(LoadType),

    // Bit shift instructions
    RL(PrefixTarget),
    RLA,
    RLC(PrefixTarget),
    RLCA,

    RR(PrefixTarget),
    RRA,
    RRC(PrefixTarget),
    RRCA,

    SLA(PrefixTarget),
    SRA(PrefixTarget),
    SRL(PrefixTarget),

    SWAP(PrefixTarget),

    // Jumps and subroutine instructions
    JR(JumpTest),
    JP(JumpTest),
    JPHLP,

    CALL(JumpTest),
    RET(JumpTest),
    RETI,

    RST(VEC),

    // Stack manipulation instructions
    ADDSP,

    POP(StackTarget),
    PUSH(StackTarget),

    // Interrupt related instructions
    DI,
    EI,
    HALT,

    // Miscellaneous instructions
    DAA,
    NOP,
    STOP,
}

pub enum LoadType {
    Byte(LoadByteTarget, LoadByteSource),
    /// Copy the value U16 into register R16.
    Word(LoadWordTarget),
    AFromIndirect(IndirectTarget),
    IndirectFromA(IndirectTarget),
    /// Copy (SP&$FF) at address U16 and (SP>>8) at address U16+1.
    IndirectFromSP,
    SPFromHL,
    /// Add the signed value e8 to SP and copy the result in HL.
    HLFromSPN,
}

pub enum IndirectTarget {
    /// Address $FF00 + C(register).
    C,
    /// Address $FF00 + U8(next byte).
    U8,
    /// Address U16(next byte).
    U16,
    BCP,
    DEP,
    HLI,
    HLD,
}

pub enum LoadByteTarget {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    HLP,
}

pub enum LoadByteSource {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    HLP,
    U8,
}

pub enum LoadWordTarget {
    BC,
    DE,
    HL,
    SP,
}

pub enum ADDHLTarget {
    BC,
    DE,
    HL,
    SP,
}

pub enum IncDecTarget {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    BC,
    DE,
    HL,
    HLP,
    SP,
}

pub enum StackTarget {
    AF,
    BC,
    DE,
    HL,
}

pub enum ArithmeticTarget {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    HLP,
    U8,
}

pub enum PrefixTarget {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    HLP,
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

/// A condition code.
pub enum JumpTest {
    Zero,
    NotZero,
    Carry,
    NotCarry,
    Always,
}

/// 3-bit unsigned bit index (0 to 7).
pub enum BitPosition {
    B0 = 0,
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
            0x00 => Some(Instruction::NOP),
            0x01 => Some(Instruction::Load(LoadType::Word(LoadWordTarget::BC))),
            0x02 => Some(Instruction::Load(LoadType::IndirectFromA(
                IndirectTarget::BCP,
            ))),
            0x03 => Some(Instruction::INC(IncDecTarget::BC)),
            0x04 => Some(Instruction::INC(IncDecTarget::B)),
            0x05 => Some(Instruction::DEC(IncDecTarget::B)),
            0x06 => Some(Instruction::Load(LoadType::Byte(
                LoadByteTarget::B,
                LoadByteSource::U8,
            ))),
            0x07 => Some(Instruction::RLCA),
            0x08 => Some(Instruction::Load(LoadType::IndirectFromSP)),
            0x09 => Some(Instruction::ADDHL(ADDHLTarget::BC)),
            0x0a => Some(Instruction::Load(LoadType::AFromIndirect(
                IndirectTarget::BCP,
            ))),
            0x0b => Some(Instruction::DEC(IncDecTarget::BC)),
            0x0c => Some(Instruction::INC(IncDecTarget::C)),
            0x0d => Some(Instruction::DEC(IncDecTarget::C)),
            0x0e => Some(Instruction::Load(LoadType::Byte(
                LoadByteTarget::C,
                LoadByteSource::U8,
            ))),
            0x0f => Some(Instruction::RRCA),

            0x10 => Some(Instruction::STOP),
            0x11 => Some(Instruction::Load(LoadType::Word(LoadWordTarget::DE))),
            0x12 => Some(Instruction::Load(LoadType::IndirectFromA(
                IndirectTarget::DEP,
            ))),
            0x13 => Some(Instruction::INC(IncDecTarget::DE)),
            0x14 => Some(Instruction::INC(IncDecTarget::D)),
            0x15 => Some(Instruction::DEC(IncDecTarget::D)),
            0x16 => Some(Instruction::Load(LoadType::Byte(
                LoadByteTarget::D,
                LoadByteSource::U8,
            ))),
            0x17 => Some(Instruction::RLA),
            0x18 => Some(Instruction::JR(JumpTest::Always)),
            0x19 => Some(Instruction::ADDHL(ADDHLTarget::DE)),
            0x1a => Some(Instruction::Load(LoadType::AFromIndirect(
                IndirectTarget::DEP,
            ))),
            0x1b => Some(Instruction::DEC(IncDecTarget::DE)),
            0x1c => Some(Instruction::INC(IncDecTarget::E)),
            0x1d => Some(Instruction::DEC(IncDecTarget::E)),
            0x1e => Some(Instruction::Load(LoadType::Byte(
                LoadByteTarget::E,
                LoadByteSource::U8,
            ))),
            0x1f => Some(Instruction::RRA),

            0x20 => Some(Instruction::JR(JumpTest::NotZero)),
            0x21 => Some(Instruction::Load(LoadType::Word(LoadWordTarget::HL))),
            0x22 => Some(Instruction::Load(LoadType::IndirectFromA(
                IndirectTarget::HLI,
            ))),
            0x23 => Some(Instruction::INC(IncDecTarget::HL)),
            0x24 => Some(Instruction::INC(IncDecTarget::H)),
            0x25 => Some(Instruction::DEC(IncDecTarget::H)),
            0x26 => Some(Instruction::Load(LoadType::Byte(
                LoadByteTarget::H,
                LoadByteSource::U8,
            ))),
            0x27 => Some(Instruction::DAA),
            0x28 => Some(Instruction::JR(JumpTest::Zero)),
            0x29 => Some(Instruction::ADDHL(ADDHLTarget::HL)),
            0x2a => Some(Instruction::Load(LoadType::AFromIndirect(
                IndirectTarget::HLI,
            ))),
            0x2b => Some(Instruction::DEC(IncDecTarget::HL)),
            0x2c => Some(Instruction::INC(IncDecTarget::L)),
            0x2d => Some(Instruction::DEC(IncDecTarget::L)),
            0x2e => Some(Instruction::Load(LoadType::Byte(
                LoadByteTarget::L,
                LoadByteSource::U8,
            ))),
            0x2f => Some(Instruction::CPL),

            0x30 => Some(Instruction::JR(JumpTest::NotCarry)),
            0x31 => Some(Instruction::Load(LoadType::Word(LoadWordTarget::SP))),
            0x32 => Some(Instruction::Load(LoadType::IndirectFromA(
                IndirectTarget::HLD,
            ))),
            0x33 => Some(Instruction::INC(IncDecTarget::SP)),
            0x34 => Some(Instruction::INC(IncDecTarget::HLP)),
            0x35 => Some(Instruction::DEC(IncDecTarget::HLP)),
            0x36 => Some(Instruction::Load(LoadType::Byte(
                LoadByteTarget::HLP,
                LoadByteSource::U8,
            ))),
            0x37 => Some(Instruction::SCF),
            0x38 => Some(Instruction::JR(JumpTest::Carry)),
            0x39 => Some(Instruction::ADDHL(ADDHLTarget::SP)),
            0x3a => Some(Instruction::Load(LoadType::AFromIndirect(
                IndirectTarget::HLD,
            ))),
            0x3b => Some(Instruction::DEC(IncDecTarget::SP)),
            0x3c => Some(Instruction::INC(IncDecTarget::A)),
            0x3d => Some(Instruction::DEC(IncDecTarget::A)),
            0x3e => Some(Instruction::Load(LoadType::Byte(
                LoadByteTarget::A,
                LoadByteSource::U8,
            ))),
            0x3f => Some(Instruction::CCF),

            0x40 => Some(Instruction::Load(LoadType::Byte(
                LoadByteTarget::B,
                LoadByteSource::B,
            ))),
            0x41 => Some(Instruction::Load(LoadType::Byte(
                LoadByteTarget::B,
                LoadByteSource::C,
            ))),
            0x42 => Some(Instruction::Load(LoadType::Byte(
                LoadByteTarget::B,
                LoadByteSource::D,
            ))),
            0x43 => Some(Instruction::Load(LoadType::Byte(
                LoadByteTarget::B,
                LoadByteSource::E,
            ))),
            0x44 => Some(Instruction::Load(LoadType::Byte(
                LoadByteTarget::B,
                LoadByteSource::H,
            ))),
            0x45 => Some(Instruction::Load(LoadType::Byte(
                LoadByteTarget::B,
                LoadByteSource::L,
            ))),
            0x46 => Some(Instruction::Load(LoadType::Byte(
                LoadByteTarget::B,
                LoadByteSource::HLP,
            ))),
            0x47 => Some(Instruction::Load(LoadType::Byte(
                LoadByteTarget::B,
                LoadByteSource::A,
            ))),
            0x48 => Some(Instruction::Load(LoadType::Byte(
                LoadByteTarget::C,
                LoadByteSource::B,
            ))),
            0x49 => Some(Instruction::Load(LoadType::Byte(
                LoadByteTarget::C,
                LoadByteSource::C,
            ))),
            0x4a => Some(Instruction::Load(LoadType::Byte(
                LoadByteTarget::C,
                LoadByteSource::D,
            ))),
            0x4b => Some(Instruction::Load(LoadType::Byte(
                LoadByteTarget::C,
                LoadByteSource::E,
            ))),
            0x4c => Some(Instruction::Load(LoadType::Byte(
                LoadByteTarget::C,
                LoadByteSource::H,
            ))),
            0x4d => Some(Instruction::Load(LoadType::Byte(
                LoadByteTarget::C,
                LoadByteSource::L,
            ))),
            0x4e => Some(Instruction::Load(LoadType::Byte(
                LoadByteTarget::C,
                LoadByteSource::HLP,
            ))),
            0x4f => Some(Instruction::Load(LoadType::Byte(
                LoadByteTarget::C,
                LoadByteSource::A,
            ))),

            0x50 => Some(Instruction::Load(LoadType::Byte(
                LoadByteTarget::D,
                LoadByteSource::B,
            ))),
            0x51 => Some(Instruction::Load(LoadType::Byte(
                LoadByteTarget::D,
                LoadByteSource::C,
            ))),
            0x52 => Some(Instruction::Load(LoadType::Byte(
                LoadByteTarget::D,
                LoadByteSource::D,
            ))),
            0x53 => Some(Instruction::Load(LoadType::Byte(
                LoadByteTarget::D,
                LoadByteSource::E,
            ))),
            0x54 => Some(Instruction::Load(LoadType::Byte(
                LoadByteTarget::D,
                LoadByteSource::H,
            ))),
            0x55 => Some(Instruction::Load(LoadType::Byte(
                LoadByteTarget::D,
                LoadByteSource::L,
            ))),
            0x56 => Some(Instruction::Load(LoadType::Byte(
                LoadByteTarget::D,
                LoadByteSource::HLP,
            ))),
            0x57 => Some(Instruction::Load(LoadType::Byte(
                LoadByteTarget::D,
                LoadByteSource::A,
            ))),
            0x58 => Some(Instruction::Load(LoadType::Byte(
                LoadByteTarget::E,
                LoadByteSource::B,
            ))),
            0x59 => Some(Instruction::Load(LoadType::Byte(
                LoadByteTarget::E,
                LoadByteSource::C,
            ))),
            0x5a => Some(Instruction::Load(LoadType::Byte(
                LoadByteTarget::E,
                LoadByteSource::D,
            ))),
            0x5b => Some(Instruction::Load(LoadType::Byte(
                LoadByteTarget::E,
                LoadByteSource::E,
            ))),
            0x5c => Some(Instruction::Load(LoadType::Byte(
                LoadByteTarget::E,
                LoadByteSource::H,
            ))),
            0x5d => Some(Instruction::Load(LoadType::Byte(
                LoadByteTarget::E,
                LoadByteSource::L,
            ))),
            0x5e => Some(Instruction::Load(LoadType::Byte(
                LoadByteTarget::E,
                LoadByteSource::HLP,
            ))),
            0x5f => Some(Instruction::Load(LoadType::Byte(
                LoadByteTarget::E,
                LoadByteSource::A,
            ))),

            0x60 => Some(Instruction::Load(LoadType::Byte(
                LoadByteTarget::H,
                LoadByteSource::B,
            ))),
            0x61 => Some(Instruction::Load(LoadType::Byte(
                LoadByteTarget::H,
                LoadByteSource::C,
            ))),
            0x62 => Some(Instruction::Load(LoadType::Byte(
                LoadByteTarget::H,
                LoadByteSource::D,
            ))),
            0x63 => Some(Instruction::Load(LoadType::Byte(
                LoadByteTarget::H,
                LoadByteSource::E,
            ))),
            0x64 => Some(Instruction::Load(LoadType::Byte(
                LoadByteTarget::H,
                LoadByteSource::H,
            ))),
            0x65 => Some(Instruction::Load(LoadType::Byte(
                LoadByteTarget::H,
                LoadByteSource::L,
            ))),
            0x66 => Some(Instruction::Load(LoadType::Byte(
                LoadByteTarget::H,
                LoadByteSource::HLP,
            ))),
            0x67 => Some(Instruction::Load(LoadType::Byte(
                LoadByteTarget::H,
                LoadByteSource::A,
            ))),
            0x68 => Some(Instruction::Load(LoadType::Byte(
                LoadByteTarget::L,
                LoadByteSource::B,
            ))),
            0x69 => Some(Instruction::Load(LoadType::Byte(
                LoadByteTarget::L,
                LoadByteSource::C,
            ))),
            0x6a => Some(Instruction::Load(LoadType::Byte(
                LoadByteTarget::L,
                LoadByteSource::D,
            ))),
            0x6b => Some(Instruction::Load(LoadType::Byte(
                LoadByteTarget::L,
                LoadByteSource::E,
            ))),
            0x6c => Some(Instruction::Load(LoadType::Byte(
                LoadByteTarget::L,
                LoadByteSource::H,
            ))),
            0x6d => Some(Instruction::Load(LoadType::Byte(
                LoadByteTarget::L,
                LoadByteSource::L,
            ))),
            0x6e => Some(Instruction::Load(LoadType::Byte(
                LoadByteTarget::L,
                LoadByteSource::HLP,
            ))),
            0x6f => Some(Instruction::Load(LoadType::Byte(
                LoadByteTarget::L,
                LoadByteSource::A,
            ))),

            0x70 => Some(Instruction::Load(LoadType::Byte(
                LoadByteTarget::HLP,
                LoadByteSource::B,
            ))),
            0x71 => Some(Instruction::Load(LoadType::Byte(
                LoadByteTarget::HLP,
                LoadByteSource::C,
            ))),
            0x72 => Some(Instruction::Load(LoadType::Byte(
                LoadByteTarget::HLP,
                LoadByteSource::D,
            ))),
            0x73 => Some(Instruction::Load(LoadType::Byte(
                LoadByteTarget::HLP,
                LoadByteSource::E,
            ))),
            0x74 => Some(Instruction::Load(LoadType::Byte(
                LoadByteTarget::HLP,
                LoadByteSource::H,
            ))),
            0x75 => Some(Instruction::Load(LoadType::Byte(
                LoadByteTarget::HLP,
                LoadByteSource::L,
            ))),
            0x76 => Some(Instruction::HALT),
            0x77 => Some(Instruction::Load(LoadType::Byte(
                LoadByteTarget::HLP,
                LoadByteSource::A,
            ))),
            0x78 => Some(Instruction::Load(LoadType::Byte(
                LoadByteTarget::A,
                LoadByteSource::B,
            ))),
            0x79 => Some(Instruction::Load(LoadType::Byte(
                LoadByteTarget::A,
                LoadByteSource::C,
            ))),
            0x7a => Some(Instruction::Load(LoadType::Byte(
                LoadByteTarget::A,
                LoadByteSource::D,
            ))),
            0x7b => Some(Instruction::Load(LoadType::Byte(
                LoadByteTarget::A,
                LoadByteSource::E,
            ))),
            0x7c => Some(Instruction::Load(LoadType::Byte(
                LoadByteTarget::A,
                LoadByteSource::H,
            ))),
            0x7d => Some(Instruction::Load(LoadType::Byte(
                LoadByteTarget::A,
                LoadByteSource::L,
            ))),
            0x7e => Some(Instruction::Load(LoadType::Byte(
                LoadByteTarget::A,
                LoadByteSource::HLP,
            ))),
            0x7f => Some(Instruction::Load(LoadType::Byte(
                LoadByteTarget::A,
                LoadByteSource::A,
            ))),

            0x80 => Some(Instruction::ADD(ArithmeticTarget::B)),
            0x81 => Some(Instruction::ADD(ArithmeticTarget::C)),
            0x82 => Some(Instruction::ADD(ArithmeticTarget::D)),
            0x83 => Some(Instruction::ADD(ArithmeticTarget::E)),
            0x84 => Some(Instruction::ADD(ArithmeticTarget::H)),
            0x85 => Some(Instruction::ADD(ArithmeticTarget::L)),
            0x86 => Some(Instruction::ADD(ArithmeticTarget::HLP)),
            0x87 => Some(Instruction::ADD(ArithmeticTarget::A)),
            0x88 => Some(Instruction::ADC(ArithmeticTarget::B)),
            0x89 => Some(Instruction::ADC(ArithmeticTarget::C)),
            0x8a => Some(Instruction::ADC(ArithmeticTarget::D)),
            0x8b => Some(Instruction::ADC(ArithmeticTarget::E)),
            0x8c => Some(Instruction::ADC(ArithmeticTarget::H)),
            0x8d => Some(Instruction::ADC(ArithmeticTarget::L)),
            0x8e => Some(Instruction::ADC(ArithmeticTarget::HLP)),
            0x8f => Some(Instruction::ADC(ArithmeticTarget::A)),

            0x90 => Some(Instruction::SUB(ArithmeticTarget::B)),
            0x91 => Some(Instruction::SUB(ArithmeticTarget::C)),
            0x92 => Some(Instruction::SUB(ArithmeticTarget::D)),
            0x93 => Some(Instruction::SUB(ArithmeticTarget::E)),
            0x94 => Some(Instruction::SUB(ArithmeticTarget::H)),
            0x95 => Some(Instruction::SUB(ArithmeticTarget::L)),
            0x96 => Some(Instruction::SUB(ArithmeticTarget::HLP)),
            0x97 => Some(Instruction::SUB(ArithmeticTarget::A)),
            0x98 => Some(Instruction::SBC(ArithmeticTarget::B)),
            0x99 => Some(Instruction::SBC(ArithmeticTarget::C)),
            0x9a => Some(Instruction::SBC(ArithmeticTarget::D)),
            0x9b => Some(Instruction::SBC(ArithmeticTarget::E)),
            0x9c => Some(Instruction::SBC(ArithmeticTarget::H)),
            0x9d => Some(Instruction::SBC(ArithmeticTarget::L)),
            0x9e => Some(Instruction::SBC(ArithmeticTarget::HLP)),
            0x9f => Some(Instruction::SBC(ArithmeticTarget::A)),

            0xa0 => Some(Instruction::AND(ArithmeticTarget::B)),
            0xa1 => Some(Instruction::AND(ArithmeticTarget::C)),
            0xa2 => Some(Instruction::AND(ArithmeticTarget::D)),
            0xa3 => Some(Instruction::AND(ArithmeticTarget::E)),
            0xa4 => Some(Instruction::AND(ArithmeticTarget::H)),
            0xa5 => Some(Instruction::AND(ArithmeticTarget::L)),
            0xa6 => Some(Instruction::AND(ArithmeticTarget::HLP)),
            0xa7 => Some(Instruction::AND(ArithmeticTarget::A)),
            0xa8 => Some(Instruction::XOR(ArithmeticTarget::B)),
            0xa9 => Some(Instruction::XOR(ArithmeticTarget::C)),
            0xaa => Some(Instruction::XOR(ArithmeticTarget::D)),
            0xab => Some(Instruction::XOR(ArithmeticTarget::E)),
            0xac => Some(Instruction::XOR(ArithmeticTarget::H)),
            0xad => Some(Instruction::XOR(ArithmeticTarget::L)),
            0xae => Some(Instruction::XOR(ArithmeticTarget::HLP)),
            0xaf => Some(Instruction::XOR(ArithmeticTarget::A)),

            0xb0 => Some(Instruction::OR(ArithmeticTarget::B)),
            0xb1 => Some(Instruction::OR(ArithmeticTarget::C)),
            0xb2 => Some(Instruction::OR(ArithmeticTarget::D)),
            0xb3 => Some(Instruction::OR(ArithmeticTarget::E)),
            0xb4 => Some(Instruction::OR(ArithmeticTarget::H)),
            0xb5 => Some(Instruction::OR(ArithmeticTarget::L)),
            0xb6 => Some(Instruction::OR(ArithmeticTarget::HLP)),
            0xb7 => Some(Instruction::OR(ArithmeticTarget::A)),
            0xb8 => Some(Instruction::CP(ArithmeticTarget::B)),
            0xb9 => Some(Instruction::CP(ArithmeticTarget::C)),
            0xba => Some(Instruction::CP(ArithmeticTarget::D)),
            0xbb => Some(Instruction::CP(ArithmeticTarget::E)),
            0xbc => Some(Instruction::CP(ArithmeticTarget::H)),
            0xbd => Some(Instruction::CP(ArithmeticTarget::L)),
            0xbe => Some(Instruction::CP(ArithmeticTarget::HLP)),
            0xbf => Some(Instruction::CP(ArithmeticTarget::A)),

            0xc0 => Some(Instruction::RET(JumpTest::NotZero)),
            0xc1 => Some(Instruction::POP(StackTarget::BC)),
            0xc2 => Some(Instruction::JP(JumpTest::NotZero)),
            0xc3 => Some(Instruction::JP(JumpTest::Always)),
            0xc4 => Some(Instruction::CALL(JumpTest::NotZero)),
            0xc5 => Some(Instruction::PUSH(StackTarget::BC)),
            0xc6 => Some(Instruction::ADD(ArithmeticTarget::U8)),
            0xc7 => Some(Instruction::RST(VEC::X00)),
            0xc8 => Some(Instruction::RET(JumpTest::Zero)),
            0xc9 => Some(Instruction::RET(JumpTest::Always)),
            0xca => Some(Instruction::JP(JumpTest::Zero)),
            0xcb => panic!("Instruction prefix 0xCB in `from_byte_not_prefixed`."),
            0xcc => Some(Instruction::CALL(JumpTest::Zero)),
            0xcd => Some(Instruction::CALL(JumpTest::Always)),
            0xce => Some(Instruction::ADC(ArithmeticTarget::U8)),
            0xcf => Some(Instruction::RST(VEC::X08)),

            0xd0 => Some(Instruction::RET(JumpTest::NotCarry)),
            0xd1 => Some(Instruction::POP(StackTarget::DE)),
            0xd2 => Some(Instruction::JP(JumpTest::NotCarry)),
            0xd4 => Some(Instruction::CALL(JumpTest::NotCarry)),
            0xd5 => Some(Instruction::PUSH(StackTarget::DE)),
            0xd6 => Some(Instruction::SUB(ArithmeticTarget::U8)),
            0xd7 => Some(Instruction::RST(VEC::X10)),
            0xd8 => Some(Instruction::RET(JumpTest::Carry)),
            0xd9 => Some(Instruction::RETI),
            0xda => Some(Instruction::JP(JumpTest::Carry)),
            0xdc => Some(Instruction::CALL(JumpTest::Carry)),
            0xde => Some(Instruction::SBC(ArithmeticTarget::U8)),
            0xdf => Some(Instruction::RST(VEC::X18)),

            0xe0 => Some(Instruction::Load(LoadType::IndirectFromA(
                IndirectTarget::U8,
            ))),
            0xe1 => Some(Instruction::POP(StackTarget::HL)),
            0xe2 => Some(Instruction::Load(LoadType::IndirectFromA(
                IndirectTarget::C,
            ))),
            0xe5 => Some(Instruction::PUSH(StackTarget::HL)),
            0xe6 => Some(Instruction::AND(ArithmeticTarget::U8)),
            0xe7 => Some(Instruction::RST(VEC::X20)),
            0xe8 => Some(Instruction::ADDSP),
            0xe9 => Some(Instruction::JPHLP),
            0xea => Some(Instruction::Load(LoadType::IndirectFromA(
                IndirectTarget::U16,
            ))),
            0xee => Some(Instruction::XOR(ArithmeticTarget::U8)),
            0xef => Some(Instruction::RST(VEC::X28)),

            0xf0 => Some(Instruction::Load(LoadType::AFromIndirect(
                IndirectTarget::U8,
            ))),
            0xf1 => Some(Instruction::POP(StackTarget::AF)),
            0xf2 => Some(Instruction::Load(LoadType::AFromIndirect(
                IndirectTarget::C,
            ))),
            0xf3 => Some(Instruction::DI),
            0xf5 => Some(Instruction::PUSH(StackTarget::AF)),
            0xf6 => Some(Instruction::OR(ArithmeticTarget::U8)),
            0xf7 => Some(Instruction::RST(VEC::X30)),
            0xf8 => Some(Instruction::Load(LoadType::HLFromSPN)),
            0xf9 => Some(Instruction::Load(LoadType::SPFromHL)),
            0xfa => Some(Instruction::Load(LoadType::AFromIndirect(
                IndirectTarget::U16,
            ))),
            0xfb => Some(Instruction::EI),
            0xfe => Some(Instruction::CP(ArithmeticTarget::U8)),
            0xff => Some(Instruction::RST(VEC::X38)),

            _ => None,
        }
    }

    fn from_byte_prefixed(byte: u8) -> Option<Self> {
        match byte {
            0x00 => Some(Instruction::RLC(PrefixTarget::B)),
            0x01 => Some(Instruction::RLC(PrefixTarget::C)),
            0x02 => Some(Instruction::RLC(PrefixTarget::D)),
            0x03 => Some(Instruction::RLC(PrefixTarget::E)),
            0x04 => Some(Instruction::RLC(PrefixTarget::H)),
            0x05 => Some(Instruction::RLC(PrefixTarget::L)),
            0x06 => Some(Instruction::RLC(PrefixTarget::HLP)),
            0x07 => Some(Instruction::RLC(PrefixTarget::A)),
            0x08 => Some(Instruction::RRC(PrefixTarget::B)),
            0x09 => Some(Instruction::RRC(PrefixTarget::C)),
            0x0a => Some(Instruction::RRC(PrefixTarget::D)),
            0x0b => Some(Instruction::RRC(PrefixTarget::E)),
            0x0c => Some(Instruction::RRC(PrefixTarget::H)),
            0x0d => Some(Instruction::RRC(PrefixTarget::L)),
            0x0e => Some(Instruction::RRC(PrefixTarget::HLP)),
            0x0f => Some(Instruction::RRC(PrefixTarget::A)),

            0x10 => Some(Instruction::RL(PrefixTarget::B)),
            0x11 => Some(Instruction::RL(PrefixTarget::C)),
            0x12 => Some(Instruction::RL(PrefixTarget::D)),
            0x13 => Some(Instruction::RL(PrefixTarget::E)),
            0x14 => Some(Instruction::RL(PrefixTarget::H)),
            0x15 => Some(Instruction::RL(PrefixTarget::L)),
            0x16 => Some(Instruction::RL(PrefixTarget::HLP)),
            0x17 => Some(Instruction::RL(PrefixTarget::A)),
            0x18 => Some(Instruction::RR(PrefixTarget::B)),
            0x19 => Some(Instruction::RR(PrefixTarget::C)),
            0x1a => Some(Instruction::RR(PrefixTarget::D)),
            0x1b => Some(Instruction::RR(PrefixTarget::E)),
            0x1c => Some(Instruction::RR(PrefixTarget::H)),
            0x1d => Some(Instruction::RR(PrefixTarget::L)),
            0x1e => Some(Instruction::RR(PrefixTarget::HLP)),
            0x1f => Some(Instruction::RR(PrefixTarget::A)),

            0x20 => Some(Instruction::SLA(PrefixTarget::B)),
            0x21 => Some(Instruction::SLA(PrefixTarget::C)),
            0x22 => Some(Instruction::SLA(PrefixTarget::D)),
            0x23 => Some(Instruction::SLA(PrefixTarget::E)),
            0x24 => Some(Instruction::SLA(PrefixTarget::H)),
            0x25 => Some(Instruction::SLA(PrefixTarget::L)),
            0x26 => Some(Instruction::SLA(PrefixTarget::HLP)),
            0x27 => Some(Instruction::SLA(PrefixTarget::A)),
            0x28 => Some(Instruction::SRA(PrefixTarget::B)),
            0x29 => Some(Instruction::SRA(PrefixTarget::C)),
            0x2a => Some(Instruction::SRA(PrefixTarget::D)),
            0x2b => Some(Instruction::SRA(PrefixTarget::E)),
            0x2c => Some(Instruction::SRA(PrefixTarget::H)),
            0x2d => Some(Instruction::SRA(PrefixTarget::L)),
            0x2e => Some(Instruction::SRA(PrefixTarget::HLP)),
            0x2f => Some(Instruction::SRA(PrefixTarget::A)),

            0x30 => Some(Instruction::SWAP(PrefixTarget::B)),
            0x31 => Some(Instruction::SWAP(PrefixTarget::C)),
            0x32 => Some(Instruction::SWAP(PrefixTarget::D)),
            0x33 => Some(Instruction::SWAP(PrefixTarget::E)),
            0x34 => Some(Instruction::SWAP(PrefixTarget::H)),
            0x35 => Some(Instruction::SWAP(PrefixTarget::L)),
            0x36 => Some(Instruction::SWAP(PrefixTarget::HLP)),
            0x37 => Some(Instruction::SWAP(PrefixTarget::A)),
            0x38 => Some(Instruction::SRL(PrefixTarget::B)),
            0x39 => Some(Instruction::SRL(PrefixTarget::C)),
            0x3a => Some(Instruction::SRL(PrefixTarget::D)),
            0x3b => Some(Instruction::SRL(PrefixTarget::E)),
            0x3c => Some(Instruction::SRL(PrefixTarget::H)),
            0x3d => Some(Instruction::SRL(PrefixTarget::L)),
            0x3e => Some(Instruction::SRL(PrefixTarget::HLP)),
            0x3f => Some(Instruction::SRL(PrefixTarget::A)),

            0x40 => Some(Instruction::BIT(BitPosition::B0, PrefixTarget::B)),
            0x41 => Some(Instruction::BIT(BitPosition::B0, PrefixTarget::C)),
            0x42 => Some(Instruction::BIT(BitPosition::B0, PrefixTarget::D)),
            0x43 => Some(Instruction::BIT(BitPosition::B0, PrefixTarget::E)),
            0x44 => Some(Instruction::BIT(BitPosition::B0, PrefixTarget::H)),
            0x45 => Some(Instruction::BIT(BitPosition::B0, PrefixTarget::L)),
            0x46 => Some(Instruction::BIT(BitPosition::B0, PrefixTarget::HLP)),
            0x47 => Some(Instruction::BIT(BitPosition::B0, PrefixTarget::A)),
            0x48 => Some(Instruction::BIT(BitPosition::B1, PrefixTarget::B)),
            0x49 => Some(Instruction::BIT(BitPosition::B1, PrefixTarget::C)),
            0x4a => Some(Instruction::BIT(BitPosition::B1, PrefixTarget::D)),
            0x4b => Some(Instruction::BIT(BitPosition::B1, PrefixTarget::E)),
            0x4c => Some(Instruction::BIT(BitPosition::B1, PrefixTarget::H)),
            0x4d => Some(Instruction::BIT(BitPosition::B1, PrefixTarget::L)),
            0x4e => Some(Instruction::BIT(BitPosition::B1, PrefixTarget::HLP)),
            0x4f => Some(Instruction::BIT(BitPosition::B1, PrefixTarget::A)),

            0x50 => Some(Instruction::BIT(BitPosition::B2, PrefixTarget::B)),
            0x51 => Some(Instruction::BIT(BitPosition::B2, PrefixTarget::C)),
            0x52 => Some(Instruction::BIT(BitPosition::B2, PrefixTarget::D)),
            0x53 => Some(Instruction::BIT(BitPosition::B2, PrefixTarget::E)),
            0x54 => Some(Instruction::BIT(BitPosition::B2, PrefixTarget::H)),
            0x55 => Some(Instruction::BIT(BitPosition::B2, PrefixTarget::L)),
            0x56 => Some(Instruction::BIT(BitPosition::B2, PrefixTarget::HLP)),
            0x57 => Some(Instruction::BIT(BitPosition::B2, PrefixTarget::A)),
            0x58 => Some(Instruction::BIT(BitPosition::B3, PrefixTarget::B)),
            0x59 => Some(Instruction::BIT(BitPosition::B3, PrefixTarget::C)),
            0x5a => Some(Instruction::BIT(BitPosition::B3, PrefixTarget::D)),
            0x5b => Some(Instruction::BIT(BitPosition::B3, PrefixTarget::E)),
            0x5c => Some(Instruction::BIT(BitPosition::B3, PrefixTarget::H)),
            0x5d => Some(Instruction::BIT(BitPosition::B3, PrefixTarget::L)),
            0x5e => Some(Instruction::BIT(BitPosition::B3, PrefixTarget::HLP)),
            0x5f => Some(Instruction::BIT(BitPosition::B3, PrefixTarget::A)),

            0x60 => Some(Instruction::BIT(BitPosition::B4, PrefixTarget::B)),
            0x61 => Some(Instruction::BIT(BitPosition::B4, PrefixTarget::C)),
            0x62 => Some(Instruction::BIT(BitPosition::B4, PrefixTarget::D)),
            0x63 => Some(Instruction::BIT(BitPosition::B4, PrefixTarget::E)),
            0x64 => Some(Instruction::BIT(BitPosition::B4, PrefixTarget::H)),
            0x65 => Some(Instruction::BIT(BitPosition::B4, PrefixTarget::L)),
            0x66 => Some(Instruction::BIT(BitPosition::B4, PrefixTarget::HLP)),
            0x67 => Some(Instruction::BIT(BitPosition::B4, PrefixTarget::A)),
            0x68 => Some(Instruction::BIT(BitPosition::B5, PrefixTarget::B)),
            0x69 => Some(Instruction::BIT(BitPosition::B5, PrefixTarget::C)),
            0x6a => Some(Instruction::BIT(BitPosition::B5, PrefixTarget::D)),
            0x6b => Some(Instruction::BIT(BitPosition::B5, PrefixTarget::E)),
            0x6c => Some(Instruction::BIT(BitPosition::B5, PrefixTarget::H)),
            0x6d => Some(Instruction::BIT(BitPosition::B5, PrefixTarget::L)),
            0x6e => Some(Instruction::BIT(BitPosition::B5, PrefixTarget::HLP)),
            0x6f => Some(Instruction::BIT(BitPosition::B5, PrefixTarget::A)),

            0x70 => Some(Instruction::BIT(BitPosition::B6, PrefixTarget::B)),
            0x71 => Some(Instruction::BIT(BitPosition::B6, PrefixTarget::C)),
            0x72 => Some(Instruction::BIT(BitPosition::B6, PrefixTarget::D)),
            0x73 => Some(Instruction::BIT(BitPosition::B6, PrefixTarget::E)),
            0x74 => Some(Instruction::BIT(BitPosition::B6, PrefixTarget::H)),
            0x75 => Some(Instruction::BIT(BitPosition::B6, PrefixTarget::L)),
            0x76 => Some(Instruction::BIT(BitPosition::B6, PrefixTarget::HLP)),
            0x77 => Some(Instruction::BIT(BitPosition::B6, PrefixTarget::A)),
            0x78 => Some(Instruction::BIT(BitPosition::B7, PrefixTarget::B)),
            0x79 => Some(Instruction::BIT(BitPosition::B7, PrefixTarget::C)),
            0x7a => Some(Instruction::BIT(BitPosition::B7, PrefixTarget::D)),
            0x7b => Some(Instruction::BIT(BitPosition::B7, PrefixTarget::E)),
            0x7c => Some(Instruction::BIT(BitPosition::B7, PrefixTarget::H)),
            0x7d => Some(Instruction::BIT(BitPosition::B7, PrefixTarget::L)),
            0x7e => Some(Instruction::BIT(BitPosition::B7, PrefixTarget::HLP)),
            0x7f => Some(Instruction::BIT(BitPosition::B7, PrefixTarget::A)),

            0x80 => Some(Instruction::RES(BitPosition::B0, PrefixTarget::B)),
            0x81 => Some(Instruction::RES(BitPosition::B0, PrefixTarget::C)),
            0x82 => Some(Instruction::RES(BitPosition::B0, PrefixTarget::D)),
            0x83 => Some(Instruction::RES(BitPosition::B0, PrefixTarget::E)),
            0x84 => Some(Instruction::RES(BitPosition::B0, PrefixTarget::H)),
            0x85 => Some(Instruction::RES(BitPosition::B0, PrefixTarget::L)),
            0x86 => Some(Instruction::RES(BitPosition::B0, PrefixTarget::HLP)),
            0x87 => Some(Instruction::RES(BitPosition::B0, PrefixTarget::A)),
            0x88 => Some(Instruction::RES(BitPosition::B1, PrefixTarget::B)),
            0x89 => Some(Instruction::RES(BitPosition::B1, PrefixTarget::C)),
            0x8a => Some(Instruction::RES(BitPosition::B1, PrefixTarget::D)),
            0x8b => Some(Instruction::RES(BitPosition::B1, PrefixTarget::E)),
            0x8c => Some(Instruction::RES(BitPosition::B1, PrefixTarget::H)),
            0x8d => Some(Instruction::RES(BitPosition::B1, PrefixTarget::L)),
            0x8e => Some(Instruction::RES(BitPosition::B1, PrefixTarget::HLP)),
            0x8f => Some(Instruction::RES(BitPosition::B1, PrefixTarget::A)),

            0x90 => Some(Instruction::RES(BitPosition::B2, PrefixTarget::B)),
            0x91 => Some(Instruction::RES(BitPosition::B2, PrefixTarget::C)),
            0x92 => Some(Instruction::RES(BitPosition::B2, PrefixTarget::D)),
            0x93 => Some(Instruction::RES(BitPosition::B2, PrefixTarget::E)),
            0x94 => Some(Instruction::RES(BitPosition::B2, PrefixTarget::H)),
            0x95 => Some(Instruction::RES(BitPosition::B2, PrefixTarget::L)),
            0x96 => Some(Instruction::RES(BitPosition::B2, PrefixTarget::HLP)),
            0x97 => Some(Instruction::RES(BitPosition::B2, PrefixTarget::A)),
            0x98 => Some(Instruction::RES(BitPosition::B3, PrefixTarget::B)),
            0x99 => Some(Instruction::RES(BitPosition::B3, PrefixTarget::C)),
            0x9a => Some(Instruction::RES(BitPosition::B3, PrefixTarget::D)),
            0x9b => Some(Instruction::RES(BitPosition::B3, PrefixTarget::E)),
            0x9c => Some(Instruction::RES(BitPosition::B3, PrefixTarget::H)),
            0x9d => Some(Instruction::RES(BitPosition::B3, PrefixTarget::L)),
            0x9e => Some(Instruction::RES(BitPosition::B3, PrefixTarget::HLP)),
            0x9f => Some(Instruction::RES(BitPosition::B3, PrefixTarget::A)),

            0xa0 => Some(Instruction::RES(BitPosition::B4, PrefixTarget::B)),
            0xa1 => Some(Instruction::RES(BitPosition::B4, PrefixTarget::C)),
            0xa2 => Some(Instruction::RES(BitPosition::B4, PrefixTarget::D)),
            0xa3 => Some(Instruction::RES(BitPosition::B4, PrefixTarget::E)),
            0xa4 => Some(Instruction::RES(BitPosition::B4, PrefixTarget::H)),
            0xa5 => Some(Instruction::RES(BitPosition::B4, PrefixTarget::L)),
            0xa6 => Some(Instruction::RES(BitPosition::B4, PrefixTarget::HLP)),
            0xa7 => Some(Instruction::RES(BitPosition::B4, PrefixTarget::A)),
            0xa8 => Some(Instruction::RES(BitPosition::B5, PrefixTarget::B)),
            0xa9 => Some(Instruction::RES(BitPosition::B5, PrefixTarget::C)),
            0xaa => Some(Instruction::RES(BitPosition::B5, PrefixTarget::D)),
            0xab => Some(Instruction::RES(BitPosition::B5, PrefixTarget::E)),
            0xac => Some(Instruction::RES(BitPosition::B5, PrefixTarget::H)),
            0xad => Some(Instruction::RES(BitPosition::B5, PrefixTarget::L)),
            0xae => Some(Instruction::RES(BitPosition::B5, PrefixTarget::HLP)),
            0xaf => Some(Instruction::RES(BitPosition::B5, PrefixTarget::A)),

            0xb0 => Some(Instruction::RES(BitPosition::B6, PrefixTarget::B)),
            0xb1 => Some(Instruction::RES(BitPosition::B6, PrefixTarget::C)),
            0xb2 => Some(Instruction::RES(BitPosition::B6, PrefixTarget::D)),
            0xb3 => Some(Instruction::RES(BitPosition::B6, PrefixTarget::E)),
            0xb4 => Some(Instruction::RES(BitPosition::B6, PrefixTarget::H)),
            0xb5 => Some(Instruction::RES(BitPosition::B6, PrefixTarget::L)),
            0xb6 => Some(Instruction::RES(BitPosition::B6, PrefixTarget::HLP)),
            0xb7 => Some(Instruction::RES(BitPosition::B6, PrefixTarget::A)),
            0xb8 => Some(Instruction::RES(BitPosition::B7, PrefixTarget::B)),
            0xb9 => Some(Instruction::RES(BitPosition::B7, PrefixTarget::C)),
            0xba => Some(Instruction::RES(BitPosition::B7, PrefixTarget::D)),
            0xbb => Some(Instruction::RES(BitPosition::B7, PrefixTarget::E)),
            0xbc => Some(Instruction::RES(BitPosition::B7, PrefixTarget::H)),
            0xbd => Some(Instruction::RES(BitPosition::B7, PrefixTarget::L)),
            0xbe => Some(Instruction::RES(BitPosition::B7, PrefixTarget::HLP)),
            0xbf => Some(Instruction::RES(BitPosition::B7, PrefixTarget::A)),

            0xc0 => Some(Instruction::SET(BitPosition::B0, PrefixTarget::B)),
            0xc1 => Some(Instruction::SET(BitPosition::B0, PrefixTarget::C)),
            0xc2 => Some(Instruction::SET(BitPosition::B0, PrefixTarget::D)),
            0xc3 => Some(Instruction::SET(BitPosition::B0, PrefixTarget::E)),
            0xc4 => Some(Instruction::SET(BitPosition::B0, PrefixTarget::H)),
            0xc5 => Some(Instruction::SET(BitPosition::B0, PrefixTarget::L)),
            0xc6 => Some(Instruction::SET(BitPosition::B0, PrefixTarget::HLP)),
            0xc7 => Some(Instruction::SET(BitPosition::B0, PrefixTarget::A)),
            0xc8 => Some(Instruction::SET(BitPosition::B1, PrefixTarget::B)),
            0xc9 => Some(Instruction::SET(BitPosition::B1, PrefixTarget::C)),
            0xca => Some(Instruction::SET(BitPosition::B1, PrefixTarget::D)),
            0xcb => Some(Instruction::SET(BitPosition::B1, PrefixTarget::E)),
            0xcc => Some(Instruction::SET(BitPosition::B1, PrefixTarget::H)),
            0xcd => Some(Instruction::SET(BitPosition::B1, PrefixTarget::L)),
            0xce => Some(Instruction::SET(BitPosition::B1, PrefixTarget::HLP)),
            0xcf => Some(Instruction::SET(BitPosition::B1, PrefixTarget::A)),

            0xd0 => Some(Instruction::SET(BitPosition::B2, PrefixTarget::B)),
            0xd1 => Some(Instruction::SET(BitPosition::B2, PrefixTarget::C)),
            0xd2 => Some(Instruction::SET(BitPosition::B2, PrefixTarget::D)),
            0xd3 => Some(Instruction::SET(BitPosition::B2, PrefixTarget::E)),
            0xd4 => Some(Instruction::SET(BitPosition::B2, PrefixTarget::H)),
            0xd5 => Some(Instruction::SET(BitPosition::B2, PrefixTarget::L)),
            0xd6 => Some(Instruction::SET(BitPosition::B2, PrefixTarget::HLP)),
            0xd7 => Some(Instruction::SET(BitPosition::B2, PrefixTarget::A)),
            0xd8 => Some(Instruction::SET(BitPosition::B3, PrefixTarget::B)),
            0xd9 => Some(Instruction::SET(BitPosition::B3, PrefixTarget::C)),
            0xda => Some(Instruction::SET(BitPosition::B3, PrefixTarget::D)),
            0xdb => Some(Instruction::SET(BitPosition::B3, PrefixTarget::E)),
            0xdc => Some(Instruction::SET(BitPosition::B3, PrefixTarget::H)),
            0xdd => Some(Instruction::SET(BitPosition::B3, PrefixTarget::L)),
            0xde => Some(Instruction::SET(BitPosition::B3, PrefixTarget::HLP)),
            0xdf => Some(Instruction::SET(BitPosition::B3, PrefixTarget::A)),

            0xe0 => Some(Instruction::SET(BitPosition::B4, PrefixTarget::B)),
            0xe1 => Some(Instruction::SET(BitPosition::B4, PrefixTarget::C)),
            0xe2 => Some(Instruction::SET(BitPosition::B4, PrefixTarget::D)),
            0xe3 => Some(Instruction::SET(BitPosition::B4, PrefixTarget::E)),
            0xe4 => Some(Instruction::SET(BitPosition::B4, PrefixTarget::H)),
            0xe5 => Some(Instruction::SET(BitPosition::B4, PrefixTarget::L)),
            0xe6 => Some(Instruction::SET(BitPosition::B4, PrefixTarget::HLP)),
            0xe7 => Some(Instruction::SET(BitPosition::B4, PrefixTarget::A)),
            0xe8 => Some(Instruction::SET(BitPosition::B5, PrefixTarget::B)),
            0xe9 => Some(Instruction::SET(BitPosition::B5, PrefixTarget::C)),
            0xea => Some(Instruction::SET(BitPosition::B5, PrefixTarget::D)),
            0xeb => Some(Instruction::SET(BitPosition::B5, PrefixTarget::E)),
            0xec => Some(Instruction::SET(BitPosition::B5, PrefixTarget::H)),
            0xed => Some(Instruction::SET(BitPosition::B5, PrefixTarget::L)),
            0xee => Some(Instruction::SET(BitPosition::B5, PrefixTarget::HLP)),
            0xef => Some(Instruction::SET(BitPosition::B5, PrefixTarget::A)),

            0xf0 => Some(Instruction::SET(BitPosition::B6, PrefixTarget::B)),
            0xf1 => Some(Instruction::SET(BitPosition::B6, PrefixTarget::C)),
            0xf2 => Some(Instruction::SET(BitPosition::B6, PrefixTarget::D)),
            0xf3 => Some(Instruction::SET(BitPosition::B6, PrefixTarget::E)),
            0xf4 => Some(Instruction::SET(BitPosition::B6, PrefixTarget::H)),
            0xf5 => Some(Instruction::SET(BitPosition::B6, PrefixTarget::L)),
            0xf6 => Some(Instruction::SET(BitPosition::B6, PrefixTarget::HLP)),
            0xf7 => Some(Instruction::SET(BitPosition::B6, PrefixTarget::A)),
            0xf8 => Some(Instruction::SET(BitPosition::B7, PrefixTarget::B)),
            0xf9 => Some(Instruction::SET(BitPosition::B7, PrefixTarget::C)),
            0xfa => Some(Instruction::SET(BitPosition::B7, PrefixTarget::D)),
            0xfb => Some(Instruction::SET(BitPosition::B7, PrefixTarget::E)),
            0xfc => Some(Instruction::SET(BitPosition::B7, PrefixTarget::H)),
            0xfd => Some(Instruction::SET(BitPosition::B7, PrefixTarget::L)),
            0xfe => Some(Instruction::SET(BitPosition::B7, PrefixTarget::HLP)),
            0xff => Some(Instruction::SET(BitPosition::B7, PrefixTarget::A)),
        }
    }
}
