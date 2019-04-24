use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, Write, Result};

fn main() -> Result<()> {
    // Generate file
    protobuf_codegen_pure::run(protobuf_codegen_pure::Args {
        // Note: we have to generate in src/ since the generated
        // code contains inner attributes, which cannot be included
        // in other files using include!.
        out_dir: &"src/servers/proto",
        input: &["proto/message.proto"],
        includes: &["proto"],
        customize: protobuf_codegen_pure::Customize {
            ..Default::default()
        },
    })
    .expect("protoc server/message.proto failed");

    // Patch file
    // See https://github.com/stepancheg/rust-protobuf/pull/332
    let in_name = "src/servers/proto/message.rs";
    let out_name = "src/servers/proto/message.new.rs";

    {
        let file = File::open(in_name)?;
        let mut out_file = File::create(out_name)?;

        for line in BufReader::new(file).lines() {
            let line = line?;

            if line == "#![allow(clippy)]" {
                write!(out_file, "#![allow(clippy::all)]\n")?;
            } else {
                write!(out_file, "{}\n", line)?;
            }
        }
    }

    fs::rename(out_name, in_name)?;

    Ok(())
}
