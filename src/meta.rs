use std::{path::Path, str::FromStr};

/// error when parsing `meta.hl` / `meta.toml`
#[derive(Debug, thiserror::Error)]
pub enum MetaError {
	#[error("# todo other")]
	Other,
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
	// todo: error variant
	// - invalid size
	// - missing file
	// - both svg and png specified
	// - no sizes set
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
						_ => return Err(MetaError::Other),
					})
				}
				"hotspot_x" => hotspot_x = Some(value.parse().map_err(|_| MetaError::Other)?),
				"hotspot_y" => hotspot_y = Some(value.parse().map_err(|_| MetaError::Other)?),

				// todo split at ';'
				"define_override" => overrides.push(value.to_owned()),
				"define_size" => {
					let size = Size::from_str(value)?;
					if let Some(kind) = &kind {
						if *kind != size.kind {
							return Err(MetaError::Other);
						}
					} else {
						kind = Some(size.kind)
					};

					sizes.push(size);
				}

				_ => continue,
			}
		}

		let resize_algorithm = resize_algorithm.unwrap_or(ResizeAlgorithm::None);
		let hotspot_x = hotspot_x.ok_or(MetaError::Other)?;
		let hotspot_y = hotspot_y.ok_or(MetaError::Other)?;

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

	// todo error
	// - invalid kind
	// - missing / invalid size
	// - missing file
	fn from_str(value: &str) -> Result<Size, Self::Err> {
		let mut split = value.split(',').map(str::trim);

		let size = split
			.next()
			.ok_or(MetaError::Other)?
			.parse::<u32>()
			.map_err(|_| MetaError::Other)?;
		let file = split.next().ok_or(MetaError::Other)?.to_owned();

		// Error::MissingExtension
		let kind = match Path::new(&file)
			.extension()
			.ok_or(MetaError::Other)?
			.to_str()
			.unwrap()
		{
			"png" => Kind::Png,
			"svg" => Kind::Svg,
			// Error::MissingExtension
			_ => return Err(MetaError::Other),
		};

		// todo give out error
		let delay = split.next().and_then(|delay| delay.parse::<u32>().ok());
		delay.inspect(|delay| assert!(*delay > 0));

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
