use std::path::Path;

/// temporary struct to parse into
#[derive(Debug)]
pub struct Manifest {
	pub name: String,
	pub description: Option<String>,
	pub version: Option<String>,
	pub author: Option<String>,
	pub cursors_directory: Option<String>,
}

impl Manifest {
	// todo make like a proper parser for this godforsaken format
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

		let name = name.or_else(|| {
			// the hyprcursor reference implementation
			// uses .stem() so i will use it too
			dir.file_stem()
				.and_then(|stem| stem.to_str())
				.map(ToOwned::to_owned)
		})?;

		Some(Manifest {
			name,
			description,
			version,
			author,
			cursors_directory,
		})
	}
}
