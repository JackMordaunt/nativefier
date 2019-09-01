mod bundle;
mod error;
mod infer;

use bundle::Bundler;
use dirs;
use infer::infer_icon;
use log::{error, trace};
use pretty_env_logger;
use serde::{Deserialize, Serialize};
use serde_json;
use std::error::Error;
use std::path::PathBuf;
use url::{ParseError, Url};
use web_view::{Content, WVResult, WebView};

fn dispatch(wv: &mut WebView<()>, event: &Event) -> WVResult {
    let js = format!(
        "Event.dispatch({})",
        serde_json::to_string(event).expect("serializing event"),
    );
    wv.eval(&js)
}

#[cfg(target_os = "windows")]
fn set_dpi_aware() {
    use winapi::um::shellscalingapi::{SetProcessDpiAwareness, PROCESS_SYSTEM_DPI_AWARE};
    unsafe { SetProcessDpiAwareness(PROCESS_SYSTEM_DPI_AWARE) };
}

#[cfg(not(target_os = "windows"))]
fn set_dpi_aware() {}

fn build(name: String, url: String, directory: String) -> Result<(), Box<dyn ::std::error::Error>> {
    let icon = infer_icon(&url.parse()?).map_err(|err| format!("inferring icon: {}", err))?;
    if cfg!(windows) {
        bundle::Windows {
            dir: &directory,
            name: &name,
            url: &url,
            icon: icon,
        }
        .bundle()
        .map_err(|err| format!("bundling Windows app: {}", err).into())
    } else {
        bundle::Darwin {
            dir: &directory,
            name: &name,
            url: &url,
            icon: icon,
        }
        .bundle()
        .map_err(|err| format!("bundling MacOS app: {}", err).into())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
enum Action {
    Build {
        name: String,
        url: String,
        directory: String,
    },
    ChooseDirectory,
    Initialize,
    Log {
        msg: String,
    },
    // Errors from the front-end.
    Error {
        msg: String,
        uri: Option<String>,
        line: Option<String>,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
enum Event {
    DirectoryChosen {
        path: PathBuf,
    },
    Initialized {
        platform: String,
        default_path: PathBuf,
    },
    BuildComplete,
    // Errors to display on the front-end.
    Error {
        msg: String,
    },
}

struct App {
    default_path: PathBuf,
}

impl App {
    // Todo:
    //  - Cleaner / Easier way to send application errors back to the frontend.
    //  - Do we bother handling transport errors?
    fn handle(&self, wv: &mut WebView<()>, action: Action) -> WVResult {
        match &action {
            Action::Log { msg } => {
                trace!("[  js  ] {}", msg.trim_matches('"'));
            }
            Action::Error { .. } => {
                error!("[  js  ] {:?}", action);
            }
            _ => {
                trace!("[action] {:?}", action);
            }
        };
        match action {
            Action::Initialize => {
                dispatch(
                    wv,
                    &Event::Initialized {
                        platform: if cfg!(windows) { "windows" } else { "unix" }.into(),
                        default_path: self.default_path.clone(),
                    },
                )
                .ok();
            }
            Action::Build {
                name,
                url,
                directory,
            } => {
                match (&url).parse::<Url>() {
                    Ok(_) => {
                        match build(name, url, directory) {
                            Ok(_) => dispatch(wv, &Event::BuildComplete).ok(),
                            Err(err) => dispatch(
                                wv,
                                &Event::Error {
                                    msg: format!("building app: {:?}", err),
                                },
                            )
                            .ok(),
                        };
                    }
                    Err(ParseError::RelativeUrlWithoutBase) => {
                        match build(name, format!("https://{}", url), directory) {
                            Ok(_) => dispatch(wv, &Event::BuildComplete).ok(),
                            Err(err) => dispatch(
                                wv,
                                &Event::Error {
                                    msg: format!("building app: {:?}", err),
                                },
                            )
                            .ok(),
                        };
                    }
                    Err(err) => {
                        dispatch(
                            wv,
                            &Event::Error {
                                msg: format!("malformed url: {:?}: {:?}", url, err),
                            },
                        )
                        .ok();
                    }
                };
            }
            Action::ChooseDirectory => {
                let path = wv
                    .dialog()
                    .choose_directory("Choose output directory", &self.default_path)
                    .expect("selecting output directory")
                    .unwrap_or_else(|| self.default_path.clone());
                dispatch(wv, &Event::DirectoryChosen { path }).ok();
            }
            _ => {}
        };
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    set_dpi_aware();
    pretty_env_logger::init();
    let html = format!(
        include_str!("ui/index.html"),
        style = format!(
            "<style>{}</style>",
            concat!(
                include_str!("ui/semantic.min.css"),
                include_str!("ui/style.css")
            )
        ),
        script = format!(
            "<script>{}</script>",
            concat!(
                include_str!("ui/jquery.min.js"),
                include_str!("ui/semantic.min.js"),
                include_str!("ui/app.js")
            )
        ),
    );
    let app = App {
        default_path: dirs::desktop_dir().expect("loading desktop directory"),
    };
    let wv = web_view::builder()
        .title("nativefier")
        .resizable(true)
        .size(400, 250)
        .content(Content::Html(html))
        .user_data(())
        .invoke_handler(move |wv, msg| {
            match serde_json::from_str::<Action>(msg)
                .map_err(|err| format!("deserializing json: {:?}", err))
            {
                Ok(action) => app.handle(wv, action),
                Err(err) => Err(web_view::Error::custom(err)),
            }
        })
        .build()?;
    wv.run()?;
    Ok(())
}
