import sys
import shutil
import subprocess as subp
from pathlib import Path
import os

VSWHERE_PATH = Path(os.environ["ProgramFiles(x86)"]) / \
    "Microsoft Visual Studio\\Installer\\vswhere.exe"


def vswhere(args: list[str]) -> str:
    cmd = subp.run([VSWHERE_PATH, *args], stdout=subp.PIPE, encoding="utf-8")
    cmd.check_returncode()
    return cmd.stdout


def vs_dev_env(arch: str) -> dict[str, str]:
    devenv_path = Path(vswhere(
        ["-latest", "-property", "installationPath"]).strip()) / "Common7\\Tools\\VsDevCmd.bat"

    cmd = subp.Popen(
        f"cmd.exe /s /c \"\"{str(devenv_path)}\" " +
        f"-no_logo -arch={arch} && set\"",
        stdout=subp.PIPE, encoding="utf-8"
    )

    env_dict: dict[str, str] = {}
    for line in cmd.stdout:
        line = line.strip()
        split_pos = line.find('=')
        if split_pos < 0:
            raise ValueError("Windows environment broke")
        env_dict[line[:split_pos]] = line[(split_pos + 1):]

    return env_dict


MSBUILD_PATH = Path(vswhere([
    "-latest",
    "-requires",
    "Microsoft.Component.MSBuild",
    "-find",
    "MSBuild\\**\\Bin\\MSBuild.exe",
]).strip())

self_dir = Path(__file__).parent
target_dir = self_dir/"target"


def msbuild_subproject(sln: Path, env: dict[str, str], out_dir: Path, int_dir: Path, msbuild_config: str, msbuild_platform: str):

    args = [
        MSBUILD_PATH, 
        f"/p:Configuration={msbuild_config}", 
        f"/p:Platform={msbuild_platform}", 
        f"/p:OutDir={os.path.join(str(out_dir), "")}", 
        f"/p:IntDir={os.path.join(str(int_dir), "")}", 
        str(sln)
    ]
    print(args)
    cmd = subp.run(
        args,
        env=env,
        stdout=sys.stderr
    )
    cmd.check_returncode()

    pass




def build():
    vs_env_arch = "amd64"
    msbuild_platform = "x64"
    match os.environ.get("CARGO_CFG_TARGET_ARCH"):
        case "x86":
            (vs_env_arch, msbuild_platform) = ("x86", "Win32")
        case "x86_64":
            (vs_env_arch, msbuild_platform) = ("amd64", "x64")
        case _:
            pass

    msbuild_config = "Debug"
    match os.environ.get("PROFILE"):
        case "debug":
            msbuild_config = "Debug"
        case "release":
            msbuild_config = "Release"
        case _:
            pass

    vs_env = vs_dev_env(vs_env_arch)

    # subp.run([MSBUILD_PATH, "/?"])

    msbuild_subproject(
        self_dir/"mupen64plus-core\\projects\\msvc\\mupen64plus-core.vcxproj",
        env=vs_env,
        out_dir=target_dir,
        int_dir=target_dir/"mupen64plus-core",
        msbuild_config=msbuild_config,
        msbuild_platform=msbuild_platform
    )
    msbuild_subproject(
        self_dir/"mupen64plus-audio-sdl\\projects\\msvc\\mupen64plus-audio-sdl.vcxproj",
        env=vs_env,
        out_dir=target_dir,
        int_dir=target_dir/"mupen64plus-audio-sdl",
        msbuild_config=msbuild_config,
        msbuild_platform=msbuild_platform
    )
    msbuild_subproject(
        self_dir/"mupen64plus-input-sdl\\projects\\msvc\\mupen64plus-input-sdl.vcxproj",
        env=vs_env,
        out_dir=target_dir,
        int_dir=target_dir/"mupen64plus-input-sdl",
        msbuild_config=msbuild_config,
        msbuild_platform=msbuild_platform
    )
    msbuild_subproject(
        self_dir/"mupen64plus-video-rice\\projects\\msvc\\mupen64plus-video-rice.vcxproj",
        env=vs_env,
        out_dir=target_dir,
        int_dir=target_dir/"mupen64plus-video-rice",
        msbuild_config=msbuild_config,
        msbuild_platform=msbuild_platform
    )
    msbuild_subproject(
        self_dir/"mupen64plus-rsp-hle\\projects\\msvc\\mupen64plus-rsp-hle.vcxproj",
        env=vs_env,
        out_dir=target_dir,
        int_dir=target_dir/"mupen64plus-rsp-hle",
        msbuild_config=msbuild_config,
        msbuild_platform=msbuild_platform
    )



def clean():
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
