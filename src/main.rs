use web_view::*;
use clap::{Arg, App, SubCommand};

mod bundle;
mod infer;

use crate::bundle::Bundler;

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
        .subcommand(SubCommand::with_name("generate")
            .about("generates a standalone binary")
            .arg(Arg::with_name("output")
                .short("o")
                .long("output")
                .takes_value(true)
                .help("output directory for generated app, defaults to current directory")))
        .get_matches();
    let title = matches.value_of("title").expect("parsing title");
    let url = matches.value_of("url").expect("parsing url");
    match matches.subcommand() {
        ("generate", args) => {
            let dir = match args {
                Some(args) => args.value_of("output").unwrap_or(""),
                None => "",
            };
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
        _ => {
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
        }
    };
}