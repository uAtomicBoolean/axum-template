//! Build script: compiles `styles/main.scss` into `static/css/main.css`
//! with the Dart Sass CLI, on every `cargo build` / `cargo run`.

use std::{
    env,
    io::ErrorKind,
    path::{Path, PathBuf},
    process::Command,
};

const INPUT: &str = "styles/main.scss";
const OUTPUT: &str = "static/css/main.css";

/// Executable names to try, in order. Windows ships Dart Sass as a batch
/// file: the official archive has `sass.bat`, an npm install `sass.cmd`.
#[cfg(windows)]
const EXE_NAMES: &[&str] = &["sass.bat", "sass.cmd", "sass.exe", "sass"];
#[cfg(not(windows))]
const EXE_NAMES: &[&str] = &["sass"];

fn main() {
    // Re-run this script (and recompile the crate) when these change.
    // `locales` is listed so that the .ftl files, embedded at compile time
    // by fluent-templates, do trigger a recompilation.
    println!("cargo:rerun-if-changed=styles");
    println!("cargo:rerun-if-changed=locales");

    // Unlike the application, the build script has to load `.env` itself.
    if Path::new(".env").exists() {
        println!("cargo:rerun-if-changed=.env");
        if let Err(e) = dotenvy::dotenv() {
            println!("cargo:error=could not read .env ({e}), please fix its contents.");
            panic!("Error while reading the .env file.");
        }
    }

    let style_generation = match env::var("PROFILE").as_deref() {
        Ok("release") => "compressed",
        _ => "expanded",
    };

    let from_path: Vec<PathBuf> = EXE_NAMES.iter().map(PathBuf::from).collect();
    if try_all(&from_path, style_generation) {
        return;
    }

    panic!(
        "Dart Sass not found.\n\
         Install Dart Sass (https://sass-lang.com/install) and make sure it is in the PATH."
    );
}

/// Tries each candidate in order. Returns true as soon as one compiled the
/// stylesheet, after reporting the warnings Dart Sass printed.
fn try_all(candidates: &[PathBuf], style: &str) -> bool {
    for sass in candidates {
        if let Ok(warnings) = run_sass(sass, style) {
            for line in warnings.lines().filter(|l| !l.trim().is_empty()) {
                println!("cargo:warning=sass: {line}");
            }
            return true;
        }
    }
    false
}

/// Runs Dart Sass once. `Ok` carries the warnings it printed.
fn run_sass(sass: &Path, style: &str) -> Result<String, ()> {
    let result = Command::new(sass)
        .arg(format!("--style={style}"))
        .arg("--no-source-map")
        .arg("--load-path=styles")
        .arg(INPUT)
        .arg(OUTPUT)
        .output();

    match result {
        Ok(out) if out.status.success() => Ok(String::from_utf8_lossy(&out.stderr).into_owned()),
        // Dart Sass ran and rejected the stylesheet: that is a real error.
        Ok(out) => panic!(
            "Dart Sass failed on {INPUT}:\n{}",
            String::from_utf8_lossy(&out.stderr)
        ),
        Err(e) if matches!(e.kind(), ErrorKind::NotFound | ErrorKind::PermissionDenied) => Err(()),
        Err(e) => panic!("could not run `{}`: {e}", sass.display()),
    }
}
