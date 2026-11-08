use image::{ExtendedColorType, ImageEncoder, ImageFormat, ImageReader, codecs::png::PngEncoder};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{Cursor, Write};
use std::path::Path;
use tempfile::NamedTempFile;

enum Checksum {
    Crc32(u32),
    Sha256([u8; 32]),
}

/// Owns the exact file copied to the ESP and the checksum of those same bytes.
/// Preparation completes before any ESP or UEFI changes; dropping it removes the snapshot.
pub(crate) struct PreparedLogo {
    file: NamedTempFile,
    extension: String,
    checksum: Checksum,
}

impl PreparedLogo {
    pub(crate) fn new(source: &Path, version: u32) -> Result<Self, String> {
        if !matches!(version, 0x20000 | 0x20003) {
            return Err(format!("unsupported logo checksum version: 0x{version:x}"));
        }
        let extension = source
            .extension()
            .and_then(|s| s.to_str())
            .ok_or_else(|| "image has no valid file extension".to_string())?
            .to_ascii_lowercase();
        let input = fs::read(source).map_err(|e| format!("read image: {e}"))?;
        let bytes = if extension == "png" {
            normalize_png(&input)?
        } else {
            input
        };
        let checksum = match version {
            0x20000 => Checksum::Crc32(crc32fast::hash(&bytes[..bytes.len().min(512)])),
            0x20003 => Checksum::Sha256(Sha256::digest(&bytes).into()),
            _ => unreachable!(),
        };
        let mut file = NamedTempFile::new().map_err(|e| format!("create image snapshot: {e}"))?;
        file.write_all(&bytes)
            .map_err(|e| format!("write image snapshot: {e}"))?;
        Ok(Self {
            file,
            extension,
            checksum,
        })
    }

    pub(crate) fn path(&self) -> Result<&str, String> {
        self.file
            .path()
            .to_str()
            .ok_or_else(|| "image snapshot path is not UTF-8".to_string())
    }

    pub(crate) fn extension(&self) -> &str {
        &self.extension
    }

    pub(crate) fn descriptor(&self, mut descriptor: [u8; 40]) -> [u8; 40] {
        match self.checksum {
            Checksum::Crc32(value) => descriptor[4..8].copy_from_slice(&value.to_le_bytes()),
            Checksum::Sha256(value) => descriptor[4..36].copy_from_slice(&value),
        }
        descriptor
    }
}

fn normalize_png(input: &[u8]) -> Result<Vec<u8>, String> {
    // ImageReader retains the decoder's default allocation limits.
    let rgba = ImageReader::with_format(Cursor::new(input), ImageFormat::Png)
        .decode()
        .map_err(|e| format!("decode PNG: {e}"))?
        .to_rgba8();
    let mut rgb = image::RgbImage::new(rgba.width(), rgba.height());
    for (source, target) in rgba.pixels().zip(rgb.pixels_mut()) {
        let alpha = u32::from(source[3]);
        for channel in 0..3 {
            // Composite onto black, rounding to nearest as in Pillow's masked paste.
            target[channel] = ((u32::from(source[channel]) * alpha + 127) / 255) as u8;
        }
    }
    // A fresh encoder emits 8-bit RGB, non-interlaced PNG, without source metadata.
    let mut output = Vec::new();
    PngEncoder::new(&mut output)
        .write_image(
            rgb.as_raw(),
            rgb.width(),
            rgb.height(),
            ExtendedColorType::Rgb8,
        )
        .map_err(|e| format!("encode RGB PNG: {e}"))?;
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn png_fixture() -> Vec<u8> {
        let mut png = Vec::new();
        PngEncoder::new(&mut png)
            .write_image(
                &[255, 40, 10, 255, 200, 100, 50, 0, 255, 101, 51, 128],
                3,
                1,
                ExtendedColorType::Rgba8,
            )
            .unwrap();
        // Insert valid optional metadata before IDAT; it must not reach the ESP.
        let payload = b"Comment\0firmware compatibility test";
        let mut chunk = Vec::new();
        chunk.extend_from_slice(&(payload.len() as u32).to_be_bytes());
        chunk.extend_from_slice(b"tEXt");
        chunk.extend_from_slice(payload);
        chunk.extend_from_slice(&crc32fast::hash(&chunk[4..]).to_be_bytes());
        png.splice(33..33, chunk);
        png
    }

    fn source(bytes: &[u8], suffix: &str) -> NamedTempFile {
        let mut file = tempfile::Builder::new().suffix(suffix).tempfile().unwrap();
        file.write_all(bytes).unwrap();
        file
    }

    #[test]
    fn png_is_rgb8_black_backed_noninterlaced_without_metadata() {
        let input = png_fixture();
        let original = source(&input, ".PNG");
        let prepared = PreparedLogo::new(original.path(), 0x20000).unwrap();
        assert_eq!(prepared.extension(), "png");
        let bytes = fs::read(prepared.path().unwrap()).unwrap();
        assert_eq!(&bytes[24..29], &[8, 2, 0, 0, 0]); // depth, RGB, compression, filter, interlace
        let image = image::load_from_memory(&bytes).unwrap().to_rgb8();
        assert_eq!(image.dimensions(), (3, 1));
        assert_eq!(image.as_raw(), &[255, 40, 10, 0, 0, 0, 128, 51, 26]);
        let mut offset = 8;
        while offset < bytes.len() {
            let length = u32::from_be_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize;
            let kind = &bytes[offset + 4..offset + 8];
            assert!(matches!(kind, b"IHDR" | b"IDAT" | b"IEND"));
            offset += 12 + length;
        }
        assert_eq!(fs::read(original.path()).unwrap(), input);
        let path = prepared.file.path().to_path_buf();
        drop(prepared);
        assert!(!path.exists());
    }

    #[test]
    fn grayscale_and_sixteen_bit_pngs_are_encoded_as_rgb8() {
        for (pixels, color, expected) in [
            (vec![128], ExtendedColorType::L8, [128, 128, 128]),
            (
                u16::MAX.to_ne_bytes().to_vec(),
                ExtendedColorType::L16,
                [255, 255, 255],
            ),
        ] {
            let mut input = Vec::new();
            PngEncoder::new(&mut input)
                .write_image(&pixels, 1, 1, color)
                .unwrap();
            let original = source(&input, ".png");
            let prepared = PreparedLogo::new(original.path(), 0x20003).unwrap();
            let bytes = fs::read(prepared.path().unwrap()).unwrap();
            assert_eq!(&bytes[24..29], &[8, 2, 0, 0, 0]);
            assert_eq!(
                image::load_from_memory(&bytes).unwrap().to_rgb8().as_raw(),
                &expected
            );
        }
    }

    #[test]
    fn both_protocols_hash_the_prepared_file_and_preserve_other_descriptor_bytes() {
        let input = png_fixture();
        let original = source(&input, ".png");
        for version in [0x20000_u32, 0x20003] {
            let prepared = PreparedLogo::new(original.path(), version).unwrap();
            let bytes = fs::read(prepared.path().unwrap()).unwrap();
            let mut before = [0xa5; 40];
            before[..4].copy_from_slice(&version.to_le_bytes());
            let after = prepared.descriptor(before);
            assert_eq!(&after[..4], &before[..4]);
            if version == 0x20000 {
                let expected = crc32fast::hash(&bytes[..bytes.len().min(512)]);
                assert_eq!(&after[4..8], &expected.to_le_bytes());
                assert_ne!(expected, crc32fast::hash(&input[..input.len().min(512)]));
                assert_eq!(&after[8..], &before[8..]);
            } else {
                assert_eq!(&after[4..36], Sha256::digest(&bytes).as_slice());
                assert_ne!(Sha256::digest(&bytes), Sha256::digest(&input));
                assert_eq!(&after[36..], &before[36..]);
            }
        }
    }

    #[test]
    fn other_formats_are_snapshotted_without_conversion() {
        let input: Vec<u8> = (0..900).map(|n| (n % 251) as u8).collect();
        for suffix in [".jpg", ".tga", ".pcx", ".gif", ".bmp"] {
            let original = source(&input, suffix);
            let prepared = PreparedLogo::new(original.path(), 0x20000).unwrap();
            assert_eq!(fs::read(prepared.path().unwrap()).unwrap(), input);
            let descriptor = prepared.descriptor([0; 40]);
            assert_eq!(
                &descriptor[4..8],
                &crc32fast::hash(&input[..512]).to_le_bytes()
            );
            // Changing the selected source later cannot change what will be copied.
            fs::write(original.path(), b"changed after preparation").unwrap();
            assert_eq!(fs::read(prepared.path().unwrap()).unwrap(), input);
        }
    }

    #[test]
    fn invalid_input_and_unsupported_protocol_fail_during_preparation() {
        let invalid = source(b"not a PNG", ".png");
        assert!(PreparedLogo::new(invalid.path(), 0x20000).is_err());
        let valid = source(&png_fixture(), ".png");
        assert!(PreparedLogo::new(valid.path(), 0x20001).is_err());
        let no_extension = source(b"data", "");
        assert!(PreparedLogo::new(no_extension.path(), 0x20000).is_err());
    }
}
