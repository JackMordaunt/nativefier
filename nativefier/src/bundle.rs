use crate::infer;
use icns;
use std::io::{BufWriter, Write};
use std::{error::Error, fs, path::PathBuf, process::Command};
use url::Url;

/// bundle for the current host OS.
#[cfg(target_os = "windows")]
pub fn bundle(
    dir: &str,
    name: &str,
    url: &Url,
    icon: Option<&infer::Icon>,
) -> Result<(), Box<dyn Error>> {
    Windows {
        dir,
        name,
        url,
        icon,
    }
    .bundle()
}

/// bundle for the current host OS.
#[cfg(target_os = "macos")]
pub fn bundle(
    dir: &str,
    name: &str,
    url: &Url,
    icon: Option<&infer::Icon>,
) -> Result<(), Box<dyn Error>> {
    Darwin {
        dir,
        name,
        url,
        icon,
    }
    .bundle()
}

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
    pub url: &'a Url,
    // Icon data.
    pub icon: Option<&'a infer::Icon>,
}

#[cfg(target_os = "macos")]
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
        let icon_path = app.join("Contents/Resources/icon.icns");
        for dir in ["Contents/MacOS", "Contents/Resources"].iter() {
            fs::create_dir_all(app.join(dir))?;
        }
        fs::File::create(app.join(format!("Contents/MacOS/{0}", &executable)))?
            .write_all(include_bytes!("../../target/release/webview"))?;
        fs::File::create(&plist)?.write_all(
            format!(
                include_str!("../../res/Info.plist"),
                executable = &executable,
            )
            .as_bytes(),
        )?;
        fs::File::create(&wrapper)?.write_all(
            format!(
                include_str!("../../res/launch.sh"),
                executable = &executable,
                title = &self.name,
                url = &self.url.as_str(),
            )
            .as_bytes(),
        )?;
        Command::new("chmod").arg("+x").arg(&wrapper).output()?;
        if let Some(icon) = self.icon {
            icns::Encoder::new(BufWriter::new(fs::File::create(&icon_path)?)).encode(&icon.img)?;
        }
        Ok(())
    }
}

// Windows bundles a windows executable.
pub struct Windows<'a> {
    pub dir: &'a str,
    pub name: &'a str,
    pub url: &'a Url,
    pub icon: Option<&'a infer::Icon>,
}

/// Bundler uses an executable "warp-packer" to create a standalone binary,
/// and "ResourceHacker" to write the icon to final binary.
/// Yeah, it's pretty hacky and bloaty. Still smaller than electron ;)
///  
/// Todo: remove dependency on warp-packer and ResourceHacker.
#[cfg(target_os = "windows")]
impl Bundler for Windows<'_> {
    fn bundle(self) -> Result<(), Box<dyn Error>> {
        use image::imageops::{resize, Lanczos3};
        let root = PathBuf::from(&self.dir);
        let workspace = root.join("tmp");
        let bundle = workspace.join(format!("{}.exe", &self.name));
        let packer = workspace.join("warp-packer.exe");
        let input = workspace.join(&self.name);
        let exec = input.join(format!("{}.exe", &self.name));
        let launcher = input.join("launch.bat");
        let icon_path = workspace.join("icon.ico");
        let rcedit = workspace.join("rcedit.exe");
        fs::create_dir_all(&input)?;
        // Fixme: Should this be a resource?
        fs::File::create(&exec)?.write_all(include_bytes!("../../target/release/webview.exe"))?;
        fs::File::create(&launcher)?.write_all(
            format!(
                include_str!("../../res/launch.bat"),
                name = &self.name,
                executable = format!("{}.exe", &self.name),
                url = &self.url,
            )
            .as_bytes(),
        )?;
        fs::File::create(&packer)?.write_all(include_bytes!("../../res/warp-packer.exe"))?;
        Command::new(&packer.to_string_lossy().as_ref())
            .arg("--arch")
            .arg("windows-x64")
            .arg("--input_dir")
            .arg(input.to_string_lossy().as_ref())
            .arg("--exec")
            .arg("launch.bat")
            .arg("--output")
            .arg(bundle.to_string_lossy().as_ref())
            .output()?;
        if let Some(icon) = self.icon {
            resize(&icon.img, 255, 255, Lanczos3).save(&icon_path)?;
            fs::File::create(&rcedit)?.write_all(include_bytes!("../../res/rcedit.exe"))?;
            Command::new(&rcedit.to_string_lossy().as_ref())
                .arg("-open")
                .arg(&bundle.to_string_lossy().as_ref())
                .arg("-save")
                .arg(&bundle.to_string_lossy().as_ref())
                .arg("-action")
                .arg("addoverwrite")
                .arg("-res")
                .arg(&icon_path.to_string_lossy().as_ref())
                .arg("-mask")
                .arg("ICONGROUP,1,1033")
                .output()?;
        }
        // Cleanup.
        fs::rename(&bundle, root.join(format!("{}.exe", &self.name)))?;
        fs::remove_dir_all(&workspace).map(|err| format!("removing temporary files: {:?}", err))?;
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    fn bundle(self) -> Result<(), Box<dyn Error>> {
        Err("cannot bundle windows application on this OS".into())
    }
}
