#[cfg(windows)]
fn main() {
    use image::imageops;
    use std::path::Path;
    use winres;
    // Check for icon.ico
    // if icon.ico doesnt exist, convert icon.png to icon.ico
    if !Path::new("assets/icon.ico").exists() {
        if Path::new("assets/icon.png").exists() {
            let src = image::open("assets/icon.png").unwrap();
            let resized = imageops::resize(&src, 255, 255, imageops::Lanczos3);
            resized.save("assets/icon.ico").unwrap();
        }
    }
    let mut res = winres::WindowsResource::new();
    res.set_icon("assets/icon.ico");
    res.compile().expect("compiling winres");
}

#[cfg(unix)]
fn main() {}
