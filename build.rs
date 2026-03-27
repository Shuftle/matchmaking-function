use std::{env, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    tonic_prost_build::configure()
        .build_client(false)
        .file_descriptor_set_path(out_dir.join("open-match2-descriptor.bin"))
        .compile_protos(
            // List of proto files to compile.
            &["./open-match2/proto/mmf.proto"],
            // List of directories where to find imports.
            &["./open-match2/proto"],
        )?;
    Ok(())
}
