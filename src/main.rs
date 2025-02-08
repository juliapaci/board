use board::board::Selectable;
use ggez::event::{self, EventHandler};
use ggez::graphics::{self, Color};
use ggez::input::keyboard::{KeyCode, KeyMods};
use ggez::{Context, ContextBuilder, GameResult};

use copypasta::{ClipboardContext, ClipboardProvider};

mod board;
mod camera;
mod notifications;

pub(crate) const LIGHT: Color = Color::new(237. / 255., 230. / 255., 230. / 255., 1.0);
pub(crate) const DARK: Color = Color::new(36. / 255., 34. / 255., 34. / 255., 1.0);

const NOTIFICATION_TIME: f32 = std::f32::consts::FRAC_PI_2;

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
    // board stuff
    board: board::board::Board,
    clipboard: ClipboardContext,

    // other context stuff
    mode: Mode,
    draw_bounds: bool,
    notifications: notifications::Notifications<notifications::MyNotification>,
}

impl BoardApp {
    fn new(ctx: &mut Context) -> GameResult<Self> {
        Ok(Self {
            board: board::board::Board::create(
                std::env::args()
                    .collect::<Vec<String>>()
                    .get(1)
                    .unwrap_or(&"test_store".to_owned()),
                ctx,
            )
            .expect("couldnt create board"),
            clipboard: ClipboardContext::new().expect("couldnt create clipboard"),

            mode: Mode::LIGHT,
            draw_bounds: false,
            notifications: notifications::Notifications::with_colour(DARK),
        })
    }

    fn background_colour(&self) -> Color {
        match self.mode {
            Mode::LIGHT => LIGHT,
            Mode::DARK => DARK,
        }
    }

    fn opposite_colour(&self) -> Color {
        match self.mode {
            Mode::LIGHT => DARK,
            Mode::DARK => LIGHT,
        }
    }

    fn switch_colours(&mut self) {
        match self.mode {
            Mode::LIGHT => {
                self.board.set_colours((DARK, LIGHT));
                self.notifications.set_colour(LIGHT);

                self.mode = Mode::DARK;
            }
            Mode::DARK => {
                self.board.set_colours((LIGHT, DARK));
                self.notifications.set_colour(DARK);

                self.mode = Mode::LIGHT;
            }
        };
    }
}

impl EventHandler for BoardApp {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        self.board.manage(ctx);
        self.notifications
            .update_all(ctx.time.delta().as_secs_f32());

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, self.background_colour());

        self.board.draw(&mut canvas, ctx);
        if self.draw_bounds {
            self.board.draw_bounds(&mut canvas, ctx)
        }

        self.notifications.display_all(&mut canvas);

        canvas.finish(ctx)
    }

    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        input: ggez::input::keyboard::KeyInput,
        repeated: bool,
    ) -> Result<(), ggez::GameError> {
        // dont care for holding anything down
        if repeated {
            return Ok(());
        }

        match input.keycode {
            Some(KeyCode::A) => {
                if let Ok(s) = self.clipboard.get_contents() {
                    // TODO: pase a path -> load from file
                    if !s.starts_with("http") || input.mods.contains(KeyMods::SHIFT) {
                        self.board.add_text(s);
                    } else {
                        if let Err(e) = self.board.add_image(&s, ctx) {
                            println!("Error: {e}");
                        }
                    }

                    self.notifications.add(notifications::MyNotification::new(
                        format!("added {}", self.board.get(self.board.len() - 1).unwrap()),
                        NOTIFICATION_TIME,
                    ));
                }
            }

            Some(KeyCode::S) => match self.board.save() {
                Ok(_) => self.notifications.add(notifications::MyNotification::new(
                    "board was saved".to_owned(),
                    NOTIFICATION_TIME,
                )),
                Err(e) => println!("error while saving: {e}"),
            },

            Some(KeyCode::X) => {
                if let Some(Selectable::Item(i)) = self.board.selected() {
                    self.notifications.add(notifications::MyNotification::new(
                        format!("removed item {i} ({})", self.board.get(i).unwrap()),
                        NOTIFICATION_TIME,
                    ));

                    self.board.remove(i)
                }
            }

            Some(KeyCode::Tab) => {
                if input.mods.is_empty() {
                    self.switch_colours()
                }
            }

            Some(KeyCode::Space) => self.draw_bounds = !self.draw_bounds,

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
        if button == event::MouseButton::Left || button == event::MouseButton::Right {
            self.board
                .set_last_press((x, y));

            match self.board.select((x, y), ctx) {
                Some(i) => self.board.set_selection(Selectable::Item(i)),
                None => self.board.set_selection(Selectable::Board),
            }
        }

        Ok(())
    }

    fn mouse_wheel_event(
        &mut self,
        _ctx: &mut Context,
        _x: f32,
        y: f32,
    ) -> Result<(), ggez::GameError> {
        self.board.camera.mouse_wheel_event(_ctx, _x, y)
    }

    fn quit_event(&mut self, _ctx: &mut Context) -> Result<bool, ggez::GameError> {
        self.board.save().expect("failed to save");
        println!("auto saved the board");

        Ok(false)
    }
}
