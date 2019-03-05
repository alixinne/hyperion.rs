fn main() {
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
}
