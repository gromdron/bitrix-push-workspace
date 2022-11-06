use std::io::Result;
fn main() -> Result<()> {
    prost_build::compile_protos(
        &[
            "proto/sender.proto",
            "proto/license.proto",
            "proto/notification.proto",
            "proto/receiver.proto",
            "proto/response.proto",
            "proto/request.proto",
        ],
        &["proto"],
    )?;
    Ok(())
}
