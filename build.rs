#[cfg(windows)]
fn main() {
    use image::imageops;
    use std::path::Path;
    use winres::WindowsResource;
    // Check for icon.ico
    // if icon.ico doesnt exist, convert icon.png to icon.ico
    if !Path::new("res/icon.ico").exists() {
        if Path::new("res/icon.png").exists() {
            let src = image::open("res/icon.png").unwrap();
            let resized = imageops::resize(&src, 255, 255, imageops::Lanczos3);
            resized.save("res/icon.ico").unwrap();
        }
    }
    let mut res = WindowsResource::new();
    res.set_icon("res/icon.ico");
    res.compile().expect("compiling winres");
}

#[cfg(unix)]
fn main() {}
