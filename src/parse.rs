#![allow(clippy::just_underscores_and_digits)]

use crate::model::*;
use nom::{
    bits::{
        bits, bytes,
        complete::{bool, tag, take},
    },
    branch::alt,
    error::Error,
    multi::many1,
    number::complete::{i8, le_i16},
    sequence::tuple,
    Parser,
};
use nom_supreme::final_parser::final_parser;

type IResult<'a, O> = nom::IResult<(&'a [u8], usize), O>;

pub(crate) fn many_move_instructions(input: &[u8]) -> Result<Vec<MoveInstruction>, Error<&[u8]>> {
    final_parser(many1(bits::<_, _, Error<(&[u8], usize)>, Error<&[u8]>, _>(
        mov_instruction,
    )))(input)
}

fn mov_instruction(input: (&[u8], usize)) -> IResult<MoveInstruction> {
    alt((
        register_or_memory_to_or_from_register,
        immediate_to_register_or_memory,
        immediate_to_register,
        memory_to_accumulator,
        accumulator_to_memory,
        register_or_memory_to_segment_register,
        segment_register_to_register_or_memory,
    ))(input)
}

fn register_or_memory_to_or_from_register(input: (&[u8], usize)) -> IResult<MoveInstruction> {
    let (input, (_, reg_is_destination, word, mode, reg_code, reg_or_mem_code)) =
        tuple((tag(0b100010, 6usize), bool, bool, mode, code, code))(input)?;
    debug_assert_eq!(input.1, 0);
    let reg = Register {
        word,
        code: reg_code,
    };
    let (input, reg_or_mem) = reg_or_mem_calc(word, mode, reg_or_mem_code, input)?;
    let (source, destination) = if reg_is_destination {
        (reg_or_mem, reg.into())
    } else {
        (reg.into(), reg_or_mem)
    };
    Ok((
        input,
        MoveInstruction {
            source,
            destination,
        },
    ))
}

fn immediate_to_register_or_memory(input: (&[u8], usize)) -> IResult<MoveInstruction> {
    let (input, (_, word, mode, _, reg_or_mem_code)) =
        tuple((tag(0b1100011, 7usize), bool, mode, tag(0b000, 3usize), code))(input)?;
    debug_assert_eq!(input.1, 0);
    let (input, reg_or_mem) = reg_or_mem_calc(word, mode, reg_or_mem_code, input)?;
    let (input, data) = byte_or_word(word, input)?;
    let data_literal = match (reg_or_mem, word) {
        (Location::MemoryCalc { .. }, false) => DataLiteral::ExplicitByte(data),
        (Location::MemoryCalc { .. }, true) => DataLiteral::ExplicitWord(data),
        _ => DataLiteral::Implicit(data),
    };
    Ok((
        input,
        MoveInstruction {
            source: data_literal.into(),
            destination: reg_or_mem,
        },
    ))
}

fn immediate_to_register(input: (&[u8], usize)) -> IResult<MoveInstruction> {
    let (input, (_, word, code)) = tuple((tag(0b1011, 4usize), bool, code))(input)?;
    debug_assert_eq!(input.1, 0);
    let (input, data) = byte_or_word(word, input)?;
    Ok((
        input,
        MoveInstruction {
            source: DataLiteral::Implicit(data).into(),
            destination: Register { word, code }.into(),
        },
    ))
}

fn memory_to_accumulator(input: (&[u8], usize)) -> IResult<MoveInstruction> {
    let (input, (_, word)) = tuple((tag(0b1010000, 7usize), bool))(input)?;
    debug_assert_eq!(input.1, 0);
    let (input, addr) = byte_or_word(word, input)?;
    Ok((
        input,
        MoveInstruction {
            source: MemoryLiteral(addr).into(),
            destination: Accumulator { word }.into(),
        },
    ))
}

fn accumulator_to_memory(input: (&[u8], usize)) -> IResult<MoveInstruction> {
    let (input, (_, word)) = tuple((tag(0b1010001, 7usize), bool))(input)?;
    debug_assert_eq!(input.1, 0);
    let (input, addr) = byte_or_word(word, input)?;
    Ok((
        input,
        MoveInstruction {
            source: Accumulator { word }.into(),
            destination: MemoryLiteral(addr).into(),
        },
    ))
}

fn register_or_memory_to_segment_register(input: (&[u8], usize)) -> IResult<MoveInstruction> {
    let (input, (_, mode, _, sr, code)) = tuple((
        tag(0b10001110, 8usize),
        mode,
        tag(0b0, 1usize),
        segment_register,
        code,
    ))(input)?;
    debug_assert_eq!(input.1, 0);
    let (input, reg_or_mem) = reg_or_mem_calc(true, mode, code, input)?;
    Ok((
        input,
        MoveInstruction {
            source: reg_or_mem,
            destination: sr.into(),
        },
    ))
}

fn segment_register_to_register_or_memory(input: (&[u8], usize)) -> IResult<MoveInstruction> {
    let (input, (_, mode, _, sr, code)) = tuple((
        tag(0b10001100, 8usize),
        mode,
        tag(0b0, 1usize),
        segment_register,
        code,
    ))(input)?;
    debug_assert_eq!(input.1, 0);
    let (input, reg_or_mem) = reg_or_mem_calc(true, mode, code, input)?;
    Ok((
        input,
        MoveInstruction {
            source: sr.into(),
            destination: reg_or_mem,
        },
    ))
}

#[derive(Debug, Clone, Copy)]
enum Mode {
    _00,
    _01,
    _10,
    _11,
}

fn mode(input: (&[u8], usize)) -> IResult<Mode> {
    use Mode::*;
    take(2usize)
        .map(|bits: u8| match bits {
            0b00 => _00,
            0b01 => _01,
            0b10 => _10,
            0b11 => _11,
            _ => unreachable!(),
        })
        .parse(input)
}

fn segment_register(input: (&[u8], usize)) -> IResult<SegmentRegister> {
    use SegmentRegister::*;
    take(2usize)
        .map(|bits: u8| match bits {
            0b00 => _00,
            0b01 => _01,
            0b10 => _10,
            0b11 => _11,
            _ => unreachable!(),
        })
        .parse(input)
}

fn code(input: (&[u8], usize)) -> IResult<Code> {
    use Code::*;
    take(3usize)
        .map(|bits: u8| match bits {
            0b000 => _000,
            0b001 => _001,
            0b010 => _010,
            0b011 => _011,
            0b100 => _100,
            0b101 => _101,
            0b110 => _110,
            0b111 => _111,
            _ => unreachable!(),
        })
        .parse(input)
}

fn parse_word(input: (&[u8], usize)) -> IResult<i16> {
    bytes::<_, _, Error<&[u8]>, _, _>(le_i16)(input)
}

fn byte_as_i16(input: (&[u8], usize)) -> IResult<i16> {
    bytes::<_, _, Error<&[u8]>, _, _>(i8)
        .map(|b| b as i16)
        .parse(input)
}

fn byte_or_word(word: bool, input: (&[u8], usize)) -> IResult<i16> {
    if word {
        parse_word.parse(input)
    } else {
        byte_as_i16.parse(input)
    }
}

fn reg_or_mem_calc(word: bool, mode: Mode, code: Code, input: (&[u8], usize)) -> IResult<Location> {
    use Mode::*;
    let (input, displacement) = match mode {
        _11 => return Ok((input, Register { word, code }.into())),
        _00 if matches!(code, Code::_110) => parse_word.map(Some).parse(input)?,
        _00 => (input, None),
        _01 => byte_as_i16.map(Some).parse(input)?,
        _10 => parse_word.map(Some).parse(input)?,
    };
    Ok((
        input,
        MemoryCalc {
            displacement,
            mode_is_0: matches!(mode, _00),
            code,
        }
        .into(),
    ))
}
