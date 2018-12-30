use std::{
    env,
    fs,
    error::Error,
    path::PathBuf,
    io::prelude::*,
    process::Command,
};
use handlebars::Handlebars;
use serde_json::json;
use crate::infer;

/// Bundler is any object that can produce an executable bundle.
/// This allows us to be polymorphic across operating systems (macos, windows,
/// linux) and their various ways of handling an app bundle. 
pub trait Bundler {
    fn bundle(self) -> Result<(), Box<Error>>;
}

// Darwin bundles a macos app bundle. 
pub struct Darwin<'a> {
    /// Output directory. Defaults to current working directory. 
    pub dir: &'a str,
    /// Title of the application. 
    pub title: &'a str,
    /// Url to wrap. 
    pub url: &'a str,
    /// Filepath to icon.
    pub icon: infer::Icon,
}

impl Bundler for Darwin<'_> {
    fn bundle(self) -> Result<(), Box<Error>> {
        let executable = self.title.chars()
            .filter(|c| !c.is_whitespace())
            .map(|c| c.to_ascii_lowercase())
            .collect::<String>();
        println!("executable: {}", &executable);
        let app = PathBuf::from(&self.dir).join(format!("{0}.app", &self.title));
        for dir in ["Contents/MacOS", "Contents/Resources"].iter() {
            fs::create_dir_all(app.join(dir))?;
        }
        fs::copy(
            env::current_exe()?.to_path_buf(),
            app.join(format!("Contents/MacOS/{0}", &executable)),
        )?;
        let h = Handlebars::new();
        let plist = app.join("Contents/Info.plist");
        fs::File::create(&plist)?
            .write_all(h.render_template(PLIST.trim(), &json!({
                "executable": &executable,
                "url": &self.url,
            }))?.as_bytes())?;
        let wrapper = app.join(format!("Contents/MacOS/{0}.sh", &executable));
        fs::File::create(&wrapper)?
            .write_all(h.render_template(BASH_WRAPPER.trim(), &json!({
                "executable": &executable,
                "title": &self.title,
                "url": &self.url,
            }))?.as_bytes())?;
        Command::new("chmod")
            .arg("+x")
            .arg(&wrapper)
            .output()?;
        let icon_path = app.join("Contents/Resources/icon.png");
        fs::File::create(&icon_path)?.write_all(self.icon.into_png()?.as_ref())?;
        Command::new("icnsify")
            .arg("-i")
            .arg(&icon_path)
            .arg("-o")
            .arg(app.join("Contents/Resources/icon.icns"))
            .output()?;
        Ok(())
    }
}

// Windows bundles a windows executable. 
pub struct Windows<'a> {
    pub dir: &'a str,
    pub title: &'a str,
    pub url: &'a str,
}

/// Bundle nativefier executable using "iexpress", which is a Windows 
/// program that creates self extracting installers.
/// In order to capture post-compilation information (ie, our arguments:
/// title and url) we embed it into a batch script that is then self extracted
/// and run.  
impl Bundler for Windows<'_> {
    /// TODO(jfm): compile icon. 
    fn bundle(self) -> Result<(), Box<Error>> {
        fs::create_dir_all(&self.dir)?;
        let h = Handlebars::new();
        let bin = PathBuf::from(&self.dir).join(format!("{0}.exe", &self.title));
        let batch_file = PathBuf::from(&self.dir).join(format!("{0}.bat", self.title));
        let sed_file = PathBuf::from(&self.dir).join("tmp.sed");
        fs::copy(env::current_exe()?.to_path_buf(), &bin)?;
        fs::File::create(&batch_file)?
            .write_all(h.render_template(BATCH_WRAPPER.trim(), &json!({
                "executable": &bin,
                "title": &self.title,
                "url": &self.url,
            }))?.as_bytes())?;
        fs::File::create(&sed_file)?
            .write_all(h.render_template(SED_FILE.trim(), &json!({
                "name": &self.title,
                "executable": &format!("{0}.exe", &self.title),
                "entry_point": &batch_file,
                "source_directory": &self.dir,
                "target": PathBuf::from(&self.dir).join(format!("target_{0}.exe", &self.title)),
            }))?.as_bytes())?;
        Command::new("iexpress.exe")
            .arg("/N")
            .arg("/Q")
            .arg(&sed_file)
            .output()?;
        Ok(())
    }
}

/// .plist files are config files which MacOS .app bundles use. 
const PLIST: &str = r#"
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

/// Bash script that invokes the generated executable with the given arguments.
const BASH_WRAPPER: &str = r#"
#!/usr/bin/env bash
DIR=$(cd "$(dirname "$0")"; pwd)
"$DIR/{{executable}}" "{{title}}" "{{url}}"
"#;

/// .sed files are config files for "iexpress", which creates self extracting 
/// installers.
const SED_FILE: &str = r#"
[Version]
Class=IEXPRESS
SEDVersion=3
[Options]
PackagePurpose=InstallApp
ShowInstallProgramWindow=1
HideExtractAnimation=1
UseLongFileName=0
InsideCompressed=0
CAB_FixedSize=0
CAB_ResvCodeSigning=0
RebootMode=N
InstallPrompt=%InstallPrompt%
DisplayLicense=%DisplayLicense%
FinishMessage=%FinishMessage%
TargetName=%TargetName%
FriendlyName=%FriendlyName%
AppLaunched=%AppLaunched%
PostInstallCmd=%PostInstallCmd%
AdminQuietInstCmd=%AdminQuietInstCmd%
UserQuietInstCmd=%UserQuietInstCmd%
SourceFiles=SourceFiles
[Strings]
InstallPrompt=
DisplayLicense=
FinishMessage=
TargetName={{target}}
FriendlyName={{name}}
AppLaunched={{entry_point}}
PostInstallCmd=<None>
AdminQuietInstCmd=
UserQuietInstCmd=
FILE0="{{entry_point}}"
FILE1="{{executable}}"
[SourceFiles]
SourceFiles0={{parent_directory}}
[SourceFiles0]
%FILE0%=
%FILE1%=
"#;

/// Batch script that invokes the generated executable with the given arguments. 
const BATCH_WRAPPER: &str = r#"
cmd.exe /c start "{{executable}}" "{{title}}" "{{url}}"
"#;