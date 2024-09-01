use eframe::NativeOptions;

use app::App;

mod app;
mod element;
mod utils;

fn main() -> eframe::Result {
    eframe::run_native(
        "circuit.rs",
        NativeOptions::default(),
        Box::new(|_ctx| Ok(Box::new(App::default()))),
    )
}
