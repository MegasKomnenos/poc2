use crate::misc::*;
use crate::asset::*;
use crate::ai::*;
use crate::component::*;
use crate::map::*;
use crate::{ NUM_ITEM, MAP_SIZE };

use amethyst::{
    prelude::*,
    core::{ math::Vector3, Transform },
    ecs::Join,
    input::{ is_close_requested, is_key_down, },
    renderer::{ camera::Camera, SpriteRender },
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
        data.world.insert(MiscTime::default());
        data.world.insert(MiscMapMode::default());

        let map_sprite_sheet_handle = load_sprite_sheet(data.world, "texture/tile_sprites.png", "texture/tile_sprites.ron");
        let character_sprite_sheet_handle = load_sprite_sheet(data.world, "texture/character_sprites.png", "texture/character_sprites.ron");


        let mut map = TileMap::<MiscTile, MortonEncoder2D>::new(
            Vector3::new(MAP_SIZE, MAP_SIZE, 1),
            Vector3::new(1, 1, 1),
            Some(map_sprite_sheet_handle),
        );

        gen_map(&mut map);

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
        data.world
            .create_entity()
            .with(
                SpriteRender { 
                    sprite_sheet: character_sprite_sheet_handle,
                    sprite_number: 0,
                },
            )
            .with(
                Transform::from(Vector3::new(0.0, 0.0, 0.0)),
            )
            .with(
                ComponentMovement { 
                    targets: Vec::new(), 
                    velocity: Vector3::new(0.0, 0.0, 0.0),
                    speed_limit: 0.1, 
                    acceleration: 0.05, 
                },
            )
            .with(ComponentPlayerControlled)
            .build();
        
        load_ui(data.world);
        
        let path = application_root_dir().unwrap().join("asset");
        
        let mut workplaces = Vec::new();
        let mut items = Vec::new();

        workplaces.push(from_str::<AssetWorkplaceData>(&read_to_string(path.join("def").join("workplace").join("Mine.ron")).unwrap()).unwrap());
        workplaces.push(from_str::<AssetWorkplaceData>(&read_to_string(path.join("def").join("workplace").join("Furnace.ron")).unwrap()).unwrap());
        workplaces.push(from_str::<AssetWorkplaceData>(&read_to_string(path.join("def").join("workplace").join("Smithy.ron")).unwrap()).unwrap());
        workplaces.push(from_str::<AssetWorkplaceData>(&read_to_string(path.join("def").join("workplace").join("Market.ron")).unwrap()).unwrap());

        items.push(from_str::<AssetItemData>(&read_to_string(path.join("def").join("item").join("Amethyst.ron")).unwrap()).unwrap());
        items.push(from_str::<AssetItemData>(&read_to_string(path.join("def").join("item").join("Ore.ron")).unwrap()).unwrap());
        items.push(from_str::<AssetItemData>(&read_to_string(path.join("def").join("item").join("Ingot.ron")).unwrap()).unwrap());
        items.push(from_str::<AssetItemData>(&read_to_string(path.join("def").join("item").join("Tools.ron")).unwrap()).unwrap());

        data.world.insert(workplaces);
        data.world.insert(items);

        let mut axis: Vec<AIAxis> = Vec::new();
        let mut actions: Vec<Box<dyn AIAction>> = Vec::new();

        axis.push(from_str::<AIAxis>(&read_to_string(path.join("def").join("axis").join("DistanceFromMe.ron")).unwrap()).unwrap()); // 0
        axis.push(from_str::<AIAxis>(&read_to_string(path.join("def").join("axis").join("OreEmpty.ron")).unwrap()).unwrap());       // 1
        axis.push(from_str::<AIAxis>(&read_to_string(path.join("def").join("axis").join("IngotEmpty.ron")).unwrap()).unwrap());     // 2
        axis.push(from_str::<AIAxis>(&read_to_string(path.join("def").join("axis").join("ToolsEmpty.ron")).unwrap()).unwrap());     // 3
        axis.push(from_str::<AIAxis>(&read_to_string(path.join("def").join("axis").join("OreFull.ron")).unwrap()).unwrap());        // 4
        axis.push(from_str::<AIAxis>(&read_to_string(path.join("def").join("axis").join("IngotFull.ron")).unwrap()).unwrap());      // 5
        axis.push(from_str::<AIAxis>(&read_to_string(path.join("def").join("axis").join("ToolsFull.ron")).unwrap()).unwrap());      // 6
        axis.push(from_str::<AIAxis>(&read_to_string(path.join("def").join("axis").join("OrePriceBuy.ron")).unwrap()).unwrap());    // 7
        axis.push(from_str::<AIAxis>(&read_to_string(path.join("def").join("axis").join("IngotPriceBuy.ron")).unwrap()).unwrap());  // 8
        axis.push(from_str::<AIAxis>(&read_to_string(path.join("def").join("axis").join("ToolsPriceBuy.ron")).unwrap()).unwrap());  // 9
        axis.push(from_str::<AIAxis>(&read_to_string(path.join("def").join("axis").join("OrePriceSell.ron")).unwrap()).unwrap());   // 10
        axis.push(from_str::<AIAxis>(&read_to_string(path.join("def").join("axis").join("IngotPriceSell.ron")).unwrap()).unwrap()); // 11
        axis.push(from_str::<AIAxis>(&read_to_string(path.join("def").join("axis").join("ToolsPriceSell.ron")).unwrap()).unwrap()); // 12
        axis.push(from_str::<AIAxis>(&read_to_string(path.join("def").join("axis").join("CanBuyOre.ron")).unwrap()).unwrap());      // 13
        axis.push(from_str::<AIAxis>(&read_to_string(path.join("def").join("axis").join("CanBuyIngot.ron")).unwrap()).unwrap());    // 14
        axis.push(from_str::<AIAxis>(&read_to_string(path.join("def").join("axis").join("CanBuyTools.ron")).unwrap()).unwrap());    // 15
        axis.push(from_str::<AIAxis>(&read_to_string(path.join("def").join("axis").join("CanSellOre.ron")).unwrap()).unwrap());     // 16
        axis.push(from_str::<AIAxis>(&read_to_string(path.join("def").join("axis").join("CanSellIngot.ron")).unwrap()).unwrap());   // 17
        axis.push(from_str::<AIAxis>(&read_to_string(path.join("def").join("axis").join("CanSellTools.ron")).unwrap()).unwrap());   // 18

        actions.push(Box::new(AIActionIdle { name: "Idle".to_string(), axis: Vec::new(), delays: HashMap::new() }));
        actions.push(Box::new(AIActionWorkAtMine { name: "Work at Mine".to_string(), axis: vec![0, 1, 6], delays: HashMap::new() }));
        actions.push(Box::new(AIActionWorkAtFurnace { name: "Work at Furnace".to_string(), axis: vec![0, 2, 4], delays: HashMap::new() }));
        actions.push(Box::new(AIActionWorkAtSmithy { name: "Work at Smithy".to_string(), axis: vec![0, 3, 5], delays: HashMap::new() }));
        actions.push(Box::new(AIActionBuyOre { name: "Buy Ore".to_string(), axis: vec![0, 1, 7, 13], delays: HashMap::new() }));
        actions.push(Box::new(AIActionBuyIngot { name: "Buy Ingot".to_string(), axis: vec![0, 2, 8, 14], delays: HashMap::new() }));
        actions.push(Box::new(AIActionBuyTools { name: "Buy Tools".to_string(), axis: vec![0, 3, 9, 15], delays: HashMap::new() }));
        actions.push(Box::new(AIActionSellOre { name: "Sell Ore".to_string(), axis: vec![0, 4, 10, 16], delays: HashMap::new() }));
        actions.push(Box::new(AIActionSellIngot { name: "Sell Ingot".to_string(), axis: vec![0, 5, 11, 17], delays: HashMap::new() }));
        actions.push(Box::new(AIActionSellTools { name: "Sell Tools".to_string(), axis: vec![0, 6, 12, 18], delays: HashMap::new() }));

        data.world.insert(axis);
        data.world.insert(actions);
    }

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        let workplaces = data.world.read_storage::<ComponentWorkplace>();
        let prices = data.world.read_storage::<ComponentPrice>();
        let stockpiles = data.world.read_storage::<ComponentStockpile>();
        let agents = data.world.read_storage::<ComponentAgent>();
        let item_datas = data.world.read_resource::<Vec<AssetItemData>>();

        for (_, price, stockpile) in (&workplaces, &prices, &stockpiles).join() {
            println!("Market");

            for i in 0..NUM_ITEM {
                println!("{}: {}, {}, {}", item_datas[i].name, stockpile.items[i], price.buy[i], price.sell[i]);
            }
        }
        for (_, price, stockpile) in (&agents, &prices, &stockpiles).join() {
            println!("Agent");

            for i in 0..NUM_ITEM {
                println!("{}: {}, {}, {}", item_datas[i].name, stockpile.items[i], price.buy[i], price.sell[i]);
            }
        }
        
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