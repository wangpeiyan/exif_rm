use std::io::{Cursor, Read as IoRead, Write as IoWrite};

use crate::error::Error;
use crate::traits::{FileFormat, MetadataRemover, RemovalOptions};

pub struct OfficeRemover;

impl MetadataRemover for OfficeRemover {
    fn format(&self) -> FileFormat {
        FileFormat::Docx
    }

    fn remove_metadata(&self, input: &[u8], options: &RemovalOptions) -> crate::Result<Vec<u8>> {
        let mut src = zip::ZipArchive::new(Cursor::new(input))
            .map_err(|_| Error::InvalidData("ZIP".into()))?;

        let output_buf = Vec::new();
        let cursor = Cursor::new(output_buf);
        let mut dst = zip::ZipWriter::new(cursor);

        for i in 0..src.len() {
            let mut entry = src.by_index(i)?;
            let name = entry.name().to_string();

            // Skip custom.xml entirely
            if name == "docProps/custom.xml" {
                continue;
            }

            // Sanitize core.xml and app.xml: empty all metadata element content
            if (name == "docProps/core.xml" || name == "docProps/app.xml")
                && options.document_properties
            {
                let mut xml_data = Vec::new();
                entry.read_to_end(&mut xml_data)?;
                let sanitized = sanitize_xml_metadata(&xml_data)?;
                let opts = zip::write::SimpleFileOptions::default()
                    .compression_method(entry.compression());
                dst.start_file(&name, opts)?;
                dst.write_all(&sanitized)?;
                continue;
            }

            // Update [Content_Types].xml to remove custom.xml override
            if name == "[Content_Types].xml" {
                let mut xml_data = Vec::new();
                entry.read_to_end(&mut xml_data)?;
                let cleaned = clean_content_types(&xml_data)?;
                let opts = zip::write::SimpleFileOptions::default()
                    .compression_method(entry.compression());
                dst.start_file(&name, opts)?;
                dst.write_all(&cleaned)?;
                continue;
            }

            // All other entries: copy verbatim
            dst.raw_copy_file(entry)?;
        }

        let cursor = dst.finish()?;
        Ok(cursor.into_inner())
    }
}

/// Empty all text content inside XML metadata elements while preserving structure.
fn sanitize_xml_metadata(xml_data: &[u8]) -> crate::Result<Vec<u8>> {
    use quick_xml::events::Event;
    use quick_xml::Reader;
    use quick_xml::Writer;

    let mut reader = Reader::from_reader(xml_data);
    let mut writer = Writer::new(Vec::new());

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                writer.write_event(Event::Start(e.into_owned())).map_err(|e| Error::InvalidData(format!("XML write: {e}")))?;
            }
            Ok(Event::Empty(e)) => {
                writer.write_event(Event::Empty(e.into_owned())).map_err(|e| Error::InvalidData(format!("XML write: {e}")))?;
            }
            Ok(Event::Text(_)) | Ok(Event::CData(_)) => {
                // Skip text/CDATA content (empties the element)
            }
            Ok(Event::End(e)) => {
                writer.write_event(Event::End(e)).map_err(|e| Error::InvalidData(format!("XML write: {e}")))?;
            }
            Ok(Event::Eof) => break,
            Ok(event) => {
                writer.write_event(event).map_err(|e| Error::InvalidData(format!("XML write: {e}")))?;
            }
            Err(e) => return Err(Error::InvalidData(format!("XML parse: {e}"))),
        }
    }

    Ok(writer.into_inner())
}

/// Remove Override entries for docProps/custom.xml from [Content_Types].xml.
fn clean_content_types(xml_data: &[u8]) -> crate::Result<Vec<u8>> {
    use quick_xml::events::Event;
    use quick_xml::Reader;
    use quick_xml::Writer;

    let mut reader = Reader::from_reader(xml_data);
    let mut writer = Writer::new(Vec::new());

    loop {
        match reader.read_event() {
            Ok(Event::Empty(e)) => {
                let is_custom_override = e.name().as_ref() == b"Override"
                    && e.attributes().any(|attr| {
                        attr.map(|a| a.key.as_ref() == b"PartName"
                            && a.value.as_ref().ends_with(b"custom.xml"))
                            .unwrap_or(false)
                    });

                if !is_custom_override {
                    writer.write_event(Event::Empty(e.into_owned())).map_err(|e| Error::InvalidData(format!("XML write: {e}")))?;
                }
            }
            Ok(Event::Eof) => break,
            Ok(event) => {
                writer.write_event(event).map_err(|e| Error::InvalidData(format!("XML write: {e}")))?;
            }
            Err(e) => return Err(Error::InvalidData(format!("XML parse: {e}"))),
        }
    }

    Ok(writer.into_inner())
}