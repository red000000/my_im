fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = tonic_build::configure().out_dir("proto-compiled");
    config.compile(
        &["proto/services.proto","proto/push.proto"],
        &["proto"],
    )?;
    Ok(())
}
