use std::{env, fs, path::PathBuf};

use gl_generator::{Api, Fallbacks, Profile, Registry, StructGenerator};

fn gl_gen() {
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let out_path = out_dir.join("gl.gen.rs");

    Registry::new(Api::Gl, (4, 5), Profile::Compatibility, Fallbacks::All, [])
        .write_bindings(StructGenerator, &mut fs::File::create(out_path).unwrap())
        .expect("gl_generator failed");
}


fn win_manifest() {
    use embed_manifest::{self, manifest::{ActiveCodePage, DpiAwareness}};
    let manifest = embed_manifest::new_manifest("M64prs.Gtk")
        .dpi_awareness(DpiAwareness::PerMonitorV2)
        .active_code_page(ActiveCodePage::Utf8);
    embed_manifest::embed_manifest(manifest)
        .expect("manifest embed failed!");
}

fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=../m64prs-native/target");
    // copy_stuff_to_target();
    gl_gen();
    
    
    if std::env::var_os("CARGO_CFG_WINDOWS").is_some() {
        win_manifest();
        println!("cargo::warning=Windows is not supported at the moment due to some outstanding issues in GTK. Proceed with caution.")
    }

}
