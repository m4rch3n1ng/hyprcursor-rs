use std::path::Path;

/// temporary struct to parse into
#[derive(Debug)]
pub struct Manifest {
	pub name: String,
	pub description: Option<String>,
	pub version: Option<String>,
	pub author: Option<String>,
	pub cursors_directory: String,
}

impl Manifest {
	// todo make like a proper parser for this godforsaken format
	// todo error variants:
	// - cursors_directory not set
	pub fn from_hyprlang(dir: &Path, file: String) -> Option<Self> {
		let mut name = None;
		let mut description = None;
		let mut version = None;
		let mut author = None;
		let mut cursors_directory = None;

		for line in file.lines() {
			let Some((ident, value)) = line.split_once('=') else {
				continue;
			};

			let ident = ident.trim();
			let value = value.trim();

			match ident {
				"name" => name = Some(value.to_owned()),
				"description" => description = Some(value.to_owned()),
				"author" => author = Some(value.to_owned()),
				"version" => version = Some(value.to_owned()),
				"cursors_directory" => cursors_directory = Some(value.to_owned()),
				_ => {}
			}
		}

		let name = name
			.or_else(|| {
				// hyprcursor uses stem so i will use it too
				dir.file_stem()
					.and_then(|stem| stem.to_str())
					.map(ToOwned::to_owned)
			})
			.unwrap_or_default();
		let cursors_directory = cursors_directory?;

		Some(Manifest {
			name,
			description,
			version,
			author,
			cursors_directory,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
	Png,
	Svg,
}

impl Size {
	// todo error
	// - invalid kind
	// - missing / invalid size
	// - missing file
	fn from_str(value: &str) -> Option<Size> {
		let mut split = value.split(',').map(str::trim);

		let size = split.next()?.parse::<u32>().ok()?;
		let file = split.next()?.to_owned();

		let kind = match Path::new(&file).extension()?.to_str().unwrap() {
			"png" => Kind::Png,
			"svg" => Kind::Svg,
			_ => return None,
		};

		// todo give out error
		let delay = split.next().and_then(|delay| delay.parse::<u32>().ok());

		Some(Size {
			size,
			file,
			delay,
			kind,
		})
	}
}

/// `meta.hl` / `meta.toml`
#[derive(Debug)]
pub struct Meta {
	#[expect(dead_code, reason = "todo")]
	pub name: String,

	#[expect(dead_code, reason = "todo")]
	pub resize_algorithm: ResizeAlgorithm,
	#[expect(dead_code, reason = "todo")]
	pub hotspot_x: f32,
	#[expect(dead_code, reason = "todo")]
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
	pub fn from_hyprlang(path: &Path, file: String) -> Option<Self> {
		let name = path.file_stem().unwrap().to_str()?.to_owned();

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
						_ => return None,
					})
				}
				"hotspot_x" => hotspot_x = Some(value.parse().ok()?),
				"hotspot_y" => hotspot_y = Some(value.parse().ok()?),

				// todo split at ';'
				"define_override" => overrides.push(value.to_owned()),
				"define_size" => {
					let size = Size::from_str(value)?;
					if let Some(kind) = &kind {
						if *kind != size.kind {
							return None;
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
		let hotspot_x = hotspot_x?;
		let hotspot_y = hotspot_y?;

		Some(Meta {
			name,

			resize_algorithm,
			hotspot_x,
			hotspot_y,

			overrides,
			sizes,
		})
	}
}
