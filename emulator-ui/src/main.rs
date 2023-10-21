mod app;
mod audio;
mod utils;

fn main() {
    let mut gui = app::GuiApp::new();
    gui.run();
}
