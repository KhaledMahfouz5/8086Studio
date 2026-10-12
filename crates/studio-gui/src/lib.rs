//! Widgets — one file per primitive.
//!
//! Each function takes `&mut egui::Ui` and a `Tokens` and renders exactly
//! one widget in one state. No global state. No hidden inputs. Caller
//! controls the data; the widget paints it.

pub mod app;
pub mod examples;
pub mod preview;
pub mod theme;
pub mod widgets;
