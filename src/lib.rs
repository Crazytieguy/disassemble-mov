#![allow(clippy::just_underscores_and_digits)]

mod display;
mod model;
mod parse;

pub fn dissasemble(bytes: &[u8]) -> Result<Vec<String>, nom::error::Error<&[u8]>> {
    Ok(parse::many_move_instructions(bytes)?
        .into_iter()
        .map(|i| i.to_string())
        .collect())
}
