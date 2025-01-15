use raylib::prelude::*;

mod board;

fn main() {
    let (mut rl, thread) = raylib::init()
        .title("board")
        .resizable()
        .vsync()
        .log_level(TraceLogLevel::LOG_ERROR)
        .build();

    let mut board = board::board::Board::create("test_store", &mut rl, &thread)
        .expect("couldnt create board");

    let mut recently_saved = 0.0;
    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::new(21, 21, 21, 255));

        board.draw(&mut d);
        board.input(&mut d);

        if d.is_key_pressed(raylib::consts::KeyboardKey::KEY_A) {
            if let Ok(s) = d.get_clipboard_text() {
                // TODO: raylib-rs issue the String can be null and i dont know how to check for that
                (!s.is_empty()).then(|| board.add_text(s));
            }
        }

        if d.is_key_pressed(raylib::consts::KeyboardKey::KEY_S) {
            board.save();
            recently_saved = 2.0;
        } else if recently_saved >= 0.0 {
            d.draw_text(
                "board was saved. see printed logs for any potential errors",
                0,
                0,
                30,
                Color::BLACK,
            );
            recently_saved -= d.get_frame_time();
        }
    }

    board.save();
    println!("auto saved the board");
}
