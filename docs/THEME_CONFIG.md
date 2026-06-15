# Theme Configuration

GitNapse supports custom color theming through an optional `theme.jsonc` file placed in the configuration directory.

## Location

The configuration directory is platform-dependent:

| Platform | Path |
|---|---|
| Linux | `~/.config/GitNapse/` |
| macOS | `~/Library/Application Support/com.GitNapse.GitNapse/` |
| Windows | `C:\Users\<user>\AppData\Roaming\GitNapse\GitNapse\config\` |

Place the file at `theme.jsonc` inside that directory. If the file does not exist, GitNapse uses the built-in default palette (16 colors based on a modified Dracula-inspired scheme).

## Format

The file uses JSON with support for `//` line comments (JSONC format).

```jsonc
{
    // GitNapse Theme Configuration
    "palette": [
        [0x36, 0x35, 0x37],   // index 0  - dark background
        [0xfc, 0x61, 0x8d],   // index 1  - pink
        [0x7b, 0xd8, 0x8f],   // index 2  - green
        [0xfc, 0xe5, 0x66],   // index 3  - yellow
        [0xfd, 0x93, 0x53],   // index 4  - orange
        [0x94, 0x8a, 0xe3],   // index 5  - purple
        [0x5a, 0xd4, 0xe6],   // index 6  - cyan
        [0xf7, 0xf1, 0xff],   // index 7  - light text
        [0x69, 0x67, 0x6c],   // index 8  - dim text
        [0xfc, 0x61, 0x8d],   // index 9  - pink (bright)
        [0x7b, 0xd8, 0x8f],   // index 10 - green (bright)
        [0xfc, 0xe5, 0x66],   // index 11 - yellow (bright)
        [0xfd, 0x93, 0x53],   // index 12 - orange (bright)
        [0x94, 0x8a, 0xe3],   // index 13 - purple (bright)
        [0x5a, 0xd4, 0xe6],   // index 14 - cyan (bright)
        [0xf7, 0xf1, 0xff]    // index 15 - white
    ]
}
```

## Palette

The `palette` field is an array of RGB color tuples `[r, g, b]`. Each value is a hexadecimal byte (0x00-0xFF). Colors are indexed modulo the palette length, so you can provide any number of colors.

Indexes are used cyclically for selection highlighting in the UI. For example, the first item in a list uses index 0, the second uses index 1, and so on.

## Text Contrast

For each palette color, GitNapse automatically selects either black or white foreground text based on the luminance of the background color. Colors with luminance >= 0.58 get black text; darker colors get white text. This ensures readability regardless of the palette values.

## Customization Example

To create a minimal theme with just two accent colors:

```jsonc
{
    "palette": [
        [0x1e, 0x1e, 0x2e],
        [0xf3, 0x8f, 0xf8]
    ]
}
```

This would cycle between a dark blue-grey and a soft pink for all selection highlights.

## No Palette File

If `theme.jsonc` is absent or contains invalid JSON, GitNapse silently falls back to the default palette. No error is shown to the user.
