// todo: any way to hide .proto file but keep the generated file?
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let proto_files = &[
    //     "src/BlueProtobuf.proto",
    // ];
    //
    // // Tell cargo to recompile if any of these proto files are changed
    // for proto_file in proto_files {
    //     println!("cargo:rerun-if-changed={proto_file}");
    // }
    //
    // let file_descriptor_set = protox::Compiler::new(["."])?
    //     .include_source_info(true)
    //     .include_imports(true)
    //     .open_files(proto_files)?
    //     .file_descriptor_set();
    //
    // let file_descriptor_set_bytes = file_descriptor_set.encode_to_vec();
    //
    // prost_build::Config::new()
    //     // Override prost-types with pbjson-types
    //     .compile_well_known_types()
    //     .type_attribute(".blueprotobuf_package", "#[derive(specta::Type)]")
    //     .extern_path(".google.protobuf", "::pbjson_types")
    //     .compile_fds(file_descriptor_set)?;
    //
    // pbjson_build::Builder::new()
    //     .register_descriptors(&file_descriptor_set_bytes)?
    //     .build(&[".blueprotobuf_package"])?;

    Ok(())
}
