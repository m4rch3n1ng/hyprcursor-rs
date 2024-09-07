use hyprcursor_rs::HyprcursorTheme;
use std::{fs::File, io::BufWriter, path::Path};

fn main() {
	let theme = HyprcursorTheme::load("rose-pine-hyprcursor").unwrap();
	let cursor = theme.load_cursor("default").unwrap();

	let size = 24;
	let frames = cursor.render_frames(size);
	let frame = &frames[0];

	let path = Path::new(r"image.png");
	let file = File::create(path).unwrap();
	let w = BufWriter::new(file);

	let mut encoder = png::Encoder::new(w, size, size);
	encoder.set_color(png::ColorType::Rgba);

	let mut writer = encoder.write_header().unwrap();
	writer.write_image_data(frame.pixels()).unwrap();
}
