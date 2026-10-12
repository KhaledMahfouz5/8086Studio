//! `studio-asm` — 8086 Intel-syntax assembler.
//!
//! Two-pass: pass 1 records label addresses and statement positions;
//! pass 2 emits bytes, resolving label operands into 16-bit absolute
//! addresses (or 8-bit relative offsets for short branches). v1 supports
//! the mnemonics in `architecture.md § 1` plus the basic directives
//! `ORG`, `DB`, `DW`, `EQU`, `END`.

#![forbid(unsafe_code)]

use std::collections::HashMap;

use studio_isa::{Reg8, Reg16};
mod encode;
mod parse;
use encode::{encode_stmt, validate_labels_resolved};
use parse::{find_label_colon, parse_statement};

/// What the user (or the GUI's auto-detector) asked for.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum OutputFormat {
    Com,
}

/// One line of source: what we assembled, where, and how big it was.
#[derive(Debug, Clone)]
pub struct EncodedLine {
    pub line: u32,
    pub address: u16,
    pub bytes: Vec<u8>,
    pub source: String,
}

/// The full output of `assemble`.
#[derive(Debug, Clone)]
pub struct AsmResult {
    pub bytes: Vec<u8>,
    pub entry_cs: u16,
    pub entry_ip: u16,
    pub lines: Vec<EncodedLine>,
    pub labels: Vec<(String, u16)>,
    pub format: OutputFormat,
}

/// One assembly-time error. Carries the 1-based line + a clear sentence.
#[derive(Debug, Clone)]
pub struct AsmError {
    pub line: u32,
    pub message: String,
}

impl std::fmt::Display for AsmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Line {}: {}", self.line, self.message)
    }
}

/// Public entry point.
pub fn assemble(source: &str, format: OutputFormat) -> Result<AsmResult, AsmError> {
    // Pass 1: build the line list, recording label addresses and `EQU`
    // constants. Statement positions use worst-case estimates so labels
    // defined after a forward reference land at a valid address.
    let mut labels: HashMap<String, u16> = HashMap::new();
    let mut constants: HashMap<String, u16> = HashMap::new();
    let mut items: Vec<Item> = Vec::new();

    let mut current_addr: u16 = 0x0100;

    for (idx, raw_line) in source.lines().enumerate() {
        let line_no = (idx + 1) as u32;
        let trimmed = raw_line.split(';').next().unwrap_or("").trim();
        if trimmed.is_empty() { continue; }

        // Label(s) at head, possibly more than one per line.
        // We preserve original case for the body (so string literals survive)
        // but match labels case-insensitively. Skips colons inside quoted strings.
        let mut body = trimmed.to_owned();
        loop {
            let colon = find_label_colon(&body);
            match colon {
                Some(c) => {
                    let name = body[..c].trim().to_owned();
                    if name.is_empty() {
                        return Err(AsmError { line: line_no, message: "empty label before `:`".into() });
                    }
                    labels.insert(name.to_ascii_uppercase(), current_addr);
                    body = body[c+1..].trim().to_owned();
                }
                None => break,
            }
        }
        if body.is_empty() { continue; }

        // Bare-label-then-directive form: `msg db "..."` registers `msg`
        // before the directive. Common textbook idiom. Keep the original
        // case of the rest so string literals survive uppercasing.
        let first_ws = body.find(char::is_whitespace);
        let (first_tok, rest) = match first_ws {
            Some(n) => (body[..n].trim(), body[n..].trim_start()),
            None => (body.trim(), ""),
        };
        let second_upper = rest.split_whitespace().next().unwrap_or("").to_ascii_uppercase();
        if matches!(second_upper.as_str(), "DB" | "DW" | "EQU") {
            labels.insert(first_tok.to_ascii_uppercase(), current_addr);
        }

        let stmt = parse_statement(line_no, &body, trimmed)?;
        match &stmt {
            Statement::Directive(Directive::Org(v)) => current_addr = *v,
            Statement::Directive(Directive::Equ(name, v)) => { constants.insert(name.clone(), *v); }
            _ => {
                let item = Item {
                    line: line_no,
                    source: trimmed.to_owned(),
                    addr: current_addr,
                    stmt,
                };
                // Advance by the TRUE encoded length. Encoding here uses the
                // label map as-is; length depends only on operand shape (never
                // on a label's value), so this is exact and makes the two
                // passes agree by construction. A failure here is the same
                // failure the user would see in pass 2, so surface it now.
                let len = encode_stmt(&item, &labels, &constants)
                    .map(|b| b.len() as u16)?;
                items.push(item);
                current_addr = current_addr.wrapping_add(len);
            }
        }
    }

    // Validation: every label referenced by any instruction must be defined
    // somewhere. Pass 1 used placeholder addresses for forward references;
    // by this point the map is complete, so anything still missing is a real
    // error and fails loudly with its source line.
    for it in &items {
        validate_labels_resolved(it, &labels, &constants)?;
    }

    // After pass 1 every label's address is fixed. Pass 2 re-encodes with the
    // complete label map (forward references now resolve) and emits bytes.
    let mut bytes: Vec<u8> = Vec::new();
    let mut lines: Vec<EncodedLine> = Vec::new();

    for it in &items {
        let encoded = encode_stmt(it, &labels, &constants)?;
        lines.push(EncodedLine {
            line: it.line,
            address: it.addr,
            bytes: encoded.clone(),
            source: it.source.clone(),
        });
        bytes.extend(encoded);
    }

    Ok(AsmResult {
        bytes,
        entry_cs: 0,
        entry_ip: items.first().map(|i| i.addr).unwrap_or(0x0100),
        lines,
        labels: labels.into_iter().collect(),
        format,
    })
}

#[derive(Debug, Clone)]
enum Directive {
    Org(u16),
    Db(Vec<u8>),
    Dw(Vec<u16>),
    Equ(String, u16),
    End,
}

#[derive(Debug, Clone)]
enum Operand {
    Reg16(Reg16),
    Reg8(Reg8),
    Imm16(u16),
    Mem16(u16),
    MemReg(Reg16),
    MemLabelOff(String, u16),
    Label(String),
}

#[derive(Debug, Clone)]
enum Instruction {
    Mov { dst: Operand, src: Operand },
    Add { dst: Operand, src: Operand },
    Sub { dst: Operand, src: Operand },
    Cmp { dst: Operand, src: Operand },
    And { dst: Operand, src: Operand },
    Or  { dst: Operand, src: Operand },
    Xor { dst: Operand, src: Operand },
    Inc(Operand),
    Dec(Operand),
    Push(Reg16),
    Pop(Reg16),
    JmpShort(Operand),
    JeShort(Operand),
    JneShort(Operand),
    JaShort(Operand),
    JaeShort(Operand),
    JbShort(Operand),
    JbeShort(Operand),
    JgeShort(Operand),
    JlShort(Operand),
    Call(Operand),
    LoopShort(Operand),
    Lea(Operand, Operand),
    Shr(Operand, Operand),
    Int(u8),
    Ret,
    Hlt,
    Nop,
}

#[derive(Debug, Clone)]
enum Statement {
    Instruction(Instruction),
    Directive(Directive),
}

struct Item {
    line: u32,
    source: String,
    addr: u16,
    stmt: Statement,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mov_ax_imm5_encodes_to_b8_05_00() {
        let r = assemble("mov ax, 5", OutputFormat::Com).expect("ok");
        assert_eq!(r.bytes, vec![0xB8, 0x05, 0x00]);
        assert_eq!(r.entry_ip, 0x0100);
        assert_eq!(r.format, OutputFormat::Com);
    }

    #[test]
    fn mov_ax_hex_imm_encodes_correctly() {
        let r = assemble("mov ax, 1234h", OutputFormat::Com).expect("ok");
        assert_eq!(r.bytes, vec![0xB8, 0x34, 0x12]);
    }

    #[test]
    fn mov_cx_imm_uses_b9() {
        let r = assemble("mov cx, 0x0001", OutputFormat::Com).expect("ok");
        assert_eq!(r.bytes, vec![0xB9, 0x01, 0x00]);
    }

    #[test]
    fn int_21h_is_cd_21() {
        let r = assemble("int 21h", OutputFormat::Com).expect("ok");
        assert_eq!(r.bytes, vec![0xCD, 0x21]);
    }

    #[test]
    fn ret_is_c3() {
        let r = assemble("ret", OutputFormat::Com).expect("ok");
        assert_eq!(r.bytes, vec![0xC3]);
    }

    #[test]
    fn comments_are_stripped() {
        let r = assemble("mov ax, 7 ; set ax to 7", OutputFormat::Com).expect("ok");
        assert_eq!(r.bytes, vec![0xB8, 0x07, 0x00]);
    }

    #[test]
    fn unknown_mnemonic_is_an_error_with_line_number() {
        let e = assemble("wat ax, 5", OutputFormat::Com).expect_err("should fail");
        assert_eq!(e.line, 1);
        assert!(e.message.contains("WAT"), "message was: {}", e.message);
    }

    #[test]
    fn bad_register_is_an_error() {
        let e = assemble("mov qq, 5", OutputFormat::Com).expect_err("should fail");
        // `qq` is parsed as a label; no form of MOV accepts it as a source.
        assert!(e.message.contains("unsupported MOV form"), "message was: {}", e.message);
    }

    #[test]
    fn hlt_nop_ret_encode() {
        let r = assemble("hlt\nnop\nnop\nret", OutputFormat::Com).expect("ok");
        assert_eq!(r.bytes, vec![0xF4, 0x90, 0x90, 0xC3]);
    }

    #[test]
    fn inc_dec_encode() {
        let r = assemble("inc ax\ndec cx", OutputFormat::Com).expect("ok");
        assert_eq!(r.bytes, vec![0x40, 0x49]);
    }

    #[test]
    fn push_pop_encode() {
        let r = assemble("push ax\npop bx", OutputFormat::Com).expect("ok");
        assert_eq!(r.bytes, vec![0x50, 0x5B]);
    }

    #[test]
    fn add_ax_imm_uses_05() {
        let r = assemble("add ax, 1", OutputFormat::Com).expect("ok");
        assert_eq!(r.bytes, vec![0x05, 0x01, 0x00]);
    }

    #[test]
    fn add_cx_imm_uses_81_c1() {
        let r = assemble("add cx, 1", OutputFormat::Com).expect("ok");
        assert_eq!(r.bytes, vec![0x81, 0xC1, 0x01, 0x00]);
    }

    #[test]
    fn sub_ax_imm_uses_2d() {
        let r = assemble("sub ax, 5", OutputFormat::Com).expect("ok");
        assert_eq!(r.bytes, vec![0x2D, 0x05, 0x00]);
    }

    #[test]
    fn cmp_ax_imm_uses_3d() {
        let r = assemble("cmp ax, 0", OutputFormat::Com).expect("ok");
        assert_eq!(r.bytes, vec![0x3D, 0x00, 0x00]);
    }

    #[test]
    fn je_jne_jmp_loop_short_branches_encode() {
        let src = "
top:
    loop top
    jmp done
    je skip
    jne done
done:
    ret
skip:
    hlt
";
        let r = assemble(src, OutputFormat::Com).expect("ok");
        // LOOP rel8, JMP short rel8, JE rel8, JNE rel8, RET, HLT
        assert_eq!(r.bytes[0], 0xE2);
        // JMP short is EB
        let jmp_pos = 2;
        assert_eq!(r.bytes[jmp_pos], 0xEB);
        // JE is 74, JNE is 75
        let je_pos = jmp_pos + 2;
        assert_eq!(r.bytes[je_pos], 0x74);
        let jne_pos = je_pos + 2;
        assert_eq!(r.bytes[jne_pos], 0x75);
    }

    #[test]
    fn org_directive_advances_origin() {
        let r = assemble("org 0200h\nnop", OutputFormat::Com).expect("ok");
        assert_eq!(r.lines[0].address, 0x0200);
        assert_eq!(r.bytes, vec![0x90]);
    }

    #[test]
    fn db_directive_emits_bytes() {
        let r = assemble(r#"msg db "hi$""#, OutputFormat::Com).expect("ok");
        assert_eq!(r.bytes, vec![b'h', b'i', b'$']);
    }

    #[test]
    fn dw_directive_emits_little_endian_words() {
        let r = assemble("dw 1234h, 5678h", OutputFormat::Com).expect("ok");
        assert_eq!(r.bytes, vec![0x34, 0x12, 0x78, 0x56]);
    }

    #[test]
    fn forward_label_in_jmp_resolves() {
        let src = "jmp done\nnop\ndone:\nret";
        let r = assemble(src, OutputFormat::Com).expect("ok");
        // jmp short: EB rel8, where rel = done - (jmp_addr + 2).
        let jmp_addr = 0x0100;
        let done_addr = jmp_addr + 3; // jmp(2) + nop(1)
        let rel = (done_addr as i32 - (jmp_addr + 2) as i32) as i8;
        assert_eq!(r.bytes, vec![0xEB, rel as u8, 0x90, 0xC3]);
    }

    #[test]
    fn hello_world_works() {
        // The canonical DOS hello-world (INT 21h/09h).
        let src = "\
mov dx, msg
mov ah, 9
int 21h
ret
msg db \"hello, world$\"
";
        let r = assemble(src, OutputFormat::Com).expect("ok");
        // Layout: mov dx,0x0108 → BA 08 01, mov ah,9 → B4 09, int 21h → CD 21,
        // ret → C3, then "hello, world$" (13 bytes).
        assert_eq!(&r.bytes[0..3], &[0xBA, 0x08, 0x01]);
        assert_eq!(&r.bytes[3..5], &[0xB4, 0x09]);
        assert_eq!(&r.bytes[5..7], &[0xCD, 0x21]);
        assert_eq!(&r.bytes[7..8], &[0xC3]);
        assert_eq!(&r.bytes[8..], b"hello, world$");
    }
}

