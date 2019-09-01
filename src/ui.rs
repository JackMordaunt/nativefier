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
use std::sync::mpsc::{channel, Receiver, Sender};
use url::{ParseError, Url};
use web_view::{Content, WVResult, WebView};

// dispatch injects js that evaluates a call to the event dispatcher.
fn dispatch(wv: &mut WebView<()>, event: &Event) -> WVResult {
    use web_view::Error;
    let js = format!(
        "Event.dispatch({})",
        serde_json::to_string(event)
            .map_err(|err| Error::custom(Box::new(format!("serializing event: {:?}", &err))))?,
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

// parse_url accepts absolute and relative urls.
fn parse_url(url: &str) -> Result<Url, Box<dyn Error>> {
    match url.parse() {
        Ok(u) => Ok(u),
        Err(ParseError::RelativeUrlWithoutBase) => parse_url(&format!("https://{}", url)),
        Err(err) => Err(format!("malformed url: {:?}", err).into()),
    }
}

fn build(name: String, url: &Url, directory: String) -> Result<(), Box<dyn ::std::error::Error>> {
    let icon = infer_icon(&url).map_err(|err| format!("inferring icon: {}", err))?;
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

impl Event {
    fn error<S: Into<String>>(msg: S) -> Self {
        Event::Error { msg: msg.into() }
    }
}

// App performs the "real" work in it's own thread.
// This allows us to separate the "ui" from the "application".
// The API is structured around actions and events.
struct App {
    actions: Receiver<Action>,
    events: Sender<Event>,
    default_path: PathBuf,
}

impl App {
    fn handle(&self, action: Action) -> Option<Event> {
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
            Action::Initialize => Some(Event::Initialized {
                platform: if cfg!(windows) { "windows" } else { "unix" }.into(),
                default_path: self.default_path.clone(),
            }),
            Action::Build {
                name,
                url,
                directory,
            } => match parse_url(&url).and_then(|u| build(name, &u, directory)) {
                Ok(_) => Some(Event::BuildComplete),
                Err(err) => Some(Event::error(format!("building app: {:?}", err))),
            },
            // Action::ChooseDirectory => {
            //     let path = wv
            //         .dialog()
            //         .choose_directory("Choose output directory", &self.default_path)
            //         .expect("selecting output directory")
            //         .unwrap_or_else(|| self.default_path.clone());
            //     Some(Event::DirectoryChosen { path })
            // }
            _ => None,
        }
    }

    fn start(self) {
        std::thread::spawn(move || {
            for action in &self.actions {
                if let Some(event) = self.handle(action) {
                    self.events.send(event).ok();
                }
            }
        });
    }
}

// Invoker closes over a channel to send Actions.
//
// Note: Allows precise capturing of state where a move closure
// would be too greedy.
struct Invoker {
    actions: Sender<Action>,
}

impl Invoker {
    fn handle(&self, _wv: &mut WebView<()>, msg: &str) -> WVResult {
        use web_view::Error;
        serde_json::from_str::<Action>(msg)
            .map_err(|err| Error::custom(Box::new(format!("deserializing json: {:?}", err))))
            .and_then(|action| {
                self.actions
                    .send(action)
                    .map_err(|err| Error::custom(Box::new(format!("sending action: {:?}", err))))
            })
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
    let (event_tx, event_rx) = channel::<Event>();
    let (action_tx, action_rx) = channel::<Action>();
    let app = App {
        actions: action_rx,
        events: event_tx,
        default_path: dirs::desktop_dir().expect("loading desktop directory"),
    };
    app.start();
    let mut wv = web_view::builder()
        .title("nativefier")
        .resizable(true)
        .size(400, 250)
        .content(Content::Html(html))
        .user_data(())
        .invoke_handler(|wv, msg| {
            Invoker {
                actions: action_tx.clone(),
            }
            .handle(wv, msg)
        })
        .build()?;
    loop {
        for event in event_rx.try_iter() {
            if let Err(err) = dispatch(&mut wv, &event) {
                error!("{:?}", err);
            }
        }
        if let Some(Err(err)) = wv.step() {
            error!("{:?}", err);
        }
    }
}
