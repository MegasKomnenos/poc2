use crate::misc::*;

use amethyst::{
    core::{ math::{ Vector3, Vector2, Point3, }, Transform, },
    ecs::{ Entities, System, ReadStorage, WriteStorage, Read, ReadExpect, Join },
    input::{ InputHandler, StringBindings }, 
    renderer::{
        camera::{ ActiveCamera, Camera, },
    },
    window::ScreenDimensions,
    winit,
};
use amethyst_tiles::{ MapStorage, TileMap, Map, };

#[derive(Default)]
pub struct SystemColorMap;
impl<'s> System<'s> for SystemColorMap {
    type SystemData = (
        Entities<'s>,
        Read<'s, ActiveCamera>,
        Read<'s, InputHandler<StringBindings>>,
        ReadExpect<'s, ScreenDimensions>,
        ReadStorage<'s, Transform>,
        ReadStorage<'s, Camera>,
        WriteStorage<'s, TileMap<MiscTile>>,
    );

    fn run(&mut self, (entities, active_camera, input, dimensions, transforms, cameras, mut tilemaps): Self::SystemData) {
        let mut color = 100;

        if input.mouse_button_is_down(winit::MouseButton::Left) {
            color = 1;
        } else if input.mouse_button_is_down(winit::MouseButton::Right) {
            color = 0;
        }

        if color < 100 {
            if let Some(mouse_position) = input.mouse_position() {
                let mut camera_join = (&cameras, &transforms).join();
                if let Some((camera, camera_transform)) = active_camera
                    .entity
                    .and_then(|a| camera_join.get(a, &entities))
                    .or_else(|| camera_join.next())
                {
                    let coord = camera.projection()
                        .screen_to_world_point(
                            Point3::new(mouse_position.0, mouse_position.1, 0.0),
                            Vector2::new(dimensions.width(), dimensions.height()),
                            camera_transform,
                        );

                    for tilemap in (&mut tilemaps).join() {
                        if let Some(tile) = tilemap.to_tile(&Vector3::new(coord[0], coord[1], coord[2])) {
                            tilemap.get_mut(&tile).unwrap().terrain = color;
                        }
                    }
                }
            }
        }
    }
}

#[derive(Default)]
pub struct SystemCameraMovement;
impl<'s> System<'s> for SystemCameraMovement {
    type SystemData = (
        Read<'s, ActiveCamera>,
        Entities<'s>,
        ReadStorage<'s, Camera>,
        WriteStorage<'s, Transform>,
        Read<'s, InputHandler<StringBindings>>,
    );

    fn run(&mut self, (active_camera, entities, cameras, mut transforms, input): Self::SystemData) {
        let x_move = input.axis_value("camera_x").unwrap();
        let y_move = input.axis_value("camera_y").unwrap();
        let z_move = input.axis_value("camera_z").unwrap();
        let z_move_scale = input.axis_value("camera_scale").unwrap();

        if x_move != 0.0 || y_move != 0.0 || z_move != 0.0 || z_move_scale != 0.0 {
            let mut camera_join = (&cameras, &mut transforms).join();
            if let Some((_, camera_transform)) = active_camera
                .entity
                .and_then(|a| camera_join.get(a, &entities))
                .or_else(|| camera_join.next())
            {
                camera_transform.prepend_translation_x(x_move);
                camera_transform.prepend_translation_y(y_move);
                camera_transform.prepend_translation_z(z_move);

                let z_scale = 0.01 * z_move_scale;
                let scale = camera_transform.scale();
                let scale = Vector3::new(scale.x + z_scale, scale.y + z_scale, scale.z + z_scale);
                camera_transform.set_scale(scale);
            }
        }
    }
}