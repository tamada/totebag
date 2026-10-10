#!/usr/bin/env python3
"""Generate the unsafe archives used by the path traversal tests (#103).

Each archive holds the same four one-byte entries, in this order:

    ok_before.txt            safe, must be extracted
    ../evil.txt              climbs out of the destination, must be rejected
    /tmp/totebag_evil.txt    absolute path, must be rejected
    ok_after.txt             safe, must still be extracted after the rejections

Only the Python standard library is used. `zipfile` and `tarfile` write these
names as given when entries are added from `ZipInfo` / `TarInfo` objects.
Timestamps and owners are fixed, so running the script again produces
byte-identical files.

Usage (from anywhere):

    python3 testdata/unsafe/generate.py
"""

import io
import tarfile
import zipfile
from pathlib import Path

HERE = Path(__file__).resolve().parent

ENTRIES = [
    "ok_before.txt",
    "../evil.txt",
    "/tmp/totebag_evil.txt",
    "ok_after.txt",
]
DATA = b"x"


def write_zip(path: Path) -> None:
    with zipfile.ZipFile(path, "w", compression=zipfile.ZIP_STORED) as archive:
        for name in ENTRIES:
            info = zipfile.ZipInfo(name, date_time=(1980, 1, 1, 0, 0, 0))
            info.external_attr = 0o644 << 16
            archive.writestr(info, DATA)


def write_tar(path: Path) -> None:
    with tarfile.open(path, "w", format=tarfile.USTAR_FORMAT) as archive:
        for name in ENTRIES:
            info = tarfile.TarInfo(name)
            info.size = len(DATA)
            info.mode = 0o644
            info.mtime = 0
            info.uid = info.gid = 0
            info.uname = info.gname = ""
            archive.addfile(info, io.BytesIO(DATA))


def check(path: Path, names: list[str]) -> None:
    if names != ENTRIES:
        raise SystemExit(f"{path.name}: entry names were changed on write: {names}")


def main() -> None:
    zip_path = HERE / "unsafe.zip"
    write_zip(zip_path)
    with zipfile.ZipFile(zip_path) as archive:
        check(zip_path, archive.namelist())

    tar_path = HERE / "unsafe.tar"
    write_tar(tar_path)
    with tarfile.open(tar_path) as archive:
        check(tar_path, archive.getnames())

    print(f"wrote {zip_path.name} and {tar_path.name} in {HERE}")


if __name__ == "__main__":
    main()
