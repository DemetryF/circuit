use egui::{epaint::PathStroke, Color32};

use circuit::default_conductors::Wire;

use crate::utils::Painter;

use super::{ElementPos, Render, HIGHLIGHTED_COLOR};

impl Render for Wire {
    fn render(&self, endpoints: [ElementPos; 2], painter: Painter) {
        render_wire(endpoints, painter, Color32::GRAY);
    }

    fn render_highlighted(&self, endpoints: [ElementPos; 2], painter: Painter) {
        render_wire(endpoints, painter, HIGHLIGHTED_COLOR);
    }
}

pub fn render_wire(endpoints: [ElementPos; 2], painter: Painter<'_>, color: Color32) {
    let endpoints = endpoints.map(ElementPos::into_pos);

    painter.line(endpoints, PathStroke::new(2.0, color));
}
