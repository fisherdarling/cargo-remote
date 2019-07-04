use failure::Fail;

use cargo_metadata::Error as MetadataError;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "IO Error: {:?}", 0)]
    IOError(#[cause] std::io::Error),
    #[fail(display = "Error transfering files: {:?}", 0)]
    TransferFilesError(#[cause] std::io::Error),
    #[fail(display = "Error running cargo command remotely: {:?}", 0)]
    RunCargoCommandError(#[cause] std::io::Error),
    #[fail(display = "Error retrieving project metadata: {:?}", 0)]
    ParseMetadataError(#[cause] MetadataError),
    #[fail(display = "Can't parse config file: {:?}", 0)]
    ParseTomlError(#[cause] toml::de::Error),
    #[fail(display = "No project found")]
    NoProjectFoundError,
    #[fail(display = "Cargo.toml must have a parent")]
    CargoTomlNoParentError,
}

impl From<MetadataError> for Error {
    fn from(error: MetadataError) -> Self {
        Error::ParseMetadataError(error)
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::IOError(error)
    }
}