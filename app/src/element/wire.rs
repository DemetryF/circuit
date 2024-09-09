use egui::{epaint::PathStroke, Color32};

use circuit::default_conductors::Wire;
use smallvec::{smallvec, SmallVec};

use super::{ElementPos, Properties, Render, HIGHLIGHTED_COLOR};
use crate::utils::Painter;

impl Render for Wire {
    fn render(&self, endpoints: [ElementPos; 2], painter: Painter) {
        render_wire(endpoints, painter, Color32::GRAY);
    }

    fn render_highlighted(&self, endpoints: [ElementPos; 2], painter: Painter) {
        render_wire(endpoints, painter, HIGHLIGHTED_COLOR);
    }
}

pub fn render_wire(endpoints: [ElementPos; 2], painter: Painter<'_>, color: Color32) {
    let endpoints = endpoints.map(ElementPos::to_pos);

    painter.line(endpoints, PathStroke::new(2.0, color));
}

impl Properties for Wire {
    fn properties(&self) -> &'static [&'static str] {
        &[]
    }

    fn properties_mut(&mut self) -> SmallVec<[&mut f32; 2]> {
        smallvec![]
    }
}
