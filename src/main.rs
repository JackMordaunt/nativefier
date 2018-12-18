use std::{
    env,
    error::Error,
    fs,
    path::PathBuf,
    io::prelude::*,
};
use web_view::*;
use clap::{Arg, App};
use handlebars::Handlebars;
use serde_json::json;

mod infer;

fn main() {
    match Mode::detect().expect("detecting execution mode") {
        Mode::Generator => {
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
            if cfg!(windows) {
                Windows {
                    dir: "",
                    title: title,
                    url: url,
                }.bundle().expect("bundling Windows app");
            } else {
                Darwin {
                    dir: "",
                    title: title,
                    url: url,
                }.bundle().expect("bundling MacOS app");
            }
        },
        Mode::Generated => {
            let config = Config::load().unwrap();
            let wv = web_view::builder()
                .title(&config.title)
                .content(Content::Url(&config.url))
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

impl Mode {
    fn detect() -> Result<Mode, Box<Error>> {
        for c in env::current_exe()?.canonicalize()?.components() {
            let cmp = c.as_os_str().to_string_lossy();
            if cmp.contains("nativefier.exe") {
                return Ok(Mode::Generator);
            }
            if cmp.contains(".app") {
                return Ok(Mode::Generated);
            }
        }
        return Ok(Mode::Generator);
    }
}

struct Config {
    title: String,
    url: String,
}

impl Config {
    fn load() -> Result<Config, Box<Error>> {
        if cfg!(windows) {
            let created = fs::File::open(env::current_exe()?.to_path_buf())?.metadata()?.created()?;
            let json_string = env::var(format!("nativefier_{:?}", created))?;
            let config: serde_json::Value = serde_json::from_str(&json_string)?;
            return Ok(Config{
                title: config["title"].to_string(),
                url: config["url"].to_string(),
            });
        } 
        if cfg!(unix) {
            let args: Vec<_> = env::args().collect();
            if args.len() < 2 {
                return Err("not enough arguments".into());
            }
            return Ok(Config{
                title: args[1].clone(),
                url: args[2].clone(),
            });
        }
        Err("unsupported platform".into())
    }
}

/// Bundler is any object that can produce an executable bundle.
/// This allows us to be polymorphic across operating systems (macos, windows,
/// linux) and their various ways of handling an app bundle. 
pub trait Bundler {
    fn bundle(&self) -> Result<(), Box<Error>>;
}

// Darwin bundles a macos app bundle. 
pub struct Darwin<'a> {
    pub dir: &'a str,
    pub title: &'a str,
    pub url: &'a str,
}

impl<'a> Bundler for Darwin<'a> {
    fn bundle(&self) -> Result<(), Box<Error>> {
        let app = PathBuf::from(format!("{0}.app", &self.title));
        for dir in vec!["Contents/MacOS", "Contents/Resources"] {
            fs::create_dir_all(app.join(dir))?;
        }
        fs::copy(
            env::current_exe()?.to_path_buf(),
            app.join(format!("Contents/MacOS/{0}", &self.title)),
        )?;
        let h = Handlebars::new();
        let plist = format!("{0}.app/Contents/Info.plist", &self.title);
        fs::File::create(&plist)?
            .write(h.render_template(PLIST.trim(), &json!({
                "executable": &self.title,
                "url": &self.url,
            }))?.as_bytes())?;
        let wrapper = format!("{0}.app/Contents/MacOS/{0}.sh", &self.title);
        fs::File::create(&wrapper)?
            .write(h.render_template(WRAPPER.trim(), &json!({
                "executable": &self.title,
                "title": &self.title,
                "url": &self.url,
            }))?.as_bytes())?;
        Command::new("chmod")
            .arg("+x")
            .arg(&wrapper)
            .output()?;
        let icon = infer::infer_icon(self.url)?;
        let icon_path = format!("{0}.app/Contents/Resources/icon.png", &self.title);
        fs::write(&icon_path, icon)?;
        use std::process::Command;
        Command::new("icnsify")
            .arg("-i")
            .arg(&icon_path)
            .arg("-o")
            .arg(format!("{0}.app/Contents/Resources/icon.icns", &self.title))
            .output()
            .expect("converting png to icns");
        // TODO(jfm): Write wrapper bash script to pass url to the binary as a flag.  
        Ok(())
    }
}

// Windows bundles a windows executable. 
pub struct Windows<'a> {
    pub dir: &'a str,
    pub title: &'a str,
    pub url: &'a str,
}

impl<'a> Bundler for Windows<'a> {
    /// TODO(jfm): compile icon. 
    fn bundle(&self) -> Result<(), Box<Error>> {
        let bin = format!("{}.exe", &self.title);
        fs::copy(env::current_exe()?.to_path_buf(), &bin)?;
        env::set_var(
            format!("nativefier_{:?}", fs::File::open(&bin)?.metadata()?.created()?),
            json!({"title": &self.title, "url": &self.url}).to_string(),
        );
        Ok(())
    }
}

static PLIST: &'static str = r#"
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>CFBundleExecutable</key>
		<string>{{executable}}.sh</string>
	<key>CFBundleIconFile</key>
		<string>icon.icns</string>
	<key>CFBundleIdentifier</key>
		<string>com.nativefier.{{executable}}</string>
	<key>NSHighResolutionCapable</key>
		<true/>
	<key>NSAppTransportSecurity</key>
		<dict>
			<key>NSExceptionDomains</key>
				<dict>
					<key>localhost</key>
						<dict>
							<key>NSExceptionAllowsInsecureHTTPLoads</key>
								<true/>
							<key>NSIncludesSubdomains</key>
								<true/>
						</dict>
				</dict>
		</dict>
</dict>
</plist>
"#;

static WRAPPER: &'static str = r#"
#!/usr/bin/env bash
DIR=$(cd "$(dirname "$0")"; pwd)
$DIR/{{executable}} {{title}} "{{url}}" 
"#;