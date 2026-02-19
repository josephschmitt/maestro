fn main() {
    let frontend_dist = std::path::Path::new("../build");
    if !frontend_dist.exists() {
        std::fs::create_dir_all(frontend_dist).expect("failed to create frontend dist directory");
        std::fs::write(frontend_dist.join("index.html"), "<!doctype html><html><head></head><body></body></html>")
            .expect("failed to create placeholder index.html");
    }

    tauri_build::build()
}
