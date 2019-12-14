use crate::misc::*;

use amethyst::{
    core::math::{ Point3, Vector3 },
    ecs::{Component, DenseVecStorage, Entity },
};

pub struct ComponentWorkplace {
    pub variant: u8,
}
impl Component for ComponentWorkplace {
    type Storage = DenseVecStorage<Self>;
}

pub struct ComponentStockpile {
    pub items: [u16; 3],
}
impl Component for ComponentStockpile {
    type Storage = DenseVecStorage<Self>;
}

pub struct ComponentAgent {
    pub actions: [u8; 23],
    pub current: u8,
    pub target: Option<Entity>,
    pub fresh: bool,
}
impl Component for ComponentAgent {
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