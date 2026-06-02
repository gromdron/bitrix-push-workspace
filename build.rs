fn main() {
    prost_build::compile_protos(
        &[
            "proto/request.proto",
            "proto/sender.proto",
            "proto/receiver.proto",
        ],
        &["proto/"],
    )
    .unwrap();
}
