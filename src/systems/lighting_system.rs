use super::{LightSource, Map, Position, Viewshed};
use rltk::RGB;
use specs::prelude::*;

pub struct LightingSystem {}

impl<'a> System<'a> for LightingSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        WriteExpect<'a, Map>,
        ReadStorage<'a, Viewshed>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, LightSource>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut map, viewshed, positions, lighting) = data;

        if map.outdoors {
            return;
        }

        let black = RGB::from_f32(0.0, 0.0, 0.0);
        for l in map.light.iter_mut() {
            *l = black;
        }

        for (viewshed, pos, light) in (&viewshed, &positions, &lighting).join() {
            let light_point = rltk::Point::new(pos.x, pos.y);
            let range_f = light.range as f32;
            for tile in viewshed.visible_tiles.iter() {
                if tile.x > 0 && tile.x < map.width && tile.y > 0 && tile.y < map.height {
                    let idx = map.xy_idx(tile.x, tile.y);
                    let distance = rltk::DistanceAlg::Pythagoras.distance2d(light_point, *tile);
                    let intensity = (range_f - distance) / range_f;

                    map.light[idx] = map.light[idx] + (light.color * intensity);
                }
            }
        }
    }
}
