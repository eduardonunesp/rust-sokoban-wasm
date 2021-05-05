extern crate good_web_game as ggez;

use cgmath::num_traits::abs;
use ggez::audio as ggzaudio;
use ggez::event::{KeyCode, KeyMods, MouseButton};
use ggez::graphics::{Color, Font};
use ggez::{event, timer, Context, GameResult};
use nalgebra as na;
use specs::{RunNow, World, WorldExt};

mod audio;
mod components;
mod constants;
mod entities;
mod events;
mod map;
mod resources;
mod systems;

use crate::audio::*;
use crate::components::*;
use crate::map::*;
use crate::resources::*;
use crate::systems::*;

const MINIMUN_SWIPE_DRAG: f32 = 100.;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum TouchPhase {
    None,
    Started,
    Moved,
    Ended,
    Cancelled,
}

struct Game {
    world: World,
    audio_store: AudioStore,
    font: Font,
    clear_color: Color,
    touch_start: na::Vector2<f32>,
    touch_end: na::Vector2<f32>,
}

impl Game {
    fn new(ctx: &mut Context) -> GameResult<Game> {
        let mut world = World::new();

        register_components(&mut world);
        register_resources(&mut world);
        initialize_level(&mut world);

        let font = Font::new(ctx, "/fonts/LiberationMono-Regular.ttf").expect("get font");
        let clear_color = Color::new(0.95, 0.95, 0.95, 1.0);

        // Create the game state
        let game = Game {
            audio_store: AudioStore::new(ctx),
            world,
            font,
            clear_color,
            touch_start: na::Vector2::new(0., 0.),
            touch_end: na::Vector2::new(0., 0.),
        };

        Ok(game)
    }
}

impl event::EventHandler for Game {
    fn update(&mut self, context: &mut Context) -> GameResult {
        // Run input system
        {
            let mut is = InputSystem {};
            is.run_now(&self.world);
        }

        // Run gameplay state system
        {
            let mut gss = GameplayStateSystem {};
            gss.run_now(&self.world);
        }

        // Get and update time resource
        {
            let mut time = self.world.write_resource::<Time>();
            time.delta += timer::delta(context);
        }

        // Run event system
        {
            let mut es = EventSystem {
                audio_store: &mut self.audio_store,
            };
            es.run_now(&self.world);
        }

        Ok(())
    }

    fn draw(&mut self, context: &mut Context) -> GameResult {
        // Render game entities
        {
            let mut rs = RenderingSystem {
                context,
                font: self.font,
                clear_color: self.clear_color,
            };
            rs.run_now(&self.world);
        }

        Ok(())
    }

    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        _button: MouseButton,
        x: f32,
        y: f32,
    ) {
        self.touch_start.x = x;
        self.touch_start.y = y;
    }

    fn mouse_button_up_event(
        &mut self,
        context: &mut Context,
        button: MouseButton,
        x: f32,
        y: f32,
    ) {
        if button == MouseButton::Left {
            ggzaudio::maybe_create_soundmixer(context);
            self.touch_end.x = x;
            self.touch_end.y = y;

            let swipe = self.touch_end - self.touch_start;
            let mut input_queue = self.world.write_resource::<InputQueue>();

            if abs(swipe.x) > MINIMUN_SWIPE_DRAG {
                if swipe.x > 0. {
                    input_queue.keys_pressed.push(KeyCode::Right);
                } else {
                    input_queue.keys_pressed.push(KeyCode::Left);
                }
            }

            if abs(swipe.y) > MINIMUN_SWIPE_DRAG {
                if swipe.y > 0. {
                    input_queue.keys_pressed.push(KeyCode::Down);
                } else {
                    input_queue.keys_pressed.push(KeyCode::Up);
                }
            }
        }
    }

    fn key_down_event(
        &mut self,
        context: &mut Context,
        keycode: KeyCode,
        _keymod: KeyMods,
        _repeat: bool,
    ) {
        ggzaudio::maybe_create_soundmixer(context);
        println!("Key pressed: {:?}", keycode);

        let mut input_queue = self.world.write_resource::<InputQueue>();
        input_queue.keys_pressed.push(keycode);

        #[cfg(not(target_arch = "wasm32"))]
        if keycode == KeyCode::Escape {
            event::quit(context)
        }
    }
}

// Initialize the level
pub fn initialize_level(world: &mut World) {
    const MAP: &str = "
    N N W W W W W W
    W W W . . . . W
    W . . . BB . . W
    W . . RB . . . W
    W . P . . . . W
    W . . . . RS . W
    W . . BS . . . W
    W . . . . . . W
    W W W W W W W W
    ";

    load_map(world, MAP.to_string());
}

pub fn main() -> GameResult {
    ggez::start(
        ggez::conf::Conf {
            cache: ggez::conf::Cache::Tar(include_bytes!("../resources/resources.tar").to_vec()),
            loading: ggez::conf::Loading::Embedded,
            window_title: "Rust Sokoban!".to_string(),
            window_width: 800,
            window_height: 600,
            ..Default::default()
        },
        |ctx| Box::new(Game::new(ctx).unwrap()),
    )
}
