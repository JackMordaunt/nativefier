use std::env;
use std::error::Error;
use std::fs;
use url::Url;
use web_view::{Content, WebView};

fn main() -> Result<(), Box<dyn Error>> {
    let content = match env::args().nth(1) {
        Some(path) => match Url::parse(&path) {
            Ok(url) => Content::Url(url.into_string()),
            Err(_) => Content::Html(fs::read_to_string(path)?),
        },
        None => return Err("pass in path to html content as url or file".into()),
    };
    println!("content {:?}", content);
    let wv = web_view::builder()
        .title("nativefier")
        .resizable(true)
        .size(400, 300)
        .content(content)
        .user_data(())
        .invoke_handler(move |mut _webview: &mut WebView<()>, _arg: &str| Ok(()))
        .build()?;
    wv.run()?;
    Ok(())
}
