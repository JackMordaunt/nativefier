mod bundle;
mod error;
mod infer;

use bundle::Bundler;
use dirs;
use infer::infer_icon;
use serde::{Deserialize, Serialize};
use serde_json;
use std::error::Error;
use std::path::PathBuf;
use web_view::{Content, WVResult, WebView};

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
enum Action {
    Build {
        name: String,
        url: String,
        directory: String,
    },
    ChooseDirectory,
    LoadConfig,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
enum Event {
    DirectoryChosen { path: PathBuf },
    ConfigLoaded { platform: String },
    BuildComplete,
}

fn main() -> Result<(), Box<dyn Error>> {
    set_dpi_aware();
    let html = format!(
        include_str!("ui/index.html"),
        style = format!("<style>{}</style>", include_str!("ui/style.css")),
        cash = format!("<script>{}</script>", include_str!("ui/cash.min.js")),
        app = format!("<script>{}</script>", include_str!("ui/app.js"),),
    );
    let wv = web_view::builder()
        .title("nativefier")
        .resizable(true)
        .size(400, 300)
        .content(Content::Html(html))
        .user_data(())
        .invoke_handler(move |wv: &mut WebView<()>, arg: &str| {
            match serde_json::from_str::<Action>(arg) {
                Ok(Action::LoadConfig) => {
                    dispatch(
                        wv,
                        &Event::ConfigLoaded {
                            platform: if cfg!(windows) { "windows" } else { "unix" }.into(),
                        },
                    )
                    .ok();
                }
                Ok(Action::Build {
                    name,
                    url,
                    directory,
                }) => {
                    build(name, url, directory).expect("building app");
                    dispatch(wv, &Event::BuildComplete).ok();
                }
                Ok(Action::ChooseDirectory) => {
                    let default = dirs::desktop_dir().expect("getting desktop directory");
                    let path = wv
                        .dialog()
                        .choose_directory("Choose output directory", &default)
                        .expect("selecting output directory")
                        .unwrap_or(default);
                    dispatch(wv, &Event::DirectoryChosen { path }).ok();
                }
                _ => {}
            };
            Ok(())
        })
        .build()?;
    wv.run()?;
    Ok(())
}

fn dispatch(wv: &mut WebView<()>, event: &Event) -> WVResult {
    let js = format!(
        "Event.dispatch(JSON.parse({}))",
        serde_json::to_string(event)
            .and_then(|s| serde_json::to_string(&s))
            .expect("serializing event"),
    );
    wv.eval(&js)
}

#[cfg(target_os = "windows")]
fn set_dpi_aware() {
    use winapi::um::shellscalingapi::{SetProcessDpiAwareness, PROCESS_SYSTEM_DPI_AWARE};
    unsafe { SetProcessDpiAwareness(PROCESS_SYSTEM_DPI_AWARE) };
}

fn set_dpi_aware() {}

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
