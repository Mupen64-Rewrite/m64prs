use bindgen::callbacks::{DeriveInfo, ParseCallbacks, TypeKind};
use heck::{ToPascalCase, ToShoutySnakeCase};
use std::{
    env, error::Error, ffi::OsString, fmt::Write, path::{Path, PathBuf}, process::{Command, Stdio}, sync::LazyLock
};

const CORE_RR_HEADERS: [&str; 3] = ["m64p_types.h", "m64p_tas.h", "m64p_plugin.h"];
const CORE_RR_BITFLAGS: [&str; 2] = ["m64p_core_caps", "m64p_video_flags"];
const CORE_RR_BITFLAGS_RUST: [&str; 2] = ["CoreCaps", "VideoFlags"];

#[derive(Debug)]
struct M64PParseCallbacks;

impl ParseCallbacks for M64PParseCallbacks {
    fn item_name(&self, original_item_name: &str) -> Option<String> {
        match original_item_name {
            // irregular type names
            "m64p_2d_size" => Some("Size2D".to_owned()),
            "m64p_GLattr" => Some("GLAttribute".to_owned()),
            "m64p_GLContextType" => Some("GLContextType".to_owned()),
            "BUTTONS" => Some("Buttons".to_owned()),
            // confusing names
            "m64p_type" => Some("ConfigType".to_owned()),
            // other items
            item if item.starts_with("m64p_") => Some(original_item_name[5..].to_pascal_case()),
            _ => None,
        }
    }

    fn add_derives(&self, info: &DeriveInfo<'_>) -> Vec<String> {
        match info.kind {
            TypeKind::Struct => vec![],
            TypeKind::Enum => {
                if CORE_RR_BITFLAGS_RUST.contains(&info.name) {
                    vec![]
                } else {
                    vec![
                        "::num_enum::IntoPrimitive".to_owned(),
                        "::num_enum::TryFromPrimitive".to_owned(),
                    ]
                }
            }
            TypeKind::Union => vec![],
        }
    }

    fn enum_variant_name(
        &self,
        enum_name: Option<&str>,
        original_variant_name: &str,
        _variant_value: bindgen::callbacks::EnumVariantValue,
    ) -> Option<String> {
        let stripped = if original_variant_name.starts_with("M64P_GL_") {
            &original_variant_name[8..]
        } else if original_variant_name.starts_with("M64") {
            match original_variant_name.find('_') {
                Some(pos) => &original_variant_name[(pos + 1)..],
                None => return None,
            }
        } else {
            match original_variant_name.find('_') {
                Some(pos) => &original_variant_name[(pos + 1)..],
                None => original_variant_name,
            }
        };

        if let Some(enum_name) = enum_name {
            match enum_name {
                "m64p_plugin_type" if original_variant_name == "M64PLUGIN_GFX" => {
                    return Some("Graphics".to_owned())
                }
                "m64p_render_mode" => {
                    return match original_variant_name {
                        "M64P_RENDER_OPENGL" => Some("OpenGl".to_owned()),
                        "M64P_RENDER_VULKAN" => Some("Vulkan".to_owned()),
                        _ => unimplemented!(),
                    }
                }
                "m64p_core_param" => {
                    return if stripped.starts_with("STATE_") && stripped.ends_with("COMPLETE") {
                        let mut name = stripped.to_pascal_case();
                        unsafe {
                            // stripped.to_pascal_case returns valid UTF-8.
                            // We're only editing ASCII bytes, so we cannot break any non-ASCII data.
                            let name_bytes = name.as_bytes_mut();
                            name_bytes[name_bytes.len() - 8] = b'C';
                        }
                        Some(name)
                    } else {
                        Some(stripped.to_pascal_case())
                    };
                }
                "m64p_GLContextType" => match stripped.rfind('_') {
                    Some(last_uscore) => {
                        return Some(stripped[(last_uscore + 1)..].to_pascal_case())
                    }
                    None => return None,
                },
                name if CORE_RR_BITFLAGS.contains(&name) => {
                    return Some(stripped.to_shouty_snake_case())
                }
                _ => (),
            };
        }
        Some(stripped.to_pascal_case())
    }
}

fn core_bindgen<P: AsRef<Path>>(core_dir: P) -> Result<(), Box<dyn Error>> {
    // Paths
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let out_file = out_dir.join("types.gen.rs");

    let src_dir = core_dir.as_ref().join("src/");
    let api_dir = core_dir.as_ref().join("src/api/");

    let mut builder = bindgen::builder()
        .impl_debug(true)
        .clang_arg(format!("-I{}", src_dir.display()));
    // parse callbacks
    builder = builder
        .parse_callbacks(Box::new(
            ::bindgen::CargoCallbacks::new().rerun_on_header_files(true),
        ))
        .parse_callbacks(Box::new(M64PParseCallbacks {}));

    // builder settings
    builder = builder
        .default_enum_style(bindgen::EnumVariation::Rust {
            non_exhaustive: false,
        })
        .prepend_enum_name(false);

    // blocklist debug types and function pointers
    builder = builder
        .blocklist_type(r"m64p_dbg_.*")
        .blocklist_type(r"m64p_breakpoint")
        .blocklist_type(r"ptr_.*")
        .blocklist_function(".*");

    // blocklist BUTTONS specifically
    builder = builder.blocklist_type(r"BUTTONS");

    // blocklist bitflag enums so I can give them the bitflags! treatment
    for name in CORE_RR_BITFLAGS {
        builder = builder.blocklist_type(name);
    }

    // Add extern crate for num_enum
    builder = builder.raw_line("extern crate num_enum;");

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

/// Compiles a mupen64plus-like project using its provided makefile.
fn m64p_compile_project<P: AsRef<Path>, Q: AsRef<Path>>(project_dir: P, output_dir: Q) -> Result<(), Box<dyn Error>> {
    #[cfg(unix)]
    {
        let makefile_dir = project_dir.as_ref().join("projects/unix");
        let mut make_proc = Command::new("make")
            .args(["all"])
            .current_dir(makefile_dir)
            .stdout(os_pipe::dup_stderr().unwrap())
            .spawn()
            .expect("Command::spawn failed");

        let make_proc = make_proc.wait().expect("Child::wait failed");
        if !make_proc.success() {
            panic!(
                "[make all] failed; exit code {}",
                make_proc
                    .code()
                    .map_or("unknown".to_string(), |code| code.to_string())
            );
        }

        // execute compiledb to make
    }

    #[cfg(all(windows, target_env = "msvc"))]
    {
        use std::os::windows::process::CommandExt;

        static VSWHERE_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
            // use vswhere.exe bundled with VS
            let program_files_x86 = PathBuf::from(std::env::var("ProgramFiles(x86)").unwrap());
            let vswhere_path =
                program_files_x86.join("Microsoft Visual Studio\\Installer\\vswhere.exe");
            assert!(vswhere_path.exists());
            vswhere_path
        });
        static VS_DEVCMD: LazyLock<PathBuf> = LazyLock::new(|| {
            // invoke vswhere to find the latest VS installation with MSVC
            let vswhere_invoke = Command::new(VSWHERE_PATH.as_path())
                .arg("-latest")
                .args([
                    "-requires",
                    "Microsoft.VisualStudio.Component.VC.Tools.x86.x64",
                ])
                .args(["-requires", "Microsoft.Component.MSBuild"])
                .args(["-property", "installationPath"])
                .output()
                .expect("vswhere invoke failed");
            assert!(vswhere_invoke.status.success());
            
            // find the developer prompt batchfile
            let install_path: PathBuf = String::from_utf8(vswhere_invoke.stdout)
                .unwrap()
                .trim()
                .into();
            install_path.join("Common7\\Tools\\VsDevCmd.bat")
        });

        let msbuild_config = {
            let cargo_profile = std::env::var("PROFILE").unwrap();
            match cargo_profile.as_str() {
                "release" => "Release",
                "debug" => "Debug",
                _ => panic!("Invalid profile!")
            }
        };
        let msbuild_platform = {
            let cargo_target = std::env::var("TARGET").unwrap();
            let target_arch = cargo_target.split('-').nth(0).unwrap();

            match target_arch {
                "x86_64" => "x64",
                "i686" | "i586" => "Win32",
                arch => panic!("Unsupported platform {}", arch)
            }
        };
        let vsdevcmd_arch = {
            let cargo_target = std::env::var("TARGET").unwrap();
            let target_arch = cargo_target.split('-').nth(0).unwrap();

            match target_arch {
                "x86_64" => "amd64",
                "i686" | "i586" => "x86",
                arch => panic!("Unsupported platform {}", arch)
            }
        };

        // output dir: strip the UNC prefix if it's there
        let output_unc = String::from(output_dir.as_ref().to_str().unwrap());
        let output = output_unc.strip_prefix("\\\\?\\").unwrap();

        let cmd_invocation = {
            // setup the final command to pass to cmd.exe
            let mut s = OsString::new();
            s.push("\"\"");
            s.push(VS_DEVCMD.as_os_str());
            s.push("\" ");
            write!(
                s, 
                "-arch={} && msbuild /p:Configuration={} /p:Platform={} /p:OutDir=\"{}\" {}",
                vsdevcmd_arch,
                msbuild_config,
                msbuild_platform,
                output,
                "mupen64plus-core.vcxproj"
            ).unwrap();
            s.push("\"");
            s
        };
        eprintln!("Invoking command: cmd.exe /s /c {}", cmd_invocation.to_string_lossy());

        // working dir: strip the UNC prefix if it's there
        let pwd_unc = String::from(project_dir.as_ref().join("projects\\msvc\\").to_str().unwrap());
        let pwd = pwd_unc.strip_prefix("\\\\?\\").unwrap();

        let build_invoke = Command::new("cmd.exe")
            .args(["/s", "/c"])
            .raw_arg(cmd_invocation.as_os_str())
            .current_dir(pwd)
            .stdout(os_pipe::dup_stderr().unwrap())
            .status()
            .expect("build command failed");

        assert!(build_invoke.success());

        // assert!(msbuild_invoke.success());
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let native_dir = std::fs::canonicalize(
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("../native/"),
    )
    .unwrap();

    let core_dir = native_dir.join("mupen64plus-core-tas");
    let build_dir = native_dir.join("build");

    m64p_compile_project(&core_dir, &build_dir)?;
    core_bindgen(&core_dir)?;

    Ok(())
}
