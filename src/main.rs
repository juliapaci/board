use ggez::event::{self, EventHandler};
use ggez::graphics::{self, Color, Text};
use ggez::input::keyboard::{KeyCode, KeyMods};
use ggez::{Context, ContextBuilder, GameResult};

use copypasta::{ClipboardContext, ClipboardProvider};

mod board;

pub(crate) const LIGHT: Color = Color::new(237. / 255., 230. / 255., 230. / 255., 1.0);
pub(crate) const DARK: Color = Color::new(36. / 255., 34. / 255., 34. / 255., 1.0);

fn main() {
    let (mut ctx, event_loop) = ContextBuilder::new("board", "")
        .add_resource_path(std::path::PathBuf::from("."))
        .window_mode(ggez::conf::WindowMode::default().resizable(true))
        .build()
        .expect("couldnt create ggez context");

    let app = BoardApp::new(&mut ctx).unwrap();

    event::run(ctx, event_loop, app);
}

enum Mode {
    LIGHT,
    DARK,
}

struct BoardApp {
    board: board::board::Board,
    clipboard: ClipboardContext,

    recently_saved: f32,

    mode: Mode,
}

impl BoardApp {
    fn new(ctx: &mut Context) -> GameResult<Self> {
        Ok(Self {
            board: board::board::Board::create("test_store", ctx).expect("couldnt create board"),
            clipboard: ClipboardContext::new().expect("couldnt create clipboard"),
            recently_saved: 0.0,
            mode: Mode::LIGHT,
        })
    }

    fn background_colour(&self) -> Color {
        match self.mode {
            Mode::LIGHT => LIGHT,
            Mode::DARK => DARK,
        }
    }

    fn switch_colours(&mut self) {
        match self.mode {
            Mode::LIGHT => {
                self.board.set_colours((DARK, LIGHT));
                self.mode = Mode::DARK;
            }
            Mode::DARK => {
                self.board.set_colours((LIGHT, DARK));
                self.mode = Mode::LIGHT;
            }
        };
    }
}

impl EventHandler for BoardApp {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        self.board.manage(ctx);

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, self.background_colour());

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

    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        input: ggez::input::keyboard::KeyInput,
        _repeated: bool,
    ) -> Result<(), ggez::GameError> {
        match input.keycode {
            Some(KeyCode::A) => {
                if let Ok(s) = self.clipboard.get_contents() {
                    if !s.starts_with("http") || input.mods.contains(KeyMods::SHIFT) {
                        self.board.add_text(s);
                    } else {
                        if let Err(e) = self.board.add_image(&s, ctx) {
                            println!("Error: {e}");
                        }
                    }
                }
            }

            Some(KeyCode::S) => match self.board.save() {
                Ok(_) => self.recently_saved = 2.0,
                Err(e) => println!("error while saving{e}"),
            },

            Some(KeyCode::Tab) => {
                if input.mods.is_empty() {
                    self.switch_colours()
                }
            }
            Some(KeyCode::Escape) => ctx.request_quit(),

            _ => {}
        }

        Ok(())
    }

    fn mouse_button_down_event(
        &mut self,
        ctx: &mut Context,
        button: event::MouseButton,
        x: f32,
        y: f32,
    ) -> Result<(), ggez::GameError> {
        if button == event::MouseButton::Left {
            self.board.set_selection(self.board.select((x, y), ctx))
        }

        Ok(())
    }

    fn quit_event(&mut self, _ctx: &mut Context) -> Result<bool, ggez::GameError> {
        self.board.save().expect("failed to save");
        println!("auto saved the board");

        Ok(false)
    }
}
