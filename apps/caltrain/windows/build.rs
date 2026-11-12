//! Compile and bake `app.contract` into `OUT_DIR/app.plan` at build time, so
//! the executable carries its plan and opens with `host.windows.window`.

use contract::DataSource;

fn main() {
    contract::rerun_if_changed(std::path::Path::new("../app.contract"));
    println!("cargo:rerun-if-changed=build.rs");
    let plan = match contract::compile_path(std::path::Path::new("../app.contract")) {
        Ok(p) => p,
        Err(e) => panic!("app.contract:{e}"),
    };
    let entries = contract::snapshot::entries(std::path::Path::new("..")).unwrap();
    contract::snapshot::check_refreshes(&entries, &mut caltrain_data::Caltrain)
        .unwrap_or_else(|e| panic!("widget refresh: {e}"));
    let baked =
        contract::bake(plan, caltrain_data::Caltrain).unwrap_or_else(|e| panic!("bake: {e:?}"));
    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    std::fs::write(out_dir.join("app.plan"), baked.encode()).unwrap();
    println!("cargo:rerun-if-changed=../app.json");
    println!("cargo:rerun-if-changed=../data");
    println!("cargo:rerun-if-changed=../gpu/shaders");
    println!("cargo:rerun-if-changed=../../../Cargo.lock");
    let app_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let target = std::env::var("TARGET").unwrap_or_default();
    let platform = "windows";
    let manifest = contract::Manifest::read(&app_dir).unwrap_or_else(|e| panic!("app.json: {e}"));
    let grants = caltrain_data::Caltrain.grants();
    let compat = exact_bake::compatibility_id_sources_with_entries(
        &app_dir,
        platform,
        &target,
        &manifest,
        Some(grants),
        None,
        Some(&entries),
    )
    .unwrap_or_else(|e| panic!("compatibility id: {e}"));
    std::fs::write(out_dir.join("compat.json"), compat.to_json()).unwrap();
    let window = manifest
        .windows_window()
        .unwrap_or_else(|e| panic!("host.windows.window: {e}"));
    std::fs::write(
        out_dir.join("window.rs"),
        format!("/// The window the app opens in (`host.windows.window`).\nfn window() -> exact_windows::Window {{ {window} }}\n"),
    )
    .unwrap();
}
