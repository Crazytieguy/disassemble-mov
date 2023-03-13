use crate::model::*;
use std::fmt::{Display, Formatter, Result};

impl Display for DataLiteral {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        use DataLiteral::*;
        match self {
            ExplicitByte(d) => write!(f, "byte {d}"),
            ExplicitWord(d) => write!(f, "word {d}"),
            Implicit(d) => write!(f, "{d}"),
        }
    }
}

impl Display for MemoryLiteral {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "[{d}]", d = self.0)
    }
}

impl Display for SegmentRegister {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        use SegmentRegister::*;
        match self {
            _00 => write!(f, "es"),
            _01 => write!(f, "cs"),
            _10 => write!(f, "ss"),
            _11 => write!(f, "ds"),
        }
    }
}

impl Display for MemoryCalc {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        use ThreeBitCode::*;
        let regs = match self.code {
            _000 => "bx + si",
            _001 => "bx + di",
            _010 => "bp + si",
            _011 => "bp + di",
            _100 => "si",
            _101 => "di",
            _110 if self.mode_is_0 => {
                let d = self
                    .displacement
                    .expect("Direct address should have a displacement");
                return write!(f, "[{d}]");
            }
            _110 => "bp",
            _111 => "bx",
        };
        match self.displacement {
            Some(d) if d > 0 => write!(f, "[{regs} + {d}]"),
            Some(d) if d < 0 => write!(f, "[{regs} - {neg}]", neg = -d),
            _ => write!(f, "[{regs}]"),
        }
    }
}

impl Display for Register {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        use ThreeBitCode::*;
        let reg = match (self.code, self.word) {
            (_000, false) => "al",
            (_001, false) => "cl",
            (_010, false) => "dl",
            (_011, false) => "bl",
            (_100, false) => "ah",
            (_101, false) => "ch",
            (_110, false) => "dh",
            (_111, false) => "bh",
            (_000, true) => "ax",
            (_001, true) => "cx",
            (_010, true) => "dx",
            (_011, true) => "bx",
            (_100, true) => "sp",
            (_101, true) => "bp",
            (_110, true) => "si",
            (_111, true) => "di",
        };
        write!(f, "{reg}")
    }
}

impl Display for Accumulator {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self.word {
            false => write!(f, "al"),
            true => write!(f, "ax"),
        }
    }
}

impl Display for MoveInstruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "mov {dest}, {source}",
            dest = self.destination,
            source = self.source
        )
    }
}
