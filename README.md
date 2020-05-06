# fix-japanese-zip

Converts Shift JIS encoded ZIP files to UTF-8 without recompression. May destroy your data.

## Installation

```
cargo +nightly install --git https://github.com/xTibor/fix-japanese-zip
```

## Never asked questions

***Why?*** - KDE applications doesn't support code page auto-detection nor manually overriding the code pages of ZIP files and I cannot read file names like "æfÉ░éτé╡éóâ\`âôâ\`âôéαé╠".

***Is this production quality?*** - No.

***Is this compliant with the ZIP specification?*** - Most likely not. [Ark](https://kde.org/applications/utilities/org.kde.ark) doesn't seem to care.

***How fast is this?*** - Never measured it. Should be faster than extracting/recompressing the archives.

***Why does this require nightly Rust?*** - https://github.com/rust-lang/rust/issues/59359

***Could you help rescuing my ZIP files destroyed by this tool?*** - Restore them from your backups.

***I have no backups and my ZIP files are still destroyed.*** - You are on your own.
