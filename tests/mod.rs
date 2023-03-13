use dissasemble_mov::dissasemble;

type TestResult = Result<(), nom::error::Error<&'static [u8]>>;

fn run_case(bytes: &[u8], answer: &str) -> TestResult {
    let result = dissasemble(bytes).unwrap();
    for (i, line) in answer.lines().filter(|s| s.starts_with("mov")).enumerate() {
        assert_eq!(line, result[i]);
    }
    Ok(())
}

#[test]
fn single_register_mov() -> TestResult {
    let bytes = include_bytes!("cases/listing_0037_single_register_mov");
    let answer = include_str!("cases/listing_0037_single_register_mov.asm");
    run_case(bytes, answer)
}

#[test]
fn many_register_move() -> TestResult {
    let bytes = include_bytes!("cases/listing_0038_many_register_mov");
    let answer = include_str!("cases/listing_0038_many_register_mov.asm");
    run_case(bytes, answer)
}

#[test]
fn more_movs() -> TestResult {
    let bytes = include_bytes!("cases/listing_0039_more_movs");
    let answer = include_str!("cases/listing_0039_more_movs.asm");
    run_case(bytes, answer)
}

#[test]
fn challenge_movs() -> TestResult {
    let bytes = include_bytes!("cases/listing_0040_challenge_movs");
    let answer = include_str!("cases/listing_0040_challenge_movs.asm");
    run_case(bytes, answer)
}
