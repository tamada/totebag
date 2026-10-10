# Unsafe archives

Test fixtures for the path traversal check (#103). **Do not extract these with other tools.**
An extractor without the check writes `evil.txt` next to the destination directory and
`/tmp/totebag_evil.txt`.

| File | Format |
| ---- | ------ |
| `unsafe.zip` | Zip, stored (no compression) |
| `unsafe.tar` | Tar, ustar |

Both archives hold the same four one-byte entries, in this order:

| Entry | Expected result |
| ----- | --------------- |
| `ok_before.txt` | extracted |
| `../evil.txt` | rejected (`Error::UnsafePath`): climbs out of the destination |
| `/tmp/totebag_evil.txt` | rejected (`Error::UnsafePath`): absolute path |
| `ok_after.txt` | extracted: one unsafe entry must not stop the rest |

## Why the files are committed

The archives are committed rather than built inside the tests, so the tests do not depend
on archive *writers* continuing to accept unsafe names. A future writer that refuses
them would otherwise break the tests for reasons unrelated to extraction.

## Regenerating

```sh
python3 testdata/unsafe/generate.py
```

[`generate.py`](generate.py) uses only the Python standard library. It fixes timestamps and
owners, so the output is byte-identical every time. It also reads each archive back
and fails if any entry name was changed on write.

To add a format, add a `write_*` function to `generate.py`, a row to the tables above,
and a test that extracts the new file.
