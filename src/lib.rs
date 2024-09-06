use std::path::PathBuf;

use manifest::Manifest;

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
	pub path: PathBuf,

	pub manifest: Manifest,
}

impl HyprcursorTheme {
	pub fn load(name: String) -> Option<HyprcursorTheme> {
		let data_dirs = xdg_data_dirs();
		let user_dirs = user_theme_dirs();

		for path in data_dirs.into_iter().chain(user_dirs.into_iter()) {
			if !path.is_dir() {
				continue;
			};

			for theme in path.read_dir().unwrap().map_while(Result::ok) {
				let theme = theme.path();

				let manifest = if let Ok(file) = std::fs::read_to_string(theme.join("manifest.hl"))
				{
					Manifest::from_hyprlang(&theme, file)
				} else if let Ok(_toml) = std::fs::read_to_string(theme.join("manifest.toml")) {
					// i don't actually fully know how the toml spec is supposed
					// to look like
					todo!("toml support");
				} else {
					continue;
				};
				let Some(manifest) = manifest else { continue };

				if name == manifest.name {
					let theme = HyprcursorTheme {
						name,
						path: theme,
						manifest,
					};
					return Some(theme);
				}
			}
		}

		None
	}
}
