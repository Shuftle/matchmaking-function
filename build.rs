fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_prost_build::compile_protos("./open-match2/proto/mmf.proto")?;
    Ok(())
}
