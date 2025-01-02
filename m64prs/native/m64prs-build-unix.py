#!/usr/bin/env python
import sys
from pathlib import Path
import shutil
import subprocess as subp
import platform


def core_soname(name: str) -> str:
    match platform.system():
        case "Windows":
            return f"{name}.dll"
        case "Darwin":
            return f"lib{name}.dylib"
        case "Linux":
            return f"lib{name}.so.2.0.0"
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


COMPILEDB_AVAILABLE = shutil.which("compiledb") is not None


def make_subproject(dir: Path, extra_args: list[str] = []):
    # check if git will ignore it
    base_args = ["make"]
    compdb_ignored = not bool(subp.run([
        "git", "check-ignore", "-q", "compile_commands.json"
    ], cwd=dir).returncode)
    if COMPILEDB_AVAILABLE and compdb_ignored:
        base_args = ["compiledb", "make"]

    args = base_args + extra_args
    subp.run(args, cwd=dir).check_returncode()


self_dir = Path(__file__).parent
target_dir = self_dir/"target"


def build():
    target_dir.mkdir(exist_ok=True, parents=True)
    make_subproject(self_dir/"mupen64plus-core/projects/unix",
                    ["all", "TAS=1"])
    make_subproject(self_dir/"mupen64plus-audio-sdl/projects/unix", ["all"])
    make_subproject(self_dir/"mupen64plus-input-sdl/projects/unix", ["all"])
    make_subproject(self_dir/"mupen64plus-video-rice/projects/unix", ["all"])
    make_subproject(self_dir/"mupen64plus-rsp-hle/projects/unix", ["all"])

    copy_if_newer(self_dir/f"mupen64plus-core/projects/unix/{
                  core_soname("mupen64plus")}", target_dir/dll_name("mupen64plus"))
    copy_if_newer(self_dir/f"mupen64plus-audio-sdl/projects/unix/{
                  plugin_name("mupen64plus-audio-sdl")}", target_dir)
    copy_if_newer(self_dir/f"mupen64plus-input-sdl/projects/unix/{
                  plugin_name("mupen64plus-input-sdl")}", target_dir)
    copy_if_newer(self_dir/f"mupen64plus-video-rice/projects/unix/{
                  plugin_name("mupen64plus-video-rice")}", target_dir)
    copy_if_newer(self_dir/f"mupen64plus-rsp-hle/projects/unix/{
                  plugin_name("mupen64plus-rsp-hle")}", target_dir)


def clean():
    make_subproject(self_dir/"mupen64plus-core/projects/unix", ["clean"])
    make_subproject(self_dir/"mupen64plus-audio-sdl/projects/unix", ["clean"])
    make_subproject(self_dir/"mupen64plus-input-sdl/projects/unix", ["clean"])
    make_subproject(self_dir/"mupen64plus-video-rice/projects/unix", ["clean"])
    make_subproject(self_dir/"mupen64plus-rsp-hle/projects/unix", ["clean"])

    shutil.rmtree(target_dir)
    pass


COMMAND_TABLE = {
    "build": build,
    "clean": clean
}
COMMAND_LIST = " | ".join(i for i in COMMAND_TABLE.keys())


if len(sys.argv) != 2:
    print(f"Usage: {sys.argv[0]} [{COMMAND_LIST}]")
    sys.exit(1)

command = COMMAND_TABLE.get(sys.argv[1])
if command is None:
    print(f"Usage: {sys.argv[0]} [{COMMAND_LIST}]")
    sys.exit(1)

command()
