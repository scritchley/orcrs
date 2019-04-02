use std::io::Error;
use std::result;

/// The result type used for all stream readers. Provides a way of 
/// identifying both errors and the end of the stream.
pub type Result<T> = result::Result<Option<T>, ReaderError>;

/// The error type for Reader operations. This can include IO errors 
/// or errors that occur when attempting to deserialize a stream into 
/// the ORC format.
#[derive(Debug)]
pub enum ReaderError {
    IO(Error),
}

