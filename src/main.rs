mod app;
mod config;
mod player;
mod track;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([960.0, 540.0]),
        ..Default::default()
    };

    eframe::run_native(
        &format!("attention v{VERSION}"),
        options,
        Box::new(|_| Ok(Box::<app::Attention>::default())),
    )
    .expect("failed to launch app");
}
