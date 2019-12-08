use amethyst::{
    core::math::{ Point3, Vector3 },
    ecs::{Component, DenseVecStorage,},
};

pub struct ComponentMovement {
    pub targets: Vec<Point3<u32>>,
    pub velocity: Vector3<f32>,
    pub speed_limit: f32,
    pub acceleration: f32,
}
impl Component for ComponentMovement {
    type Storage = DenseVecStorage<Self>;
}