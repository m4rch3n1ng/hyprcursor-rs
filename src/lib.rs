use manifest::Manifest;
use std::path::PathBuf;

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

	#[expect(dead_code, reason = "todo")]
	path: PathBuf,
}

impl HyprcursorTheme {
	// todo error:
	// - does not exist
	// - cursors_directory is not set
	// - cursors_directory does not exist
	pub fn load(name: &str) -> Option<HyprcursorTheme> {
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
					};
					return Some(theme);
				}
			}
		}

		None
	}
}
