//! WebView responsible for rendering the provided url in a webview.
use clap::{App, Arg};
use web_view;

fn main() {
    set_dpi_aware();
    let matches = App::new("webview")
        .about("render a webpage in an OS provided webview")
        .arg(
            Arg::with_name("title")
                .required(true)
                .takes_value(true)
                .help("title for the webview window"),
        )
        .arg(
            Arg::with_name("url")
                .required(true)
                .takes_value(true)
                .help("url of webpage to render"),
        )
        .arg(
            Arg::with_name("width")
                .takes_value(true)
                .help("initial width of the webview")
                .short("w"),
        )
        .arg(
            Arg::with_name("height")
                .takes_value(true)
                .help("initial height of the webview")
                .short("h"),
        )
        .get_matches();
    let title = matches.value_of("title").unwrap();
    let url = matches.value_of("url").unwrap();
    let width = matches
        .value_of("width")
        .unwrap_or("800")
        .parse()
        .expect("invalid width");
    let height = matches
        .value_of("width")
        .unwrap_or("600")
        .parse()
        .expect("invalid height");
    web_view::builder()
        .title(&title)
        .size(width, height)
        .resizable(true)
        .user_data(())
        .invoke_handler(|_, _| Ok(()))
        .content(web_view::Content::Url(url))
        .run()
        .expect("running webview");
}

#[cfg(target_os = "windows")]
fn set_dpi_aware() {
    use winapi::um::shellscalingapi::{SetProcessDpiAwareness, PROCESS_SYSTEM_DPI_AWARE};
    unsafe { SetProcessDpiAwareness(PROCESS_SYSTEM_DPI_AWARE) };
}

#[cfg(not(target_os = "windows"))]
fn set_dpi_aware() {}
