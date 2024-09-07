use manifest::{Kind, Manifest, Meta};
use resvg::usvg::{Options, Tree};
use std::{
	fmt::Debug,
	fs::File,
	io::{Read, Seek},
	path::PathBuf,
};
use zip::ZipArchive;

mod manifest;

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
	pub fn load(name: &str) -> Option<HyprcursorTheme> {
		let mut theme = HyprcursorTheme::read(name)?;

		for cursor in theme.path.read_dir().ok()?.map_while(Result::ok) {
			let cursor_path = cursor.path();
			if !cursor_path.extension().is_some_and(|ext| ext == "hlc") {
				continue;
			}

			let archive = File::open(&cursor_path).ok()?;
			let mut archive = ZipArchive::new(archive).ok()?;

			let (index, is_toml) = if let Some(index) = archive.index_for_path("meta.hl") {
				(index, false)
			} else if let Some(index) = archive.index_for_path("meta.toml") {
				(index, true)
			} else {
				todo!();
			};

			let mut file = archive.by_index(index).ok()?;
			let mut content = String::new();
			file.read_to_string(&mut content).ok()?;
			drop(file);

			let meta = if is_toml {
				todo!();
			} else {
				Meta::from_hyprlang(&cursor_path, content)?
			};

			let cursor = Hyprcursor::new(meta, archive)?;
			theme.cache.push(cursor);
		}

		Some(theme)
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
	#[expect(dead_code, reason = "todo")]
	size: u32,
	#[expect(dead_code, reason = "todo")]
	delay: Option<u32>,
}

#[expect(dead_code, reason = "todo")]
enum Data {
	Png,
	Svg(Tree),
}

impl Debug for Data {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Data::Png => f.write_str("Png"),
			Data::Svg(_) => f.debug_struct("Svg").finish_non_exhaustive(),
		}
	}
}

#[derive(Debug)]
pub struct Hyprcursor {
	meta: Meta,
	#[expect(dead_code, reason = "todo")]
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

			let data = match size.kind {
				Kind::Svg => {
					let tree = Tree::from_data(&buffer, &Options::default()).ok()?;
					Data::Svg(tree)
				}
				Kind::Png => todo!(),
			};

			let image = Image {
				data,
				size: size.size,
				delay: size.delay,
			};
			images.push(image);
		}

		debug_assert!(
			images.iter().all(|img| matches!(img.data, Data::Png))
				|| images.iter().all(|img| matches!(img.data, Data::Svg(_)))
		);

		Some(Hyprcursor { meta, images })
	}
}
