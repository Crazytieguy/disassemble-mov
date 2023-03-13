use crate::model::*;
use std::fmt::{self, Display, Formatter};

impl Display for DataLiteral {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            DataLiteral::ExplicitByte(d) => write!(f, "byte {d}"),
            DataLiteral::ExplicitWord(d) => write!(f, "word {d}"),
            DataLiteral::Implicit(d) => write!(f, "{d}"),
        }
    }
}

impl Display for MemoryLiteral {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "[{d}]", d = self.0)
    }
}

impl Display for SegmentRegister {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            SegmentRegister::ES => write!(f, "es"),
            SegmentRegister::CS => write!(f, "cs"),
            SegmentRegister::SS => write!(f, "ss"),
            SegmentRegister::DS => write!(f, "ds"),
        }
    }
}

impl Display for MemoryCalc {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let regs = match self.code {
            Code::_000 => "bx + si",
            Code::_001 => "bx + di",
            Code::_010 => "bp + si",
            Code::_011 => "bp + di",
            Code::_100 => "si",
            Code::_101 => "di",
            Code::_110 if self.mode_is_0 => {
                let d = self
                    .displacement
                    .expect("Direct address should have a displacement");
                return write!(f, "[{d}]");
            }
            Code::_110 => "bp",
            Code::_111 => "bx",
        };
        match self.displacement {
            Some(d) if d > 0 => write!(f, "[{regs} + {d}]"),
            Some(d) if d < 0 => write!(f, "[{regs} - {neg}]", neg = -d),
            _ => write!(f, "[{regs}]"),
        }
    }
}

impl Display for Register {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let reg = match (self.code, self.word) {
            (Code::_000, false) => "al",
            (Code::_000, true) => "ax",
            (Code::_001, false) => "cl",
            (Code::_001, true) => "cx",
            (Code::_010, false) => "dl",
            (Code::_010, true) => "dx",
            (Code::_011, false) => "bl",
            (Code::_011, true) => "bx",
            (Code::_100, false) => "ah",
            (Code::_100, true) => "sp",
            (Code::_101, false) => "ch",
            (Code::_101, true) => "bp",
            (Code::_110, false) => "dh",
            (Code::_110, true) => "si",
            (Code::_111, false) => "bh",
            (Code::_111, true) => "di",
        };
        write!(f, "{reg}")
    }
}

impl Display for Accumulator {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.word {
            false => write!(f, "al"),
            true => write!(f, "ax"),
        }
    }
}

impl Display for MoveInstruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "mov {dest}, {source}",
            dest = self.destination,
            source = self.source
        )
    }
}
