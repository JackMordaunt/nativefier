#[cfg(windows)]
fn main() {
    use image::imageops;
    use std::path::Path;
    use winres::WindowsResource;
    // Check for icon.ico
    // if icon.ico doesnt exist, convert icon.png to icon.ico
    if !Path::new("../res/icon.ico").exists() {
        if Path::new("../res/icon.png").exists() {
            let src = image::open("../res/icon.png").unwrap();
            imageops::resize(&src, 255, 255, imageops::Lanczos3)
                .save("../res/icon.ico")
                .expect("saving icon file");
        }
    }
    WindowsResource::new()
        .set_icon("../res/icon.ico")
        .compile()
        .expect("compiling winres");
}

#[cfg(unix)]
fn main() {}
