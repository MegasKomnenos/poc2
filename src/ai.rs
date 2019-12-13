use crate::misc::*;
use crate::component::*;
use crate::asset::*;

use std::f32::consts::E;

use amethyst::{
    core::{
        Transform,
    },
    ecs::{
        Entity, Entities, Read, WriteStorage, Join,
    },
};
use amethyst_tiles::{ TileMap, Map };

use serde::{ Serialize, Deserialize };

use std::collections::HashMap;

type AIData<'a> = (
    Entities<'a>,
    Read<'a, Vec<AssetWorkplaceData>>,
    Read<'a, Vec<AssetItemData>>,
    Read<'a, Vec<AIAxis>>,
    WriteStorage<'a, TileMap<MiscTile>>,
    WriteStorage<'a, Transform>,
    WriteStorage<'a, ComponentWorkplace>,
    WriteStorage<'a, ComponentStockpile>,
    WriteStorage<'a, ComponentMovement>,
);

#[derive(Serialize, Deserialize)]
pub enum AICurveType {
    Quadratic,
    Logistic,
}

#[derive(Serialize, Deserialize)]
pub enum AIInputType {
    MyStockpileOre,
    MyStockpileIngot,
    MyStockpileTools,
    DistanceFromMe,
}

#[derive(Serialize, Deserialize)]
pub struct AIAxis {
    pub name: String,
    pub curve: AICurveType,
    pub input: AIInputType,
    pub foo: f32,
    pub m: f32,
    pub k: f32,
    pub b: f32,
    pub c: f32,
}

pub trait AIAction<'a> {
    fn get_name(&self) -> &String;

    fn eval(&self, _: &Entity, _: AIData<'a>) -> Option<(u8, Option<Entity>, f32)>;

    fn init(&mut self, _: &Entity, _: &Entity, _: AIData<'a>) -> bool;
    fn run(&mut self, _: &Entity, _: &Entity, _: AIData<'a>) -> bool;
}

pub struct AIActionWorkAtMine {
    pub name: String,
    pub axis: Vec<u8>,
    pub delays: HashMap<Entity, u32>,
}

impl<'a> AIAction<'a> for AIActionWorkAtMine {
    fn get_name(&self) -> &String {
        &self.name
    }

    fn eval(&self, me: &Entity, ai_data: AIData) -> Option<(u8, Option<Entity>, f32)> {
        let (entities, _, _, axis_datas, _, _, workplaces, _, _) = &ai_data;

        let mut out = (0, None, -1.0);

        for (target, workplace) in (entities, workplaces).join() {
            if workplace.variant == 0 {
                let mut weight = 1.0;

                for axis_index in self.axis.iter() {
                    let axis = &axis_datas[*axis_index as usize];

                    let x = clearing_house(&axis.input, me, &target, axis.foo, &ai_data);
                    let y = response_curve(&axis.curve, x, axis.m, axis.k, axis.b, axis.c);

                    weight *= y;
                }

                if weight > out.2 {
                    out.1 = Some(target);
                    out.2 = weight;
                }
            }
        }

        if out.2 > -1.0 {
            return Some(out);
        } else {
            return None;
        }
    }

    fn init(&mut self, me: &Entity, target: &Entity, ai_data: AIData) -> bool {
        let (_, _, _, _, tilemaps, transforms, _, _, mut movements) = ai_data;
        let tilemap = (&tilemaps).join().next().unwrap();

        let me_point = tilemap.to_tile(transforms.get(*me).unwrap().translation()).unwrap();
        let target_point = tilemap.to_tile(transforms.get(*target).unwrap().translation()).unwrap();

        if me_point != target_point {
            let targets = get_targets(&me_point, &target_point, &tilemap);

            if targets.len() > 0 {
                movements.get_mut(*me).unwrap().targets = targets;
            } else {
                return false;
            }
        } else {
            movements.get_mut(*me).unwrap().targets.clear();
        }

        return true;
    }
    fn run(&mut self, me: &Entity, target: &Entity, ai_data: AIData) -> bool {
        let (_, workplace_datas, _, _, _, _, workplaces, mut stockpiles, movements) = ai_data;

        if movements.get(*me).unwrap().targets.len() > 0 {
            return false;
        }

        if let Some(delay) = self.delays.get_mut(me) {
            if *delay > 0 {
                *delay -= 1;

                return false;
            }

            self.delays.remove(me);

            let workplace = workplaces.get(*target).unwrap();
            let stockpile = stockpiles.get_mut(*me).unwrap();

            for (i, out) in workplace_datas[workplace.variant as usize].outs.iter().enumerate() {
                if *out > 0 {
                    stockpile.items[i] += *out as u16;
                }
            }
            for (i, input) in workplace_datas[workplace.variant as usize].inputs.iter().enumerate() {
                if *input > 0 {
                    stockpile.items[i] -= *input as u16;
                }
            }

            return true;
        } else {
            let workplace = workplaces.get(*target).unwrap();

            self.delays.insert(*me, workplace_datas[workplace.variant as usize].duration);

            return false;
        }
    }
}

pub fn clearing_house(variant: &AIInputType, me: &Entity, target: &Entity, foo: f32, ai_data: &AIData) -> f32 {
    let (entities, workplace_datas, item_datas, _, tilemaps, transforms, workplaces, stockpiles, movements) = ai_data;
    
    match variant {
        MyStockpileOre => {
            return stockpiles.get(*me).unwrap().items[0] as f32;
        }
        MyStockpileIngot => {
            return stockpiles.get(*me).unwrap().items[1] as f32;
        }
        MyStockpileTools => {
            return stockpiles.get(*me).unwrap().items[2] as f32;
        }
        DistanceFromMe => {
            let diff = transforms.get(*me).unwrap().translation() - transforms.get(*target).unwrap().translation();
            let dist = (diff[0].powf(2.0) + diff[1].powf(2.0) + diff[2].powf(2.0)).sqrt();
            return clamp(dist / foo);
        }
    }
}

pub fn response_curve(variant: &AICurveType, x: f32, m: f32, k: f32, b: f32, c: f32) -> f32 {
    match variant {
        Quadratic => {
            return clamp(m * (x - c).powf(k) + b);
        },
        Logistic => {
            return clamp(k / (1.0 + 1000.0 * E * m.powf(-1.0 * x + c)) + b);
        }
    }
}

pub fn clamp(x: f32) -> f32 {
    if x > 1.0 {
        return 1.0;
    } else if x < 0.0 {
        return 0.0;
    } else {
        return x;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clamp() {
        assert_eq!(clamp(-1.0), 0.0);
        assert_eq!(clamp(2.0), 1.0);
        assert_eq!(clamp(0.123), 0.123);
        assert_eq!(clamp(0.987), 0.987);
    }
}