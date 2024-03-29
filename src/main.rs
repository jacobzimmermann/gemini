use adw::prelude::*;
use gtk4::glib;
use log;
use simple_logger;

mod about;
mod app;
mod res;
mod window;

fn main() -> glib::ExitCode {
    simple_logger::init_with_env().expect("Logger must initialise");

    match app::Gemini::builder().build() {
        Ok(app) => app.run(),
        Err(err) => {
            log::error!("{err}");
            glib::ExitCode::FAILURE
        }
    }
}
