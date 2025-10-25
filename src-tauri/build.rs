use tauri_build::is_dev;

fn main() {
    if is_dev() {
        tauri_build::build();
    } else {
        // Run app as admin by default
        // https://github.com/tauri-apps/tauri/issues/7173#issuecomment-1584928815
        let windows = tauri_build::WindowsAttributes::new().app_manifest(include_str!("app.manifest"));

        tauri_build::try_build(tauri_build::Attributes::new().windows_attributes(windows))
            .expect("failed to run build script");
    }
}
