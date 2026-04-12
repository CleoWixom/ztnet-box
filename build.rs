// build.rs — assembles www/src/**  →  www/build/index.html
// Embedded into binary via include_str! in router.rs
use std::{
    fs,
    path::{Path, PathBuf},
};

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|_| String::new())
}

fn collect_files(dir: &Path, ext: &str) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        let mut entries: Vec<_> = entries.flatten().collect();
        entries.sort_by_key(|e| e.file_name());
        for entry in entries {
            let path = entry.path();
            if path.is_dir() {
                files.extend(collect_files(&path, ext));
            } else if path.extension().and_then(|s| s.to_str()) == Some(ext) {
                files.push(path);
            }
        }
    }
    files
}

fn main() {
    let src = Path::new("www/src");
    let out = Path::new("www/build");
    let dest = out.join("index.html");

    // Trigger rebuild on any www/src change
    println!("cargo:rerun-if-changed=www/src");
    println!("cargo:rerun-if-changed=build.rs");

    if !src.exists() {
        // No frontend source yet — write placeholder
        let _ = fs::create_dir_all(out);
        let _ = fs::write(
            &dest,
            "<!DOCTYPE html><html><head><meta charset=\"utf-8\"><title>ZeroBox</title></head>\
             <body><h1>ZeroBox</h1><p>Frontend building...</p></body></html>",
        );
        return;
    }

    let _ = fs::create_dir_all(out);

    // ── CSS (ordered) ──────────────────────────────────────────────────────
    let css_dir = src.join("css");
    let css_order = ["variables", "reset", "layout", "components", "pages"];
    let mut css = String::new();
    for name in css_order {
        let p = css_dir.join(format!("{name}.css"));
        if p.exists() {
            css.push_str(&read(&p));
            css.push('\n');
        }
    }
    // Any remaining CSS files not in order list
    for f in collect_files(&css_dir, "css") {
        let stem = f.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        if !css_order.contains(&stem) {
            css.push_str(&read(&f));
            css.push('\n');
        }
    }

    // ── JS (ordered: core → components → pages) ────────────────────────────
    // Core must come first: api/state/router are referenced by components and
    // pages. Shell HTML calls Router.on() / Router.start() at the very end,
    // so Router must already be defined when those calls execute.
    let js_dir = src.join("js");
    let comp_dir = js_dir.join("components");
    let page_dir = js_dir.join("pages");
    let mut js = String::new();

    // 1. Core scripts first (api → state → router)
    for name in ["api", "state", "router"] {
        let p = js_dir.join(format!("{name}.js"));
        if p.exists() {
            js.push_str(&read(&p));
            js.push('\n');
        }
    }
    // 2. Component scripts
    for f in collect_files(&comp_dir, "js") {
        js.push_str(&read(&f));
        js.push('\n');
    }
    // 3. Page scripts
    for f in collect_files(&page_dir, "js") {
        js.push_str(&read(&f));
        js.push('\n');
    }

    // ── Shell HTML ─────────────────────────────────────────────────────────
    let shell_path = src.join("html").join("shell.html");
    let shell = if shell_path.exists() {
        let version = env!("CARGO_PKG_VERSION");
        read(&shell_path).replace("{{VERSION}}", version)
    } else {
        "<div id=\"app\"><main id=\"content\"></main></div>\
         <div id=\"toast-container\"></div><div id=\"modal-overlay\"></div>"
            .to_string()
    };

    // ── Assemble index.html ────────────────────────────────────────────────
    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<meta name="color-scheme" content="dark">
<title>ZeroBox — ZeroTier Web UI</title>
<style>
{css}
</style>
</head>
<body>
{shell}
<script>
{js}
</script>
</body>
</html>
"#
    );

    fs::write(&dest, &html).expect("Failed to write www/build/index.html");

    let size_kb = html.len() / 1024;
    println!(
        "cargo:warning=Frontend built: {} KB ({} chars)",
        size_kb,
        html.len()
    );
}
