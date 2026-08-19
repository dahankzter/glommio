use std::{env, fs, path::*, process::Command};

use cc::Build;
use rustc_version::Channel;

fn main() {
    let project = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .canonicalize()
        .unwrap();

    let liburing = match env::var("GLOMMIO_LIBURING_DIR") {
        Ok(path) => PathBuf::from(path).canonicalize().unwrap(),
        Err(_) => {
            let _ = Command::new("git")
                .arg("submodule")
                .arg("update")
                .arg("--init")
                .status();

            project.join("liburing")
        }
    };

    // Run the configure script in OUT_DIR to get `compat.h`
    let configured_include = configure(&liburing);

    let src = liburing.join("src");

    // liburing
    Build::new()
        .file(src.join("setup.c"))
        .file(src.join("queue.c"))
        .file(src.join("syscall.c"))
        .file(src.join("register.c"))
        .flag("-D_GNU_SOURCE")
        .include(src.join("include"))
        .include(&configured_include)
        .extra_warnings(false)
        .compile("uring");

    // (our additional, linkable C bindings)
    Build::new()
        .file(project.join("rusturing.c"))
        .flag("-D_GNU_SOURCE")
        .include(src.join("include"))
        .include(&configured_include)
        .compile("rusturing");
    if rustc_version::version_meta()
        .map(|meta| Channel::Nightly == meta.channel)
        .unwrap_or(false)
    {
        println!("cargo:rustc-cfg=nightly");
    }
}

/// Configure liburing outside the crate source directory, and return the
/// include directory holding the headers it generated.
///
/// DataDog/glommio does this by copying just the configure script into
/// OUT_DIR. That does not work against the liburing revision vendored here:
/// the newer script sources Makefile.common and gives up before writing
/// `src/include/liburing/compat.h`, while still exiting 0. Copy the whole
/// tree and configure the copy instead.
fn configure(liburing: &Path) -> PathBuf {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap())
        .canonicalize()
        .unwrap();
    let configured = out_dir.join("liburing");
    copy_dir(liburing, &configured);

    Command::new("./configure")
        .current_dir(&configured)
        .output()
        .expect("configure script failed");

    let compat = configured.join("src/include/liburing/compat.h");
    assert!(
        compat.exists(),
        "configure did not generate {}",
        compat.display()
    );

    configured.join("src/include")
}

/// Recursively copy `from` onto `to`, replacing whatever was there before so a
/// stale configure run cannot leak into a fresh build.
fn copy_dir(from: &Path, to: &Path) {
    if to.exists() {
        fs::remove_dir_all(to).unwrap();
    }
    fs::create_dir_all(to).unwrap();

    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), &target).unwrap();
        }
    }
}
