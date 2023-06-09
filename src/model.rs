use crate::display::FormatOnto;
use enum_dispatch::enum_dispatch;

#[derive(Debug, Clone, Copy)]
pub(crate) struct MoveInstruction {
    pub(crate) source: Location,
    pub(crate) destination: Location,
}

#[enum_dispatch(FormatOnto)]
#[derive(Debug, Clone, Copy)]
pub(crate) enum Location {
    DataLiteral,
    MemoryLiteral,
    Accumulator,
    SegmentRegister,
    Register,
    MemoryCalc,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum DataLiteral {
    ExplicitByte(i16),
    ExplicitWord(i16),
    Implicit(i16),
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct MemoryLiteral(pub(crate) i16);

#[derive(Debug, Clone, Copy)]
pub(crate) struct Accumulator {
    pub(crate) word: bool,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum SegmentRegister {
    _00,
    _01,
    _10,
    _11,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Register {
    pub(crate) word: bool,
    pub(crate) code: ThreeBitCode,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct MemoryCalc {
    pub(crate) displacement: Option<i16>,
    pub(crate) mode_is_0: bool,
    pub(crate) code: ThreeBitCode,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum ThreeBitCode {
    _000,
    _001,
    _010,
    _011,
    _100,
    _101,
    _110,
    _111,
}
