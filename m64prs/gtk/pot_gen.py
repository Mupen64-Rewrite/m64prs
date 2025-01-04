#!/usr/bin/env python3
from pathlib import Path
import platform
import shutil
import sys
import subprocess as subp
from tempfile import mkdtemp

BASE_PATH = Path(__file__).parent
SRC_PATH = BASE_PATH / "src"
POT_PATH = BASE_PATH / "i18n"

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

    # Rust
    subp.run(["xtr", SRC_PATH/"main.rs", "-o", rs_pot]).check_returncode()

    # GTK Builder XML
    subp.run(["xgettext", "-L", "glade", *SRC_PATH.rglob("*.ui"),
             "-o", ui_pot]).check_returncode()

    # merge
    merge_cmd = ["xgettext", "-o", "-", "--", rs_pot, ui_pot]
    merge_proc = subp.Popen(
        merge_cmd,
        stdout=subp.PIPE, encoding="utf-8"
    )
    with open(POT_PATH / "m64prs.pot", "w") as out_file:
        for line in merge_proc.stdout:
            if line.startswith("\"Project-Id-Version"):
                out_file.write("\"Project-Id-Version: m64prs 0.1.0\"\n")
            else:
                out_file.write(line)
    merge_proc_rc = merge_proc.wait()

    if merge_proc_rc != 0:
        raise subp.CalledProcessError(merge_proc_rc, merge_cmd)


tmpdir = None
try:
    tmpdir = Path(mkdtemp(prefix="m64prs-gtk-pot"))
    pot_gen(tmpdir)
finally:
    shutil.rmtree(tmpdir)


# update existing .po files
for file in POT_PATH.glob("*.po"):
    backup = file.with_suffix(".po.bak")
    shutil.move(file, backup)
    with open(file, "w") as outfile:
        subp.run([
            "msgmerge", "-N", "--", backup, POT_PATH / "m64prs.pot"
        ], stdout=outfile).check_returncode()
    backup.unlink()
