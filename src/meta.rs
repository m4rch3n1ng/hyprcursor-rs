use crate::error::{InvalidFloat, InvalidInt};
use std::{path::Path, str::FromStr};

/// error when parsing `meta.hl` / `meta.toml`
#[derive(Debug, thiserror::Error)]
pub enum MetaError {
	#[error("no size defined")]
	NoSizeDefined,
	#[error("invalid resize algorithm {0:?}")]
	InvalidResizeAlgorithm(String),
	#[error("hotspot_x not set")]
	MissingHotspotX,
	#[error("hotspot_x is invalid")]
	InvalidHotspotX(#[source] InvalidFloat),
	#[error("hotspot_y not set")]
	MissingHotspotY,
	#[error("hotspot_y is invalid")]
	InvalidHotspotY(#[source] InvalidFloat),

	/// size definition empty
	#[error("size definition empty")]
	EmptyDefinition,
	#[error("invalid size")]
	InvalidSize(#[source] InvalidInt),
	#[error("file not set in size definition")]
	FileMissing,
	#[error("no extension specified for file")]
	MissingExtension,
	#[error("unknown extension {0}, expected svg or png")]
	InvalidExtension(String),
	#[error("invalid delay")]
	InvalidDelay(#[source] InvalidInt),
	#[error("delay has to be > 0")]
	DelayIsZero,

	#[error("both png and svg defined")]
	MoreThanOneFormat,
}

/// `meta.hl` / `meta.toml`
#[derive(Debug)]
pub struct Meta {
	pub name: String,

	pub resize_algorithm: ResizeAlgorithm,
	pub hotspot_x: f32,
	pub hotspot_y: f32,

	pub overrides: Vec<String>,
	pub sizes: Vec<Size>,
}

impl Meta {
	pub fn from_hyprlang(path: &Path, file: String) -> Result<Self, MetaError> {
		let name = path.file_stem().unwrap().to_str().unwrap().to_owned();

		let mut resize_algorithm: Option<ResizeAlgorithm> = None;
		let mut hotspot_x: Option<f32> = None;
		let mut hotspot_y: Option<f32> = None;

		let mut overrides = vec![name.clone()];
		let mut sizes = Vec::new();

		let mut kind = None;

		for line in file.lines() {
			let Some((ident, value)) = line.split_once('=') else {
				continue;
			};

			let ident = ident.trim();
			let value = value.trim();

			match ident {
				"resize_algorithm" => {
					// todo properly
					resize_algorithm = Some(match value {
						"none" => ResizeAlgorithm::None,
						"bilinear" => ResizeAlgorithm::Bilinear,
						"nearest" => ResizeAlgorithm::Nearest,
						algorithm => {
							return Err(MetaError::InvalidResizeAlgorithm(algorithm.to_owned()))
						}
					})
				}

				"hotspot_x" => {
					let hotx = value.parse::<f32>().map_err(|err| {
						MetaError::InvalidHotspotX(InvalidFloat {
							err,
							number: value.to_owned(),
						})
					})?;
					hotspot_x = Some(hotx)
				}
				"hotspot_y" => {
					let hoty = value.parse::<f32>().map_err(|err| {
						MetaError::InvalidHotspotX(InvalidFloat {
							err,
							number: value.to_owned(),
						})
					})?;
					hotspot_y = Some(hoty)
				}

				// todo split at ';'
				"define_override" => overrides.push(value.to_owned()),
				"define_size" => {
					let size = Size::from_str(value)?;
					if let Some(kind) = &kind {
						if *kind != size.kind {
							return Err(MetaError::MoreThanOneFormat);
						}
					} else {
						kind = Some(size.kind)
					};

					sizes.push(size);
				}

				_ => continue,
			}
		}

		if sizes.is_empty() {
			return Err(MetaError::NoSizeDefined);
		}

		let resize_algorithm = resize_algorithm.unwrap_or(ResizeAlgorithm::None);
		let hotspot_x = hotspot_x.ok_or(MetaError::MissingHotspotX)?;
		let hotspot_y = hotspot_y.ok_or(MetaError::MissingHotspotY)?;

		Ok(Meta {
			name,

			resize_algorithm,
			hotspot_x,
			hotspot_y,

			overrides,
			sizes,
		})
	}
}

#[derive(Debug)]
pub enum ResizeAlgorithm {
	None,
	Bilinear,
	Nearest,
}

#[derive(Debug)]
pub struct Size {
	pub size: u32,
	pub file: String,
	pub delay: Option<u32>,

	pub kind: Kind,
}

impl FromStr for Size {
	type Err = MetaError;

	fn from_str(value: &str) -> Result<Size, Self::Err> {
		let mut split = value.split(',').map(str::trim);

		let size = split.next().ok_or(MetaError::EmptyDefinition)?;
		let size = size.parse::<u32>().map_err(|err| {
			MetaError::InvalidSize(InvalidInt {
				err,
				number: size.to_owned(),
			})
		})?;

		let file = split.next().ok_or(MetaError::FileMissing)?.to_owned();

		let kind = match Path::new(&file)
			.extension()
			.ok_or(MetaError::MissingExtension)?
			.to_str()
		{
			Some("png") => Kind::Png,
			Some("svg") => Kind::Svg,
			Some(ext) => return Err(MetaError::InvalidExtension(ext.to_owned())),
			None => return Err(MetaError::MissingExtension),
		};

		let delay = split
			.next()
			.map(|delay| match delay.parse::<u32>() {
				Err(err) => Err(MetaError::InvalidDelay(InvalidInt {
					err,
					number: delay.to_owned(),
				})),
				Ok(0) => Err(MetaError::DelayIsZero),
				Ok(e) => Ok(e),
			})
			.transpose()?;

		Ok(Size {
			size,
			file,
			delay,
			kind,
		})
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
	Png,
	Svg,
}
