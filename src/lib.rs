use manifest::{Kind, Manifest, Meta};
use resvg::{
	tiny_skia::Pixmap,
	usvg::{Options, Transform, Tree},
};
use std::{
	fmt::Debug,
	fs::File,
	io::{Read, Seek},
	path::PathBuf,
};
use zip::ZipArchive;
use zune_png::PngDecoder;

mod error;
mod manifest;

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

// todo ~/.local/share/icons, ~/.icons
fn user_theme_dirs() -> Vec<PathBuf> {
	vec![]
}

#[derive(Debug)]
pub struct HyprcursorTheme {
	pub name: String,
	pub description: Option<String>,
	pub version: Option<String>,
	pub author: Option<String>,
	pub cursors_directory: String,

	path: PathBuf,
	cache: Vec<Hyprcursor>,
}

impl HyprcursorTheme {
	// todo error:
	// - does not exist
	// - cursors_directory is not set
	// - cursors_directory does not exist
	// - all the stuff with meta.hl
	pub fn load(name: &str) -> Result<HyprcursorTheme, Error> {
		let mut theme = HyprcursorTheme::read(name).ok_or(Error::ThemeNotFound)?;

		for cursor in theme.path.read_dir()?.map_while(Result::ok) {
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
					let path = cursor_path.join(if is_toml { "meta.toml" } else { "meta.hl" });
					Error::MetaError { err, path }
				})?
			};

			let cursor = Hyprcursor::new(meta, archive).ok_or(Error::Other)?;
			theme.cache.push(cursor);
		}

		Ok(theme)
	}

	fn read(name: &str) -> Option<HyprcursorTheme> {
		let data_dirs = xdg_data_dirs();
		let user_dirs = user_theme_dirs();

		for path in data_dirs.into_iter().chain(user_dirs.into_iter()) {
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
					theme_dir.push(&manifest.cursors_directory);

					let theme = HyprcursorTheme {
						name: manifest.name,
						description: manifest.description,
						version: manifest.version,
						author: manifest.author,
						cursors_directory: manifest.cursors_directory,

						path: theme_dir,
						cache: Vec::new(),
					};
					return Some(theme);
				}
			}
		}

		None
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
	fn render(&self, size: u32) -> RenderData {
		match self {
			Data::Png(data) => {
				let mut decoder = PngDecoder::new(data);
				let pixels = decoder.decode_raw().unwrap();

				RenderData::Png(pixels)
			}
			Data::Svg(tree) => {
				let transform = Transform::from_scale(
					size as f32 / tree.size().height(),
					size as f32 / tree.size().width(),
				);

				let mut pixmap = Pixmap::new(size, size).unwrap();
				resvg::render(tree, transform, &mut pixmap.as_mut());
				RenderData::Svg(pixmap)
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
	// todo error values
	fn new<R: Read + Seek>(meta: Meta, mut archive: ZipArchive<R>) -> Option<Self> {
		let mut images = Vec::new();
		for size in &meta.sizes {
			let mut file = archive.by_name(&size.file).ok()?;
			let mut buffer = Vec::new();
			file.read_to_end(&mut buffer).ok()?;

			// todo reject if both png and svg
			let data = match size.kind {
				Kind::Svg => {
					let tree = Tree::from_data(&buffer, &Options::default()).ok()?;
					Data::Svg(tree)
				}
				Kind::Png => {
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

		Some(Hyprcursor { meta, images })
	}

	pub fn render_frames(&self, size: u32) -> Vec<Frame> {
		// todo do this only for pngs
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
}

enum RenderData {
	Svg(Pixmap),
	Png(Vec<u8>),
}

impl Debug for RenderData {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			RenderData::Svg(_) => f.debug_struct("Svg").finish_non_exhaustive(),
			RenderData::Png(_) => f.debug_list().entry(&..).finish(),
		}
	}
}

// todo figure out what to do with
// the hotspot
#[derive(Debug)]
pub struct Frame {
	data: RenderData,
	pub size: u32,
	pub delay: Option<u32>,
}

impl Frame {
	fn new(img: &Image, size: u32) -> Self {
		let data = img.data.render(size);

		Frame {
			data,
			size,
			delay: img.delay,
		}
	}

	// todo i think this is rgba?
	pub fn pixels(&self) -> &[u8] {
		match &self.data {
			RenderData::Svg(pixmap) => pixmap.data(),
			RenderData::Png(pixels) => pixels,
		}
	}

	pub fn width(&self) -> u32 {
		self.size
	}

	pub fn height(&self) -> u32 {
		self.size
	}
}
