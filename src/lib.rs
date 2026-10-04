pub mod applet;
pub mod i18n;
pub mod trash;

pub const APP_ID: &str = "io.github.marcelogomes90.cosmic-ext-applet-trash";

pub fn init_tracing() {
    use tracing_subscriber::EnvFilter;

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("warn,cosmic_ext_applet_trash=info"));

    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .try_init();
}
