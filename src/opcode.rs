#[derive(Debug, PartialEq, Copy, Clone)]
#[repr(u8)]
pub enum OpCode {
    Constant,
    Add,
    Subtract,
    Multiply,
    Divide,
    Negate,
    Not,
    True,
    False,
    Return,
    Nil,
    Equal,
    Greater,
    Less,
    Pop,
    DefineGlobal,
    GetGlobal,
    GetLocal,
    SetGlobal,
    SetLocal,
    Call,
    Jump,
    JumpIfFalse,
    Loop,
}

pub fn byte_to_opcode(byte: u8) -> Result<OpCode, u8> {
    match byte {
        0 => Ok(OpCode::Constant),
        1 => Ok(OpCode::Add),
        2 => Ok(OpCode::Subtract),
        3 => Ok(OpCode::Multiply),
        4 => Ok(OpCode::Divide),
        5 => Ok(OpCode::Negate),
        6 => Ok(OpCode::Not),
        7 => Ok(OpCode::True),
        8 => Ok(OpCode::False),
        9 => Ok(OpCode::Return),
        10 => Ok(OpCode::Nil),
        11 => Ok(OpCode::Equal),
        12 => Ok(OpCode::Greater),
        13 => Ok(OpCode::Less),
        14 => Ok(OpCode::Pop),
        15 => Ok(OpCode::DefineGlobal),
        16 => Ok(OpCode::GetGlobal),
        17 => Ok(OpCode::GetLocal),
        18 => Ok(OpCode::SetGlobal),
        19 => Ok(OpCode::SetLocal),
        20 => Ok(OpCode::Call),
        21 => Ok(OpCode::Jump),
        22 => Ok(OpCode::JumpIfFalse),
        23 => Ok(OpCode::Loop),

        _ => Err(byte),
    }
}

// #[test]
// fn opcode_round_trip() {
//     let opcodes = [
//         OpCode::Constant,
//         OpCode::Add,
//         OpCode::Subtract,
//         OpCode::Multiply,
//         OpCode::Divide,
//         OpCode::Negate,
//         OpCode::Not,
//         OpCode::True,
//         OpCode::False,
//         OpCode::Return,
//         OpCode::Nil,
//     ];

//     for opcode in opcodes {
//         assert_eq!(byte_to_opcode(opcode as u8), Ok(opcode));
//     }
// }