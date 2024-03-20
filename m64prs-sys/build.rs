use bindgen::callbacks::ParseCallbacks;
use heck::ToPascalCase;
use std::{
    env,
    error::Error,
    path::{Path, PathBuf},
};

const CORE_RR_HEADERS: [&str; 3] = ["m64p_types.h", "m64p_vcr.h", "m64p_plugin.h"];
const CORE_RR_BITFLAGS: [&str; 2] = ["m64p_core_caps", "m64p_video_flags"];

#[derive(Debug)]
struct M64PParseCallbacks;

impl ParseCallbacks for M64PParseCallbacks {
    fn item_name(&self, _original_item_name: &str) -> Option<String> {
        match _original_item_name {
            // irregular type names
            "m64p_2d_size" => Some("Size2D".to_owned()),
            "m64p_GLattr" => Some("GLAttribute".to_owned()),
            "m64p_GLContextType" => Some("GLContextType".to_owned()),
            // BUTTONS and fields
            "BUTTONS" => Some("Buttons".to_owned()),
            // other items
            item if item.starts_with("m64p_") => Some(_original_item_name[5..].to_pascal_case()),
            _ => None,
        }
    }
}

fn run_bindgen<P: AsRef<Path>>(core_dir: P) -> Result<(), Box<dyn Error>> {
    // Paths
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let out_file = out_dir.join("types.gen.rs");

    let src_dir = core_dir.as_ref().join("src/");
    let api_dir = core_dir.as_ref().join("src/api/");

    let mut builder = bindgen::builder()
        .impl_debug(true)
        .clang_arg(format!("-I{}", src_dir.display()))
        .parse_callbacks(Box::new(::bindgen::CargoCallbacks::new().rerun_on_header_files(true)))
        .parse_callbacks(Box::new(M64PParseCallbacks {}))
        .default_enum_style(bindgen::EnumVariation::Consts)
        .prepend_enum_name(false);

    // blocklist
    builder = builder
        .blocklist_type(r"m64p_dbg_.*")
        .blocklist_type(r"m64p_breakpoint")
        .blocklist_type(r"ptr_.*")
        .blocklist_function(".*");

    // add bitflag enums
    for name in CORE_RR_BITFLAGS {
        builder = builder.bitfield_enum(name);
    }

    // add headers
    for header in CORE_RR_HEADERS {
        let path = api_dir.join(header);
        if !path.exists() {
            eprintln!("Header path `{:?}` not found.", path);
            continue;
        }
        let path_str = path.to_string_lossy();
        builder = builder.header(path_str.clone()).allowlist_file(path_str);
    }
    builder.generate()?.write_to_file(out_file)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let core_dir =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("../mupen64plus-core-rr");
    run_bindgen(&core_dir)?;

    Ok(())
}
