use amethyst::{
    prelude::*,
    core::{ math::{ Point3, Vector2, Vector3 }, Transform, },
    ecs::{ World, WorldExt, Join, SystemData, Entities, Read, ReadExpect, ReadStorage, },
    assets::{ Loader, AssetStorage, },
    renderer::{
        formats::texture::ImageFormat,
        camera::{ ActiveCamera, Camera, },
        sprite::{SpriteSheet, SpriteSheetFormat, SpriteSheetHandle},
        Texture,
    },
    ui::{
        Anchor, TtfFormat, UiText, UiTransform
    },
    window::ScreenDimensions,
};
use amethyst_tiles::{ Tile, TileMap, CoordinateEncoder, MortonEncoder2D, MapStorage, Map, DrawTiles2DBounds, Region, };
use pathfinding::prelude::{ astar, absdiff };

pub struct MiscTime {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub mnt: u8,
    pub scnd: u8,
    pub am: bool,
}
impl Default for MiscTime {
    fn default() -> Self {
        MiscTime {
            year: 0,
            month: 9,
            day: 1,
            hour: 6,
            mnt: 0,
            scnd: 0,
            am: true,
        }
    }
}

#[derive(Default, Clone)]
pub struct MiscTile {
    pub terrain: usize,
}
impl Tile for MiscTile {
    fn sprite(&self, _: Point3<u32>, _: &World) -> Option<usize> {
        Some(self.terrain)
    }
}

#[derive(Debug)]
pub struct MiscTileBounds;
impl DrawTiles2DBounds for MiscTileBounds {
    fn bounds<T: Tile, E: CoordinateEncoder>(map: &TileMap<T, E>, world: &World) -> Region {
        let camera_fetch = amethyst::renderer::submodules::gather::CameraGatherer::gather_camera_entity(world);
        
        assert!(camera_fetch.is_some());

        let (entities, active_camera, dimensions, transforms, cameras) =
            <(
                Entities<'_>,
                Read<'_, ActiveCamera>,
                ReadExpect<'_, ScreenDimensions>,
                ReadStorage<'_, Transform>,
                ReadStorage<'_, Camera>,
            )>::fetch(world);
        
        let mut camera_join = (&cameras, &transforms).join();
        if let Some((camera, camera_transform)) = active_camera
            .entity
            .and_then(|a| camera_join.get(a, &entities))
            .or_else(|| camera_join.next())
        {
            let coord = camera.projection()
                .screen_to_world_point(
                    Point3::new(0.0, 0.0, 0.0),
                    Vector2::new(dimensions.width(), dimensions.height()),
                    camera_transform,
                );
            let top_left = Vector3::new(coord[0], coord[1], coord[2]);

            let coord = camera.projection()
                .screen_to_world_point(
                    Point3::new(dimensions.width(), dimensions.height(), 0.0),
                    Vector2::new(dimensions.width(), dimensions.height()),
                    camera_transform,
                );
            let bottom_right = Vector3::new(coord[0], coord[1], coord[2]);

            let half_dimensions = Vector2::new(
                (map.tile_dimensions().x * map.dimensions().x) as f32 / 2.0,
                (map.tile_dimensions().x * map.dimensions().y) as f32 / 2.0,
            );
            let bottom_right = Vector3::new(
                bottom_right
                    .x
                    .min(half_dimensions.x - map.tile_dimensions().x as f32)
                    .max(-half_dimensions.x),
                bottom_right
                    .y
                    .min(half_dimensions.y - map.tile_dimensions().y as f32)
                    .max(-half_dimensions.y + map.tile_dimensions().y as f32),
                bottom_right
                    .z
                    .min(-0.0)
                    .max(0.0),
            );
            
            let min = map
                .to_tile(&top_left)
                .unwrap_or_else(|| Point3::new(0, 0, 0));
            let max = map
                .to_tile(&bottom_right)
                .unwrap_or_else(|| Point3::new(map.dimensions().x - 1, map.dimensions().y - 1, 0));
                
            Region::new(min, max)
        } else {
            Region::empty()
        }
    }
}

pub fn load_ui(world: &mut World) {
    let font = world.read_resource::<Loader>().load(
        "font/square.ttf",
        TtfFormat,
        (),
        &world.read_resource(),
    );
    let transform = UiTransform::new(
        "Time".to_string(), Anchor::TopMiddle, Anchor::TopMiddle,
        0., 0., 1., 200., 50.,
    );

    world
        .create_entity()
        .with(transform)
        .with(UiText::new(
            font.clone(),
            "0".to_string(),
            [1., 1., 1., 1.],
            50.,
        )).build();
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

    let mut out = Vec::new();

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
        for (i, target) in targets.iter().rev().enumerate() {
            let i = targets.len() - i - 1;

            if i == 0 || i + 1 == targets.len() {
                out.push(*target);
            } else {
                let t0 = targets[i + 1];
                let t1 = targets[i - 1];
                let t2 = Point3::new(t0[0] + t1[0], t0[1] + t1[1], t0[2] + t1[2]);

                if t2 != target * 2 {
                    out.push(*target);
                }
            }
        }
    }

    out
}

pub fn get_indiff(k: f32, x: u16, a0: f32, a1: f32, p0: f32, p1: f32) -> u16 {
    return ((k - (a0 * x as f32).powf(p0)).powf(1.0 / p1) / a1).ceil() as u16;
}
pub fn get_util(x: u16, a: f32, p: f32) -> f32 {
    return (x as f32 * a).powf(p);
}

pub fn get_price(buying: bool, amount: u16, currency: (u16, f32, f32), item: (u16, f32, f32)) -> u16 {
    let k = get_util(currency.0, currency.1, currency.2) + get_util(item.0, item.1, item.2);

    if buying {
        return currency.0 - get_indiff(k, item.0 + amount, item.1, currency.1, item.2, currency.2);
    } else {
        return get_indiff(k, item.0 - amount, item.1, currency.1, item.2, currency.2) - currency.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_util() {
        assert!(get_util(2, 5.551, 0.766) <= 6.32087696336 + 0.001 || get_util(2, 5.551, 0.766) >= 6.32087696336 - 0.001);
        assert!(get_util(4, 9.7331254, 0.586978216) <= 8.57982999635 + 0.001 || get_util(4, 9.7331254, 0.586978216) >= 8.57982999635 - 0.001);
    }

    #[test]
    fn test_indiff() {
        assert_eq!(get_indiff(5.0, 2, 5.0, 1.0, 0.5, 0.5), 4);
        assert_eq!(get_indiff(3.5, 3, 5.5, 1.9, 0.4, 0.6), 1);
    }
}