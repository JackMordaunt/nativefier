// Build webview project first so that webview binary if available for 
// embedding with `include_bytes!`.
// 
// Only do this if the file doesn't exist already. 
//
// Note: This does mean that webview could become stale if we don't recompile 
// it after editing it. 
fn main() {
    use std::process::Command;
    use std::fs::File;
    let should_build = &[
            "../target/release/webview",
            "../target/release/webview.exe",
        ]
        .iter()
        .fold(true, |should_build, path| {
            if should_build == false {
                false
            } else {
                if let Err(_) = File::open(path) {
                    true
                } else {
                    false
                }
            }
        });
    if *should_build {
        Command::new("cargo")
            .arg("build")
            .arg("-p")
            .arg("webview")
            .arg("--release")
            .output()
            .expect("building webview project");
    }
}