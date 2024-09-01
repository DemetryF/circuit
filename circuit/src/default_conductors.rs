use crate::Conductor;

pub struct Wire;

impl Conductor for Wire {
    fn emf(&self) -> f32 {
        0.0
    }

    fn resistance(&self) -> f32 {
        0.0
    }
}

pub struct Resistor {
    pub resistance: f32,
}

impl Conductor for Resistor {
    fn emf(&self) -> f32 {
        0.0
    }

    fn resistance(&self) -> f32 {
        self.resistance
    }
}

pub struct CurrentSource {
    pub resistance: f32,
    pub emf: f32,
}

impl Conductor for CurrentSource {
    fn emf(&self) -> f32 {
        self.emf
    }

    fn resistance(&self) -> f32 {
        self.resistance
    }
}
