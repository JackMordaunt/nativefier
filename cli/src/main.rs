use clap::{App, Arg};
use nativefier::{bundle, infer_icon, infer_name};
use pretty_env_logger;
use url::Url;

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
    let dir = matches.value_of("output").unwrap_or("");
    let icon_url: Url = match matches.value_of("icon-override") {
        Some(icon_url) => icon_url.parse().expect("malformed URL"),
        None => url.clone(),
    };
    let icon = infer_icon(&icon_url).expect("inferring icon");
    // Todo: handle loading the appropriate webview binary for windows/unix
    bundle(
        &dir,
        &name,
        &url,
        Some(&icon),
    )
    .expect("bundling windows app");
}
