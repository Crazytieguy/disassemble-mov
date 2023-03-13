use crate::model::*;
use enum_dispatch::enum_dispatch;
use itoa::Buffer;

#[enum_dispatch]
pub(crate) trait FormatOnto {
    fn format_onto(&self, s: &mut String);
}

impl FormatOnto for i16 {
    fn format_onto(&self, s: &mut String) {
        s.push_str(Buffer::new().format(*self));
    }
}

impl FormatOnto for DataLiteral {
    fn format_onto(&self, s: &mut String) {
        use DataLiteral::*;
        match self {
            ExplicitByte(d) => {
                s.push_str("byte ");
                d.format_onto(s);
            }
            ExplicitWord(d) => {
                s.push_str("word ");
                d.format_onto(s);
            }
            Implicit(d) => {
                d.format_onto(s);
            }
        }
    }
}

impl FormatOnto for MemoryLiteral {
    fn format_onto(&self, s: &mut String) {
        s.push('[');
        self.0.format_onto(s);
        s.push(']');
    }
}

impl FormatOnto for SegmentRegister {
    fn format_onto(&self, s: &mut String) {
        use SegmentRegister::*;
        match self {
            _00 => s.push_str("es"),
            _01 => s.push_str("cs"),
            _10 => s.push_str("ss"),
            _11 => s.push_str("ds"),
        }
    }
}

impl FormatOnto for MemoryCalc {
    fn format_onto(&self, s: &mut String) {
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
                s.push('[');
                d.format_onto(s);
                s.push(']');
                return;
            }
            _110 => "bp",
            _111 => "bx",
        };
        s.push('[');
        match self.displacement {
            Some(d) if d > 0 => {
                s.push_str(regs);
                s.push_str(" + ");
                d.format_onto(s);
            }
            Some(d) if d < 0 => {
                s.push_str(regs);
                s.push_str(" - ");
                (-d).format_onto(s);
            }
            _ => s.push_str(regs),
        }
        s.push(']');
    }
}

impl FormatOnto for Register {
    fn format_onto(&self, s: &mut String) {
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
        s.push_str(reg);
    }
}

impl FormatOnto for Accumulator {
    fn format_onto(&self, s: &mut String) {
        match self.word {
            false => s.push_str("al"),
            true => s.push_str("ax"),
        }
    }
}

impl FormatOnto for MoveInstruction {
    fn format_onto(&self, s: &mut String) {
        s.push_str("mov ");
        self.destination.format_onto(s);
        s.push_str(", ");
        self.source.format_onto(s);
    }
}
