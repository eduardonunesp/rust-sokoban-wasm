use crate::components::*;
use crate::constants::TILE_WIDTH;
use crate::resources::*;
use ggez::graphics;
use ggez::graphics::Color;
use ggez::graphics::DrawParam;
use ggez::graphics::Font;
use ggez::graphics::Image;
use ggez::{timer, Context};
use graphics::spritebatch::SpriteBatch;
use itertools::Itertools;
use nalgebra as na;
use specs::{Join, Read, ReadStorage, System};
use std::{collections::HashMap, time::Duration};

pub struct RenderingSystem<'a> {
    pub context: &'a mut Context,
    pub font: Font,
    pub clear_color: Color,
}

impl RenderingSystem<'_> {
    pub fn draw_text(&mut self, text_string: &str, x: f32, y: f32) {
        let text = graphics::Text::new((text_string, self.font, 20.));
        let destination = mint::Point2 { x, y };
        let color = Color::new(0.0, 0.0, 0.0, 1.0);

        let x: f32 = 0.0;
        let y: f32 = 20.0;
        let dimensions = text.dimensions(self.context);

        let font_bounding_rect = graphics::Rect::new(
            destination.x,
            destination.y,
            dimensions.0 as f32,
            dimensions.1 as f32,
        );

        let font_polygon = graphics::Mesh::new_rectangle(
            self.context,
            graphics::DrawMode::fill(),
            font_bounding_rect,
            self.clear_color,
        )
        .expect("expected font polygon");

        let draw_params = DrawParam::new().dest(na::Point2::new(x, y));

        graphics::draw(self.context, &font_polygon, draw_params).expect("expected render");

        let mut draw_params = graphics::DrawParam::new();
        draw_params.dest = destination;
        draw_params.color = color;

        graphics::draw(self.context, &text, draw_params).expect("expected render");
    }

    pub fn get_image(&mut self, renderable: &Renderable, delta: Duration) -> String {
        let path_index = match renderable.kind() {
            RenderableKind::Static => {
                // We only have one image, so we just return that
                0
            }
            RenderableKind::Animated => {
                // If we have multiple, we want to select the right one based on the delta time.
                // First we get the delta in milliseconds, we % by 1000 to get the seconds only
                // and finally we divide by 250 to get a number between 0 and 4. If it's 4
                // we technically are on the next iteration of the loop (or on 0), but we will let
                // the renderable handle this logic of wrapping frames.
                ((delta.as_millis() % 1000) / 250) as usize
            }
        };

        renderable.path(path_index)
    }
}

// System implementation
impl<'a> System<'a> for RenderingSystem<'a> {
    // Data
    type SystemData = (
        Read<'a, Gameplay>,
        Read<'a, Time>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Renderable>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (gameplay, time, positions, renderables) = data;

        // Clearing the screen (this gives us the backround colour)
        graphics::clear(self.context, self.clear_color);

        // Get all the renderables with their positions.
        let rendering_data = (&positions, &renderables).join().collect::<Vec<_>>();
        let mut rendering_batches: HashMap<u8, HashMap<String, Vec<DrawParam>>> = HashMap::new();

        // Iterate each of the renderables, determine which image path should be rendered
        // at which drawparams, and then add that to the rendering_batches.
        for (position, renderable) in rendering_data.iter() {
            // Load the image
            let image_path = self.get_image(renderable, time.delta);

            let x = position.x as f32 * TILE_WIDTH;
            let y = position.y as f32 * TILE_WIDTH;
            let z = position.z;

            // Add to rendering batches
            let draw_param = DrawParam::new().dest(na::Point2::new(x, y));
            rendering_batches
                .entry(z)
                .or_default()
                .entry(image_path)
                .or_default()
                .push(draw_param);
        }

        // Iterate spritebatches ordered by z and actually render each of them
        for (_z, group) in rendering_batches
            .iter()
            .sorted_by(|a, b| Ord::cmp(&a.0, &b.0))
        {
            for (image_path, draw_params) in group {
                let image = Image::new(self.context, image_path).expect("expected image");
                let mut sprite_batch = SpriteBatch::new(image);

                for draw_param in draw_params.iter() {
                    sprite_batch.add(*draw_param);
                }

                graphics::draw(self.context, &sprite_batch, graphics::DrawParam::new())
                    .expect("expected render");
            }
        }

        // Render any text
        self.draw_text(&gameplay.state.to_string(), 525.0, 80.0);
        self.draw_text(&gameplay.moves_count.to_string(), 525.0, 100.0);
        let fps = format!("FPS: {:.0}", timer::fps(self.context));
        self.draw_text(&fps, 525.0, 120.0);

        // Finally, present the context, this will actually display everything
        // on the screen.
        graphics::present(self.context).expect("expected to present");
    }
}
