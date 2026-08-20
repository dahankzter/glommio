//! The crate talks to io_uring through the pure-Rust `io-uring` crate, so
//! there is no C to compile: this build script exists only to detect a nightly
//! toolchain for the `nightly` feature's benefit.

use rustc_version::Channel;

fn main() {
    if rustc_version::version_meta()
        .map(|meta| Channel::Nightly == meta.channel)
        .unwrap_or(false)
    {
        println!("cargo:rustc-cfg=nightly");
    }
}
