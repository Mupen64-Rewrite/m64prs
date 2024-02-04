use bindgen::callbacks::ParseCallbacks;
use heck::ToPascalCase;
use reqwest::blocking as reqwest;
use std::{
    env, error::Error, fs::{self, File}, io::{self, Read, Seek}, path::{Path, PathBuf}
};
use zip::{result::{ZipError, ZipResult}, ZipArchive};

const CORE_RR_URL: &str = "https://github.com/Mupen64-Rewrite/mupen64plus-core-rr/archive/8954d83624d7a3ae0f600b634055702032b9266d.zip";
const CORE_RR_HEADERS: [&str; 1] = [
    "m64p_types.h"
];
const CORE_RR_ENUMS: [&str; 11] = [
    "m64p_type",
    "m64p_msg_level",
    "m64p_error",
    "m64p_plugin_type",
    "m64p_emu_state",
    "m64p_video_mode",
    "m64p_core_param",
    "m64p_command",
    "m64p_system_type",
    "m64p_rom_save_type",
    "m64p_disk_region"
];
const CORE_RR_BITFLAGS: [&str; 2] = [
    "m64p_core_caps",
    "m64p_video_flags"
];

fn zip_extract_cut_root<R: Read + Seek, P: AsRef<Path>>(zip_archive: &mut ZipArchive<R>, directory: P) -> ZipResult<()> {
    for i in 0..zip_archive.len() {
        let mut file = zip_archive.by_index(i)?;
        // let filepath = file
        //     .enclosed_name()
        //     .ok_or(ZipError::InvalidArchive("Invalid file path"))?;
        let filepath = match file.enclosed_name() {
            Some(name) => {
                // strip off exactly one component
                let mut i = name.iter();
                i.next();
                Ok(i.as_path())
            },
            None => Err(ZipError::InvalidArchive("Invalid file path"))
        }?;

        let outpath = directory.as_ref().join(filepath);

        if !filepath.starts_with("src/api") {
            continue;
        }
        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p)?;
                }
            }
            let mut outfile = fs::File::create(&outpath)?;
            io::copy(&mut file, &mut outfile)?;
        }
        // Get and Set permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Some(mode) = file.unix_mode() {
                fs::set_permissions(&outpath, fs::Permissions::from_mode(mode))?;
            }
        }
    }
    Ok(())
}

fn download_core_rr() -> Result<PathBuf, Box<dyn Error>> {
    // Paths
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let core_zip = out_dir.join("mupen64plus-core-rr.zip");
    let core_dir = out_dir.join("mupen64plus-core-rr");

    // Download mupen64plus-core-rr (too lazy to use git right now)
    {
        let mut resp = reqwest::get(CORE_RR_URL)?;
        let mut file_out = File::create(&core_zip)?;
        resp.copy_to(&mut file_out)?;
    }
    // extract m64p_types.h
    {
        let mut file_in = File::open(&core_zip)?;
        let mut zip_in = ZipArchive::new(&mut file_in)?;

        fs::create_dir_all(&core_dir)?;
        zip_extract_cut_root(&mut zip_in, &core_dir)?;
    }
    Ok(core_dir)
}

#[derive(Debug)]
struct M64PParseCallbacks;

impl ParseCallbacks for M64PParseCallbacks {
    fn enum_variant_name(
            &self,
            _enum_name: Option<&str>,
            _original_variant_name: &str,
            _variant_value: bindgen::callbacks::EnumVariantValue,
        ) -> Option<String> {
        if let Some(name) = _enum_name {
            if CORE_RR_ENUMS.contains(&name) || CORE_RR_BITFLAGS.contains(&name) {
                // mupen64plus prefixes all its enums because C, so we have to unprefix them
                let underscore_pos = _original_variant_name.find('_').unwrap();
                return Some(_original_variant_name[(underscore_pos + 1)..].to_owned())
            }
        }
        None
    }
    fn item_name(&self, _original_item_name: &str) -> Option<String> {
        match _original_item_name {
            "m64p_2d_size" => Some("Size2D".to_owned()),
            "m64p_GLattr" => Some("GLAttribute".to_owned()),
            "m64p_GLContextType" => Some("GLContextType".to_owned()),
            item => {
                if item.starts_with("m64p_") {
                    Some(_original_item_name[5..].to_pascal_case())
                }
                else {
                    None
                }
            }
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
        .parse_callbacks(Box::new(M64PParseCallbacks {}))
        .prepend_enum_name(false);

    // blocklist
    builder = builder
        .blocklist_type("m64p_dbg_.*")
        .blocklist_type("m64p_breakpoint");

    // add enums
    for name in CORE_RR_ENUMS {
        builder = builder.newtype_enum(name);
    }
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
        builder = builder
            .header(path_str.clone())
            .allowlist_file(path_str);
    }
    builder.generate()?.write_to_file(out_file)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let core_dir = download_core_rr()?;
    run_bindgen(&core_dir)?;

    Ok(())
}
