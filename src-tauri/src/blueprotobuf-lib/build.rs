use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let descriptor_path = out_dir.join("proto_descriptor.bin");

    prost_build::Config::new()
        .file_descriptor_set_path(&descriptor_path)
        .compile_well_known_types()
        .extern_path(".google.protobuf", "::pbjson_types")
        .type_attribute(".blueprotobuf_package", "#[derive(specta::Type)]")
        .out_dir(&manifest_dir.join("src"))
        .compile_protos(&["src/pb.proto"], &["src/"])?;

    let descriptor_set_bytes = std::fs::read(&descriptor_path)?;

    pbjson_build::Builder::new()
        .register_descriptors(&descriptor_set_bytes)?
        .build(&[".blueprotobuf_package"])?;

    let serde_source = out_dir.join("blueprotobuf_package.serde.rs");
    let serde_dest = manifest_dir.join("src/blueprotobuf_package.serde.rs");
    if serde_source.exists() {
        std::fs::copy(&serde_source, &serde_dest)?;
    }

    println!("cargo:rerun-if-changed=src/pb.proto");

    Ok(())
}
