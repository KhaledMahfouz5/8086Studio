//! `examples` — the bundled `.asm` programs, embedded at compile time.
//!
//! Per the architecture, the GUI is the only crate that knows about file
//! paths; embedding the examples keeps the binary self-contained and means
//! "File → Examples" works regardless of the current working directory.
//!
//! The set here is the Tier-1 list from `replica/examples-todo.md`. Each
//! file is written fresh under our own name — see `examples/README.md`.

use std::collections::BTreeMap;

/// `(name, source)` pairs in the order they appear in the picker.
pub const EMBEDDED: &[(&str, &str)] = &[
    ("hello.asm",               include_str!("../../../examples/hello.asm")),
    ("sum.asm",                 include_str!("../../../examples/sum.asm")),
    ("1_sample.asm",            include_str!("../../../examples/1_sample.asm")),
    ("int21.asm",               include_str!("../../../examples/int21.asm")),
    ("keybrd.asm",              include_str!("../../../examples/keybrd.asm")),
    ("count_chars.asm",         include_str!("../../../examples/count_chars.asm")),
    ("loops.asm",               include_str!("../../../examples/loops.asm")),
    ("add_two_arrays.asm",      include_str!("../../../examples/add_two_arrays.asm")),
    ("sub_test.asm",            include_str!("../../../examples/sub_test.asm")),
    ("pushpop.asm",             include_str!("../../../examples/pushpop.asm")),
];

/// Pull a bundled example by name.
pub fn load_builtin(name: &str) -> Option<&'static str> {
    EMBEDDED.iter().find(|(n, _)| *n == name).map(|(_, s)| *s)
}

/// One-line purpose per example, written fresh (the original's text is not
/// copied). Used as the tooltip in the picker.
pub fn blurb(name: &str) -> &'static str {
    match name {
        "hello.asm"           => "Write a $-terminated string via INT 21h/09h.",
        "sum.asm"             => "Add two 16-bit numbers; exit with the low byte.",
        "1_sample.asm"        => "First lesson: MOV, INT 21h/02h, print one char.",
        "int21.asm"           => "The four common INT 21h sub-functions: 01/02/09/4C.",
        "keybrd.asm"          => "Read a keystroke through INT 16h and echo it.",
        "count_chars.asm"     => "Walk a $-terminated string, count the bytes.",
        "loops.asm"           => "LOOP instruction versus CMP + Jcc.",
        "add_two_arrays.asm"  => "Element-wise sum of two 5-byte arrays.",
        "sub_test.asm"        => "SUB and CMP, with a branch on the result.",
        "pushpop.asm"         => "Save and restore registers with PUSH/POP.",
        _ => "",
    }
}

/// Names only, sorted — what the picker shows.
pub fn names() -> BTreeMap<&'static str, &'static str> {
    EMBEDDED.iter().map(|(n, _)| (*n, blurb(n))).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_embedded_example_assembles() {
        for (name, src) in EMBEDDED {
            let r = studio_asm::assemble(src, studio_asm::OutputFormat::Com);
            assert!(r.is_ok(), "{name} failed to assemble: {:?}", r.err());
        }
    }

    #[test]
    fn load_builtin_round_trips() {
        assert!(load_builtin("hello.asm").is_some());
        assert!(load_builtin("nope.asm").is_none());
    }
}
