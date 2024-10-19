use hyprcursor_rs::HyprcursorTheme;
use png::Encoder as PngEncoder;
use std::fs::File;

fn main() {
	let theme = HyprcursorTheme::load("rose-pine-hyprcursor").unwrap();
	let cursor = theme.load_cursor("default").unwrap();

	let size = 24;
	let frames = cursor.render_frames(size);
	let frame = &frames[0];

	let file = File::create("image.png").unwrap();

	let mut encoder = PngEncoder::new(file, frame.size, frame.size);
	encoder.set_color(png::ColorType::Rgba);
	encoder.set_depth(png::BitDepth::Eight);

	let mut writer = encoder.write_header().unwrap();
	writer.write_image_data(&frame.pixels).unwrap();
}
