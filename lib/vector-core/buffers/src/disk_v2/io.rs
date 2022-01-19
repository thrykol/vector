use std::{path::PathBuf, io};

use tokio::io::{AsyncRead, AsyncWrite};

/// Generalized interface for opening and deleting files from a filesystem.
pub trait Filesystem {
	type File: AsyncRead + AsyncWrite;

	/// Opens a file for writing, creating it if it does not exist.
	/// 
	/// The file will also be readable, as well.  Files will be opened at the logical end position,
	/// and never truncated.
	///
	/// # Errors
	/// 
	/// If an I/O error occurred when attempting to open the file for writing, an error variant will
	/// be returned describing the underlying error.
	fn open_writable(&self, path: &PathBuf) -> io::Result<Self::File>;

	/// Opens a file for writing, creating it if it does not already exist, but atomically.
	///
	/// The file will also be readable, as well.  Files will be opened at the logical end position,
	/// and never truncated.
	/// 
	/// # Errors
	/// If the file already existed, then an error will be returned with an `ErrorKind` of `AlreadyExists`.
	///
	/// If a general I/O error occurred when attempting to open the file for writing, an error variant will
	/// be returned describing the underlying error.
	fn open_writable_atomic(&self, path: &PathBuf) -> io::Result<Self::File>;

	/// Opens a file for readaing, creating it if it does not exist.
	/// 
	/// Files will be opened at the logical end position,
	///
	/// # Errors
	/// 
	/// If an I/O error occurred when attempting to open the file for reading, an error variant will
	/// be returned describing the underlying error.	
	fn open_readable(&self, path: &PathBuf) -> io::Result<Self::File>;

	/// Deletes a file.
	/// 
	/// # Errors
	/// 
	/// If an I/O error occurred when attempting to delete the file, an error variant will be
	/// returned describing the underlying error.     
	fn delete(&self, path: &PathBuf) -> io::Result<()>;
}