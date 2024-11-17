#!/usr/bin/env python3
import argparse
from pathlib import Path
import subprocess as subp
import shutil
import platform
import os


def copy_if_newer(src: Path, dst: Path) -> None:
    assert src.exists()

    if dst.is_dir():
        dst = dst.joinpath(src.name)

    if dst.exists() and dst.stat().st_mtime_ns > src.stat().st_mtime_ns:
        return
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


def install_exe(srcdir: Path, dstdir: Path, srcfile: str, dstfile: str | None = None):
    if dstfile is None:
        dstfile = srcfile
    srcfile = exe_name(srcfile)
    dstfile = exe_name(dstfile)

    copy_if_newer(srcdir/srcfile, dstdir/dstfile)


def install_dll(srcdir: Path, dstdir: Path, srcfile: str, dstfile: str | None = None):
    if dstfile is None:
        dstfile = srcfile
    srcfile = dll_name(srcfile)
    dstfile = dll_name(dstfile)

    copy_if_newer(srcdir/srcfile, dstdir/dstfile)


def build(args: argparse.Namespace, extra: list[str]):
    root_dir = Path(__file__).parent

    # setup directories
    bin_dir = None
    target_dir = None
    if args.release:
        bin_dir = root_dir.joinpath("install/release")
        target_dir = root_dir.joinpath("target/release")
    else:
        bin_dir = root_dir.joinpath("install/debug")
        target_dir = root_dir.joinpath("target/debug")
    bin_dir.mkdir(parents=True, exist_ok=True)

    native_target_dir = root_dir.joinpath("m64prs-native/target")
    core_data_dir = root_dir.joinpath(
        "m64prs-native/mupen64plus-core-tas/data")

    data_dir = bin_dir.joinpath("data")
    plugins_dir = bin_dir.joinpath("plugins")
    data_dir.mkdir(parents=True, exist_ok=True)
    plugins_dir.mkdir(parents=True, exist_ok=True)

    # compile cargo
    cargo_args = ["cargo", "build", "--bin", "m64prs-gtk"]
    if args.release:
        cargo_args.append("--release")
    subp.run(
        cargo_args,
        cwd=root_dir
    ).check_returncode()

    # copy binaries
    install_exe(target_dir, bin_dir, "m64prs-gtk")
    install_dll(native_target_dir, bin_dir, "mupen64plus")

    # Copy Windows dependencies
    if platform.system() == "Windows":
        arch_name = None
        if platform.machine() in ["x86"]:
            arch_name = "x86"
        elif platform.machine() in ["AMD64"]:
            arch_name = "x64"
        else:
            assert False

        deps_dir = root_dir.joinpath("m64prs-native/mupen64plus-win32-deps")
            
        install_dll(deps_dir.joinpath(f"freetype-2.13.0/lib/{arch_name}"), bin_dir, "freetype")
        install_dll(deps_dir.joinpath(f"libpng-1.6.39/lib/{arch_name}"), bin_dir, "libpng16")
        install_dll(deps_dir.joinpath(f"SDL2_net-2.2.0/lib/{arch_name}"), bin_dir, "SDL2_net")
        install_dll(deps_dir.joinpath(f"SDL2-2.26.3/lib/{arch_name}"), bin_dir, "SDL2")
        install_dll(deps_dir.joinpath(f"zlib-1.2.13/lib/{arch_name}"), bin_dir, "zlib")

    # copy data
    for item in core_data_dir.iterdir():
        copy_if_newer(item, data_dir.joinpath(item.name))


def run(args: argparse.Namespace, extra: list[str]):
    build(args, extra)
    root_dir = Path(__file__).parent

    bin_dir = None
    if args.release:
        bin_dir = root_dir.joinpath("install/release")
    else:
        bin_dir = root_dir.joinpath("install/debug")

    run_args = [bin_dir.joinpath(exe_name("m64prs-gtk"))]
    run_args.extend(extra)

    subp.run(
        run_args
    ).check_returncode()


def create_cli():
    cli = argparse.ArgumentParser(
        description="Master build script for m64prs.",
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
        help="Build and install all dependencies"
    )
    build_cli.add_argument(
        "--release",
        help="Create a release build instead of a debug build.",
        action="store_true",
        default=False
    )
    build_cli.set_defaults(func=build)

    run_cli = subclis.add_parser(
        "run",
        help="Run the program"
    )
    run_cli.add_argument(
        "--release",
        help="Create a release build instead of a debug build.",
        action="store_true",
        default=False
    )
    run_cli.set_defaults(func=run)

    return cli


args, extra = create_cli().parse_known_args()
extra = [arg for arg in extra if arg != "--"]

try:
    args.func(args, extra)
except subp.CalledProcessError as e:
    print("Subprocess failed!")
    print(e)
