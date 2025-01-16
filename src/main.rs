use macroquad::prelude::*;
use miniquad::window::{clipboard_get, quit};

mod board;

fn conf() -> Conf {
    Conf {
        window_title: "board".to_owned(),
        window_resizable: true,
        ..Default::default()
    }
}

#[macroquad::main("conf")]
async fn main() {
    prevent_quit();

    let mut board = board::board::Board::create("test_store")
        .await
        .expect("couldnt create board");

    let mut recently_saved = 0.0;
    while !is_quit_requested() {
        if is_key_pressed(KeyCode::Escape) {
            break;
        }

        // clear_background(Color::from_rgba(21, 21, 21, 255));
        // draw_text(&format!("{}", get_fps()), 0., 0., 20., WHITE);
        // draw_text("test", 0., 0., 1., WHITE);
        // println!("{}", get_fps());

        board.draw();
        board.input();

        if is_key_pressed(KeyCode::A) {
            if let Some(s) = clipboard_get() {
                if s.starts_with("http") {
                    board.add_image(&s).await;
                } else {
                    board.add_text(s);
                }
            }
        }

        if is_key_pressed(KeyCode::S) {
            match board.save() {
                Ok(_) => recently_saved = 2.0,
                Err(e) => println!("error while saving{e}"),
            }
        } else if recently_saved >= 0.0 {
            let mut colour = WHITE;
            colour.a = recently_saved;
            draw_text("board was saved", 0.0, 0.0, 30.0, colour);
            recently_saved -= get_frame_time();
        }

        next_frame().await;
    }

    // board.save().expect("failed to save");
    println!("auto saved the board");
}
