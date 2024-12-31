#!/usr/bin/env python3
from pathlib import Path
import platform
import shutil
import sys
import subprocess as subp
from tempfile import mkdtemp

BASE_PATH = Path(__file__).parent
SRC_PATH = BASE_PATH / "src"
POT_PATH = BASE_PATH / "i18n" / "messages.pot"

if len(sys.argv) == 2 and sys.argv[1] == "--help":
    print(f"Usage: {sys.argv[0]} [--help]")
    print("Generates the .pot file from the available source files.")
    sys.exit(0)
elif len(sys.argv) == 1:
    pass
else:
    print(f"Usage: {sys.argv[0]} [--help]")
    sys.exit(1)

# check for installed tools
if not shutil.which("xgettext"):
    print("Missing `xgettext`. Ensure you have the gettext tools installed.")
    if platform.system() == "Windows":
        print("Hint: you may need to configure the environment variables after installing gvsbuild")
    sys.exit(1)
if not shutil.which("xtr"):
    print("Missing Cargo binary `xtr`. Install it with `cargo install xtr`.")
    sys.exit(1)

# generate base .pot files


def pot_gen(tmpdir: Path):
    rs_pot = tmpdir / "rs.pot"
    ui_pot = tmpdir / "ui.pot"
    blp_pot = tmpdir / "blp.pot"

    # Rust
    subp.run(["xtr", SRC_PATH/"main.rs", "-o", rs_pot]).check_returncode()

    # GTK Builder XML
    subp.run(["xgettext", "-L", "glade", *SRC_PATH.rglob("*.ui"),
             "-o", ui_pot]).check_returncode()

    # Blueprint
    subp.run([
        "xgettext", *
        SRC_PATH.rglob("*.blp"), "--from-code=UTF-8", "--add-comments",
             "--keyword=_", "--keyword=C_:1c,2", "-o", blp_pot
             ]).check_returncode()

    # merge
    merge_proc = subp.Popen(
        ["xgettext", rs_pot, ui_pot, blp_pot, "-o", "-"], stdout=subp.PIPE, encoding="utf-8")
    with open(POT_PATH, "w") as out_file:
        for line in merge_proc.stdout:
            if line.startswith("\"Project-Id-Version"):
                out_file.write("\"Project-Id-Version: m64prs 0.1.0\"\n")
            else:
                out_file.write(line)

    pass


tmpdir = None
try:
    tmpdir = Path(mkdtemp(prefix="m64prs-gtk-pot"))
    pot_gen(tmpdir)
finally:
    shutil.rmtree(tmpdir)
