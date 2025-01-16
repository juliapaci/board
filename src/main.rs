use ggez::event::{self, EventHandler};
use ggez::graphics::{self, Color, Text};
use ggez::input::keyboard::KeyCode;
use ggez::{Context, ContextBuilder, GameResult};

use copypasta::{ClipboardContext, ClipboardProvider};

mod board;

fn main() {
    let (mut ctx, event_loop) = ContextBuilder::new("board", "")
        .add_resource_path(std::path::PathBuf::from("."))
        .window_mode(ggez::conf::WindowMode::default().resizable(true))
        .build()
        .expect("couldnt create ggez context");

    let app = BoardApp::new(&mut ctx).unwrap();

    event::run(ctx, event_loop, app);
}

struct BoardApp {
    board: board::board::Board,
    clipboard: ClipboardContext,

    recently_saved: f32,
}

impl BoardApp {
    pub fn new(ctx: &mut Context) -> GameResult<Self> {
        Ok(Self {
            board: board::board::Board::create("test_store", ctx).expect("couldnt create board"),
            clipboard: ClipboardContext::new().expect("couldnt create clipboard"),
            recently_saved: 0.0,
        })
    }
}

impl EventHandler for BoardApp {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        // if is_key_pressed(KeyCode::Escape) {
        //     break;
        // }

        self.board.input(ctx);

        if ctx.keyboard.is_key_pressed(KeyCode::A) {
            if let Ok(s) = self.clipboard.get_contents() {
                if s.starts_with("http") {
                    self.board.add_image(&s, ctx);
                } else {
                    self.board.add_text(s);
                }
            }
        }

        if ctx.keyboard.is_key_pressed(KeyCode::S) {
            match self.board.save() {
                Ok(_) => self.recently_saved = 2.0,
                Err(e) => println!("error while saving{e}"),
            }
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::from_rgb(21, 21, 21));

        self.board.draw(&mut canvas, ctx);

        if self.recently_saved >= 0.0 {
            let mut colour = Color::WHITE;
            colour.a = self.recently_saved;
            canvas.draw(
                Text::new("board was saved").set_scale(30.0),
                graphics::DrawParam::new()
                    .color(colour)
                    .dest([0.0, 0.0])
                    .rotation(self.recently_saved),
            );
            self.recently_saved -= ctx.time.delta().as_secs_f32();
        }

        canvas.finish(ctx)
    }

    fn quit_event(&mut self, _ctx: &mut Context) -> Result<bool, ggez::GameError> {
        self.board.save().expect("failed to save");
        println!("auto saved the board");

        Ok(false)
    }
}
