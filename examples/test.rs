use hyprcursor_rs::HyprcursorTheme;

fn main() {
	let theme = HyprcursorTheme::load("rose-pine-hyprcursor").unwrap();
	let cursor = theme.load_cursor("default");
	dbg!(cursor);
}
