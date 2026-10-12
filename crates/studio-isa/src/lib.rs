//! Pure data types for the Intel 8086.
//!
//! This crate has zero behavioural logic and zero dependencies on other
//! workspace crates. Both `studio-asm` and `studio-cpu` depend on it.
//!
//! Per the unix covenant: data, not logic. Opcode tables, register
//! enums, and the flag bitflags live here as plain types.

#![forbid(unsafe_code)]

use bitflags::bitflags;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Reg16 { AX, CX, DX, BX, SP, BP, SI, DI }

impl Reg16 {
    pub const ALL: [Reg16; 8] = [
        Reg16::AX, Reg16::CX, Reg16::DX, Reg16::BX,
        Reg16::SP, Reg16::BP, Reg16::SI, Reg16::DI,
    ];
    pub fn name(self) -> &'static str {
        match self {
            Reg16::AX => "AX", Reg16::CX => "CX", Reg16::DX => "DX", Reg16::BX => "BX",
            Reg16::SP => "SP", Reg16::BP => "BP", Reg16::SI => "SI", Reg16::DI => "DI",
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Reg8 { AL, CL, DL, BL, AH, CH, DH, BH }

impl Reg8 {
    pub fn name(self) -> &'static str {
        match self {
            Reg8::AL => "AL", Reg8::CL => "CL", Reg8::DL => "DL", Reg8::BL => "BL",
            Reg8::AH => "AH", Reg8::CH => "CH", Reg8::DH => "DH", Reg8::BH => "BH",
        }
    }
    /// Map the 3-bit register field of an 8086 instruction to a `Reg8`.
    /// 0=AL, 1=CL, 2=DL, 3=BL, 4=AH, 5=CH, 6=DH, 7=BH.
    pub fn from_usize(i: usize) -> Reg8 {
        match i {
            0 => Reg8::AL, 1 => Reg8::CL, 2 => Reg8::DL, 3 => Reg8::BL,
            4 => Reg8::AH, 5 => Reg8::CH, 6 => Reg8::DH, 7 => Reg8::BH,
            _ => Reg8::AL,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum SegReg { ES, CS, SS, DS }

impl SegReg {
    pub const ALL: [SegReg; 4] = [SegReg::ES, SegReg::CS, SegReg::SS, SegReg::DS];
    pub fn name(self) -> &'static str {
        match self { SegReg::ES => "ES", SegReg::CS => "CS", SegReg::SS => "SS", SegReg::DS => "DS" }
    }
}

bitflags! {
    /// 8086 FLAGS register. Only the 9 active condition-code / control bits
    /// are defined; bits 1, 3, 5 and 12..15 are always 0 on a real 8086.
    #[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
    pub struct Flags: u16 {
        const CF = 1 << 0;
        const PF = 1 << 2;
        const AF = 1 << 4;
        const ZF = 1 << 6;
        const SF = 1 << 7;
        const TF = 1 << 8;
        const IF = 1 << 9;
        const DF = 1 << 10;
        const OF = 1 << 11;
    }
}

impl Flags {
    /// Order used by the Flags panel in the GUI.
    pub const PANE_ORDER: &'static [(Flags, &'static str)] = &[
        (Flags::OF, "OF"), (Flags::DF, "DF"), (Flags::IF, "IF"), (Flags::TF, "TF"),
        (Flags::SF, "SF"), (Flags::ZF, "ZF"), (Flags::AF, "AF"),
        (Flags::PF, "PF"), (Flags::CF, "CF"),
    ];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flag_layout_matches_intel_datasheet() {
        // OF=11, DF=10, IF=9, TF=8, SF=7, ZF=6, AF=4, PF=2, CF=0
        let f = Flags::OF | Flags::DF | Flags::IF | Flags::TF
              | Flags::SF | Flags::ZF | Flags::AF | Flags::PF | Flags::CF;
        assert_eq!(f.bits(), 0b0000_1111_1101_0101);
    }

    #[test]
    fn reg16_names_round_trip() {
        for r in Reg16::ALL {
            assert_eq!(r.name().len(), 2);
        }
    }
}
