use std::{env, fs, io::Write, path::Path};

fn main() {
    gen_version_info();
}

fn gen_version_info() {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=Cargo.toml");
    let manifest_dir = Path::new(&env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("Cargo.toml");

    let data = fs::read_to_string(&manifest_dir).expect("Failed to read Cargo.toml");
    let data = toml::from_str::<toml::Table>(&data).expect("msg");

    let package_sect = data["package"]
        .as_table()
        .expect("Cargo.toml missing [package]");

    let metadata = (package_sect["metadata"].as_table())
        .and_then(|metadata| metadata["m64plugin"].as_table())
        .expect("[package.metadata.m64plugin] is required to generate plugin API constants");

    let plugin_name = (metadata["plugin_name"].as_str())
        .expect("missing plugin_name in [package.metadata.m64plugin]");
    let api_version = (metadata["api_version"].as_str())
        .expect("missing api_version in [package.metadata.m64plugin]")
        .parse::<semver::Version>()
        .expect("invalid api_version in [package.metadata.m64plugin");

    let version = (package_sect["version"].as_str())
        .expect("missing version in [package]")
        .parse::<semver::Version>()
        .expect("invalid version in [package]");

    if plugin_name.contains("######") {
        panic!("Who puts that many hashes in a plugin name??");
    }

    // generation

    let m64p_version = to_m64p_version(&version);
    let m64p_api_version = to_m64p_version(&api_version);

    let outfile = Path::new(&env::var_os("OUT_DIR").unwrap()).join("version_gen.rs");
    let mut writer = fs::File::create(outfile).expect("Failed to open generated file");

    write!(
        writer,
        "\
        pub const API_VERSION: i32 = 0x{:06X}; \n\
        pub const PLUGIN_VERSION: i32 = 0x{:06X}; \n\
        pub const PLUGIN_NAME: &'static ::std::ffi::CStr = cr######\"{}\"######;\n
        ",
        m64p_api_version, m64p_version, plugin_name
    )
    .unwrap();
}

fn to_m64p_version(ver: &semver::Version) -> i32 {
    let major: i32 =
        u8::try_from(ver.major).expect("M64+ major version capped to 255") as u32 as i32;
    let minor: i32 =
        u8::try_from(ver.minor).expect("M64+ minor version capped to 255") as u32 as i32;
    let patch: i32 =
        u8::try_from(ver.patch).expect("M64+ patch version capped to 255") as u32 as i32;

    (major << 16) | (minor << 8) | patch
}
