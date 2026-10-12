use studio_gui::preview::PreviewApp;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([900.0, 800.0]),
        ..Default::default()
    };
    eframe::run_native(
        "8086Studio — design preview",
        native_options,
        Box::new(|cc| {
            let t = studio_gui::theme::DEFAULT_TOKENS;
            cc.egui_ctx.style_mut(|s| studio_gui::theme::apply(s, &t));
            Ok(Box::new(PreviewApp::default()))
        }),
    )
}
