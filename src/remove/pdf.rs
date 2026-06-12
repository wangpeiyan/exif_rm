use crate::traits::{FileFormat, MetadataRemover, RemovalOptions};

pub struct PdfRemover;

impl MetadataRemover for PdfRemover {
    fn format(&self) -> FileFormat {
        FileFormat::Pdf
    }

    fn remove_metadata(&self, input: &[u8], options: &RemovalOptions) -> crate::Result<Vec<u8>> {
        let mut doc = lopdf::Document::load_mem(input)?;

        // Remove /Info dictionary from trailer
        if options.document_properties {
            if let Ok(info_ref) = doc.trailer.get(b"Info") {
                if let lopdf::Object::Reference(obj_id) = info_ref {
                    doc.delete_object(*obj_id);
                }
                doc.trailer.remove(b"Info");
            }
        }

        // Remove /Metadata from the document catalog
        if options.xmp {
            if let Ok(lopdf::Object::Reference(catalog_id)) = doc.trailer.get(b"Root") {
                if let Some(catalog_obj) = doc.objects.get_mut(catalog_id) {
                    if let Ok(catalog_dict) = catalog_obj.as_dict_mut() {
                        if catalog_dict.has(b"Metadata") {
                            catalog_dict.remove(b"Metadata");
                        }
                    }
                }
            }
        }

        doc.prune_objects();
        doc.renumber_objects();

        let mut output = Vec::new();
        doc.save_to(&mut output)?;
        Ok(output)
    }
}
