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
use amethyst_tiles::Tile;
use voronoi::Point;

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

pub fn segment_intersection(seg1: &[Point; 2], seg2: &[Point; 2]) -> Option<Point> {
    let a = seg1[0];
    let c = seg2[0];
    let r = seg1[1] - a;
    let s = seg2[1] - c;

    let denom = r.cross(s);
    if denom == 0.0 { return None; }

    let numer_a = (c - a).cross(s);
    let numer_c = (c - a).cross(r);

    let t = numer_a / denom;
    let u = numer_c / denom;

    if t < 0.0 || t > 1.0 || u < 0.0 || u > 1.0 { return None; }

    return Some(a + r * t);
}