#!/usr/bin/env python3
import argparse
from pathlib import Path
import subprocess as subp
import shutil
import platform
import os

# INSTALL FUNCTIONS
# ======================

INSTALL_DIRS = {
    # For Windows, AppImage, etc.
    "portable": {
        "bin": ".",
        "core": ".",
        "plugin": "./plugin",
        "data": "./data",
        "i18n": "./i18n",
    },
    # For Linux distro packages
    "unix": {
        "bin": "./bin",
        "core": "./lib/m64prs",
        "plugin": "./lib/m64prs/plugin",
        "data": "./share/m64prs",
        "i18n": "./share/locale"
    }
}


def copy_if_newer(src: Path, dst: Path) -> None:
    if not src.exists():
        raise FileNotFoundError(src)

    if dst.is_dir():
        dst = dst.joinpath(src.name)

    if dst.exists() and dst.stat().st_mtime_ns > src.stat().st_mtime_ns:
        return

    print(f"copy: {src} -> {dst}")
    shutil.copy(src, dst)
    pass


def exe_name(name: str) -> str:
    match platform.system():
        case "Windows":
            return f"{name}.exe"
        case _:
            return name


def dll_name(name: str) -> str:
    match platform.system():
        case "Windows":
            return f"{name}.dll"
        case "Darwin":
            return f"lib{name}.dylib"
        case "Linux":
            return f"lib{name}.so"
        case _:
            return name


def plugin_name(name: str) -> str:
    match platform.system():
        case "Windows":
            return f"{name}.dll"
        case "Darwin":
            return f"{name}.dylib"
        case "Linux":
            return f"{name}.so"
        case _:
            return name


def install_debug_info(srcdir: Path, dstdir: Path, srcfile: str, dstfile: str | None = None):
    if platform.system() != "Windows":
        return
    if not (srcdir/f"{srcfile}.pdb").exists():
        return
    if dstfile is None:
        dstfile = srcfile

    copy_if_newer(srcdir/f"{srcfile}.pdb", dstdir/f"{dstfile}.pdb")


def install_exe(srcdir: Path, dstdir: Path, srcfile: str, dstfile: str | None = None):
    if dstfile is None:
        dstfile = srcfile

    copy_if_newer(srcdir/exe_name(srcfile), dstdir/exe_name(dstfile))


def install_dll(srcdir: Path, dstdir: Path, srcfile: str, dstfile: str | None = None):
    if dstfile is None:
        dstfile = srcfile

    copy_if_newer(srcdir/dll_name(srcfile), dstdir/dll_name(dstfile))


def install_plugin(srcdir: Path, dstdir: Path, srcfile: str, dstfile: str | None = None):
    if dstfile is None:
        dstfile = srcfile
    srcfile = plugin_name(srcfile)
    dstfile = plugin_name(dstfile)

    copy_if_newer(srcdir/srcfile, dstdir/dstfile)


def install_data(srcdir: Path, dstdir: Path):
    for item in srcdir.iterdir():
        copy_if_newer(item, dstdir.joinpath(item.name))

#
# ======================

# COMMANDS
# ======================


def build(args: argparse.Namespace, extra: list[str]):
    project_dir = Path(__file__).parent

    # setup directories
    install_root_dir = None
    target_dir = None
    if args.release:
        install_root_dir = project_dir.joinpath("install/release")
        target_dir = project_dir.joinpath("target/release")
    else:
        install_root_dir = project_dir.joinpath("install/debug")
        target_dir = project_dir.joinpath("target/debug")
    install_root_dir.mkdir(parents=True, exist_ok=True)

    native_root_dir = project_dir.joinpath("m64prs/native")
    native_target_dir = native_root_dir.joinpath("target")

    scheme = INSTALL_DIRS[args.install_scheme]
    default_scheme_name = next(iter(INSTALL_DIRS.keys()))

    [bin_dir, core_dir, data_dir, plugin_dir, i18n_dir] = [
        (install_root_dir / scheme[key]) for key in
        ["bin", "core", "data", "plugin", "i18n"]
    ]

    for dir in [bin_dir, core_dir, data_dir, plugin_dir, i18n_dir]:
        dir.mkdir(parents=True, exist_ok=True)

    # compile cargo
    cargo_args = [
        "cargo", "build",
        "-p", "m64prs-gtk",
        "-p", "tasinput-bridge",
        "-p", "tasinput-ui",
    ]
    if args.release:
        cargo_args.append("--release")
    if args.install_scheme != default_scheme_name:
        cargo_args.extend(["-F", f"install-{args.install_scheme}"])
    subp.run(
        cargo_args,
        cwd=project_dir
    ).check_returncode()

    # copy binaries
    install_exe(target_dir, bin_dir, "m64prs-gtk")
    install_debug_info(target_dir, bin_dir, "m64prs_gtk")
    install_dll(native_target_dir, core_dir, "mupen64plus")
    install_debug_info(native_target_dir, core_dir, "mupen64plus")

    # copy native plugins
    install_plugin(native_target_dir, plugin_dir, "mupen64plus-video-rice")
    install_plugin(native_target_dir, plugin_dir, "mupen64plus-audio-sdl")
    install_plugin(native_target_dir, plugin_dir, "mupen64plus-input-sdl")
    install_plugin(native_target_dir, plugin_dir, "mupen64plus-rsp-hle")
    install_debug_info(native_target_dir, plugin_dir, "mupen64plus-video-rice")
    install_debug_info(native_target_dir, plugin_dir, "mupen64plus-audio-sdl")
    install_debug_info(native_target_dir, plugin_dir, "mupen64plus-input-sdl")
    install_debug_info(native_target_dir, plugin_dir, "mupen64plus-rsp-hle")

    # copy tasinput-rs
    copy_if_newer(target_dir/dll_name("tasinput_bridge"),
                  plugin_dir/plugin_name("mupen64plus-input-tasinput"))
    install_debug_info(target_dir, plugin_dir,
                       "tasinput_bridge", "mupen64plus_input_tasinput")
    install_exe(target_dir, plugin_dir, "tasinput-ui")
    install_debug_info(target_dir, plugin_dir, "tasinput-ui")

    # copy Windows dependencies
    if platform.system() == "Windows":
        arch_name = None
        if platform.machine() in ["x86"]:
            arch_name = "x86"
        elif platform.machine() in ["AMD64"]:
            arch_name = "x64"
        else:
            assert False

        deps_dir = install_root_dir.joinpath(
            "m64prs/native/mupen64plus-win32-deps")

        NATIVE_DEP_NAMES = [
            ("freetype-2.13.0", "freetype"),
            ("libpng-1.6.39", "libpng16"),
            ("SDL2_net-2.2.0", "SDL2_net"),
            ("SDL2-2.26.3", "SDL2"),
            ("zlib-1.2.13", "zlib"),
        ]
        for dep_name, lib_name in NATIVE_DEP_NAMES:
            install_dll(deps_dir / dep_name / "lib" /
                        arch_name, bin_dir, lib_name)

    # copy data files
    NATIVE_DATA_PATHS = [
        "mupen64plus-core/data",
        "mupen64plus-input-sdl/data",
        "mupen64plus-video-rice/data",
    ]
    for data_path in NATIVE_DATA_PATHS:
        install_data(native_root_dir / data_path, data_dir)


def run(args: argparse.Namespace, extra: list[str]):
    build(args, extra)
    project_dir = Path(__file__).parent

    install_root_dir = None
    if args.release:
        install_root_dir = project_dir.joinpath("install/release")
    else:
        install_root_dir = project_dir.joinpath("install/debug")
    
    bin_dir = install_root_dir / INSTALL_DIRS[args.install_scheme]['bin']

    run_args = [bin_dir.joinpath(exe_name("m64prs-gtk"))]
    run_args.extend(extra)

    subp.run(
        run_args
    ).check_returncode()


def clean(args: argparse.Namespace, extra: list[str]):
    root_dir = Path(__file__).parent
    shutil.rmtree(root_dir.joinpath("target"))
    shutil.rmtree(root_dir.joinpath("install"))

    native_dir = root_dir.joinpath("m64prs/native")
    if platform.system() == "Windows":
        shutil.rmtree(native_dir.joinpath("target"))
    else:
        subp.run([
            "python3", native_dir.joinpath("m64prs-build-all.py"), "clean"
        ]).check_returncode()

    pass


# CLI
# ======================

def build_options(subcli: argparse.ArgumentParser):
    scheme_list = list(INSTALL_DIRS.keys())

    subcli.add_argument(
        "-r", "--release",
        help="Do a release build (instead of a debug build).",
        action="store_true",
        default=False
    )
    subcli.add_argument(
        "-s", "--install-scheme",
        help="The install scheme to use.",
        choices=scheme_list,
        default=scheme_list[0],
        metavar="<scheme>"
    )


def create_cli():
    cli = argparse.ArgumentParser(
        description="Master build script for m64prs.",
        add_help=False,
    )
    cli.add_argument(
        "--help",
        help="Show this help message and exit.",
        action="help"
    )
    subclis = cli.add_subparsers(
        title="subcommands",
        metavar="<command>",
        required=True,
        help="The build command to use."
    )

    # build command
    build_cli = subclis.add_parser(
        "build",
        add_help=False,
        help="Build and install all dependencies"
    )
    build_cli.add_argument(
        "--help",
        help="Show this help message and exit.",
        action="help"
    )
    build_options(build_cli)
    build_cli.set_defaults(func=build)

    run_cli = subclis.add_parser(
        "run",
        add_help=False,
        help="Run the program"
    )
    run_cli.add_argument(
        "--help",
        help="Show this help message and exit.",
        action="help"
    )
    build_options(run_cli)
    run_cli.set_defaults(func=run)

    clean_cli = subclis.add_parser(
        "clean",
        add_help=False,
        help="Cleans all build artifacts."
    )
    clean_cli.add_argument(
        "--help",
        help="Show this help message and exit.",
        action="help"
    )
    clean_cli.set_defaults(func=clean)

    return cli


args, extra = create_cli().parse_known_args()
extra = [arg for arg in extra if arg != "--"]

try:
    args.func(args, extra)
except subp.CalledProcessError as e:
    print("Subprocess failed!")
    print(e)
except KeyboardInterrupt:
    print("Ctrl+C pressed! Stopping...")
