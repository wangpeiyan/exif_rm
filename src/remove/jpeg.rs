use crate::error::Error;
use crate::traits::{FileFormat, MetadataRemover, RemovalOptions};

pub struct JpegRemover;

impl MetadataRemover for JpegRemover {
    fn format(&self) -> FileFormat {
        FileFormat::Jpeg
    }

    fn remove_metadata(&self, input: &[u8], options: &RemovalOptions) -> crate::Result<Vec<u8>> {
        let mut output = Vec::with_capacity(input.len());

        // Validate JPEG SOI marker
        if input.len() < 4 || input[0] != 0xFF || input[1] != 0xD8 {
            return Err(Error::InvalidData("JPEG".into()));
        }

        output.extend_from_slice(&input[0..2]); // SOI
        let mut pos = 2;

        while pos < input.len() {
            if input[pos] != 0xFF {
                return Err(Error::InvalidData("expected marker".into()));
            }

            let marker = input[pos + 1];

            // SOS: copy header + entropy-coded data until next marker
            if marker == 0xDA {
                output.extend_from_slice(&input[pos..]);
                break;
            }

            // EOI
            if marker == 0xD9 {
                output.extend_from_slice(&input[pos..pos + 2]);
                break;
            }

            // Standalone markers (no length field): RST0-RST7, SOI
            if (0xD0..=0xD7).contains(&marker) || marker == 0x01 {
                output.extend_from_slice(&input[pos..pos + 2]);
                pos += 2;
                continue;
            }

            // Read segment length (big-endian, includes the 2 length bytes)
            if pos + 4 > input.len() {
                return Err(Error::InvalidData("truncated marker segment".into()));
            }
            let seg_len = u16::from_be_bytes([input[pos + 2], input[pos + 3]]) as usize;
            let seg_end = pos + 2 + seg_len;
            if seg_end > input.len() {
                return Err(Error::InvalidData("truncated marker segment".into()));
            }

            let should_strip = should_strip_marker(marker, options);
            if !should_strip {
                output.extend_from_slice(&input[pos..seg_end]);
            }

            pos = seg_end;
        }

        Ok(output)
    }
}

fn should_strip_marker(marker: u8, options: &RemovalOptions) -> bool {
    match marker {
        // APP1: EXIF (0xE1 with "Exif" header) or XMP
        0xE1 => options.exif || options.xmp,
        // APP2: ICC profile
        0xE2 => options.icc_profile,
        // APP13: IPTC/NAA
        0xED => options.iptc || options.xmp,
        // COM: comment
        0xFE => options.comments,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_jpeg_with_exif() -> Vec<u8> {
        let mut jpeg = vec![0xFF, 0xD8]; // SOI
        // APP1 with Exif header and minimal valid TIFF structure
        // TIFF structure (big-endian):
        // - Header: MM (byte order) + 0x002A (magic) + IFD0 offset (8)
        // - IFD0 at offset 8: entry count (1) + one entry + next IFD offset (0)
        let tiff_data: Vec<u8> = vec![
            // TIFF header
            b'M', b'M',             // Big-endian byte order
            0x00, 0x2A,             // TIFF magic number (42)
            0x00, 0x00, 0x00, 0x08, // Offset to IFD0 (8 bytes from TIFF start)
            // IFD0 at offset 8
            0x00, 0x01,             // 1 directory entry
            // Entry 1: ImageWidth (tag 0x0100), SHORT type (3), count 1, value 800
            0x01, 0x00,             // Tag: ImageWidth
            0x00, 0x03,             // Type: SHORT (3)
            0x00, 0x00, 0x00, 0x01, // Count: 1
            0x03, 0x20, 0x00, 0x00, // Value: 800 (0x320)
            // Next IFD offset
            0x00, 0x00, 0x00, 0x00, // No next IFD
        ];
        let exif_header = b"Exif\x00\x00";
        let seg_len = (exif_header.len() + tiff_data.len() + 2) as u16;
        jpeg.push(0xFF);
        jpeg.push(0xE1);
        jpeg.extend_from_slice(&seg_len.to_be_bytes());
        jpeg.extend_from_slice(exif_header);
        jpeg.extend_from_slice(&tiff_data);
        // DQT (fake)
        jpeg.extend_from_slice(&[0xFF, 0xDB, 0x00, 0x02]);
        // SOS + data + EOI
        jpeg.extend_from_slice(&[0xFF, 0xDA, 0x00, 0x02]);
        jpeg.extend_from_slice(&[0x00]); // fake entropy data
        jpeg.extend_from_slice(&[0xFF, 0xD9]); // EOI
        jpeg
    }

    #[test]
    fn test_strip_exif() {
        let input = minimal_jpeg_with_exif();
        let remover = JpegRemover;
        let output = remover.remove_metadata(&input, &RemovalOptions::default()).unwrap();
        // Should not contain APP1 marker
        assert!(!window_contains(&output, &[0xFF, 0xE1]));
        // Should still contain DQT
        assert!(window_contains(&output, &[0xFF, 0xDB]));
    }

    fn window_contains(haystack: &[u8], needle: &[u8]) -> bool {
        haystack.windows(needle.len()).any(|w| w == needle)
    }
}
