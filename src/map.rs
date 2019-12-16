use crate::misc::*;
use crate::component::*;
use crate::MAP_SIZE;

use amethyst::core::math::Point3;
use amethyst_tiles::{TileMap, MapStorage, MortonEncoder2D, Map};
use rand::Rng;
use voronoi::{voronoi, Point, lloyd_relaxation, DCEL, make_polygons};

type Segment = [Point; 2];

pub fn gen_map(tiles: &mut TileMap<MiscTile, MortonEncoder2D>) {
    let mut rng = rand::thread_rng();
    let mut vor_pts = Vec::<Point>::new();
    let map_size = MAP_SIZE as f64;

    for _ in 0..100 {
        vor_pts.push(Point::new(rng.gen::<f64>() * map_size, rng.gen::<f64>() * map_size))
    }

    let vor_pts = lloyd_relaxation(vor_pts, map_size);
    let vor_pts = lloyd_relaxation(vor_pts, map_size);

    let vor_diagram = voronoi(vor_pts, map_size);
    let vor_polys = make_polygons(&vor_diagram);

    let mut vor_seas = Vec::<(f64, f64)>::new();
    let mut vor_seas_poly = Vec::<Vec<Point>>::new();

    for poly in vor_polys.iter() {
        for point in poly.iter() {
            if point.x.into_inner() > map_size * 0.8 {
                vor_seas_poly.push(poly.clone());

                for point in poly.iter() {
                    if !vor_seas.contains(&(*point.x, *point.y)) {
                        vor_seas.push((*point.x, *point.y));
                    }
                }

                break;
            }
        }
    }

    for y in 0..MAP_SIZE as usize {
        for x in 0..MAP_SIZE as usize {
            let tile = tiles.get_mut(&Point3::new(x as u32, y as u32, 0)).unwrap();

            for poly in vor_seas_poly.iter() {
                let mut segments = Vec::<Segment>::new();

                for (i, point) in poly.iter().enumerate() {
                    if i + 1 < poly.len() {
                        segments.push([point.clone(), poly[i + 1].clone()]);
                    } else {
                        segments.push([point.clone(), poly[0].clone()]);
                    }
                }

                let check = [Point::new(x as f64, y as f64), Point::new(map_size + 10.0, y as f64)];
                let mut count = 0;

                for segment in segments.iter() {
                    match segment_intersection(check, *segment) {
                        Some(_) => count += 1,
                        None => (),
                    };
                }

                if count % 2 == 1 {
                    tile.terrain = 1;

                    break;
                }
            }
        }
    }
}

fn segment_intersection(seg1: Segment, seg2: Segment) -> Option<Point> {
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