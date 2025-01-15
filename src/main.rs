use copypasta::{ClipboardContext, ClipboardProvider};
use raylib::prelude::*;

mod board;

fn main() {
    let mut cb = ClipboardContext::new().unwrap();

    let (mut rl, thread) = raylib::init()
        .title("board")
        .resizable()
        .vsync()
        .log_level(TraceLogLevel::LOG_ERROR)
        .build();

    let mut board = board::board::Board::create("test_store".into(), &mut rl, &thread).unwrap();

    let mut recently_saved = 0.0;
    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::GRAY);

        board.draw(&mut d);

        if d.is_key_pressed(key_from_i32('A' as _).unwrap()) {
            match cb.get_contents() {
                Ok(x) => board.add_text(x),
                Err(_) => todo!(),
            }
        }

        if d.is_key_pressed(key_from_i32('S' as _).unwrap()) {
            board.save();
            recently_saved = 2.0;
        }

        if recently_saved >= 0.0 {
            d.draw_text("board was saved. see printed logs for any potential errors", 0, 0, 30, Color::WHITE);
            recently_saved -= d.get_frame_time();
        }
    }

    board.save();
    println!("auto saved the board");
}
