//! `studio-debug` — the data types the debugger needs.
//!
//! v0 (vertical slice): just a `SourceMap` and a `Breakpoints` set. The
//! GUI's CodePanel + BreakpointDot widgets consume these.

#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use studio_asm::EncodedLine;

/// Source line ↔ address. Built from the assembler's `lines` output.
#[derive(Debug, Clone, Default)]
pub struct SourceMap {
    pub line_to_addr: Vec<(u32, u16)>,
    pub addr_to_line: Vec<(u16, u32)>,
}

impl SourceMap {
    pub fn from_asm(lines: &[EncodedLine]) -> Self {
        let mut line_to_addr = Vec::with_capacity(lines.len());
        let mut addr_to_line = Vec::with_capacity(lines.len());
        for l in lines {
            line_to_addr.push((l.line, l.address));
            addr_to_line.push((l.address, l.line));
        }
        Self { line_to_addr, addr_to_line }
    }

    pub fn addr_for_line(&self, line: u32) -> Option<u16> {
        self.line_to_addr
            .iter()
            .find(|(l, _)| *l == line)
            .map(|(_, a)| *a)
    }

    pub fn line_for_addr(&self, addr: u16) -> Option<u32> {
        self.addr_to_line
            .iter()
            .find(|(a, _)| *a == addr)
            .map(|(_, l)| *l)
    }
}

#[derive(Debug, Clone, Default)]
pub struct Breakpoints {
    pub addrs: BTreeSet<u16>,
}

impl Breakpoints {
    pub fn set(&mut self, addr: u16) {
        self.addrs.insert(addr);
    }

    pub fn clear(&mut self, addr: u16) {
        self.addrs.remove(&addr);
    }

    pub fn contains(&self, addr: u16) -> bool {
        self.addrs.contains(&addr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_map_round_trips_line_and_addr() {
        let lines = vec![
            EncodedLine { line: 1, address: 0x0100, bytes: vec![0xB8, 0x05, 0x00], source: "mov ax, 5".into() },
            EncodedLine { line: 2, address: 0x0103, bytes: vec![0xCD, 0x21], source: "int 21h".into() },
        ];
        let sm = SourceMap::from_asm(&lines);
        assert_eq!(sm.addr_for_line(2), Some(0x0103));
        assert_eq!(sm.line_for_addr(0x0100), Some(1));
    }

    #[test]
    fn breakpoints_toggle() {
        let mut bp = Breakpoints::default();
        bp.set(0x0100);
        assert!(bp.contains(0x0100));
        bp.clear(0x0100);
        assert!(!bp.contains(0x0100));
    }
}
