use cosmic_ext_applet_trash::{init_tracing, trash};

fn main() {
    trash::adopt_host_data_home();
    init_tracing();

    for bin in trash::watch::bins() {
        println!("bin:    {}", bin.display());
    }

    println!("status: {:?}", trash::status());

    if std::env::args()
        .skip(1)
        .any(|argument| argument == "--empty")
    {
        let outcome = trash::empty();
        println!(
            "purged: {} item(s), {} failure(s)",
            outcome.purged, outcome.failed
        );
        println!("status: {:?}", trash::status());
    }
}
