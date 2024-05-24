fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = tonic_build::configure().out_dir("src/proto-gen");

    config.compile(
        &[
            "proto/data.proto",
            "proto/services.proto",
            "proto/push.proto",
        ],
        &["proto/"],
    )?;
    Ok(())
}
