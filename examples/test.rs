use hyprcursor_rs::HyprcursorTheme;
use std::{
	fs::File,
	io::{BufWriter, Write},
};
use zune_png::{
	zune_core::{bit_depth::BitDepth, colorspace::ColorSpace, options::EncoderOptions},
	PngEncoder,
};

fn main() {
	let theme = HyprcursorTheme::load("Nordzy-cursors-white").unwrap();
	let cursor = theme.load_cursor("default").unwrap();

	let size = 24;
	let frames = cursor.render_frames(size);
	let frame = &frames[0];

	let size = size as usize;
	let options = EncoderOptions::new(size, size, ColorSpace::RGBA, BitDepth::Eight);
	let mut encoder = PngEncoder::new(frame.pixels(), options);

	let file = File::create("image.png").unwrap();
	let mut file = BufWriter::new(file);

	let pxiels = encoder.encode();
	file.write_all(&pxiels).unwrap();
}
