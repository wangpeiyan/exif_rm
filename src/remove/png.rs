use crate::error::Error;
use crate::traits::{FileFormat, MetadataRemover, RemovalOptions};

pub struct PngRemover;

impl MetadataRemover for PngRemover {
    fn format(&self) -> FileFormat {
        FileFormat::Png
    }

    fn remove_metadata(&self, input: &[u8], options: &RemovalOptions) -> crate::Result<Vec<u8>> {
        if input.len() < 8 || !input.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) {
            return Err(Error::InvalidData("PNG".into()));
        }

        let mut output = Vec::with_capacity(input.len());
        output.extend_from_slice(&input[0..8]); // PNG signature

        let mut pos = 8;
        while pos + 12 <= input.len() {
            let length = u32::from_be_bytes([input[pos], input[pos + 1], input[pos + 2], input[pos + 3]]) as usize;
            let chunk_type = &input[pos + 4..pos + 8];
            let chunk_end = pos + 12 + length; // 4 len + 4 type + data + 4 crc

            if chunk_end > input.len() {
                return Err(Error::InvalidData("truncated PNG chunk".into()));
            }

            if should_strip_chunk(chunk_type, options) {
                pos = chunk_end;
                continue;
            }

            output.extend_from_slice(&input[pos..chunk_end]);
            pos = chunk_end;
        }

        Ok(output)
    }
}

fn should_strip_chunk(chunk_type: &[u8], options: &RemovalOptions) -> bool {
    match chunk_type {
        b"eXIf" => options.exif,
        b"tEXt" | b"zTXt" | b"iTXt" => options.xmp,
        b"iCCP" => options.icc_profile,
        b"tIME" => options.timestamps,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_png_with_text() -> Vec<u8> {
        let mut png = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]; // signature
        // IHDR chunk: 1x1 8-bit RGB
        let ihdr_data: &[u8] = &[0, 0, 0, 1, 0, 0, 0, 1, 8, 2, 0, 0, 0];
        png.extend_from_slice(&(ihdr_data.len() as u32).to_be_bytes());
        png.extend_from_slice(b"IHDR");
        png.extend_from_slice(ihdr_data);
        let _crc = crc32fast::hash(ihdr_data);
        // CRC includes chunk type + data
        let mut crc_input = b"IHDR".to_vec();
        crc_input.extend_from_slice(ihdr_data);
        let crc = crc32fast::hash(&crc_input);
        png.extend_from_slice(&crc.to_be_bytes());
        // tEXt chunk with metadata
        let text_data = b"Author\x00Test Author";
        png.extend_from_slice(&(text_data.len() as u32).to_be_bytes());
        png.extend_from_slice(b"tEXt");
        png.extend_from_slice(text_data);
        let mut crc_input = b"tEXt".to_vec();
        crc_input.extend_from_slice(text_data);
        let crc = crc32fast::hash(&crc_input);
        png.extend_from_slice(&crc.to_be_bytes());
        // IDAT chunk (minimal 1x1 pixel)
        let idat_data: &[u8] = &[0x08, 0xD7, 0x63, 0xF8, 0xCF, 0xC0, 0x00, 0x00, 0x00, 0x02, 0x00, 0x01];
        png.extend_from_slice(&(idat_data.len() as u32).to_be_bytes());
        png.extend_from_slice(b"IDAT");
        png.extend_from_slice(idat_data);
        let mut crc_input = b"IDAT".to_vec();
        crc_input.extend_from_slice(idat_data);
        let crc = crc32fast::hash(&crc_input);
        png.extend_from_slice(&crc.to_be_bytes());
        // IEND chunk
        png.extend_from_slice(&0u32.to_be_bytes());
        png.extend_from_slice(b"IEND");
        let crc = crc32fast::hash(b"IEND");
        png.extend_from_slice(&crc.to_be_bytes());
        png
    }

    #[test]
    fn test_strip_text_chunk() {
        let input = minimal_png_with_text();
        let remover = PngRemover;
        let output = remover.remove_metadata(&input, &RemovalOptions::default()).unwrap();
        // Should not contain tEXt chunk
        assert!(!window_contains(&output, b"tEXt"));
        // Should contain IHDR and IDAT
        assert!(window_contains(&output, b"IHDR"));
        assert!(window_contains(&output, b"IDAT"));
    }

    fn window_contains(haystack: &[u8], needle: &[u8]) -> bool {
        haystack.windows(needle.len()).any(|w| w == needle)
    }
}
