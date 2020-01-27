use std::process::Command;

fn main() {
    prost_build::compile_protos(&["proto/message.proto"], &["proto"]).unwrap();

    if let Ok(output) = Command::new("git").args(&["describe", "--tags"]).output() {
        println!(
            "cargo:rustc-env=HYPERION_VERSION_ID=hyperion.rs {}",
            String::from_utf8(output.stdout).unwrap()
        );
    }
}
