//! Build script: compiles `styles/main.scss` into `static/css/main.css`
//! with the Dart Sass CLI, on every `cargo build` / `cargo run`.

use std::{
    env, fs,
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
    println!("cargo:rerun-if-env-changed=SASS_BIN");

    // Unlike the application, the build script has to load `.env` itself.
    // Variables already set in the environment win, as dotenvy never overrides.
    if Path::new(".env").exists() {
        println!("cargo:rerun-if-changed=.env");
        // Don't fail the build over `.env`, but never drop the line silently:
        // an unquoted Windows path (`C:\dir`) is a parse error, since dotenvy
        // reads `\d` as an escape.
        if let Err(e) = dotenvy::dotenv() {
            println!("cargo:warning=could not read .env ({e}); its values are ignored");
        }
    }

    let style = match env::var("PROFILE").as_deref() {
        Ok("release") => "compressed",
        _ => "expanded",
    };

    fs::create_dir_all("static/css").expect("could not create static/css");

    // A custom location, when given, is tried before PATH.
    let custom = custom_candidates();
    if !custom.is_empty() {
        if try_all(&custom, style) {
            return;
        }
        // A SASS_BIN that leads nowhere is a configuration mistake, so say
        // so rather than silently falling back to PATH.
        println!(
            "cargo:warning=SASS_BIN ({}) is not a Dart Sass executable nor a folder \
             containing one: falling back to PATH",
            env::var("SASS_BIN").unwrap_or_default()
        );
    }

    let from_path: Vec<PathBuf> = EXE_NAMES.iter().map(PathBuf::from).collect();
    if try_all(&from_path, style) {
        return;
    }

    panic!(
        "Dart Sass not found.\n\
         Install Dart Sass (https://sass-lang.com/install), or point SASS_BIN at its \
         executable or at the folder that contains it."
    );
}

/// `SASS_BIN` either names the executable directly, or the folder holding it.
fn custom_candidates() -> Vec<PathBuf> {
    let Some(raw) = env::var_os("SASS_BIN") else {
        return Vec::new();
    };
    let path = PathBuf::from(raw);
    if path.as_os_str().is_empty() {
        return Vec::new();
    }

    if path.is_dir() {
        EXE_NAMES.iter().map(|name| path.join(name)).collect()
    } else {
        vec![path]
    }
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

/// The executable could not be started: it does not exist here.
struct NotFound;

/// Runs Dart Sass once. `Ok` carries the warnings it printed.
fn run_sass(sass: &Path, style: &str) -> Result<String, NotFound> {
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
        Err(e) if matches!(e.kind(), ErrorKind::NotFound | ErrorKind::PermissionDenied) => {
            Err(NotFound)
        }
        Err(e) => panic!("could not run `{}`: {e}", sass.display()),
    }
}
