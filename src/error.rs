use failure::Fail;

use cargo_metadata::Error as MetadataError;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Error retrieving project metadata: {:?}", 0)]
    ParseMetadataError(#[cause] MetadataError),
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