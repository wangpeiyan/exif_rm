#[derive(Debug, thiserror::Error, uniffi::Error)]
#[uniffi(flat_error)]
pub enum Error {
    #[error("unsupported file format")]
    UnsupportedFormat,

    #[error("failed to detect file format from magic bytes")]
    FormatDetectionFailed,

    #[error("invalid or corrupted {0}")]
    InvalidData(String),

    #[error("PDF is encrypted; password required")]
    EncryptedPdf,

    #[error("I/O error: {0}")]
    Io(String),

    #[error("{0}")]
    External(String),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e.to_string())
    }
}

#[cfg(feature = "pdf")]
impl From<lopdf::Error> for Error {
    fn from(e: lopdf::Error) -> Self {
        if matches!(e, lopdf::Error::Decryption(_)) {
            Error::EncryptedPdf
        } else {
            Error::External(format!("PDF: {e:?}"))
        }
    }
}

#[cfg(feature = "office")]
impl From<zip::result::ZipError> for Error {
    fn from(e: zip::result::ZipError) -> Self {
        Error::External(format!("ZIP: {e:?}"))
    }
}

pub type Result<T> = std::result::Result<T, Error>;
