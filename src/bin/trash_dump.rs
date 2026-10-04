use cosmic_ext_applet_trash::{init_tracing, trash};

fn main() {
    trash::adopt_host_data_home();
    init_tracing();

    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let wants = |flag: &str| arguments.iter().any(|argument| argument == flag);

    for bin in trash::watch::bins() {
        println!("bin:    {}", bin.display());
    }

    println!("status: {:?}", trash::status());

    if wants("--open") {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("a current-thread runtime");

        runtime.block_on(trash::open());
        println!("open:   asked the desktop to show the trash");
    }

    if wants("--empty") {
        let outcome = trash::empty();
        println!(
            "purged: {} item(s), {} failure(s)",
            outcome.purged, outcome.failed
        );
        println!("status: {:?}", trash::status());
    }
}
