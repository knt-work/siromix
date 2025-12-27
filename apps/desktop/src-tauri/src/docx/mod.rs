use std::path::PathBuf;

pub mod read;
pub mod assets;
pub mod model;
pub mod parser;
pub mod validator;

#[allow(dead_code)]
#[derive(Debug)]
pub enum AppError {
    Io(std::io::Error),
    Zip(zip::result::ZipError),
    Utf8(std::string::FromUtf8Error),
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Io(err)
    }
}

impl From<zip::result::ZipError> for AppError {
    fn from(err: zip::result::ZipError) -> Self {
        AppError::Zip(err)
    }
}

impl From<std::string::FromUtf8Error> for AppError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        AppError::Utf8(err)
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ExtractedAsset {
    pub file_name: String,
    pub absolute_path: PathBuf,
}
