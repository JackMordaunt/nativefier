use web_view::*;
use clap::{Arg, App};
use std::{
    env,
    fs,
    path::PathBuf,
    io::prelude::*,
    error::Error,
};
use handlebars::Handlebars;
use serde_json::json;

// TODO(jfm): Template this out with real app name. 
static PLIST: &'static str = r#"
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>CFBundleDevelopmentRegion</key>
	<string>English</string>
	<key>CFBundleExecutable</key>
	<string>{{executable}}</string>
	<key>CFBundleIdentifier</key>
	<string>com.nativefier.{{executable}}</string>
	<key>CFBundleName</key>
	<string>{{executable}}</string>
	<key>CFBundleSupportedPlatforms</key>
	<array>
		<string>MacOSX</string>
	</array>
	<key>NSSupportsSuddenTermination</key>
	<string>YES</string>
        <key>NSHighResolutionCapable</key>
        <string>True</string>
</dict>
</plist>

"#;

enum Mode {
    Generator,
    Generated,
}

impl Mode {
    fn detect() -> Result<Mode, Box<Error>> {
        // TODO(jfm): implement for Windows/Linux.
        for c in env::current_exe()?.canonicalize()?.components() {
            if c.as_os_str().to_string_lossy().contains(".app") {
                return Ok(Mode::Generated);
            }
        }
        return Ok(Mode::Generator);
    }
}

fn generator() {
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
        .get_matches();
    let title = matches.value_of("title").expect("parsing title");
    let url = matches.value_of("url").expect("parsing url");
    // TODO(jfm): bundle for Windows/Linux.
    bundle_darwin(title, url).expect("bundling site into MacOS .app bundle");
}

fn bundle_darwin(title: &str, url: &str) -> Result<(), Box<Error>> {
    let app = PathBuf::from(format!("{0}.app/Contents/MacOS/{0}", title));
    let plist = PathBuf::from(format!("{0}.app/Contents/Info.plist", title));
    fs::create_dir_all(app.parent().unwrap())?;
    fs::copy(env::current_exe()?.to_path_buf(), app)?;
    let h = Handlebars::new();
    fs::File::create(plist)?.write(h.render_template(PLIST, &json!({"executable": title}))?.as_bytes())?;
    // TODO(jfm): Write wrapper bash script to pass url to the binary as a flag.  
    Ok(())
}

fn generated() {
    let (title, url) = find_config().unwrap();
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

fn find_config() -> Result<(String, String), Box<Error>> {
    Ok((String::from("SoundCloud"), String::from("https://soundcloud.com/discover")))
}

fn main() {
    match Mode::detect().expect("detecting execution mode") {
        Mode::Generator => generator(),
        Mode::Generated => generated(),
    }
}