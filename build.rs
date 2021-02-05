fn main() {
    let src_path = "src/servers/proto/message.proto";
    let dst_path = "src/servers/proto/message.rs";

    let src = std::fs::metadata(src_path)
        .expect("failed to read input")
        .modified()
        .expect("failed to get mtime");

    let dst = std::fs::metadata(dst_path).and_then(|d| d.modified()).ok();

    if dst.map(|dst| src > dst).unwrap_or(true) {
        protobuf_codegen_pure::Codegen::new()
            .out_dir("src/servers/proto")
            .inputs(&[src_path])
            .include("src/servers/proto")
            .run()
            .expect("protoc codegen failed");
    }
}
