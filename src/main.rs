use web_view::*;
use clap::{Arg, App};

mod bundle;
mod infer;

use crate::bundle::{
    Bundler,
};

fn main() {
    let matches = App::new("nativefier")
        .version("0.0.1")
        .author("Jack Mordaunt <jackmordaunt@gmail.com>")
        .about("create native apps for your favourite site")
        .arg(Arg::with_name("title")
            .required(true)
            .takes_value(true)
            .help("title of site"))
        .arg(Arg::with_name("url")
            .required(true)
            .takes_value(true)
            .help("url of site to nativefy"))
        .arg(Arg::with_name("generate")
            .short("g")
            .long("generate")
            .help("generate the native app"))
        .arg(Arg::with_name("dir")
            .short("d")
            .long("dir")
            .takes_value(true)
            .conflicts_with("run")
            .help("output directory for generated app, defaults to current directory"))
        .get_matches();
    let title = matches.value_of("title").expect("parsing title");
    let url = matches.value_of("url").expect("parsing url");
    let dir = matches.value_of("dir").unwrap_or("");
    let mode = match matches.value_of("generate") {
        Some(_) => Mode::Generator,
        None => Mode::Generated,
    };
    match mode {
        Mode::Generator => {
            if cfg!(windows) {
                bundle::Windows {
                    dir: &dir,
                    title: &title,
                    url: &url,
                }.bundle().expect("bundling Windows app");
            } else {
                bundle::Darwin {
                    dir: &dir,
                    title: &title,
                    url: &url,
                }.bundle().expect("bundling MacOS app");
            }
        },
        Mode::Generated => {
            let wv = web_view::builder()
                .title(&title)
                .content(Content::Url(&url))
                .size(800, 600)
                .resizable(true)
                .debug(true)
                .user_data(())
                .invoke_handler(|_wv, _arg| { Ok(()) } )
                .build()
                .expect("building webview");
            wv.run().expect("running webview");
        },
    }
}

// Mode specifies how to behave. 
enum Mode {
    Generator,
    Generated,
}
