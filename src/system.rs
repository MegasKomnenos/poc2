use crate::misc::*;
use crate::component::*;

use amethyst::{
    core::{ math::{ Vector3, Vector2, Point3, }, Transform, },
    assets::{ Loader, AssetStorage, },
    ecs::{ Entities, System, ReadStorage, WriteStorage, Read, ReadExpect, Join, ParJoin, },
    input::{ InputHandler, StringBindings, VirtualKeyCode }, 
    renderer::{
        Texture,
        sprite::{ SpriteRender, SpriteSheet, },
        camera::{ ActiveCamera, Camera, },
    },
    window::ScreenDimensions,
    winit,
};
use amethyst_tiles::{ MapStorage, TileMap, Map, Region, };
use rayon::iter::ParallelIterator;

#[derive(Default)]
pub struct SystemSpawnChar;
impl<'s> System<'s> for SystemSpawnChar {
    type SystemData = (
        Entities<'s>,
        ReadExpect<'s, Loader>,
        Read<'s, AssetStorage<Texture>>,
        Read<'s, AssetStorage<SpriteSheet>>,
        Read<'s, ActiveCamera>,
        Read<'s, InputHandler<StringBindings>>,
        ReadExpect<'s, ScreenDimensions>,
        ReadStorage<'s, Camera>,
        WriteStorage<'s, ComponentMovement>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, SpriteRender>,
    );

    fn run(&mut self, (
        entities, loader, texture_storage, sprite_sheet_storage, active_camera, input, dimensions, cameras, 
        mut movements, mut transforms, mut sprite_renders): Self::SystemData
    ) {
        if input.key_is_down(VirtualKeyCode::G) {
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

                    entities
                        .build_entity()
                        .with(
                            SpriteRender { 
                                sprite_sheet: load_sprite_sheet_system(
                                    &loader, &texture_storage, &sprite_sheet_storage, 
                                    "texture/tile_sprites.png", "texture/tile_sprites.ron"
                                ), 
                                sprite_number: 3,
                            },
                            &mut sprite_renders,
                        )
                        .with(
                            Transform::from(Vector3::new(coord[0], coord[1], 0.0)),
                            &mut transforms,
                        )
                        .with(
                            ComponentMovement { 
                                targets: vec![Point3::new(10, 10, 0), Point3::new(10, 90, 0), Point3::new(90, 90, 0)], 
                                velocity: Vector3::new(0.0, 0.0, 0.0),
                                speed_limit: 0.5, 
                                acceleration: 0.05, 
                            },
                            &mut movements,
                        )
                        .build();
                }
            }
        }
    }
}

#[derive(Default)]
pub struct SystemMovement;
impl<'s> System<'s> for SystemMovement {
    type SystemData = (
        Entities<'s>,
        WriteStorage<'s, ComponentMovement>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, TileMap<MiscTile>>,
    );

    fn run(&mut self, (entities, mut movements, mut transforms, mut tilemaps): Self::SystemData) {
        (&mut tilemaps).par_join().for_each(|tilemap| {
            let region = Region::new(Point3::new(0, 0, 0), Point3::new(tilemap.dimensions()[0] - 1, tilemap.dimensions()[1] - 1, tilemap.dimensions()[2] - 1));

            region.iter().for_each(|coord| {
                tilemap.get_mut(&coord).unwrap().chars.clear();
            });
        });

        for (entity, movement, mut transform) in (&entities, &mut movements, &mut transforms.restrict_mut()).join() {
            if movement.targets.len() > 0 {
                let transform = transform.get_mut_unchecked();

                for tilemap in (&mut tilemaps).join() {
                    if let Some(_) = tilemap.to_tile(transform.translation()) {
                        let mut velocity = tilemap.to_world(movement.targets.last().unwrap()) - transform.translation();
                        let distance = (velocity[0].powf(2.0) + velocity[1].powf(2.0) + velocity[2].powf(2.0)).sqrt();

                        if distance > movement.acceleration {
                            velocity *= movement.acceleration / distance;
                        }

                        movement.velocity += velocity;
                        let speed = (movement.velocity[0].powf(2.0) + movement.velocity[1].powf(2.0) + movement.velocity[2].powf(2.0)).sqrt();
                        let mut speed_limit = movement.speed_limit;

                        if distance < 10.0 {
                            if movement.targets.len() > 0 {
                                if distance < speed / 2.0 {
                                    movement.targets.pop();
                                }
                            } else {
                                speed_limit *= distance / 10.0;
                            }
                        }

                        if speed > speed_limit {
                            movement.velocity *= speed_limit / speed;
                        }

                        *transform.translation_mut() += movement.velocity;

                        if distance == 0.0 {
                            movement.targets.pop();
                        }

                        tilemap.get_mut(&tilemap.to_tile(transform.translation()).unwrap()).unwrap().chars.push(entity.clone());
                    }
                }
            }
        }
    }
}

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
                    let coord = Vector3::new(coord[0], coord[1], coord[2]);

                    for tilemap in (&mut tilemaps).join() {
                        if let Some(tile) = tilemap.to_tile(&coord) {
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