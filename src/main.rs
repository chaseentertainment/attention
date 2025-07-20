mod app;
mod track;

fn main() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([960.0, 540.0]),
        ..Default::default()
    };

    eframe::run_native(
        "attention",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::<app::Attention>::default())
        }),
    )
    .expect("failed to launch app");
}
