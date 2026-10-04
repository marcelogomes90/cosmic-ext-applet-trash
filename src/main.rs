use cosmic_ext_applet_trash::{applet, i18n, init_tracing, trash};

fn main() -> cosmic::iced::Result {
    trash::adopt_host_data_home();
    init_tracing();
    i18n::init();
    tracing::info!(version = env!("CARGO_PKG_VERSION"), "starting trash");

    applet::run()
}
