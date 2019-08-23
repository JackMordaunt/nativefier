use std::error::Error;
use web_view::{Content, WebView};

// TODO: Add `build.rs` which builds project `gui` then include! and serve the
// artifact into this webview.
fn main() -> Result<(), Box<dyn Error>> {
    let _wv = web_view::builder()
        .title("nativefier")
        .resizable(true)
        .size(1000, 500)
        .content(Content::Html(""))
        .user_data(())
        .invoke_handler(move |mut _webview: &mut WebView<()>, _arg: &str| Ok(()))
        .build()?;
    Ok(())
}
