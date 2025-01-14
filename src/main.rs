use copypasta::{ClipboardContext, ClipboardProvider};
use raylib::prelude::*;

mod board;

fn main() {
    let mut ctx = ClipboardContext::new().unwrap();
    println!("{}", ctx.get_contents().unwrap_or(String::from("")));

    let (mut rl, thread) = raylib::init().title("board").resizable().vsync().build();

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::GRAY);
        d.draw_text("test", 0, 0, 20, Color::WHITE);
    }
}
