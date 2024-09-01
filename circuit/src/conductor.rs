pub trait Conductor {
    fn zap(&mut self, _amperage: f32, _delta_time: f32) {}

    fn emf(&self) -> f32;
    fn resistance(&self) -> f32;
}
