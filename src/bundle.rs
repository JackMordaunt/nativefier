use crate::infer;
use icns;
use std::io::{BufWriter, Write};
use std::{env, error::Error, fs, path::PathBuf, process::Command};

/// Bundler is any object that can produce an executable bundle.
/// This allows us to be polymorphic across operating systems (macos, windows,
/// linux) and their various ways of handling an app bundle.
pub trait Bundler {
    fn bundle(self) -> Result<(), Box<dyn Error>>;
}

// Darwin bundles a macos app bundle.
pub struct Darwin<'a> {
    /// Output directory. Defaults to current working directory.
    pub dir: &'a str,
    /// Name of the application.
    pub name: &'a str,
    /// Url to wrap.
    pub url: &'a str,
    /// Filepath to icon.
    pub icon: infer::Icon,
}

impl Bundler for Darwin<'_> {
    fn bundle(self) -> Result<(), Box<dyn Error>> {
        let executable = self
            .name
            .chars()
            .filter(|c| !c.is_whitespace())
            .map(|c| c.to_ascii_lowercase())
            .collect::<String>();
        let app = PathBuf::from(&self.dir).join(format!("{0}.app", &self.name));
        let plist = app.join("Contents/Info.plist");
        let wrapper = app.join(format!("Contents/MacOS/{0}.sh", &executable));
        for dir in ["Contents/MacOS", "Contents/Resources"].iter() {
            fs::create_dir_all(app.join(dir))?;
        }
        fs::copy(
            env::current_exe()?.to_path_buf(),
            app.join(format!("Contents/MacOS/{0}", &executable)),
        )?;
        fs::File::create(&plist)?.write_all(
            format!(
                include_str!("../templates/Info.plist"),
                executable = &executable,
            )
            .as_bytes(),
        )?;
        fs::File::create(&wrapper)?.write_all(
            format!(
                include_str!("../templates/shell_wrapper.sh"),
                executable = &executable,
                title = &self.name,
                url = &self.url,
            )
            .as_bytes(),
        )?;
        Command::new("chmod").arg("+x").arg(&wrapper).output()?;
        let icon_path = app.join("Contents/Resources/icon.icns");
        let icon_file = fs::File::create(&icon_path)?;
        icns::Encoder::new(BufWriter::new(icon_file)).encode(&self.icon.img)?;
        Ok(())
    }
}

// Windows bundles a windows executable.
pub struct Windows<'a> {
    pub dir: &'a str,
    pub name: &'a str,
    pub url: &'a str,
}

/// Bundle nativefier executable using "iexpress", which is a Windows
/// program that creates self extracting installers.
/// In order to capture post-compilation information (ie, our arguments:
/// title and url) we embed it into a batch script that is then self extracted
/// and run.  
impl Bundler for Windows<'_> {
    /// TODO(jfm): compile icon.
    fn bundle(self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}
