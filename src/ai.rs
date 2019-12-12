use crate::misc::*;
use crate::component::*;
use crate::asset::*;

use amethyst::{
    core::{
        math::{
            Point3,
        },
        Transform,
    },
    ecs::{
        Entity, Entities, Read, WriteStorage, Join
    },
};
use amethyst_tiles::TileMap;

use std::collections::HashMap;
use std::any::Any;

#[derive(Eq, PartialEq, Hash, Clone)]
pub enum AIWSKey {
    PointSelf,
    PointStockpile,
    PointVeinAmethyst,
    EntitySelf,
    EntityStockpile,
    EntityVeinAmethyst,
    EntityItemAmethyst,
    CanReachStockpile,
    CanReachVeinAmethyst,
    CanMineAmethyst,
    CanHoldAmethyst,
    HasFeet,
    HasHands,
    TaskDuration,
    AmountMinedAmethyst,
}

pub enum AITaskType {
    Compound,
    Primitive,
}

pub trait AITask {
    fn get_name(&self) -> &String;
    fn get_type(&self) -> AITaskType;

    fn get_pre(&self) -> Option<&HashMap<AIWSKey, Box<dyn Any>>> {
        None
    }
    fn get_eff(&self) -> Option<&HashMap<AIWSKey, Box<dyn Any>>> {
        None
    }
    fn get_term(&self, _: &u8) -> Option<&Vec<AIWSKey>> {
        None
    }

    fn get_oper(&self) -> Option<&Vec<u8>> {
        None
    }
    fn get_subtask(&self, _: &HashMap<AIWSKey, Box<dyn Any>>) -> Option<Vec<u8>> {
        None
    }
}

pub struct AITaskComp {
    pub name: String,

    pub methods: Vec<(HashMap<AIWSKey, Box<dyn Any>>, Vec<u8>)>,
}

impl AITask for AITaskComp {
    fn get_name(&self) -> &String {
        &self.name
    }
    fn get_type(&self) -> AITaskType {
        AITaskType::Compound
    }

    fn get_subtask(&self, memory: &HashMap<AIWSKey, Box<dyn Any>>) -> Option<Vec<u8>> {
        'outer: for (pre, task) in self.methods.iter() {
            for (key, val) in pre.iter() {
                if let Some(valval) = memory.get(key) {
                    if val.is::<bool>() {
                        if val.downcast_ref::<bool>().unwrap() != valval.downcast_ref::<bool>().unwrap() {
                            continue 'outer;
                        }
                    } else if val.is::<i32>() {
                        if val.downcast_ref::<i32>().unwrap() >= &(val.downcast_ref::<i32>().unwrap().signum() * valval.downcast_ref::<i32>().unwrap()) {
                            continue 'outer;
                        }
                    } else if val.is::<Entity>() {
                        if val.downcast_ref::<Entity>().unwrap() != valval.downcast_ref::<Entity>().unwrap() {
                            continue 'outer;
                        }
                    } else if val.is::<Point3<u32>>() {
                        if val.downcast_ref::<Point3<u32>>().unwrap() != valval.downcast_ref::<Point3<u32>>().unwrap() {
                            continue 'outer;
                        }
                    } else {
                        panic!("memory mapped value of method's condition has type other than bool, i32, Entity, and Point3<u32>");
                    }
                } else {
                    continue 'outer;
                }
            }

            return Some(task.clone())
        }

        return None
    }
}

pub struct AITaskPrim {
    pub name: String,

    pub pre: HashMap<AIWSKey, Box<dyn Any>>,
    pub eff: HashMap<AIWSKey, Box<dyn Any>>,
    pub term: HashMap<u8, Vec<AIWSKey>>,
    pub oper: Vec<u8>,
}

impl AITask for AITaskPrim {
    fn get_name(&self) -> &String {
        &self.name
    }
    fn get_type(&self) -> AITaskType {
        AITaskType::Primitive
    }

    fn get_pre(&self) -> Option<&HashMap<AIWSKey, Box<dyn Any>>> {
        Some(&self.pre)
    }
    fn get_eff(&self) -> Option<&HashMap<AIWSKey, Box<dyn Any>>> {
        Some(&self.eff)
    }
    fn get_term(&self, oper: &u8) -> Option<&Vec<AIWSKey>> {
        Some(self.term.get(oper).unwrap())
    }
    fn get_oper(&self) -> Option<&Vec<u8>> {
        Some(&self.oper)
    }
}

type AIData<'a> = (
    Entities<'a>,
    Read<'a, Vec<AssetExtractableData>>,
    Read<'a, Vec<AssetItemData>>,
    WriteStorage<'a, TileMap<MiscTile>>,
    WriteStorage<'a, Transform>,
    WriteStorage<'a, ComponentExtractable>,
    WriteStorage<'a, ComponentStockpile>,
    WriteStorage<'a, ComponentMovement>,
);

pub enum AIOperStatus {
    Done,
    Running,
    Failed,
}

pub trait AIOper<'a> {
    fn get_name(&self) -> &String;

    fn init(&self, _: &Vec<AIWSKey>, _: &mut HashMap<AIWSKey, Box<dyn Any>>, _: AIData<'a>) -> AIOperStatus {
        AIOperStatus::Done
    }
    fn run(&self, _: &Vec<AIWSKey>, _: &mut HashMap<AIWSKey, Box<dyn Any>>, _: AIData<'a>) -> AIOperStatus {
        AIOperStatus::Done
    }
}

pub struct AIOperWalkTo { name: String }
pub struct AIOperMoveItem { name: String }
pub struct AIOperExtract { name: String }

impl<'a> AIOper<'a> for AIOperWalkTo {
    fn get_name(&self) -> &String {
        &self.name
    }

    fn init(
        &self, 
        term: &Vec<AIWSKey>,
        memory: &mut HashMap<AIWSKey, Box<dyn Any>>, 
        (_, _, _, tilemaps, _, _, _, mut movements): AIData
    ) -> AIOperStatus {
        let target = memory.get(&term[0]).unwrap().downcast_ref::<Point3<u32>>().unwrap();
        let start = memory.get(&AIWSKey::PointSelf).unwrap().downcast_ref::<Point3<u32>>().unwrap();

        let tilemap = (&tilemaps).join().next().unwrap();

        let targets = get_targets(start, target, tilemap);

        if targets.len() > 0 {
            let entity = memory.get(&AIWSKey::EntitySelf).unwrap().downcast_ref::<Entity>().unwrap();

            movements.get_mut(*entity).unwrap().targets = targets;

            return AIOperStatus::Done;
        }
            
        return AIOperStatus::Failed;
    }
}

impl<'a> AIOper<'a> for AIOperMoveItem {
    fn get_name(&self) -> &String {
        &self.name
    }

    fn run(
        &self, 
        term: &Vec<AIWSKey>,
        memory: &mut HashMap<AIWSKey, Box<dyn Any>>, 
        (_, _, _, _, _, _, mut stockpiles, _): AIData
    ) -> AIOperStatus {
        let to = memory.get(&term[0]).unwrap().downcast_ref::<Entity>().unwrap();
        let from = memory.get(&AIWSKey::EntitySelf).unwrap().downcast_ref::<Entity>().unwrap();
        let index = memory.get(&term[1]).unwrap().downcast_ref::<i32>().unwrap();
        let mut amount = memory.get(&term[2]).unwrap().downcast_ref::<i32>().unwrap().clone();

        let to_stockpile = stockpiles.get_mut(*to).unwrap();

        if amount > (to_stockpile.size_limit - to_stockpile.size) as i32 {
            amount = (to_stockpile.size_limit - to_stockpile.size) as i32;
        }

        to_stockpile.items[*index as usize] += amount as u16;
        to_stockpile.size += amount as u32;

        let from_stockpile = stockpiles.get_mut(*from).unwrap();

        from_stockpile.items[*index as usize] -= amount as u16;
        from_stockpile.size -= amount as u32;

        memory.insert(term[2].clone(), Box::<i32>::new(0));

        return AIOperStatus::Done;
    }
}

impl<'a> AIOper<'a> for AIOperExtract {
    fn get_name(&self) -> &String {
        &self.name
    }

    fn init(
        &self, 
        term: &Vec<AIWSKey>,
        memory: &mut HashMap<AIWSKey, Box<dyn Any>>, 
        (_, extractable_datas, _, _, _, extractables, _, _): AIData
    ) -> AIOperStatus {
        let target = memory.get(&term[0]).unwrap().downcast_ref::<Entity>().unwrap();

        let extractable = extractables.get(*target).unwrap();
        let duration = extractable_datas[extractable.variant as usize].duration;

        memory.insert(AIWSKey::TaskDuration, Box::<i32>::new(duration as i32));

        return AIOperStatus::Done;
    }
    fn run(
        &self, 
        term: &Vec<AIWSKey>,
        memory: &mut HashMap<AIWSKey, Box<dyn Any>>, 
        (_, extractable_datas, _, _, _, mut extractables, mut stockpiles, _): AIData
    ) -> AIOperStatus {
        let duration = memory.get_mut(&AIWSKey::TaskDuration).unwrap().downcast_mut::<i32>().unwrap();

        if *duration > 1 {
            *duration -= 1;

            return AIOperStatus::Running;
        } 

        let target = memory.get(&term[0]).unwrap().downcast_ref::<Entity>().unwrap();
        let stockpile = memory.get(&AIWSKey::EntitySelf).unwrap().downcast_ref::<Entity>().unwrap();

        let extractable = extractables.get_mut(*target).unwrap();
        let stockpile = stockpiles.get_mut(*stockpile).unwrap();
    
        extractable.deposit -= 1;

        for (i, out) in extractable_datas[extractable.variant as usize].outs.iter().enumerate() {
            if *out as u32 + stockpile.size > stockpile.size_limit {
                stockpile.items[i] += (stockpile.size_limit - stockpile.size) as u16;
                stockpile.size = stockpile.size_limit;

                break;
            } else {
                stockpile.items[i] += *out as u16;
                stockpile.size += *out as u32;
            }
        }

        return AIOperStatus::Done;
    }
}