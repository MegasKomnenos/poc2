use crate::misc::*;
use crate::component::*;
use crate::ai::*;
use crate::asset::*;
use crate::NUM_ITEM;

use amethyst::{
    core::{ math::{ Vector3, Vector2, Point3, }, Transform, },
    assets::{ Loader, AssetStorage, },
    ecs::{ Entity, Entities, System, ReadStorage, WriteStorage, Read, ReadExpect, Write, Join, ParJoin, },
    input::{ InputHandler, StringBindings, VirtualKeyCode }, 
    renderer::{
        Texture,
        sprite::{ SpriteRender, SpriteSheet, },
        camera::{ ActiveCamera, Camera, },
    },
    ui::{
        UiFinder, UiText
    },
    window::ScreenDimensions,
    winit,
};
use amethyst_tiles::{ MapStorage, TileMap, Map, };
use rayon::iter::ParallelIterator;
use rand::prelude::*;
use rand::distributions::WeightedIndex;

#[derive(Default)]
pub struct SystemMapMode;
impl<'s> System<'s> for SystemMapMode {
    type SystemData = (
        Read<'s, InputHandler<StringBindings>>,
        Write<'s, MiscMapMode>,
    );

    fn run(&mut self, (input, mut mapmode): Self::SystemData) {
        if input.key_is_down(VirtualKeyCode::Key1) {
            *mapmode = MiscMapMode::Terrain;
        } else if input.key_is_down(VirtualKeyCode::Key2) {
            *mapmode = MiscMapMode::Nothing;
        } else if input.key_is_down(VirtualKeyCode::Key3) {
            *mapmode = MiscMapMode::Amethyst;
        } else if input.key_is_down(VirtualKeyCode::Key4) {
            *mapmode = MiscMapMode::Gold;
        } else if input.key_is_down(VirtualKeyCode::Key5) {
            *mapmode = MiscMapMode::Metal;
        } else if input.key_is_down(VirtualKeyCode::Key6) {
            *mapmode = MiscMapMode::Stone;
        } else if input.key_is_down(VirtualKeyCode::Key7) {
            *mapmode = MiscMapMode::Coal;
        }
    }
}

#[derive(Default)]
pub struct SystemTime;
impl<'s> System<'s> for SystemTime {
    type SystemData = (
        UiFinder<'s>,
        WriteStorage<'s, UiText>,
        Write<'s, MiscTime>
    );

    fn run(&mut self, (ui_finder, mut ui_texts, mut time): Self::SystemData) {
        time.scnd += 1;

        if time.scnd >= 60 {
            time.scnd = 0;
            time.mnt += 1;

            if time.mnt >= 60 {
                time.mnt = 0;
                time.hour += 1;

                if time.hour >= 12 {
                    time.hour = 0;
                    time.am = !time.am;
        
                    if time.am {
                        time.day += 1;

                        if time.day >= 31 {
                            time.day = 1;
                            time.month += 1;

                            if time.month >= 5 {
                                time.year += 1;
                                time.month = 1;
                            }
                        }
                    }
                }
            }

            let mut am: String;

            if time.am {
                am = "AM".to_string();
            } else {
                am = "PM".to_string();
            }

            ui_texts.get_mut(ui_finder.find("Time").unwrap()).unwrap().text = format!("{}|{}|{}", am, time.hour, time.mnt);
        }
    }
}

#[derive(Default)]
pub struct SystemPrice;
impl<'s> System<'s> for SystemPrice {
    type SystemData = (
        ReadStorage<'s, ComponentStockpile>,
        WriteStorage<'s, ComponentPrice>,
    );

    fn run(&mut self, (stockpiles, mut prices): Self::SystemData) {
        (&stockpiles, &mut prices).par_join().for_each(|(stockpile, price)| {
            for i in 1..NUM_ITEM {
                if price.update[i] {
                    price.buy[i] = get_price(true, 1, (stockpile.items[0], price.weight[0], price.decay[0]), (stockpile.items[i], price.weight[i], price.decay[i]));
                    price.sell[i] = get_price(false, 1, (stockpile.items[0], price.weight[0], price.decay[0]), (stockpile.items[i], price.weight[i], price.decay[i]));
                }
            }
        });
    }
}

#[derive(Default)]
pub struct SystemAI;
impl<'s> System<'s> for SystemAI {
    type SystemData = (
        Entities<'s>,
        Read<'s, Vec<AssetWorkplaceData>>,
        Read<'s, Vec<AssetItemData>>,
        Read<'s, Vec<AIAxis>>,
        Write<'s, Vec<Box<dyn AIAction>>>,
        WriteStorage<'s, ComponentAgent>,
        WriteStorage<'s, TileMap<MiscTile>>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, ComponentWorkplace>,
        WriteStorage<'s, ComponentStockpile>,
        WriteStorage<'s, ComponentMovement>,
        WriteStorage<'s, ComponentPrice>,
    );

    fn run(&mut self, (entities, workplace_datas, item_datas, axis_datas, mut action_datas, mut agents, mut tilemaps, mut transforms, mut workplaces, mut stockpiles, mut movements, mut prices): Self::SystemData ) {
        let mut ai_data = (&entities, workplace_datas, item_datas, axis_datas, tilemaps, transforms, workplaces, stockpiles, movements, prices);

        (&entities, &mut agents).par_join().for_each(|(entity, agent)| {
            if agent.current == 255 {
                let mut evals: Vec<(u8, Option<Entity>, f32)> = Vec::new();

                for action in agent.actions.iter() {
                    if *action == 255 {
                        continue;
                    }

                    if let Some(eval) = action_datas[*action as usize].eval(&entity, &ai_data) {
                        evals.push(eval);
                    }
                }

                let current: (u8, Option<Entity>, f32);

                if evals.len() > 1 {
                    let dist = WeightedIndex::new(evals.iter().map(|eval| eval.2.powf(5.0))).unwrap();

                    current = evals[dist.sample(&mut thread_rng())];
                } else {
                    current = evals[0];
                }

                agent.current = current.0;

                if let Some(t) = current.1 {
                    agent.target = Some(t.clone());
                } else {
                    agent.target = None;
                }

                agent.fresh = true;
            }
        });

        for (entity, agent) in (&entities, &mut agents).join() {
            if agent.fresh {
                agent.fresh = false;
                
                if !action_datas[agent.current as usize].init(&entity, &agent.target.unwrap_or(entity.clone()), &mut ai_data) {
                    agent.current = 255;
                }
            }

            if agent.current != 255 {
                if action_datas[agent.current as usize].run(&entity, &agent.target.unwrap_or(entity.clone()), &mut ai_data) {
                    agent.current = 255;
                }
            }
        }
    }
}

#[derive(Default)]
pub struct SystemSetWorkplace;
impl<'s> System<'s> for SystemSetWorkplace {
    type SystemData = (
        Entities<'s>,
        ReadExpect<'s, Loader>,
        Read<'s, AssetStorage<Texture>>,
        Read<'s, AssetStorage<SpriteSheet>>,
        Read<'s, ActiveCamera>,
        Read<'s, InputHandler<StringBindings>>,
        ReadExpect<'s, ScreenDimensions>,
        ReadStorage<'s, Camera>,
        ReadStorage<'s, TileMap<MiscTile>>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, SpriteRender>,
        WriteStorage<'s, ComponentWorkplace>,
        WriteStorage<'s, ComponentPrice>,
        WriteStorage<'s, ComponentStockpile>,
    );

    fn run(&mut self, (entities, loader, texture_storage, sprite_sheet_storage, active_camera, input, dimensions, cameras, tilemaps, mut transforms, mut sprite_renders, mut workplaces, mut prices, mut stockpiles): Self::SystemData) {
        if input.key_is_down(VirtualKeyCode::F1) || input.key_is_down(VirtualKeyCode::F2) || input.key_is_down(VirtualKeyCode::F3) || input.key_is_down(VirtualKeyCode::F4) {
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

                    let tilemap = (&tilemaps).join().next().unwrap();
                    let coord = tilemap.to_world(&tilemap.to_tile(&coord).unwrap());

                    if input.key_is_down(VirtualKeyCode::F1) {
                        entities
                            .build_entity()
                            .with(
                                SpriteRender { 
                                    sprite_sheet: load_sprite_sheet_system(
                                        &loader, &texture_storage, &sprite_sheet_storage, 
                                        "texture/tile_sprites.png", "texture/tile_sprites.ron"
                                    ), 
                                    sprite_number: 4,
                                },
                                &mut sprite_renders,
                            )
                            .with(
                                Transform::from(Vector3::new(coord[0], coord[1], 0.0)),
                                &mut transforms,
                            )
                            .with(
                                ComponentWorkplace {
                                    variant: 0,
                                },
                                &mut workplaces
                            )
                            .build();
                    } else if input.key_is_down(VirtualKeyCode::F2) {
                        entities
                            .build_entity()
                            .with(
                                SpriteRender { 
                                    sprite_sheet: load_sprite_sheet_system(
                                        &loader, &texture_storage, &sprite_sheet_storage, 
                                        "texture/tile_sprites.png", "texture/tile_sprites.ron"
                                    ), 
                                    sprite_number: 5,
                                },
                                &mut sprite_renders,
                            )
                            .with(
                                Transform::from(Vector3::new(coord[0], coord[1], 0.0)),
                                &mut transforms,
                            )
                            .with(
                                ComponentWorkplace {
                                    variant: 1,
                                },
                                &mut workplaces
                            )
                            .build();
                    } else if input.key_is_down(VirtualKeyCode::F3) {
                        entities
                            .build_entity()
                            .with(
                                SpriteRender { 
                                    sprite_sheet: load_sprite_sheet_system(
                                        &loader, &texture_storage, &sprite_sheet_storage, 
                                        "texture/tile_sprites.png", "texture/tile_sprites.ron"
                                    ), 
                                    sprite_number: 6,
                                },
                                &mut sprite_renders,
                            )
                            .with(
                                Transform::from(Vector3::new(coord[0], coord[1], 0.0)),
                                &mut transforms,
                            )
                            .with(
                                ComponentWorkplace {
                                    variant: 2,
                                },
                                &mut workplaces
                            )
                            .build();
                    } else if input.key_is_down(VirtualKeyCode::F4) {
                        entities
                            .build_entity()
                            .with(
                                SpriteRender { 
                                    sprite_sheet: load_sprite_sheet_system(
                                        &loader, &texture_storage, &sprite_sheet_storage, 
                                        "texture/tile_sprites.png", "texture/tile_sprites.ron"
                                    ), 
                                    sprite_number: 7,
                                },
                                &mut sprite_renders,
                            )
                            .with(
                                Transform::from(Vector3::new(coord[0], coord[1], 0.0)),
                                &mut transforms,
                            )
                            .with(
                                ComponentWorkplace {
                                    variant: 3,
                                },
                                &mut workplaces
                            )
                            .with(
                                ComponentPrice {
                                    update: [true; NUM_ITEM],
                                    buy: [0; NUM_ITEM],
                                    sell: [0; NUM_ITEM],
                                    weight: [0.005, 1.0, 1.0, 1.0],
                                    decay: [0.9, 0.5, 0.5, 0.5],
                                },
                                &mut prices
                            )
                            .with(
                                ComponentStockpile {
                                    items: [10000, 50, 50, 50],
                                },
                                &mut stockpiles
                            )
                            .build();
                    }
                }
            }
        }
    }
}

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
        WriteStorage<'s, ComponentAgent>,
        WriteStorage<'s, ComponentStockpile>,
        WriteStorage<'s, ComponentPrice>,
    );

    fn run(&mut self, (
        entities, loader, texture_storage, sprite_sheet_storage, active_camera, input, dimensions, cameras, 
        mut movements, mut transforms, mut sprite_renders, mut agents, mut stockpiles, mut prices): Self::SystemData
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

                    let mut actions = [255; 23];
                    let mut items = [1; NUM_ITEM];
                    let mut weight = [1.0; NUM_ITEM];
                    let mut decay = [0.5; NUM_ITEM];
                    
                    actions[0] = 0;
                    items[0] = 1000;
                    weight[0] = 0.005;
                    decay[0] = 0.9;

                    if input.key_is_down(VirtualKeyCode::Key1) {
                        actions[1] = 1;
                        actions[2] = 6;
                        actions[3] = 7;

                        items[1] = 50;
                        items[3] = 100;
                    } else if input.key_is_down(VirtualKeyCode::Key2) {
                        actions[1] = 2;
                        actions[2] = 4;
                        actions[3] = 8;

                        items[1] = 100;
                        items[2] = 50;
                    } else if input.key_is_down(VirtualKeyCode::Key3) {
                        actions[1] = 3;
                        actions[2] = 5;
                        actions[3] = 9;

                        items[2] = 100;
                        items[3] = 50;
                    } else {
                        return;
                    }

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
                                targets: Vec::new(), 
                                velocity: Vector3::new(0.0, 0.0, 0.0),
                                speed_limit: 0.1, 
                                acceleration: 0.05, 
                            },
                            &mut movements,
                        )
                        .with(
                            ComponentAgent {
                                actions: actions,
                                current: 255,
                                target: None,
                                fresh: false,
                            },
                            &mut agents,
                        )
                        .with(
                            ComponentStockpile {
                                items: items,
                            },
                            &mut stockpiles
                        )
                        .with(
                            ComponentPrice {
                                update: [true; NUM_ITEM],
                                buy: [0; NUM_ITEM],
                                sell: [0; NUM_ITEM],
                                weight: weight,
                                decay: decay,
                            },
                            &mut prices
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
        WriteStorage<'s, ComponentMovement>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, TileMap<MiscTile>>,
    );

    fn run(&mut self, (mut movements, mut transforms, mut tilemaps): Self::SystemData) {
        for (movement, mut transform) in (&mut movements, &mut transforms.restrict_mut()).join() {
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

                        if distance < 2.0 && movement.targets.len() <= 1 {
                            speed_limit *= distance / 2.0;
                        }

                        if speed > speed_limit {
                            movement.velocity *= speed_limit / speed;
                        }

                        *transform.translation_mut() += movement.velocity;

                        if (movement.targets.len() > 1 && distance < 0.2)
                        || (movement.targets.len() <= 1 && distance < 0.002) {
                            movement.targets.pop();
                        }

                        if movement.targets.len() == 0 {
                            movement.velocity[0] = 0.0;
                            movement.velocity[1] = 0.0;
                            movement.velocity[2] = 0.0;
                        }
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