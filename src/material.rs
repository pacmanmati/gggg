use std::fmt::Debug;

pub trait Material: Debug {}

impl Material for Box<dyn Material> {}

// #[derive(Material)]
#[derive(Debug)]
pub struct BasicMaterial {}

impl Material for BasicMaterial {}
