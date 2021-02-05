fn main() {
    let src_path = "src/servers/proto/message.proto";
    prost_build::compile_protos(&[src_path], &["src/servers/proto"]).unwrap();
}
