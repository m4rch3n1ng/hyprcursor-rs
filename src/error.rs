use crate::meta::MetaError;
use resvg::usvg::Error as UsvgErr;
use std::path::PathBuf;
use zip::result::ZipError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("couldn't find theme")]
	ThemeNotFound,

	#[error("error parsing meta in {path:?}")]
	MetaError {
		#[source]
		err: MetaError,
		path: PathBuf,
	},
	#[error("couldn't find meta file in {0:?}")]
	MetaNotFound(PathBuf),

	// todo attach more information
	#[error("io error")]
	Io(#[from] std::io::Error),
	// todo attach information for file not found errors
	#[error("error unzipping {path:?}")]
	ZipError {
		#[source]
		err: ZipError,
		path: PathBuf,
	},
	/// svg rendering error
	#[error("error rendering svg file {file:?}")]
	UsvgErr {
		#[source]
		err: UsvgErr,
		file: String,
	},

	#[error("todo more proper error handling")]
	Other,
}

#[derive(Debug, thiserror::Error)]
#[error("error parsing number {number}")]
pub struct InvalidInt {
	#[source]
	pub err: std::num::ParseIntError,
	pub number: String,
}

#[derive(Debug, thiserror::Error)]
#[error("error parsing number {number}")]
pub struct InvalidFloat {
	#[source]
	pub err: std::num::ParseFloatError,
	pub number: String,
}
