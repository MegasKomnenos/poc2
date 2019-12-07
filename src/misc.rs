use amethyst::{
    core::math::Point3,
    ecs::{ World, WorldExt, },
    assets::{ Loader, AssetStorage, },
    renderer::{
        formats::texture::ImageFormat,
        sprite::{SpriteSheet, SpriteSheetFormat, SpriteSheetHandle},
        Texture,
    },
};
use amethyst_tiles::Tile;

#[derive(Default, Clone)]
pub struct MiscTile;

impl Tile for MiscTile {
    fn sprite(&self, _: Point3<u32>, _: &World) -> Option<usize> {
        Some(0)
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