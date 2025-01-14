use copypasta::{ClipboardContext, ClipboardProvider};
use raylib::prelude::*;

mod board;

fn main() {
    let mut ctx = ClipboardContext::new().unwrap();
    println!("{}", ctx.get_contents().unwrap_or(String::from("")));


    let (mut rl, thread) = raylib::init()
        .title("board")
        .resizable()
        .vsync()
        .log_level(TraceLogLevel::LOG_ERROR)
        .build();
    let board = board::board::Board::create("test_store".into(), &mut rl, &thread).unwrap();

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);

        board.draw(&mut d);
        d.clear_background(Color::GRAY);
        d.draw_text("test", 0, 0, 20, Color::WHITE);
    }
}
