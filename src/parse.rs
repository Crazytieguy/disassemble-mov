use crate::{display::FormatOnto, model::*};
use nom::{
    bits::{
        bits, bytes,
        complete::{bool, tag, take},
    },
    branch::alt,
    error::Error,
    multi::fold_many1,
    number::complete::{i8, le_i16},
    sequence::tuple,
    Parser,
};
use nom_supreme::final_parser::final_parser;

type IResult<'a, O> = nom::IResult<(&'a [u8], usize), O>;

pub fn disassemble(input: &[u8]) -> Result<String, Error<&[u8]>> {
    let byte_mov_parser = bits::<_, _, Error<(&[u8], usize)>, Error<&[u8]>, _>(mov_instruction);
    let parse_many = fold_many1(
        byte_mov_parser,
        || String::with_capacity(input.len() * 5),
        |mut acc, instruction| {
            instruction.format_onto(&mut acc);
            acc.push('\n');
            acc
        },
    );
    final_parser(parse_many)(input)
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
    let (input, (_, reg_is_destination, word, mode, reg_code, reg_or_mem_code)) = tuple((
        tag(0b100010, 6usize),
        bool,
        bool,
        mode,
        three_bit_code,
        three_bit_code,
    ))(input)?;
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
    ok_move_instruction(input, source, destination)
}

fn immediate_to_register_or_memory(input: (&[u8], usize)) -> IResult<MoveInstruction> {
    let (input, (_, word, mode, _, reg_or_mem_code)) = tuple((
        tag(0b1100011, 7usize),
        bool,
        mode,
        tag(0b000, 3usize),
        three_bit_code,
    ))(input)?;
    debug_assert_eq!(input.1, 0);
    let (input, destination) = reg_or_mem_calc(word, mode, reg_or_mem_code, input)?;
    let (input, data) = byte_or_word(word, input)?;
    let source = match (destination, word) {
        (Location::MemoryCalc { .. }, false) => DataLiteral::ExplicitByte(data),
        (Location::MemoryCalc { .. }, true) => DataLiteral::ExplicitWord(data),
        _ => DataLiteral::Implicit(data),
    };
    ok_move_instruction(input, source, destination)
}

fn immediate_to_register(input: (&[u8], usize)) -> IResult<MoveInstruction> {
    let (input, (_, word, code)) = tuple((tag(0b1011, 4usize), bool, three_bit_code))(input)?;
    debug_assert_eq!(input.1, 0);
    let (input, data) = byte_or_word(word, input)?;
    let source = DataLiteral::Implicit(data);
    let destination = Register { word, code };
    ok_move_instruction(input, source, destination)
}

fn memory_to_accumulator(input: (&[u8], usize)) -> IResult<MoveInstruction> {
    let (input, (_, word)) = tuple((tag(0b1010000, 7usize), bool))(input)?;
    debug_assert_eq!(input.1, 0);
    let (input, addr) = byte_or_word(word, input)?;
    let source = MemoryLiteral(addr);
    let destination = Accumulator { word };
    ok_move_instruction(input, source, destination)
}

fn accumulator_to_memory(input: (&[u8], usize)) -> IResult<MoveInstruction> {
    let (input, (_, word)) = tuple((tag(0b1010001, 7usize), bool))(input)?;
    debug_assert_eq!(input.1, 0);
    let (input, addr) = byte_or_word(word, input)?;
    let source = Accumulator { word };
    let destination = MemoryLiteral(addr);
    ok_move_instruction(input, source, destination)
}

fn register_or_memory_to_segment_register(input: (&[u8], usize)) -> IResult<MoveInstruction> {
    let (input, (_, mode, _, destination, code)) = tuple((
        tag(0b10001110, 8usize),
        mode,
        tag(0b0, 1usize),
        segment_register,
        three_bit_code,
    ))(input)?;
    debug_assert_eq!(input.1, 0);
    let (input, source) = reg_or_mem_calc(true, mode, code, input)?;
    ok_move_instruction(input, source, destination)
}

fn segment_register_to_register_or_memory(input: (&[u8], usize)) -> IResult<MoveInstruction> {
    let (input, (_, mode, _, source, code)) = tuple((
        tag(0b10001100, 8usize),
        mode,
        tag(0b0, 1usize),
        segment_register,
        three_bit_code,
    ))(input)?;
    debug_assert_eq!(input.1, 0);
    let (input, destination) = reg_or_mem_calc(true, mode, code, input)?;
    ok_move_instruction(input, source, destination)
}

fn ok_move_instruction(
    input: (&[u8], usize),
    source: impl Into<Location>,
    destination: impl Into<Location>,
) -> IResult<MoveInstruction> {
    Ok((
        input,
        MoveInstruction {
            source: source.into(),
            destination: destination.into(),
        },
    ))
}

fn reg_or_mem_calc(
    word: bool,
    mode: Mode,
    code: ThreeBitCode,
    input: (&[u8], usize),
) -> IResult<Location> {
    use Mode::*;
    let (input, displacement) = match mode {
        _11 => return Ok((input, Register { word, code }.into())),
        _00 if matches!(code, ThreeBitCode::_110) => parse_word.map(Some).parse(input)?,
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

fn three_bit_code(input: (&[u8], usize)) -> IResult<ThreeBitCode> {
    use ThreeBitCode::*;
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
