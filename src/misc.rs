use amethyst::{
    core::math::{ Point3 },
    ecs::{ World, WorldExt, Entity, },
    assets::{ Loader, AssetStorage, },
    renderer::{
        formats::texture::ImageFormat,
        sprite::{SpriteSheet, SpriteSheetFormat, SpriteSheetHandle},
        Texture,
    },
};
use amethyst_tiles::{ Tile, TileMap, MortonEncoder2D, MapStorage, Map, };
use pathfinding::prelude::{ astar, absdiff };

#[derive(Default, Clone)]
pub struct MiscTile {
    pub terrain: usize,
    pub chars: Vec<Entity>,
}

impl Tile for MiscTile {
    fn sprite(&self, _: Point3<u32>, _: &World) -> Option<usize> {
        Some(self.terrain)
    }
}

pub fn load_sprite_sheet(world: &mut World, png_path: &str, ron_path: &str) -> SpriteSheetHandle {
    let texture_handle = {
        let loader = world.read_resource::<Loader>();
        let texture_storage = world.read_resource::<AssetStorage<Texture>>();
        loader.load(png_path, ImageFormat::default(), (), &texture_storage)
    };
    let loader = world.read_resource::<Loader>();
    let sprite_sheet_store = world.read_resource::<AssetStorage<SpriteSheet>>();
    loader.load(
        ron_path,
        SpriteSheetFormat(texture_handle),
        (),
        &sprite_sheet_store,
    )
}

pub fn load_sprite_sheet_system(
    loader: &Loader, 
    texture_storage: &AssetStorage<Texture>, 
    sprite_sheet_store: &AssetStorage<SpriteSheet>,
    png_path: &str, 
    ron_path: &str,
) -> SpriteSheetHandle {
    let texture_handle = {
        loader.load(png_path, ImageFormat::default(), (), texture_storage)
    };

    loader.load(ron_path, SpriteSheetFormat(texture_handle), (), sprite_sheet_store)
}

pub fn get_targets(start: &Point3<u32>, goal: &Point3<u32>, tilemap: &TileMap<MiscTile, MortonEncoder2D>) -> Vec<Point3<u32>> {
    let dimensions = tilemap.dimensions();

    if let Some((targets, _)) = astar(
        start,
        |&node| {
            let mut out = Vec::new();
            
            if node[0] >= 1 {
                let point = Point3::new(node[0] - 1, node[1], node[2]);

                if tilemap.get(&point).unwrap().terrain == 0 {
                    out.push((point, 1));
                }
            }
            if node[0] + 1 < dimensions[0] {
                let point = Point3::new(node[0] + 1, node[1], node[2]);

                if tilemap.get(&point).unwrap().terrain == 0 {
                    out.push((point, 1));
                }
            }
            if node[1] >= 1 {
                let point = Point3::new(node[0], node[1] - 1, node[2]);

                if tilemap.get(&point).unwrap().terrain == 0 {
                    out.push((point, 1));
                }
            }
            if node[1] + 1 < dimensions[1] {
                let point = Point3::new(node[0], node[1] + 1, node[2]);

                if tilemap.get(&point).unwrap().terrain == 0 {
                    out.push((point, 1));
                }
            }
            if node[0] >= 1 && node[1] >= 1 {
                let point = Point3::new(node[0] - 1, node[1] - 1, node[2]);

                if tilemap.get(&point).unwrap().terrain == 0 {
                    out.push((point, 1));
                }
            }
            if node[0] + 1 < dimensions[0] && node[1] >= 1 {
                let point = Point3::new(node[0] + 1, node[1] - 1, node[2]);

                if tilemap.get(&point).unwrap().terrain == 0 {
                    out.push((point, 1));
                }
            }
            if node[0] + 1 < dimensions[0] && node[1] + 1 < dimensions[1] {
                let point = Point3::new(node[0] + 1, node[1] + 1, node[2]);

                if tilemap.get(&point).unwrap().terrain == 0 {
                    out.push((point, 1));
                }
            }
            if node[0] >= 1 && node[1] + 1 < dimensions[1] {
                let point = Point3::new(node[0] - 1, node[1] + 1, node[2]);

                if tilemap.get(&point).unwrap().terrain == 0 {
                    out.push((point, 1));
                }
            }

            out
        },
        |&node| absdiff(node[0], goal[0]) + absdiff(node[1], goal[1]),
        |&node| node == *goal
    ) {
        targets
    } else {
        Vec::new()
    }
}