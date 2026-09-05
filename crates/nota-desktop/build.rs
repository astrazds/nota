use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=resources/nota.gresource.xml");
    println!("cargo:rerun-if-changed=resources/nota.css");
    println!("cargo:rerun-if-changed=resources/nota-icon.svg");

    let output = PathBuf::from(std::env::var_os("OUT_DIR").expect("Cargo always sets OUT_DIR"))
        .join("nota.gresource");
    let status = Command::new("glib-compile-resources")
        .arg("--sourcedir=resources")
        .arg(format!("--target={}", output.display()))
        .arg("resources/nota.gresource.xml")
        .status()
        .expect("glib-compile-resources is required to build nota-desktop");
    assert!(status.success(), "glib-compile-resources failed");
}
