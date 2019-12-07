use crate::misc::*;

use amethyst::{
    prelude::*,
    core::{ math::Vector3, Transform },
    input::{ is_close_requested, is_key_down, },
    renderer::camera::Camera,
    window::ScreenDimensions,
    winit,
};
use amethyst_tiles::{ TileMap, MortonEncoder2D, };

#[derive(Default)]
pub struct PocLoad;

impl SimpleState for PocLoad {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let map_sprite_sheet_handle =
            load_sprite_sheet(data.world, "texture/tile_sprites.png", "texture/tile_sprites.ron");

        let mut map = TileMap::<MiscTile, MortonEncoder2D>::new(
            Vector3::new(100, 100, 1),
            Vector3::new(1, 1, 1),
            Some(map_sprite_sheet_handle),
        );

        data.world
            .create_entity()
            .with(map)
            .with(Transform::default())
            .build();

        let (width, height) = {
            let dim = data.world.read_resource::<ScreenDimensions>();
            (dim.width(), dim.height())
        };

        data.world
            .create_entity()
            .with(Transform::from(Vector3::new(0.0, 0.0, 0.1)))
            .with(Camera::standard_2d(width, height))
            .build();
    }

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        Trans::None
    }

    fn handle_event(
        &mut self,
        data: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        let StateData { .. } = data;
        if let StateEvent::Window(event) = &event {
            if is_close_requested(&event) || is_key_down(&event, winit::VirtualKeyCode::Escape) {
                Trans::Quit
            } else {
                Trans::None
            }
        } else {
            Trans::None
        }
    }
}