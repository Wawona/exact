//! Bake `app.contract` for the AsteroidOS platform identity.

use contract::DataSource;

fn main() {
    contract::rerun_if_changed(std::path::Path::new("../app.contract"));
    println!("cargo:rerun-if-changed=build.rs");
    let plan = match contract::compile_path(std::path::Path::new("../app.contract")) {
        Ok(p) => p,
        Err(e) => panic!("app.contract:{e}"),
    };
    let baked = contract::bake(plan, hello_data::Hello).unwrap_or_else(|e| panic!("bake: {e:?}"));
    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    std::fs::write(out_dir.join("app.plan"), baked.encode()).unwrap();

    println!("cargo:rerun-if-changed=../app.json");
    println!("cargo:rerun-if-changed=../data");
    println!("cargo:rerun-if-changed=../../../Cargo.lock");
    let app_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let target = std::env::var("TARGET").unwrap_or_default();
    let platform = "asteroid";
    let manifest = contract::Manifest::read(&app_dir).unwrap_or_else(|e| panic!("app.json: {e}"));
    let grants = hello_data::Hello.grants();
    let compat = exact_bake::compatibility_id(&app_dir, platform, &target, &manifest, Some(grants))
        .unwrap_or_else(|e| panic!("compatibility id: {e}"));
    std::fs::write(out_dir.join("compat.json"), compat.to_json()).unwrap();
    let host = "exact_asteroid";
    let (hatches, run) = contract::native::rust_hatch_entry(&app_dir, &manifest, &target)
        .unwrap_or_else(|e| panic!("hatches: {e}"));
    std::fs::write(
        out_dir.join("entry.rs"),
        contract::linux_entry(
            "hello_data::Hello",
            "hello_data::Hello",
            compat.inputs["rustMode"].as_str().unwrap(),
            host,
            &hatches,
            run,
        )
        .unwrap_or_else(|e| panic!("entry: {e}")),
    )
    .unwrap();
}
