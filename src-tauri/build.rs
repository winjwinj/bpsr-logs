use std::env;
use std::path::PathBuf;
use tauri_build::is_dev;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);

    // Generate protobuf code
    prost_build::Config::new()
        .type_attribute(".pb", "#[derive(specta::Type)]")
        .out_dir(manifest_dir.join("src/protocol"))
        .compile_protos(&["src/protocol/pb.proto"], &["src/protocol/"])?;

    println!("cargo:rerun-if-changed=src/protocol/pb.proto");

    // Tauri build
    if is_dev() {
        tauri_build::build();
    } else {
        // Run app as admin by default
        // https://github.com/tauri-apps/tauri/issues/7173#issuecomment-1584928815
        let windows =
            tauri_build::WindowsAttributes::new().app_manifest(include_str!("app.manifest"));

        tauri_build::try_build(tauri_build::Attributes::new().windows_attributes(windows))
            .expect("failed to run build script");
    }

    Ok(())
}
