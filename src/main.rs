use eframe::{egui, CreationContext};

fn main() -> eframe::Result<()> {
    let app_name: &str = "MyApp";
    let native_options: eframe::NativeOptions = eframe::NativeOptions::default();
    let app_creator: eframe::AppCreator =
        Box::new(|cc: &CreationContext<'_>| Box::new(MyEguiApp::new(cc)));
    eframe::run_native(app_name, native_options, app_creator)
}

#[derive(Default)]
struct MyEguiApp {}

impl MyEguiApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }
}

impl eframe::App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| ui.heading("Hello World!"));
    }
}
