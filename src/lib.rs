#![allow(clippy::just_underscores_and_digits)]

use display::FormatOnto;

mod display;
mod model;
mod parse;

pub fn dissasemble(bytes: &[u8]) -> Result<String, nom::error::Error<&[u8]>> {
    let instructions = parse::many_move_instructions(bytes)?;
    let mut result = String::with_capacity(instructions.len() * 10);
    for instruction in instructions {
        instruction.format_onto(&mut result);
        result.push('\n');
    }
    Ok(result)
}
