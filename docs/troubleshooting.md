# Image compatibility and Linux boot splash troubleshooting

## What a successful operation means

For PNG files, the tool decodes the selected image, composites transparency onto
black, and encodes an 8-bit RGB, non-interlaced PNG without source metadata. Image
dimensions are preserved and the selected source file is not modified. Other image
formats retain their original bytes.

The prepared bytes are stored in a temporary snapshot. The tool calculates the
checksum from this snapshot and copies that same snapshot to the EFI System
Partition (ESP), then updates Lenovo's UEFI variables. Preparation and checksum
calculation complete before any ESP or UEFI writes. Invalid PNGs and unsupported
checksum versions are rejected at this stage. The temporary snapshot is removed
when the operation finishes.

A successful operation still does not prove that a particular firmware can display
the image. Save your work and reboot to verify the actual boot display.

The `mylogo_<width>x<height>.<extension>` filename uses the dimensions reported by
`LBLDESP`; it does not mean the image was resized to those dimensions. The UI
reports these as the maximum image size. Keep the actual image within that limit.

`LBLDVC` version `0x20000` uses CRC32 of the first 512 bytes (or the whole file if
shorter); version `0x20003` uses SHA256 of the whole file. Replacing the file on the
ESP manually without updating its checksum can prevent the firmware from using it.
Use **Change Logo** again when changing the image.

## Try a simple RGB PNG

A declared format such as PNG does not guarantee that every encoding supported by
desktop image viewers works in the firmware decoder. **Change Logo** now performs
the RGB PNG normalization automatically on both Windows and Linux. Select your
original PNG directly; no Python preprocessing is needed.

If you are using an older release without automatic normalization, the following
manual conversion provides the same black-background pixel compositing. Different
encoders can produce different compressed file bytes for identical RGB pixels;
checksums must always be calculated from the file actually installed.

The following command requires Python 3 and Pillow (`PIL`). It creates a new file
without modifying the input or writing to the ESP or UEFI variables:

```bash
python3 - input.png firmware-logo.png <<'PY'
import sys
from pathlib import Path
from PIL import Image

source, destination = map(Path, sys.argv[1:])
if source.resolve() == destination.resolve():
    raise SystemExit("Choose a different output path to preserve the original.")
if destination.exists():
    raise SystemExit("Output already exists; choose a new output path.")
with Image.open(source) as original:
    rgba = original.convert("RGBA")
    rgb = Image.new("RGB", rgba.size, "black")
    rgb.paste(rgba, mask=rgba.getchannel("A"))
    rgb.save(destination, format="PNG")
print(f"Created {destination}: {rgb.width}x{rgb.height}, 8-bit RGB PNG")
PY
```

For an older release, back up your existing configuration as described in the
disclaimer, then select `firmware-logo.png` and click **Change Logo**. Save your work
and reboot to check the actual boot display. Do not just rename a file's extension to convert
its format.

## Why Ubuntu may display its own logo

The firmware splash and the operating system splash are separate stages. On Linux,
Plymouth's BGRT theme can reuse the image exposed by the firmware through the ACPI
Boot Graphics Resource Table (BGRT). If that image is unavailable, the theme can use
a fallback image instead. On the Ubuntu installation in the case below,
`/usr/share/plymouth/themes/spinner/bgrt-fallback.png` was the orange Ubuntu logo.

Check whether the firmware image is available after boot:

```bash
if [ -r /sys/firmware/acpi/bgrt/image ]; then
    echo "Firmware BGRT image is available"
else
    echo "No readable firmware BGRT image"
fi
```

An absent BGRT image alone does not prove that the firmware never displayed a logo;
observe the firmware stage separately from the operating system stage. Check image
compatibility and reapply the custom logo before changing Plymouth. Replacing a
Plymouth fallback only changes the Linux splash; it does not repair firmware image
decoding or make the firmware publish BGRT. This tool does not configure Plymouth.

## Tested recovery: ThinkBook 14 G4+ IAP

This is a single-device report, not a guarantee for every BIOS or PNG file.

| Item | Observed value |
|------|----------------|
| Device | ThinkBook 14 G4+ IAP, model 21CX |
| BIOS | HYCN51WW |
| OS | Ubuntu 26.04.1 LTS |
| Firmware-reported maximum image size | 2880 x 1800 |
| `LBLDVC` version | `0x20000` (CRC32) |
| Actual image size, before and after | 1275 x 440 |
| Original PNG | 8-bit RGBA with transparent pixels, ICC profile and DPI metadata |
| Replacement PNG | 8-bit RGB, black background, non-interlaced, no extra metadata |

Before recovery, the custom-logo enable flag was set, the ESP file matched the
selected original image byte for byte, and its CRC32 matched `LBLDVC`. The firmware
filename variable also matched the installed PNG. Despite these consistent settings,
the desired logo was missing and Ubuntu displayed its own logo. Neither the BGRT
sysfs image nor the ACPI BGRT table was present.

Recovery replaced the PNG at the same ESP path with the normalized RGB PNG and
updated its CRC32 using the existing `0x20000` protocol. The enable flag, actual
image dimensions and filename were preserved. The user reported that this firmware
change alone fixed the display after reboot. A subsequent check confirmed that
`/sys/firmware/acpi/bgrt/image` was available and the installed PNG and its checksum
matched. The Plymouth fallback file was unchanged.

Transparency and metadata were removed together. This confirms the normalized PNG
as a working recovery on this configuration, but does not isolate which original
PNG feature triggered the failure. The original recovery was performed with a targeted
external script. The Rust implementation was subsequently retested on the same
ThinkBook using the Linux build, and the user confirmed that the boot logo worked.
Read-back checks confirmed an RGB PNG without extra metadata, a matching CRC32 of
`0x236cc08e`, and an available BGRT image. The installed RGB pixels matched the
successful reference image. This validates automatic PNG normalization and the
`0x20000` firmware path on this configuration; `0x20003` and other image formats
have automated coverage but were not hardware-tested in this report.
