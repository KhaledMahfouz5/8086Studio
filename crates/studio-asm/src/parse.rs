//! Text → AST: lexing, parsing, operand resolution.

use studio_isa::{Reg8, Reg16};
use super::*;

pub(crate) fn parse_statement(line: u32, s: &str, original: &str) -> Result<Statement, AsmError> {
    let upper = s.to_ascii_uppercase();
    let tokens: Vec<String> = upper.split_whitespace()
        .map(|t| t.trim_end_matches(',').to_owned())
        .collect();
    if tokens.is_empty() {
        return Ok(Statement::Directive(Directive::End));
    }
    // Special handling: bare-label-then-directive.
    if tokens.len() == 1 && (tokens[0] == "PROC" || tokens[0] == "ENDP") {
        // MASM procedure markers — v1: ignore them, treat as a no-op.
        return Ok(Statement::Directive(Directive::End));
    }
    let head = tokens[0].as_str();
    // If the line has a leading "name" + DB/DW/EQU, treat as the bare-label
    // form even if `head` isn't a known directive.
    if tokens.len() >= 3 && matches!(tokens[1].as_str(), "DB" | "DW" | "EQU") {
        // Find the position of the directive keyword (`tokens[1]`) in the
        // original (preserved-case) line so we can pass the rest of the
        // line — with its case intact — to the directive parser.
        let upper = original.to_ascii_uppercase();
        let kw = tokens[1].as_str();
        let kw_pos = upper.find(kw).ok_or_else(|| AsmError {
            line, message: format!("could not find `{}` in line", kw),
        })?;
        let raw = original[kw_pos + kw.len()..].trim();
        let dir = match tokens[1].as_str() {
            "DB" => {
                let data = parse_data_raw(line, raw)?;
                Directive::Db(data)
            }
            "DW" => {
                let data = parse_data_words(line, &tokens[2..])?;
                Directive::Dw(data)
            }
            "EQU" => {
                let v = parse_imm(line, &tokens[2])?;
                Directive::Equ(tokens[0].clone(), v)
            }
            _ => unreachable!(),
        };
        return Ok(Statement::Directive(dir));
    }
    match head {
        "ORG" => {
            let v = parse_imm(line, &tokens[1])?;
            Ok(Statement::Directive(Directive::Org(v)))
        }
        "DB" => {
            // For DB we want string literals preserved verbatim. Pull the
            // raw tokens from the original (pre-uppercase) source.
            let head_len = head.len();
            let raw = original[head_len..].trim();
            let data = parse_data_raw(line, raw)?;
            Ok(Statement::Directive(Directive::Db(data)))
        }
        "DW" => {
            let data = parse_data_words(line, &tokens[1..])?;
            Ok(Statement::Directive(Directive::Dw(data)))
        }
        "EQU" => {
            if tokens.len() != 3 {
                return Err(AsmError { line, message: "EQU expects `<name> EQU <value>`".into() });
            }
            let name = tokens[1].clone();
            let v = parse_imm(line, &tokens[2])?;
            Ok(Statement::Directive(Directive::Equ(name, v)))
        }
        "END" => Ok(Statement::Directive(Directive::End)),
        _ => {
            // Re-tokenize so `OFFSET X` becomes a single operand.
            let fixed = collapse_offset(&tokens);
            Ok(Statement::Instruction(parse_instr(line, &fixed)?))
        }
    }
}

/// Glue `OFFSET` + name into a single token so it counts as one operand.
fn collapse_offset(tokens: &[String]) -> Vec<String> {
    let mut out = Vec::with_capacity(tokens.len());
    let mut i = 0;
    while i < tokens.len() {
        if tokens[i] == "OFFSET" && i + 1 < tokens.len() {
            out.push(format!("OFFSET {}", tokens[i + 1]));
            i += 2;
        } else {
            out.push(tokens[i].clone());
            i += 1;
        }
    }
    out
}

/// Like `parse_data` but for `db` where strings should be preserved in their
/// original case. Numbers are uppercased before parsing.
/// Tokenises on whitespace while keeping `"..."` as a single token.
fn parse_data_raw(line: u32, raw: &str) -> Result<Vec<u8>, AsmError> {
    let mut out = Vec::new();
    for t in split_tokens(raw) {
        if t.starts_with('"') && t.ends_with('"') && t.len() >= 2 {
            let inner = &t[1..t.len()-1];
            out.extend(inner.as_bytes().to_vec());
        } else {
            let t = t.trim_end_matches(',');
            let v = parse_imm(line, &t.to_ascii_uppercase())?;
            out.push(v as u8);
        }
    }
    Ok(out)
}

/// Find the first `:` that is outside any quoted string.
pub(crate) fn find_label_colon(s: &str) -> Option<usize> {
    let mut in_quote = false;
    for (i, c) in s.char_indices() {
        match c {
            '"' => in_quote = !in_quote,
            ':' if !in_quote => return Some(i),
            _ => {}
        }
    }
    None
}

/// Split a raw directive body into tokens while keeping quoted strings whole.
fn split_tokens(raw: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut cur = String::new();
    let mut in_quote = false;
    let mut chars = raw.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '"' {
            in_quote = !in_quote;
            cur.push(c);
        } else if !in_quote && c.is_whitespace() {
            if !cur.is_empty() {
                tokens.push(std::mem::take(&mut cur));
            }
        } else {
            cur.push(c);
        }
    }
    if !cur.is_empty() {
        tokens.push(cur);
    }
    tokens
}

fn parse_data_words(line: u32, tokens: &[String]) -> Result<Vec<u16>, AsmError> {
    let mut out = Vec::new();
    for t in tokens {
        let v = parse_imm(line, t)?;
        out.push(v);
    }
    Ok(out)
}

fn parse_instr(line: u32, t: &[String]) -> Result<Instruction, AsmError> {
    let m = t[0].as_str();
    let arg_required = |i: usize| -> Result<Operand, AsmError> {
        if i >= t.len() {
            Err(AsmError { line, message: format!("`{m}` needs an operand") })
        } else {
            // MASM `offset X` → label X (address of X).
            let tok = &t[i];
            let trimmed = if let Some(rest) = tok.strip_prefix("OFFSET ") {
                rest
            } else {
                tok
            };
            parse_operand(line, trimmed)
        }
    };
    let reg_only = |i: usize| -> Result<Reg16, AsmError> {
        match arg_required(i)? {
            Operand::Reg16(r) => Ok(r),
            _ => Err(AsmError { line, message: format!("`{m}` expects a register") }),
        }
    };
    Ok(match m {
        "MOV"  => Instruction::Mov  { dst: arg_required(1)?, src: arg_required(2)? },
        "ADD"  => Instruction::Add  { dst: arg_required(1)?, src: arg_required(2)? },
        "SUB"  => Instruction::Sub  { dst: arg_required(1)?, src: arg_required(2)? },
        "CMP"  => Instruction::Cmp  { dst: arg_required(1)?, src: arg_required(2)? },
        "AND"  => Instruction::And  { dst: arg_required(1)?, src: arg_required(2)? },
        "OR"   => Instruction::Or   { dst: arg_required(1)?, src: arg_required(2)? },
        "XOR"  => Instruction::Xor  { dst: arg_required(1)?, src: arg_required(2)? },
        "INC"  => Instruction::Inc(arg_required(1)?),
        "DEC"  => Instruction::Dec(arg_required(1)?),
        "PUSH" => Instruction::Push(reg_only(1)?),
        "POP"  => Instruction::Pop(reg_only(1)?),
        "JMP"  => Instruction::JmpShort(arg_required(1)?),
        "JE" | "JZ"  => Instruction::JeShort(arg_required(1)?),
        "JNE" | "JNZ"=> Instruction::JneShort(arg_required(1)?),
        "JA" | "JNBE"=> Instruction::JaShort(arg_required(1)?),
        "JBE" | "JNA"=> Instruction::JbeShort(arg_required(1)?),
        "JB" | "JNAE"=> Instruction::JbShort(arg_required(1)?),
        "JAE" | "JNB"=> Instruction::JaeShort(arg_required(1)?),
        "JGE" | "JNL"=> Instruction::JgeShort(arg_required(1)?),
        "JL"  | "JNGE"=> Instruction::JlShort(arg_required(1)?),
        "CALL" => Instruction::Call(arg_required(1)?),
        "LOOP" => Instruction::LoopShort(arg_required(1)?),
        "LEA"  => Instruction::Lea(arg_required(1)?, arg_required(2)?),
        "SHR"  => Instruction::Shr(arg_required(1)?, arg_required(2)?),
        "INT"  => {
            let n = parse_imm(line, &t[1])?;
            if n > 0xFF { return Err(AsmError { line, message: format!("INT {n} out of range 0..=255") }); }
            Instruction::Int(n as u8)
        }
        "RET"  => Instruction::Ret,
        "HLT"  => Instruction::Hlt,
        "NOP"  => Instruction::Nop,
        other  => return Err(AsmError { line, message: format!("unknown mnemonic `{other}`") }),
    })
}

fn parse_operand(line: u32, s: &str) -> Result<Operand, AsmError> {
    // MASM `OFFSET name` → just the label address.
    if let Some(name) = s.strip_prefix("OFFSET ") {
        return Ok(Operand::Label(name.to_owned()));
    }
    // Memory operand in brackets: [NNNN], [reg], [label], or [label+NNNN].
    if s.starts_with('[') && s.ends_with(']') {
        let inner = s[1..s.len()-1].replace(' ', "");
        if let Ok(r) = parse_reg16_simple(&inner) {
            return Ok(Operand::MemReg(r));
        }
        if let Some(plus) = inner.find('+') {
            let name = &inner[..plus];
            let disp = parse_imm(line, &inner[plus+1..])?;
            if parse_imm(line, name).is_ok() {
                return Ok(Operand::Mem16(parse_imm(line, name)? + disp));
            }
            return Ok(Operand::MemLabelOff(name.to_owned(), disp));
        }
        if let Ok(v) = parse_imm(line, &inner) {
            return Ok(Operand::Mem16(v));
        }
        return Ok(Operand::MemLabelOff(inner, 0));
    }
    if let Ok(r) = parse_reg16_simple(s) {
        return Ok(Operand::Reg16(r));
    }
    if let Ok(r) = parse_reg8_simple(s) {
        return Ok(Operand::Reg8(r));
    }
    // Single-quoted character: 'A' → 0x41
    if s.starts_with('\'') && s.ends_with('\'') && s.len() >= 3 {
        let inner = &s[1..s.len()-1];
        if let Some(c) = inner.chars().next() {
            return Ok(Operand::Imm16(c as u16));
        }
    }
    if let Ok(v) = parse_imm(line, s) {
        return Ok(Operand::Imm16(v));
    }
    Ok(Operand::Label(s.to_owned()))
}

fn parse_reg8_simple(s: &str) -> Result<Reg8, AsmError> {
    match s {
        "AL" => Ok(Reg8::AL),
        "CL" => Ok(Reg8::CL),
        "DL" => Ok(Reg8::DL),
        "BL" => Ok(Reg8::BL),
        "AH" => Ok(Reg8::AH),
        "CH" => Ok(Reg8::CH),
        "DH" => Ok(Reg8::DH),
        "BH" => Ok(Reg8::BH),
        _ => Err(AsmError { line: 0, message: format!("not an 8-bit reg: {s}") }),
    }
}

fn parse_reg16_simple(s: &str) -> Result<Reg16, AsmError> {
    match s {
        "AX" => Ok(Reg16::AX),
        "CX" => Ok(Reg16::CX),
        "DX" => Ok(Reg16::DX),
        "BX" => Ok(Reg16::BX),
        "SP" => Ok(Reg16::SP),
        "BP" => Ok(Reg16::BP),
        "SI" => Ok(Reg16::SI),
        "DI" => Ok(Reg16::DI),
        _ => Err(AsmError { line: 0, message: format!("not a reg: {s}") }),
    }
}

fn parse_imm(line: u32, s: &str) -> Result<u16, AsmError> {
    if s.is_empty() {
        return Err(AsmError { line, message: "empty numeric literal".into() });
    }
    let (body, radix) = if let Some(rest) = s.strip_suffix('H') {
        (rest, 16)
    } else if let Some(rest) = s.strip_prefix("0X") {
        (rest, 16)
    } else {
        (s, 10)
    };
    if body.is_empty() {
        return Err(AsmError { line, message: format!("empty numeric literal `{s}`") });
    }
    u16::from_str_radix(body, radix).map_err(|_| AsmError {
        line, message: format!("bad numeric literal `{s}`"),
    })
}
