//! Widgets — one file per primitive.
//!
//! Each function takes `&mut egui::Ui` and a `Tokens` and renders exactly
//! one widget in one state. No global state. No hidden inputs. Caller
//! controls the data; the widget paints it.

pub mod button;
pub mod tool_button;
pub mod register_cell;
pub mod flag_bit;
pub mod memory_cell;
pub mod breakpoint_dot;
pub mod line_gutter;
pub mod toolbar;
pub mod pane_tab_strip;
pub mod status_bar;
pub mod code_panel;
pub mod editor;
pub mod disassembler;
