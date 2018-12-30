mod bundle;
mod infer;
mod error;

use std::fs;
use web_view::*;
use clap::{Arg, App, SubCommand};
use tempfile::tempdir;
use pretty_env_logger;
use crate::bundle::Bundler;
use crate::infer::infer_icon;

fn main() {
    pretty_env_logger::init();
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
                .help("output directory for generated app, defaults to current directory"))
            .arg(Arg::with_name("icon-override")
                .short("f")
                .long("icon-override")
                .takes_value(true)
                .help("an alternative url to scrape the icon from")))
        .get_matches();
    let title = matches.value_of("title").expect("parsing title");
    let url = matches.value_of("url").expect("parsing url");
    match matches.subcommand() {
        ("generate", args) => {
            let (dir, icon_url) = match args {
                Some(args) => {
                    (
                        args.value_of("output").unwrap_or(""),
                        args.value_of("icon-override").unwrap_or(&url),
                    )
                },
                None => ("", url),
            };
            let icon = infer_icon(&icon_url)
                .expect("inferring icon")
                .into_png()
                .expect("converting icon to png");
            let icon_path = tempdir()
                .expect("opening temporary directory")
                .into_path()
                .join(format!("icon.{}", &icon.ext));
            fs::write(&icon_path, &icon).expect("writing icon to disk");
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
                    icon: &icon_path.to_string_lossy(),
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