use board::board::Selectable;
use ggez::event::{self, EventHandler};
use ggez::graphics::{self, Color};
use ggez::input::keyboard::{KeyCode, KeyMods};
use ggez::{Context, ContextBuilder, GameResult};

use cli_clipboard::{ClipboardContext, ClipboardProvider};
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Deserialize, Serialize, Default)]
enum Mode {
    #[default]
    LIGHT,
    DARK,
}

// TODO: probably move this to "store.rs"
#[derive(Debug, Deserialize, Serialize, Default)]
struct BoardAppState {
    mode: Mode,
    // TODO: use bitfields
    draw_bounds: bool,
    draw_selection_info: bool,
}

struct BoardApp {
    board: board::board::Board,
    store_path: String,

    clipboard: ClipboardContext,
    notifications: notifications::Notifications<notifications::MyNotification>,
    state: BoardAppState,
}

impl BoardAppState {
    const STORE_CACHE_PATH: &str = "app_state.store";

    fn new<P: AsRef<std::path::Path>>(store_path: P) -> Self {
        match Self::read_cache(store_path) {
            Ok(c) => c,
            Err(e) => {
                println!("couldnt read board app state from cache: {e}");
                Self::default()
            }
        }
    }

    fn read_cache<P: AsRef<std::path::Path>>(
        store_path: P,
    ) -> Result<BoardAppState, Box<dyn std::error::Error>> {
        Ok(serde_json::from_str::<BoardAppState>(
            &std::fs::read_to_string(store_path.as_ref().join(Self::STORE_CACHE_PATH))?,
        )?)
    }

    fn save<P: AsRef<std::path::Path>>(
        &self,
        store_path: P,
    ) -> std::io::Result<()> {
        use std::io::Write;
        let path = store_path.as_ref().join(Self::STORE_CACHE_PATH);

        if let Ok(false) = std::fs::exists(&path) {
            std::fs::File::create_new(&path)?;
        }

        let mut cache = std::fs::File::open(&path)?;
        writeln!(cache, "{}", serde_json::to_string_pretty(self)?)?;

        Ok(())
    }
}

impl BoardApp {
    fn new(ctx: &mut Context) -> GameResult<Self> {
        let default_store_path = "test_store".to_owned();
        let args = std::env::args().collect::<Vec<String>>();

        let store_path = args.get(1).unwrap_or(&default_store_path);

        Ok(Self {
            board: board::board::Board::create(store_path, ctx).expect("couldnt create board"),
            store_path: store_path.clone(),

            clipboard: ClipboardContext::new().expect("couldnt create clipboard"),
            notifications: notifications::Notifications::with_colour(DARK),

            state: BoardAppState::new(store_path),
        })
    }

    fn background_colour(&self) -> Color {
        match self.state.mode {
            Mode::LIGHT => LIGHT,
            Mode::DARK => DARK,
        }
    }

    fn switch_colours(&mut self) {
        match self.state.mode {
            Mode::LIGHT => {
                self.board.set_colours((DARK, LIGHT));
                self.notifications.set_colour(LIGHT);

                self.state.mode = Mode::DARK;
            }
            Mode::DARK => {
                self.board.set_colours((LIGHT, DARK));
                self.notifications.set_colour(DARK);

                self.state.mode = Mode::LIGHT;
            }
        };
    }

    fn save(&mut self) -> std::io::Result<()> {
        self.board.save()?;
        self.state.save(&self.store_path)?;
        Ok(())
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
        if self.state.draw_bounds {
            self.board.draw_bounds(&mut canvas, ctx)
        }
        if self.state.draw_selection_info {
            self.board.draw_selection_info(&mut canvas, ctx)
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
        if repeated || input.keycode.is_none() {
            return Ok(());
        }

        match input.keycode.unwrap() {
            KeyCode::A => {
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

            KeyCode::S => {
                match self.save() {
                    Ok(_) => self.notifications.add(notifications::MyNotification::new(
                        "board was saved".to_owned(),
                        NOTIFICATION_TIME,
                    )),
                    Err(e) => println!("error while saving: {e}"),
                }
            }

            KeyCode::X => {
                if let Some(Selectable::Item(i)) = self.board.selected() {
                    self.notifications.add(notifications::MyNotification::new(
                        format!("removed item {i} ({})", self.board.get(i).unwrap()),
                        NOTIFICATION_TIME,
                    ));

                    self.board.remove(i)
                }
            }

            KeyCode::Tab => {
                if input.mods.is_empty() {
                    self.switch_colours()
                }
            }

            KeyCode::Space => self.state.draw_bounds = !self.state.draw_bounds,
            KeyCode::D => self.state.draw_selection_info = !self.state.draw_selection_info,

            // TODO: make dynamic
            KeyCode::H => self.notifications.add(notifications::MyNotification::new(
                "
Key         Action
A           Add item from clipboard
S           Save the board
X           Delete the selected item
Tab         switch between dark and light mode
Space       Debug: show item bounds
D           Debug: show selection information
H           This help menu :)
                "
                .to_owned(),
                NOTIFICATION_TIME,
            )),

            KeyCode::Escape => ctx.request_quit(),

            _ => (),
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
        if self.board.selected().is_none()
            && (button == event::MouseButton::Left || button == event::MouseButton::Right)
        {
            self.board.set_last_press((x, y));

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

    fn resize_event(
        &mut self,
        _ctx: &mut Context,
        width: f32,
        height: f32,
    ) -> Result<(), ggez::GameError> {
        Ok(self.board.camera.resolution = (width, height))
    }

    fn quit_event(&mut self, _ctx: &mut Context) -> Result<bool, ggez::GameError> {
        self.save().expect("failed to save");
        println!("auto saved the board");

        Ok(false)
    }
}
