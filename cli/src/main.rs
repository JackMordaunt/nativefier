#![windows_subsystem = "windows"]
use clap::{App, Arg, SubCommand};
use nativefier::{infer_icon, infer_name, Bundler, Darwin, Windows};
use pretty_env_logger;
use url::Url;
use web_view::*;

fn main() {
    pretty_env_logger::init();
    let matches = App::new("nativefier")
        .version("0.2.0")
        .author("Jack Mordaunt <jackmordaunt@gmail.com>")
        .about("Create native apps for your favourite site!")
        .arg(
            Arg::with_name("url")
                .required(true)
                .takes_value(true)
                .help("Url of site to nativefy"),
        )
        .arg(
            Arg::with_name("name")
                .takes_value(true)
                .short("n")
                .long("name")
                .help("Name of app"),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .takes_value(true)
                .help("Output directory for generated app, defaults to current directory"),
        )
        .arg(
            Arg::with_name("icon-override")
                .short("i")
                .long("icon-override")
                .takes_value(true)
                .help("Alternative url to scrape the icon from"),
        )
        .subcommand(
            SubCommand::with_name("inplace").about("Open the webview without creating an app"),
        )
        .get_matches();
    let url: Url = match matches.value_of("url").unwrap().parse() {
        Ok(url) => url,
        Err(_) => format!("https://{}", matches.value_of("url").unwrap())
            .parse()
            .expect("malformed URL"),
    };
    let name: String = match matches.value_of("name") {
        Some(name) => name.into(),
        None => infer_name(&url).expect("inferring name"),
    };
    match matches.subcommand() {
        ("inplace", _) => {
            let wv = web_view::builder()
                .title(&name)
                .content(Content::Url(&url))
                .size(800, 600)
                .resizable(true)
                .debug(true)
                .user_data(())
                .invoke_handler(|_wv, _arg| Ok(()))
                .build()
                .expect("building webview");
            wv.run().expect("running webview");
        }
        _ => {
            let dir = matches.value_of("output").unwrap_or("");
            let icon_url: Url = match matches.value_of("icon-override") {
                Some(icon_url) => icon_url.parse().expect("malformed URL"),
                None => url.clone(),
            };
            let icon = Some(infer_icon(&icon_url).expect("inferring icon"));
            if cfg!(windows) {
                Windows {
                    dir: &dir,
                    name: &name,
                    url: &url,
                    icon: icon,
                    executable: &[0],
                }
                .bundle()
                .expect("bundling Windows app");
            } else {
                Darwin {
                    dir: &dir,
                    name: &name,
                    url: &url,
                    icon: icon,
                    executable: &[0],
                }
                .bundle()
                .expect("bundling MacOS app");
            }
        }
    };
}
