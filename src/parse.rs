use crate::{display::FormatOnto, model::*};
use nom::{
    bits::complete::{bool, tag, take},
    error::Error,
    multi::fold_many1,
    number::complete::{i8, le_i16, u8},
    sequence::tuple,
    Parser,
};
use nom_supreme::final_parser::final_parser;

type BitResult<'a, O> = nom::IResult<(&'a [u8], usize), O>;
type ByteResult<'a, O> = nom::IResult<&'a [u8], O>;

pub fn disassemble(input: &[u8]) -> Result<String, Error<&[u8]>> {
    final_parser(fold_many1(
        mov_instruction,
        || String::with_capacity(input.len() * 6),
        |mut acc, instruction| {
            instruction.format_onto(&mut acc);
            acc.push('\n');
            acc
        },
    ))(input)
}

fn mov_instruction(input: &[u8]) -> ByteResult<MoveInstruction> {
    let (_, first_byte) = u8(input)?;
    match first_byte {
        0b10001000..=0b10001011 => register_or_memory_to_or_from_register(input),
        0b11000110..=0b11000111 => immediate_to_register_or_memory(input),
        0b10110000..=0b10111111 => immediate_to_register(input),
        0b10100000..=0b10100001 => memory_to_accumulator(input),
        0b10100010..=0b10100011 => accumulator_to_memory(input),
        0b10001110 => register_or_memory_to_segment_register(input),
        0b10001100 => segment_register_to_register_or_memory(input),
        _ => Err(nom::Err::Error(Error::new(
            input,
            nom::error::ErrorKind::Tag,
        ))),
    }
}

fn register_or_memory_to_or_from_register(input: &[u8]) -> ByteResult<MoveInstruction> {
    let (input, (_, reg_is_destination, word, mode, reg_code, reg_or_mem_code)) = bits(tuple((
        tag(0b100010, 6usize),
        bool,
        bool,
        mode,
        three_bit_code,
        three_bit_code,
    )))(input)?;
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

fn immediate_to_register_or_memory(input: &[u8]) -> ByteResult<MoveInstruction> {
    let (input, (_, word, mode, _, reg_or_mem_code)) = bits(tuple((
        tag(0b1100011, 7usize),
        bool,
        mode,
        tag(0b000, 3usize),
        three_bit_code,
    )))(input)?;
    let (input, destination) = reg_or_mem_calc(word, mode, reg_or_mem_code, input)?;
    let (input, data) = byte_or_word(word, input)?;
    let source = match (destination, word) {
        (Location::MemoryCalc { .. }, false) => DataLiteral::ExplicitByte(data),
        (Location::MemoryCalc { .. }, true) => DataLiteral::ExplicitWord(data),
        _ => DataLiteral::Implicit(data),
    };
    ok_move_instruction(input, source, destination)
}

fn immediate_to_register(input: &[u8]) -> ByteResult<MoveInstruction> {
    let (input, (_, word, code)) = bits(tuple((tag(0b1011, 4usize), bool, three_bit_code)))(input)?;
    let (input, data) = byte_or_word(word, input)?;
    let source = DataLiteral::Implicit(data);
    let destination = Register { word, code };
    ok_move_instruction(input, source, destination)
}

fn memory_to_accumulator(input: &[u8]) -> ByteResult<MoveInstruction> {
    let (input, (_, word)) = bits(tuple((tag(0b1010000, 7usize), bool)))(input)?;
    let (input, addr) = byte_or_word(word, input)?;
    let source = MemoryLiteral(addr);
    let destination = Accumulator { word };
    ok_move_instruction(input, source, destination)
}

fn accumulator_to_memory(input: &[u8]) -> ByteResult<MoveInstruction> {
    let (input, (_, word)) = bits(tuple((tag(0b1010001, 7usize), bool)))(input)?;
    let (input, addr) = byte_or_word(word, input)?;
    let source = Accumulator { word };
    let destination = MemoryLiteral(addr);
    ok_move_instruction(input, source, destination)
}

fn register_or_memory_to_segment_register(input: &[u8]) -> ByteResult<MoveInstruction> {
    let (input, (_, mode, _, destination, code)) = bits(tuple((
        tag(0b10001110, 8usize),
        mode,
        tag(0b0, 1usize),
        segment_register,
        three_bit_code,
    )))(input)?;
    let (input, source) = reg_or_mem_calc(true, mode, code, input)?;
    ok_move_instruction(input, source, destination)
}

fn segment_register_to_register_or_memory(input: &[u8]) -> ByteResult<MoveInstruction> {
    let (input, (_, mode, _, source, code)) = bits(tuple((
        tag(0b10001100, 8usize),
        mode,
        tag(0b0, 1usize),
        segment_register,
        three_bit_code,
    )))(input)?;
    let (input, destination) = reg_or_mem_calc(true, mode, code, input)?;
    ok_move_instruction(input, source, destination)
}

fn ok_move_instruction(
    input: &[u8],
    source: impl Into<Location>,
    destination: impl Into<Location>,
) -> ByteResult<MoveInstruction> {
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
    input: &[u8],
) -> ByteResult<Location> {
    use Mode::*;
    let (input, displacement) = match mode {
        _11 => return Ok((input, Register { word, code }.into())),
        _00 if matches!(code, ThreeBitCode::_110) => le_i16.map(Some).parse(input)?,
        _00 => (input, None),
        _01 => i8.map(|b| b as i16).map(Some).parse(input)?,
        _10 => le_i16.map(Some).parse(input)?,
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

fn mode(input: (&[u8], usize)) -> BitResult<Mode> {
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

fn segment_register(input: (&[u8], usize)) -> BitResult<SegmentRegister> {
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

fn three_bit_code(input: (&[u8], usize)) -> BitResult<ThreeBitCode> {
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

fn byte_or_word(word: bool, input: &[u8]) -> ByteResult<i16> {
    if word {
        le_i16(input)
    } else {
        i8.map(|b| b as i16).parse(input)
    }
}

fn bits<'a, O, P>(parser: P) -> impl FnMut(&'a [u8]) -> ByteResult<O>
where
    P: Parser<(&'a [u8], usize), O, Error<(&'a [u8], usize)>>,
{
    nom::bits::bits(parser)
}
