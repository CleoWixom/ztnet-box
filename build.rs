// build.rs — собирает фронтенд в www/build/index.html
// В PART 1 фронтенд — заглушка. Реальная сборка в PART 3.
use std::{fs, path::Path};

fn main() {
    let out_dir = Path::new("www/build");
    let index = out_dir.join("index.html");

    // Создать директорию если нет
    if !out_dir.exists() {
        fs::create_dir_all(out_dir).expect("failed to create www/build");
    }

    // Записать заглушку только если файла нет
    if !index.exists() {
        fs::write(
            &index,
            "<!DOCTYPE html><html><head><title>ZeroBox</title></head>\
             <body><h1>ZeroBox — UI building...</h1></body></html>",
        )
        .expect("failed to write placeholder index.html");
    }

    // Пересобирать если изменился фронтенд
    println!("cargo:rerun-if-changed=www/src/");
    println!("cargo:rerun-if-changed=build.rs");
}
