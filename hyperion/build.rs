fn main() {
    prost_build::compile_protos(&["proto/message.proto"], &["proto"]).unwrap();
}
