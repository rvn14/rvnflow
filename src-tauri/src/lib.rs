pub fn run() {
    if let Err(error) = tauri::Builder::default().run(tauri::generate_context!()) {
        eprintln!("RVNFlow could not start: {error}");
    }
}
