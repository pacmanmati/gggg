pub trait Material {}

impl Material for Box<dyn Material> {}

// #[derive(Material)]
pub struct BasicMaterial {}

impl Material for BasicMaterial {}
