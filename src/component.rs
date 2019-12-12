use crate::misc::*;
use crate::ai::AIWSKey;

use amethyst::{
    core::math::{ Point3, Vector3 },
    ecs::{Component, DenseVecStorage, },
};

use std::collections::HashMap;
use std::any::Any;

pub struct ComponentExtractable {
    pub variant: u8,
    pub deposit: u16,
}
impl Component for ComponentExtractable {
    type Storage = DenseVecStorage<Self>;
}

pub struct ComponentStockpile {
    pub items: [u16; 1],
    pub size: u32,
    pub size_limit: u32,
}
impl Component for ComponentStockpile {
    type Storage = DenseVecStorage<Self>;
}

pub struct ComponentAgent {
    pub goals: [u8; 8],
    pub actions: [u8; 24],
    pub plan: [u8; 31],
    pub plan_length: u8,
}
impl Component for ComponentAgent {
    type Storage = DenseVecStorage<Self>;
}

pub struct ComponentMemory {
    pub ws: HashMap<AIWSKey, Box<dyn Any>>,
}
unsafe impl Send for ComponentMemory {}
unsafe impl Sync for ComponentMemory {}
impl Component for ComponentMemory {
    type Storage = DenseVecStorage<Self>;
}

pub struct ComponentMovement {
    pub targets: Vec<Point3<u32>>,
    pub velocity: Vector3<f32>,
    pub speed_limit: f32,
    pub acceleration: f32,
}
impl Component for ComponentMovement {
    type Storage = DenseVecStorage<Self>;
}