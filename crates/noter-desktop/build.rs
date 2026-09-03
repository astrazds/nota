use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=resources/noter.gresource.xml");
    println!("cargo:rerun-if-changed=resources/noter.css");
    println!("cargo:rerun-if-changed=resources/noter-icon.svg");

    let output = PathBuf::from(std::env::var_os("OUT_DIR").expect("Cargo always sets OUT_DIR"))
        .join("noter.gresource");
    let status = Command::new("glib-compile-resources")
        .arg("--sourcedir=resources")
        .arg(format!("--target={}", output.display()))
        .arg("resources/noter.gresource.xml")
        .status()
        .expect("glib-compile-resources is required to build noter-desktop");
    assert!(status.success(), "glib-compile-resources failed");
}
