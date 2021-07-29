use std::path::Path;
use std::process::Command;

fn main() {
    // Watches for any modifications to the client's sources.
    println!("cargo:rerun-if-changed=../www/src");

    // The working directory.
    let wd = env!("CARGO_MANIFEST_DIR");

    // Builds the client.
    let res = Command::new("npm")
        .args(&["run", "build"])
        .current_dir(Path::new(wd).join("../www"))
        .status();

    if let Err(e) = res {
        println!("cargo:warning=Could not build client: {}", e)
    }
}