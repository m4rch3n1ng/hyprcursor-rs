//! a rust implementation of the hyprcursor format
//!
//! ```
//! use hyprcursor_rs::HyprcursorTheme;
//!
//! let theme = HyprcursorTheme::load("rose-pine-hyprcursor")?;
//! let cursor = theme.load_cursor("default").unwrap();
//!
//! let size = 24;
//! let frames = cursor.render_frames(size);
//!
//! # Ok::<(), hyprcursor_rs::Error>(())
//! ```

use self::manifest::Manifest;
use self::meta::Meta;
use png::Decoder as PngDecoder;
use resvg::{
	tiny_skia::Pixmap,
	usvg::{Options, Transform, Tree},
};
use std::{
	fmt::Debug,
	fs::File,
	io::{Read, Seek},
	path::{Path, PathBuf},
};
use zip::ZipArchive;

mod error;
mod manifest;
pub mod meta;

pub use self::error::Error;

fn xdg_data_dirs() -> Vec<PathBuf> {
	let Some(data_dirs) = std::env::var_os("XDG_DATA_DIRS") else {
		return vec![PathBuf::from("/usr/share/icons")];
	};

	std::env::split_paths(&data_dirs)
		.map(|mut path| {
			path.push("icons");
			path
		})
		.collect()
}

fn user_theme_dirs() -> [PathBuf; 2] {
	let home = std::env::var_os("XDG_HOME")
		.or_else(|| std::env::var_os("HOME"))
		.expect("$HOME is not set");
	let home = PathBuf::from(home);

	let xdg_data_home = std::env::var_os("XDG_DATA_HOME")
		.map(PathBuf::from)
		.unwrap_or_else(|| home.join(".local/share"));

	[xdg_data_home.join("icons"), home.join(".icons")]
}

/// a hyprcursor theme
#[derive(Debug)]
pub struct HyprcursorTheme {
	/// the name of this theme
	pub name: String,
	/// the description of this theme
	pub description: Option<String>,
	/// the version of this theme
	pub version: Option<String>,
	/// the author of this theme
	pub author: Option<String>,
	/// the directory where cursors are stored
	pub cursors_directory: String,

	path: PathBuf,
	cache: Vec<Hyprcursor>,
}

impl HyprcursorTheme {
	pub fn load(name: &str) -> Result<HyprcursorTheme, Error> {
		let mut theme = HyprcursorTheme::read(name)?;

		for cursor in theme.path.read_dir()?.map(Result::unwrap) {
			let cursor_path = cursor.path();
			if !cursor_path.extension().is_some_and(|ext| ext == "hlc") {
				continue;
			}

			let archive = File::open(&cursor_path)?;
			let mut archive = ZipArchive::new(archive).map_err(|err| Error::ZipError {
				err,
				path: cursor_path.clone(),
			})?;

			let (index, is_toml) = if let Some(index) = archive.index_for_path("meta.hl") {
				(index, false)
			} else if let Some(index) = archive.index_for_path("meta.toml") {
				(index, true)
			} else {
				return Err(Error::MetaNotFound(cursor_path));
			};

			let mut file = archive.by_index(index).map_err(|err| Error::ZipError {
				err,
				path: cursor_path.clone(),
			})?;

			let mut content = String::new();
			file.read_to_string(&mut content)?;
			drop(file);

			let meta = if is_toml {
				todo!();
			} else {
				Meta::from_hyprlang(&cursor_path, content).map_err(|err| {
					let path = cursor_path.join("meta.hl");
					Error::MetaError { err, path }
				})?
			};

			let cursor = Hyprcursor::new(meta, &cursor_path, archive)?;
			theme.cache.push(cursor);
		}

		Ok(theme)
	}

	fn read(name: &str) -> Result<HyprcursorTheme, Error> {
		let user_dirs = user_theme_dirs();
		let data_dirs = xdg_data_dirs();

		for path in user_dirs.into_iter().chain(data_dirs.into_iter()) {
			if !path.is_dir() {
				continue;
			};

			for theme in path.read_dir().unwrap().map_while(Result::ok) {
				let mut theme_dir = theme.path();

				let manifest = if let Ok(file) =
					std::fs::read_to_string(theme_dir.join("manifest.hl"))
				{
					Manifest::from_hyprlang(&theme_dir, file)
				} else if let Ok(_toml) = std::fs::read_to_string(theme_dir.join("manifest.toml")) {
					// i don't actually fully know how the toml spec is supposed
					// to look like
					todo!("toml support");
				} else {
					continue;
				};
				let Some(manifest) = manifest else { continue };

				if name == manifest.name {
					let cursors_directory = manifest
						.cursors_directory
						.ok_or(Error::CursorsDirectoryNotSet)?;

					theme_dir.push(&cursors_directory);

					if !theme_dir.exists() || !theme_dir.is_dir() {
						return Err(Error::CursorsDirectoryDoesntExist(cursors_directory));
					}

					let theme = HyprcursorTheme {
						name: manifest.name,
						description: manifest.description,
						version: manifest.version,
						author: manifest.author,
						cursors_directory,

						path: theme_dir,
						cache: Vec::new(),
					};
					return Ok(theme);
				}
			}
		}

		Err(Error::ThemeNotFound)
	}

	// todo maybe find_cursor
	pub fn load_cursor(&self, name: &str) -> Option<&Hyprcursor> {
		self.cache
			.iter()
			.find(|cursor| cursor.meta.overrides.iter().any(|over| over == name))
	}
}

#[derive(Debug)]
struct Image {
	data: Data,
	size: u32,
	delay: Option<u32>,
}

enum Data {
	Png(Vec<u8>),
	Svg(Tree),
}

impl Data {
	fn render(&self, size: u32) -> Vec<u8> {
		match self {
			Data::Png(data) => {
				let data = std::io::Cursor::new(data);
				let decoder = PngDecoder::new(data);
				let mut reader = decoder.read_info().unwrap();

				let mut pixels = vec![0; reader.output_buffer_size()];
				let info = reader.next_frame(&mut pixels).unwrap();

				// todo convert other color types to rgba
				assert_eq!(info.color_type, png::ColorType::Rgba);

				pixels
			}
			Data::Svg(tree) => {
				let transform = Transform::from_scale(
					size as f32 / tree.size().height(),
					size as f32 / tree.size().width(),
				);

				let mut pixmap = Pixmap::new(size, size).unwrap();
				resvg::render(tree, transform, &mut pixmap.as_mut());

				pixmap.take()
			}
		}
	}
}

impl Debug for Data {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Data::Png(_) => f.debug_struct("Png").finish_non_exhaustive(),
			Data::Svg(_) => f.debug_struct("Svg").finish_non_exhaustive(),
		}
	}
}

#[derive(Debug)]
pub struct Hyprcursor {
	meta: Meta,
	images: Vec<Image>,
}

impl Hyprcursor {
	fn new<R: Read + Seek>(
		meta: Meta,
		archive_path: &Path,
		mut archive: ZipArchive<R>,
	) -> Result<Self, Error> {
		let mut images = Vec::new();
		for size in &meta.sizes {
			let mut file = archive.by_name(&size.file).map_err(|err| Error::ZipError {
				err,
				path: archive_path.to_owned(),
			})?;

			let mut buffer = Vec::new();
			file.read_to_end(&mut buffer)?;

			let data = match size.kind {
				meta::Kind::Svg => {
					let tree = Tree::from_data(&buffer, &Options::default()).map_err(|err| {
						Error::UsvgErr {
							err,
							file: size.file.clone(),
						}
					})?;

					Data::Svg(tree)
				}
				meta::Kind::Png => {
					// todo validate png buffer
					Data::Png(buffer)
				}
			};

			let image = Image {
				data,
				size: size.size,
				delay: size.delay,
			};
			images.push(image);
		}

		debug_assert!(
			images.iter().all(|img| matches!(img.data, Data::Png(_)))
				|| images.iter().all(|img| matches!(img.data, Data::Svg(_))),
			"there should only ever be either png or svg, not both"
		);

		Ok(Hyprcursor { meta, images })
	}

	pub fn render_frames(&self, size: u32) -> Vec<Frame> {
		match self.meta.kind {
			meta::Kind::Png => {
				// todo resize pngs
				let nearest = self
					.images
					.iter()
					.min_by_key(|img| u32::abs_diff(img.size, size))
					.unwrap();
				let nearest_size = nearest.size;

				self.images
					.iter()
					.filter(|img| img.size == nearest_size)
					.map(|img| Frame::new(img, size))
					.collect()
			}
			meta::Kind::Svg => self
				.images
				.iter()
				.map(|img| Frame::new(img, size))
				.collect(),
		}
	}
}

// todo figure out what to do with
// the hotspot
#[derive(Debug)]
pub struct Frame {
	pub size: u32,

	pub delay: Option<u32>,

	pub pixels: Vec<u8>,
}

impl Frame {
	fn new(img: &Image, size: u32) -> Self {
		let pixels = img.data.render(size);

		Frame {
			size,
			delay: img.delay,
			pixels,
		}
	}
}
