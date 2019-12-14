use crate::misc::*;
use crate::asset::*;
use crate::ai::*;

use amethyst::{
    prelude::*,
    core::{ math::Vector3, Transform },
    input::{ is_close_requested, is_key_down, },
    renderer::camera::Camera,
    window::ScreenDimensions,
    winit,
    utils::application_root_dir,
};
use amethyst_tiles::{ TileMap, MortonEncoder2D, };

use ron::de::from_str;
use std::fs::read_to_string;
use std::collections::HashMap;

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
        
        let path = application_root_dir().unwrap().join("asset");
        
        let mut workplaces = Vec::new();
        let mut items = Vec::new();

        workplaces.push(from_str::<AssetWorkplaceData>(&read_to_string(path.join("def").join("workplace").join("Mine.ron")).unwrap()).unwrap());
        workplaces.push(from_str::<AssetWorkplaceData>(&read_to_string(path.join("def").join("workplace").join("Furnace.ron")).unwrap()).unwrap());
        workplaces.push(from_str::<AssetWorkplaceData>(&read_to_string(path.join("def").join("workplace").join("Smithy.ron")).unwrap()).unwrap());

        items.push(from_str::<AssetItemData>(&read_to_string(path.join("def").join("item").join("Ore.ron")).unwrap()).unwrap());
        items.push(from_str::<AssetItemData>(&read_to_string(path.join("def").join("item").join("Ingot.ron")).unwrap()).unwrap());
        items.push(from_str::<AssetItemData>(&read_to_string(path.join("def").join("item").join("Tools.ron")).unwrap()).unwrap());

        data.world.insert(workplaces);
        data.world.insert(items);

        let mut axis: Vec<AIAxis> = Vec::new();
        let mut actions: Vec<Box<dyn AIAction + Send + Sync>> = Vec::new();

        axis.push(from_str::<AIAxis>(&read_to_string(path.join("def").join("axis").join("DistanceFromMe.ron")).unwrap()).unwrap());
        axis.push(from_str::<AIAxis>(&read_to_string(path.join("def").join("axis").join("OreEmpty.ron")).unwrap()).unwrap());
        axis.push(from_str::<AIAxis>(&read_to_string(path.join("def").join("axis").join("IngotEmpty.ron")).unwrap()).unwrap());
        axis.push(from_str::<AIAxis>(&read_to_string(path.join("def").join("axis").join("ToolsEmpty.ron")).unwrap()).unwrap());
        axis.push(from_str::<AIAxis>(&read_to_string(path.join("def").join("axis").join("OreFull.ron")).unwrap()).unwrap());
        axis.push(from_str::<AIAxis>(&read_to_string(path.join("def").join("axis").join("IngotFull.ron")).unwrap()).unwrap());

        actions.push(Box::new(AIActionWorkAtMine { name: "Work at Mine".to_string(), axis: vec![0, 1], delays: HashMap::new() }));
        actions.push(Box::new(AIActionWorkAtFurnace { name: "Work at Furnace".to_string(), axis: vec![0, 2, 4], delays: HashMap::new() }));
        actions.push(Box::new(AIActionWorkAtSmithy { name: "Work at Smithy".to_string(), axis: vec![0, 3, 5], delays: HashMap::new() }));
        actions.push(Box::new(AIActionIdle { name: "Idle".to_string(), axis: Vec::new(), delays: HashMap::new() }));

        data.world.insert(axis);
        data.world.insert(actions);
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