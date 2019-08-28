mod bundle;
mod error;
mod infer;

// use base64;
use bundle::Bundler;
use infer::infer_icon;
use serde::{Deserialize, Serialize};
use serde_json;
use std::error::Error;
use std::path::PathBuf;
use web_view::{Content, WebView};

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
enum Action {
    Build {
        name: String,
        url: String,
        directory: String,
    },
    // Required since the web has no way to allowing user to select a directory.
    // Therefore, we need to implement our own dialogue.
    ChooseDirectory,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
enum Event {
    DirectoryChosen { path: PathBuf },
}

fn main() -> Result<(), Box<dyn Error>> {
    if cfg!(windows) {
        set_dpi_aware();
    }
    let html = format!(
        include_str!("ui/index.html"),
        style = format!("<style>{}</style>", include_str!("ui/style.css")),
        cash = format!("<script>{}</script>", include_str!("ui/cash.min.js")),
        app = format!("<script>{}</script>", include_str!("ui/app.js")),
    );
    let wv = web_view::builder()
        .title("nativefier")
        .resizable(true)
        .size(400, 300)
        .content(Content::Html(html))
        .user_data(())
        .invoke_handler(move |mut _wv: &mut WebView<()>, arg: &str| {
            match serde_json::from_str::<Action>(arg) {
                Ok(Action::Build {
                    name,
                    url,
                    directory,
                }) => {
                    build(name, url, directory).expect("building app");
                }
                _ => {}
            };
            Ok(())
        })
        .build()?;
    wv.run()?;
    Ok(())
}

#[cfg(target_os = "windows")]
fn set_dpi_aware() {
    use winapi::um::shellscalingapi::{SetProcessDpiAwareness, PROCESS_SYSTEM_DPI_AWARE};
    unsafe { SetProcessDpiAwareness(PROCESS_SYSTEM_DPI_AWARE) };
}

fn build(name: String, url: String, directory: String) -> Result<(), Box<dyn ::std::error::Error>> {
    if cfg!(windows) {
        bundle::Windows {
            dir: &directory,
            name: &name,
            url: &url,
        }
        .bundle()
        .map_err(|err| format!("bundling Windows app: {}", err).into())
    } else {
        bundle::Darwin {
            dir: &directory,
            name: &name,
            url: &url,
            icon: infer_icon(&url.parse()?).map_err(|err| format!("inferring icon: {}", err))?,
        }
        .bundle()
        .map_err(|err| format!("bundling MacOS app: {}", err).into())
    }
}
