use crate::misc::*;
use crate::component::*;
use crate::asset::*;
use crate::NUM_ITEM;

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

pub type AIData<'a> = (
    &'a Entities<'a>,
    Read<'a, Vec<AssetWorkplaceData>>,
    Read<'a, Vec<AssetItemData>>,
    Read<'a, Vec<AIAxis>>,
    WriteStorage<'a, TileMap<MiscTile>>,
    WriteStorage<'a, Transform>,
    WriteStorage<'a, ComponentWorkplace>,
    WriteStorage<'a, ComponentStockpile>,
    WriteStorage<'a, ComponentMovement>,
    WriteStorage<'a, ComponentPrice>,
);

#[derive(Serialize, Deserialize)]
pub enum AICurveType {
    Quadratic,
    Logistic,
    Logit,
}

#[derive(Serialize, Deserialize)]
pub enum AIInputType {
    MyStockpileOre,
    MyStockpileIngot,
    MyStockpileTools,
    DistanceFromMe,
    PriceDiffBuyOre,
    PriceDiffBuyIngot,
    PriceDiffBuyTools,
    PriceDiffSellOre,
    PriceDiffSellIngot,
    PriceDiffSellTools,
    CanBuyOre,
    CanBuyIngot,
    CanBuyTools,
    CanSellOre,
    CanSellIngot,
    CanSellTools,
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

pub trait AIAction: Send + Sync {
    fn get_name(&self) -> &String;
    fn get_delay(&self) -> &HashMap<Entity, u32>;

    fn eval(&self, _: &Entity, _: &AIData) -> Option<(u8, Option<Entity>, f32)>;

    fn init(&mut self, _: &Entity, _: &Entity, _: &mut AIData) -> bool;
    fn run(&mut self, _: &Entity, _: &Entity, _: &mut AIData) -> bool;
}

pub struct AIActionIdle {
    pub name: String,
    pub axis: Vec<u16>,
    pub delays: HashMap<Entity, u32>,
}
pub struct AIActionWorkAtSmithy {
    pub name: String,
    pub axis: Vec<u16>,
    pub delays: HashMap<Entity, u32>,
}
pub struct AIActionWorkAtFurnace {
    pub name: String,
    pub axis: Vec<u16>,
    pub delays: HashMap<Entity, u32>,
}
pub struct AIActionWorkAtMine {
    pub name: String,
    pub axis: Vec<u16>,
    pub delays: HashMap<Entity, u32>,
}
pub struct AIActionBuyOre {
    pub name: String,
    pub axis: Vec<u16>,
    pub delays: HashMap<Entity, u32>,
}
pub struct AIActionBuyIngot {
    pub name: String,
    pub axis: Vec<u16>,
    pub delays: HashMap<Entity, u32>,
}
pub struct AIActionBuyTools {
    pub name: String,
    pub axis: Vec<u16>,
    pub delays: HashMap<Entity, u32>,
}
pub struct AIActionSellOre {
    pub name: String,
    pub axis: Vec<u16>,
    pub delays: HashMap<Entity, u32>,
}
pub struct AIActionSellIngot {
    pub name: String,
    pub axis: Vec<u16>,
    pub delays: HashMap<Entity, u32>,
}
pub struct AIActionSellTools {
    pub name: String,
    pub axis: Vec<u16>,
    pub delays: HashMap<Entity, u32>,
}


impl AIAction for AIActionIdle {
    fn get_name(&self) -> &String {
        &self.name
    }
    fn get_delay(&self) -> &HashMap<Entity, u32> {
        &self.delays
    }

    fn eval(&self, _: &Entity, _: &AIData) -> Option<(u8, Option<Entity>, f32)> {
        return Some((0, None, 0.0));
    }

    fn init(&mut self, _: &Entity, _: &Entity, _: &mut AIData) -> bool {
        return true;
    }
    fn run(&mut self, _: &Entity, _: &Entity, _: &mut AIData) -> bool {
        return true;
    }
}

impl AIAction for AIActionWorkAtSmithy {
    fn get_name(&self) -> &String {
        &self.name
    }
    fn get_delay(&self) -> &HashMap<Entity, u32> {
        &self.delays
    }

    fn eval(&self, me: &Entity, ai_data: &AIData) -> Option<(u8, Option<Entity>, f32)> {
        let (entities, _, _, axis_datas, _, _, workplaces, _, _, _) = &ai_data;

        let mut out = (3, None, 0.0);

        for (target, workplace) in (*entities, workplaces).join() {
            if workplace.variant == 2 {
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

        //println!("{}: {}", self.name, out.2);

        if out.2 > 0.0 {
            return Some(out);
        } else {
            return None;
        }
    }

    fn init(&mut self, me: &Entity, target: &Entity, ai_data: &mut AIData) -> bool {
        let (_, _, _, _, tilemaps, transforms, _, _, movements, _) = ai_data;
        let tilemap = (tilemaps).join().next().unwrap();

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
    fn run(&mut self, me: &Entity, target: &Entity, ai_data: &mut AIData) -> bool {
        let (_, workplace_datas, _, _, _, _, workplaces, stockpiles, movements, prices) = ai_data;

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

            prices.get_mut(*me).unwrap().update[1] = true;

            return true;
        } else {
            let workplace = workplaces.get(*target).unwrap();

            self.delays.insert(*me, workplace_datas[workplace.variant as usize].duration);

            return false;
        }
    }
}

impl AIAction for AIActionWorkAtFurnace {
    fn get_name(&self) -> &String {
        &self.name
    }
    fn get_delay(&self) -> &HashMap<Entity, u32> {
        &self.delays
    }

    fn eval(&self, me: &Entity, ai_data: &AIData) -> Option<(u8, Option<Entity>, f32)> {
        let (entities, _, _, axis_datas, _, _, workplaces, _, _, _) = &ai_data;

        let mut out = (2, None, 0.0);

        for (target, workplace) in (*entities, workplaces).join() {
            if workplace.variant == 1 {
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

        //println!("{}: {}", self.name, out.2);

        if out.2 > 0.0 {
            return Some(out);
        } else {
            return None;
        }
    }

    fn init(&mut self, me: &Entity, target: &Entity, ai_data: &mut AIData) -> bool {
        let (_, _, _, _, tilemaps, transforms, _, _, movements, _) = ai_data;
        let tilemap = (tilemaps).join().next().unwrap();

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
    fn run(&mut self, me: &Entity, target: &Entity, ai_data: &mut AIData) -> bool {
        let (_, workplace_datas, _, _, _, _, workplaces, stockpiles, movements, prices) = ai_data;

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

            prices.get_mut(*me).unwrap().update[2] = true;

            return true;
        } else {
            let workplace = workplaces.get(*target).unwrap();

            self.delays.insert(*me, workplace_datas[workplace.variant as usize].duration);

            return false;
        }
    }
}

impl AIAction for AIActionWorkAtMine {
    fn get_name(&self) -> &String {
        &self.name
    }
    fn get_delay(&self) -> &HashMap<Entity, u32> {
        &self.delays
    }

    fn eval(&self, me: &Entity, ai_data: &AIData) -> Option<(u8, Option<Entity>, f32)> {
        let (entities, _, _, axis_datas, _, _, workplaces, _, _, _) = ai_data;

        let mut out = (1, None, 0.0);

        for (target, workplace) in (*entities, workplaces).join() {
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

        //println!("{}: {}", self.name, out.2);

        if out.2 > 0.0 {
            return Some(out);
        } else {
            return None;
        }
    }

    fn init(&mut self, me: &Entity, target: &Entity, ai_data: &mut AIData) -> bool {
        let (_, _, _, _, tilemaps, transforms, _, _, movements, _) = ai_data;
        let tilemap = (tilemaps).join().next().unwrap();

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
    fn run(&mut self, me: &Entity, target: &Entity, ai_data: &mut AIData) -> bool {
        let (_, workplace_datas, _, _, _, _, workplaces, stockpiles, movements, prices) = ai_data;

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

            prices.get_mut(*me).unwrap().update[3] = true;

            return true;
        } else {
            let workplace = workplaces.get(*target).unwrap();

            self.delays.insert(*me, workplace_datas[workplace.variant as usize].duration);

            return false;
        }
    }
}

impl AIAction for AIActionBuyOre {
    fn get_name(&self) -> &String {
        &self.name
    }
    fn get_delay(&self) -> &HashMap<Entity, u32> {
        &self.delays
    }

    fn eval(&self, me: &Entity, ai_data: &AIData) -> Option<(u8, Option<Entity>, f32)> {
        let (entities, _, _, axis_datas, _, _, workplaces, stockpiles, _, prices) = ai_data;

        let mut out = (4, None, 0.0);

        for (target, workplace, _, stockpile) in (*entities, workplaces, prices, stockpiles).join() {
            if workplace.variant == 3 && stockpile.items[1] >= 1 {
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

        //println!("{}: {}", self.name, out.2);

        if out.2 > 0.0 {
            return Some(out);
        } else {
            return None;
        }
    }

    fn init(&mut self, me: &Entity, target: &Entity, ai_data: &mut AIData) -> bool {
        let (_, _, _, _, tilemaps, transforms, _, _, movements, _) = ai_data;
        let tilemap = (tilemaps).join().next().unwrap();

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
    fn run(&mut self, me: &Entity, target: &Entity, ai_data: &mut AIData) -> bool {
        let (_, _, _, _, _, _, _, stockpiles, movements, prices) = ai_data;

        if movements.get(*me).unwrap().targets.len() > 0 {
            return false;
        }
        
        let stockpile = stockpiles.get_mut(*target).unwrap();
        let price = prices.get(*target).unwrap();

        if prices.get(*me).unwrap().buy[1] > price.sell[1] && stockpile.items[1] >= 1 {
            stockpile.items[1] -= 1;
            stockpile.items[0] += price.sell[1];

            let stockpile = stockpiles.get_mut(*me).unwrap();

            stockpile.items[1] += 1;
            stockpile.items[0] -= price.sell[1];

            prices.get_mut(*me).unwrap().update = [true; NUM_ITEM];
            prices.get_mut(*target).unwrap().update = [true; NUM_ITEM];
        }

        return true;
    }
}

impl AIAction for AIActionBuyIngot {
    fn get_name(&self) -> &String {
        &self.name
    }
    fn get_delay(&self) -> &HashMap<Entity, u32> {
        &self.delays
    }

    fn eval(&self, me: &Entity, ai_data: &AIData) -> Option<(u8, Option<Entity>, f32)> {
        let (entities, _, _, axis_datas, _, _, workplaces, stockpiles, _, prices) = ai_data;

        let mut out = (5, None, 0.0);

        for (target, workplace, _, stockpile) in (*entities, workplaces, prices, stockpiles).join() {
            if workplace.variant == 3 && stockpile.items[2] >= 1 {
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

        //println!("{}: {}", self.name, out.2);

        if out.2 > 0.0 {
            return Some(out);
        } else {
            return None;
        }
    }

    fn init(&mut self, me: &Entity, target: &Entity, ai_data: &mut AIData) -> bool {
        let (_, _, _, _, tilemaps, transforms, _, _, movements, _) = ai_data;
        let tilemap = (tilemaps).join().next().unwrap();

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
    fn run(&mut self, me: &Entity, target: &Entity, ai_data: &mut AIData) -> bool {
        let (_, _, _, _, _, _, _, stockpiles, movements, prices) = ai_data;

        if movements.get(*me).unwrap().targets.len() > 0 {
            return false;
        }
        
        let stockpile = stockpiles.get_mut(*target).unwrap();
        let price = prices.get(*target).unwrap();

        if prices.get(*me).unwrap().buy[2] > price.sell[2] && stockpile.items[2] >= 1 {
            stockpile.items[2] -= 1;
            stockpile.items[0] += price.sell[2];

            let stockpile = stockpiles.get_mut(*me).unwrap();

            stockpile.items[2] += 1;
            stockpile.items[0] -= price.sell[2];

            prices.get_mut(*me).unwrap().update = [true; NUM_ITEM];
            prices.get_mut(*target).unwrap().update = [true; NUM_ITEM];
        }

        return true;
    }
}

impl AIAction for AIActionBuyTools {
    fn get_name(&self) -> &String {
        &self.name
    }
    fn get_delay(&self) -> &HashMap<Entity, u32> {
        &self.delays
    }

    fn eval(&self, me: &Entity, ai_data: &AIData) -> Option<(u8, Option<Entity>, f32)> {
        let (entities, _, _, axis_datas, _, _, workplaces, stockpiles, _, prices) = ai_data;

        let mut out = (6, None, 0.0);

        for (target, workplace, _, stockpile) in (*entities, workplaces, prices, stockpiles).join() {
            if workplace.variant == 3 && stockpile.items[3] >= 1 {
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

        //println!("{}: {}", self.name, out.2);

        if out.2 > 0.0 {
            return Some(out);
        } else {
            return None;
        }
    }

    fn init(&mut self, me: &Entity, target: &Entity, ai_data: &mut AIData) -> bool {
        let (_, _, _, _, tilemaps, transforms, _, _, movements, _) = ai_data;
        let tilemap = (tilemaps).join().next().unwrap();

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
    fn run(&mut self, me: &Entity, target: &Entity, ai_data: &mut AIData) -> bool {
        let (_, _, _, _, _, _, _, stockpiles, movements, prices) = ai_data;

        if movements.get(*me).unwrap().targets.len() > 0 {
            return false;
        }
        
        let stockpile = stockpiles.get_mut(*target).unwrap();
        let price = prices.get(*target).unwrap();

        if prices.get(*me).unwrap().buy[3] > price.sell[3] && stockpile.items[3] >= 1 {
            stockpile.items[3] -= 1;
            stockpile.items[0] += price.sell[3];

            let stockpile = stockpiles.get_mut(*me).unwrap();

            stockpile.items[3] += 1;
            stockpile.items[0] -= price.sell[3];

            prices.get_mut(*me).unwrap().update = [true; NUM_ITEM];
            prices.get_mut(*target).unwrap().update = [true; NUM_ITEM];
        }

        return true;
    }
}

impl AIAction for AIActionSellOre {
    fn get_name(&self) -> &String {
        &self.name
    }
    fn get_delay(&self) -> &HashMap<Entity, u32> {
        &self.delays
    }

    fn eval(&self, me: &Entity, ai_data: &AIData) -> Option<(u8, Option<Entity>, f32)> {
        let (entities, _, _, axis_datas, _, _, workplaces, stockpiles, _, prices) = ai_data;

        let mut out = (7, None, 0.0);

        let stockpile = stockpiles.get(*me).unwrap();

        for (target, workplace, _, _) in (*entities, workplaces, prices, stockpiles).join() {
            if workplace.variant == 3 && stockpile.items[1] >= 1 {
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

        //println!("{}: {}", self.name, out.2);

        if out.2 > 0.0 {
            return Some(out);
        } else {
            return None;
        }
    }

    fn init(&mut self, me: &Entity, target: &Entity, ai_data: &mut AIData) -> bool {
        let (_, _, _, _, tilemaps, transforms, _, _, movements, _) = ai_data;
        let tilemap = (tilemaps).join().next().unwrap();

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
    fn run(&mut self, me: &Entity, target: &Entity, ai_data: &mut AIData) -> bool {
        let (_, _, _, _, _, _, _, stockpiles, movements, prices) = ai_data;

        if movements.get(*me).unwrap().targets.len() > 0 {
            return false;
        }
        
        let stockpile = stockpiles.get_mut(*target).unwrap();
        let price = prices.get(*target).unwrap();

        if prices.get(*me).unwrap().sell[1] < price.buy[1] && stockpile.items[0] >= price.buy[1] {
            stockpile.items[1] += 1;
            stockpile.items[0] -= price.buy[1];

            let stockpile = stockpiles.get_mut(*me).unwrap();

            stockpile.items[1] -= 1;
            stockpile.items[0] += price.buy[1];

            prices.get_mut(*me).unwrap().update = [true; NUM_ITEM];
            prices.get_mut(*target).unwrap().update = [true; NUM_ITEM];
        }

        return true;
    }
}

impl AIAction for AIActionSellIngot {
    fn get_name(&self) -> &String {
        &self.name
    }
    fn get_delay(&self) -> &HashMap<Entity, u32> {
        &self.delays
    }

    fn eval(&self, me: &Entity, ai_data: &AIData) -> Option<(u8, Option<Entity>, f32)> {
        let (entities, _, _, axis_datas, _, _, workplaces, stockpiles, _, prices) = ai_data;

        let mut out = (8, None, 0.0);

        let stockpile = stockpiles.get(*me).unwrap();

        for (target, workplace, _, _) in (*entities, workplaces, prices, stockpiles).join() {
            if workplace.variant == 3 && stockpile.items[2] >= 1 {
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

        //println!("{}: {}", self.name, out.2);

        if out.2 > 0.0 {
            return Some(out);
        } else {
            return None;
        }
    }

    fn init(&mut self, me: &Entity, target: &Entity, ai_data: &mut AIData) -> bool {
        let (_, _, _, _, tilemaps, transforms, _, _, movements, _) = ai_data;
        let tilemap = (tilemaps).join().next().unwrap();

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
    fn run(&mut self, me: &Entity, target: &Entity, ai_data: &mut AIData) -> bool {
        let (_, _, _, _, _, _, _, stockpiles, movements, prices) = ai_data;

        if movements.get(*me).unwrap().targets.len() > 0 {
            return false;
        }
        
        let stockpile = stockpiles.get_mut(*target).unwrap();
        let price = prices.get(*target).unwrap();

        if prices.get(*me).unwrap().sell[2] < price.buy[2] && stockpile.items[0] >= price.buy[2] {
            stockpile.items[2] += 1;
            stockpile.items[0] -= price.buy[2];

            let stockpile = stockpiles.get_mut(*me).unwrap();

            stockpile.items[2] -= 1;
            stockpile.items[0] += price.buy[2];

            prices.get_mut(*me).unwrap().update = [true; NUM_ITEM];
            prices.get_mut(*target).unwrap().update = [true; NUM_ITEM];
        }

        return true;
    }
}

impl AIAction for AIActionSellTools {
    fn get_name(&self) -> &String {
        &self.name
    }
    fn get_delay(&self) -> &HashMap<Entity, u32> {
        &self.delays
    }

    fn eval(&self, me: &Entity, ai_data: &AIData) -> Option<(u8, Option<Entity>, f32)> {
        let (entities, _, _, axis_datas, _, _, workplaces, stockpiles, _, prices) = ai_data;

        let mut out = (9, None, 0.0);

        let stockpile = stockpiles.get(*me).unwrap();

        for (target, workplace, _, _) in (*entities, workplaces, prices, stockpiles).join() {
            if workplace.variant == 3 && stockpile.items[3] >= 1 {
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

        //println!("{}: {}", self.name, out.2);

        if out.2 > 0.0 {
            return Some(out);
        } else {
            return None;
        }
    }

    fn init(&mut self, me: &Entity, target: &Entity, ai_data: &mut AIData) -> bool {
        let (_, _, _, _, tilemaps, transforms, _, _, movements, _) = ai_data;
        let tilemap = (tilemaps).join().next().unwrap();

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
    fn run(&mut self, me: &Entity, target: &Entity, ai_data: &mut AIData) -> bool {
        let (_, _, _, _, _, _, _, stockpiles, movements, prices) = ai_data;

        if movements.get(*me).unwrap().targets.len() > 0 {
            return false;
        }
        
        let stockpile = stockpiles.get_mut(*target).unwrap();
        let price = prices.get(*target).unwrap();

        if prices.get(*me).unwrap().sell[3] < price.buy[3] && stockpile.items[0] >= price.buy[3] {
            stockpile.items[3] += 1;
            stockpile.items[0] -= price.buy[3];

            let stockpile = stockpiles.get_mut(*me).unwrap();

            stockpile.items[3] -= 1;
            stockpile.items[0] += price.buy[3];

            prices.get_mut(*me).unwrap().update = [true; NUM_ITEM];
            prices.get_mut(*target).unwrap().update = [true; NUM_ITEM];
        }

        return true;
    }
}

pub fn clearing_house(variant: &AIInputType, me: &Entity, target: &Entity, foo: f32, ai_data: &AIData) -> f32 {
    let (entities, workplace_datas, item_datas, _, tilemaps, transforms, workplaces, stockpiles, movements, prices) = ai_data;
    
    match variant {
        AIInputType::MyStockpileOre => {
            return clamp(stockpiles.get(*me).unwrap().items[1] as f32 / foo);
        }
        AIInputType::MyStockpileIngot => {
            return clamp(stockpiles.get(*me).unwrap().items[2] as f32 / foo);
        }
        AIInputType::MyStockpileTools => {
            return clamp(stockpiles.get(*me).unwrap().items[3] as f32 / foo);
        }
        AIInputType::DistanceFromMe => {
            let diff = transforms.get(*me).unwrap().translation() - transforms.get(*target).unwrap().translation();
            let dist = (diff[0].powf(2.0) + diff[1].powf(2.0) + diff[2].powf(2.0)).sqrt();
            return clamp(dist / foo);
        }
        AIInputType::PriceDiffBuyOre => {
            return clamp((prices.get(*me).unwrap().buy[1] as f32 / prices.get(*target).unwrap().sell[1] as f32) / foo);
        }
        AIInputType::PriceDiffBuyIngot => {
            return clamp((prices.get(*me).unwrap().buy[2] as f32 / prices.get(*target).unwrap().sell[2] as f32) / foo);
        }
        AIInputType::PriceDiffBuyTools => {
            return clamp((prices.get(*me).unwrap().buy[3] as f32 / prices.get(*target).unwrap().sell[3] as f32) / foo);
        }
        AIInputType::PriceDiffSellOre => {
            return clamp((prices.get(*target).unwrap().buy[1] as f32 / prices.get(*me).unwrap().sell[1] as f32) / foo);
        }
        AIInputType::PriceDiffSellIngot => {
            return clamp((prices.get(*target).unwrap().buy[2] as f32 / prices.get(*me).unwrap().sell[2] as f32) / foo);
        }
        AIInputType::PriceDiffSellTools => {
            return clamp((prices.get(*target).unwrap().buy[3] as f32 / prices.get(*me).unwrap().sell[3] as f32) / foo);
        }
        AIInputType::CanBuyOre => {
            return clamp((stockpiles.get(*me).unwrap().items[0] as f32 / prices.get(*me).unwrap().buy[1] as f32) / foo);
        }
        AIInputType::CanBuyIngot => {
            return clamp((stockpiles.get(*me).unwrap().items[0] as f32 / prices.get(*me).unwrap().buy[2] as f32) / foo);
        }
        AIInputType::CanBuyTools => {
            return clamp((stockpiles.get(*me).unwrap().items[0] as f32 / prices.get(*me).unwrap().buy[3] as f32) / foo);
        }
        AIInputType::CanSellOre => {
            return clamp(stockpiles.get(*me).unwrap().items[1] as f32 / foo);
        }
        AIInputType::CanSellIngot => {
            return clamp(stockpiles.get(*me).unwrap().items[2] as f32 / foo);
        }
        AIInputType::CanSellTools => {
            return clamp(stockpiles.get(*me).unwrap().items[3] as f32 / foo);
        }
    }
}

pub fn response_curve(variant: &AICurveType, x: f32, m: f32, k: f32, b: f32, c: f32) -> f32 {
    match variant {
        AICurveType::Quadratic => {
            return clamp(m * (x - c).powf(k) + b);
        },
        AICurveType::Logistic => {
            return clamp(k / (1.0 + E.powf(m * (c - x))) + b);
        }
        AICurveType::Logit => {
            return clamp((k * (x - c) / (1.0 - x + c)).ln() * m + b);
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