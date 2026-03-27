fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_prost_build::configure()
        .build_client(false)
        .compile_protos(
            // List of proto files to compile.
            &["./open-match2/proto/mmf.proto"],
            // List of directories where to find imports.
            &["./open-match2/proto"],
        )?;
    Ok(())
}
