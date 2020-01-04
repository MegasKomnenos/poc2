use crate::misc::*;
use crate::component::*;
use crate::ai::*;
use crate::asset::*;
use crate::ui::*;
use crate::NUM_ITEM;

use amethyst::{
    core::{ 
        math::{ 
            Vector3, Vector2, Point3, Point2, 
        }, 
        shrev:: {
            EventChannel, ReaderId,
        },
        Transform, ParentHierarchy, HiddenPropagate, Parent,
    },
    derive::SystemDesc,
    ecs::{ Entity, Entities, System, SystemData, ReadStorage, WriteStorage, Read, ReadExpect, Write, Join, ParJoin, },
    input::{ InputHandler, StringBindings, VirtualKeyCode, InputEvent }, 
    renderer::{
        camera::{ ActiveCamera, Camera, },
    },
    ui::{
        UiFinder, UiText, UiTransform, UiImage,
    },
    window::ScreenDimensions,
    winit,
    tiles::{
        TileMap, Map,
    },
};
use rayon::iter::ParallelIterator;
use rand::prelude::*;
use rand::distributions::WeightedIndex;
use std::collections::HashMap;

#[derive(Debug, SystemDesc)]
#[system_desc(name(SystemCustomUiDesc))]
pub struct SystemCustomUi {
    #[system_desc(event_channel_reader)]
    event_reader: ReaderId<CustomUiAction>,
}
impl SystemCustomUi {
    pub fn new(event_reader: ReaderId<CustomUiAction>) -> Self {
        SystemCustomUi { event_reader }
    }
}
impl<'s> System<'s> for SystemCustomUi {
    type SystemData = (
        Entities<'s>,
        Read<'s, EventChannel<CustomUiAction>>,
        ReadExpect<'s, ParentHierarchy>,
        ReadStorage<'s, ComponentInventory>,
        WriteStorage<'s, Parent>,
        WriteStorage<'s, HiddenPropagate>,
        WriteStorage<'s, UiTransform>,
        WriteStorage<'s, UiImage>,
        WriteStorage<'s, UiText>,
        WriteStorage<'s, ComponentItem>,
    );

    fn run(&mut self, (entities, events, hierarchy, inventories, mut parents, mut hiddens, mut ui_transforms, mut ui_images, mut ui_texts, mut items): Self::SystemData) {
        'outer: for event in events.read(&mut self.event_reader) {
            match event.event_type {
                CustomUiActionType::KillSelf => {
                    hiddens.insert(event.target, HiddenPropagate::default()).expect("Failed to kill widget");
                }
                CustomUiActionType::KillParent => {
                    if let Some(parent) = hierarchy.parent(event.target) {
                        hiddens.insert(parent, HiddenPropagate::default()).expect("Failed to kill parent widget");
                    }
                }
                CustomUiActionType::DragStartedItem => {
                    let mut t = ui_transforms.get(event.target).unwrap().clone();
                    t.opaque = false;

                    let entity = entities
                        .build_entity()
                        .with(parents.get(event.target).unwrap().clone(), &mut parents)
                        .with(t, &mut ui_transforms)
                        .with(ui_images.get(event.target).unwrap().clone(), &mut ui_images)
                        .with(items.get(event.target).unwrap().clone(), &mut items)
                        .build();
                    
                    items.get_mut(event.target).unwrap().dummy = Some(entity);
                    ui_transforms.get_mut(event.target).unwrap().local_z += 1.;
                }
                CustomUiActionType::DroppedItem => {
                    if let Some(other) = event.other {
                        if inventories.get(other).is_some() {
                            let t0 = ui_transforms.get(event.target).unwrap();
                            let t1 = ui_transforms.get(other).unwrap();

                            let mut t2 = t0.clone();

                            t2.local_x = t0.pixel_x() - t1.pixel_x() + t1.width / 2.;
                            t2.local_y = t0.pixel_y() - t1.pixel_y() - t1.height / 2.;

                            if t2.local_x >= 0.0 && t2.local_x <= t2.width / 2. {
                                t2.local_x = t2.width / 2.
                            } else {
                                t2.local_x = ((t2.local_x - t2.width / 2.) / 25.).floor() * 25. + t2.width / 2.;
                            }
                            if t2.local_y <= 0.0 && -t2.local_y <= t2.height / 2. {
                                t2.local_y = -t2.height / 2.
                            } else {
                                t2.local_y = ((t2.local_y + t2.height / 2.) / 25.).floor() * 25. - t2.height / 2.;
                            }

                            if t2.local_x > 0. && t2.local_x < t1.width
                            && t2.local_y < 0. && -t2.local_y < t1.height {
                                let dummy = items.get(event.target).unwrap().dummy.unwrap();
                                
                                if hierarchy
                                    .children(other)
                                    .iter()
                                    .filter(|c| **c != event.target && **c != dummy)
                                    .find(|c| {
                                        let t3 = ui_transforms.get(**c).unwrap();

                                        t2.local_x + t2.width / 2. > t3.local_x - t3.width / 2.
                                        && t2.local_x - t2.width / 2. < t3.local_x + t3.width / 2.
                                        && t2.local_y - t2.height / 2. < t3.local_y + t3.height / 2.
                                        && t2.local_y + t2.height / 2. > t3.local_y - t3.height / 2.
                                    })
                                    .is_none() {
                                    let t = ui_transforms.get_mut(event.target).unwrap();
                                    let p = parents.get_mut(event.target).unwrap();

                                    t.local_x = t2.local_x;
                                    t.local_y = t2.local_y;
                                    t.local_z -= 1.;

                                    p.entity = other;

                                    entities.delete(dummy).expect("Failed to kill dummy item");

                                    continue 'outer;
                                }
                            }
                        }
                    }

                    let dummy = items.get(event.target).unwrap().dummy.unwrap();
                    let dummy_transform = ui_transforms.get(dummy).unwrap();

                    let x = dummy_transform.local_x;
                    let y = dummy_transform.local_y;
                    let z = dummy_transform.local_z;

                    let ui_transform = ui_transforms.get_mut(event.target).unwrap();

                    ui_transform.local_x = x;
                    ui_transform.local_y = y;
                    ui_transform.local_z = z;

                    entities.delete(dummy).expect("Failed to kill dummy item");
                }
                CustomUiActionType::SortInventory => {
                    let inventory = parents.get(event.target).unwrap().entity;

                    let mut item_storage = Vec::new();

                    for child in hierarchy.children(inventory) {
                        if items.get(*child).is_some() {
                            item_storage.push(*child);
                        }
                    }

                    item_storage.sort_unstable_by(|a, b| {
                        let a = ui_transforms.get(*a).unwrap();
                        let b = ui_transforms.get(*b).unwrap();

                        (b.width * b.height).partial_cmp(&(a.width * a.height)).unwrap()
                    });

                    let inventory_transform = ui_transforms.get(inventory).unwrap();
                    let width = (inventory_transform.width / 25.) as usize;
                    let height = (inventory_transform.height / 25.) as usize;

                    let mut grid = vec![true; width * height];
                    let mut places = HashMap::new();
                    
                    for item in item_storage.iter() {
                        let rect = get_cover(&grid, width, height);

                        let ui_transform = ui_transforms.get(*item).unwrap();
                        let width_item = (ui_transform.width / 25.) as usize;
                        let height_item = (ui_transform.height / 25.) as usize;

                        if let Some(place) = rect
                            .iter()
                            .filter(|(_, size)| size[0] >= width_item && size[1] >= height_item)
                            .min_by(|(_, s0), (_, s1)| (s0[0] * s0[1]).cmp(&(s1[0] * s1[1]))) {
                            for y in 0..height_item {
                                for x in 0..width_item {
                                    grid[(place.0[1] + y) * width + place.0[0] + x] = false;
                                }
                            }
                            
                            places.insert(*item, *place);
                        } else {
                            continue 'outer;
                        }
                    }

                    for (entity, place) in places.iter() {
                        let ui_transform = ui_transforms.get_mut(*entity).unwrap();

                        ui_transform.local_x = place.0[0] as f32 * 25. + ui_transform.width / 2.;
                        ui_transform.local_y = place.0[1] as f32 * -25. - ui_transform.height / 2.;
                    }
                }
                CustomUiActionType::ShowItemInfo => {
                    let container = (&*entities, &ui_transforms)
                        .join()
                        .find(|(_, transform)| transform.id == "item_info_container")
                        .map(|(entity, _)| entity)
                        .unwrap();
                        
                    let item = items.get(event.target).unwrap();
                    
                    parents.insert(container, Parent { entity: event.target }).expect("Failed to insert new parent to item info container");
                    hiddens.remove(container);
                    ui_texts.get_mut(hierarchy.children(container)[0]).unwrap().text = format!("{}", item.name);
                }
                CustomUiActionType::KillItemInfo => {
                    let container = (&*entities, &ui_transforms)
                        .join()
                        .find(|(_, transform)| transform.id == "item_info_container")
                        .map(|(entity, _)| entity)
                        .unwrap();

                    hiddens.insert(container, HiddenPropagate::default()).expect("failed to insert new hidden status to item info container");
                }
            }
        }
    }
}

#[derive(SystemDesc)]
#[system_desc(name(SystemMovementPlayerDesc))]
pub struct SystemMovementPlayer {
    #[system_desc(event_channel_reader)]
    event_reader: ReaderId<InputEvent<StringBindings>>,
}
impl SystemMovementPlayer {
    pub fn new(event_reader: ReaderId<InputEvent<StringBindings>>) -> Self {
        SystemMovementPlayer { event_reader }
    }
}
impl<'s> System<'s> for SystemMovementPlayer {
    type SystemData = (
        Entities<'s>,
        Read<'s, EventChannel<InputEvent<StringBindings>>>,
        Read<'s, InputHandler<StringBindings>>,
        Read<'s, ActiveCamera>,
        ReadExpect<'s, ScreenDimensions>,
        ReadStorage<'s, Transform>,
        ReadStorage<'s, Camera>,
        ReadStorage<'s, TileMap<MiscTile>>,
        ReadStorage<'s, ComponentPlayerControlled>,
        WriteStorage<'s, ComponentMovement>,
    );

    fn run(&mut self, (entities, events, input, active_camera, dimensions, transforms, cameras, tilemaps, player, mut movements): Self::SystemData) {
        for event in events.read(&mut self.event_reader) {
            if let InputEvent::MouseButtonPressed(button) = *event {
                match button {
                    winit::MouseButton::Right => {
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
                                
                                let tilemap = (&tilemaps).join().next().unwrap();
                                let coord = Vector3::new(coord[0], coord[1], coord[2]);

                                if let Ok(goal) = tilemap.to_tile(&coord, None) {
                                    let (_, transform, mut movement) = (&player, &transforms, &mut movements).join().next().unwrap();
                                
                                    let start = tilemap.to_tile(transform.translation(), None).unwrap();

                                    movement.targets = get_targets(&start, &goal, tilemap);
                                }
                            }
                        }
                    },

                    _ => (),
                }
            }
        }
    }
}

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

            let am: String;

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
                    if let Ok(_) = tilemap.to_tile(transform.translation(), None) {
                        let mut velocity = tilemap.to_world(movement.targets.last().unwrap(), None) - transform.translation();
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