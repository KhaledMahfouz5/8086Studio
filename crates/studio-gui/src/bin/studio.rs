//! `studio` — 8086Studio entry point. v0 is the vertical-slice App.

fn main() -> eframe::Result<()> {
    let app = studio_gui::app::App::new();
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "8086Studio",
        native_options,
        Box::new(|cc| {
            let t = studio_gui::theme::DEFAULT_TOKENS;
            cc.egui_ctx.style_mut(|s| studio_gui::theme::apply(s, &t));
            Ok(Box::new(app))
        }),
    )
}
