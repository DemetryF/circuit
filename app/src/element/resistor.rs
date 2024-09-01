use egui::epaint::PathShape;
use egui::{Color32, Stroke};

use circuit::default_conductors::Resistor;

use crate::utils::Painter;

use super::{ElementPos, Render, HIGHLIGHTED_COLOR};

const RESISTOR_WIDTH: f32 = 20.0;
const RESISTOR_HEIGHT: f32 = 5.0;

impl Render for Resistor {
    fn render(&self, endpoints: [ElementPos; 2], painter: Painter) {
        render_resistor(endpoints, painter, Color32::GRAY);
    }

    fn render_highlighted(&self, endpoints: [ElementPos; 2], painter: Painter) {
        render_resistor(endpoints, painter, HIGHLIGHTED_COLOR);
    }
}

pub fn render_resistor(endpoints: [ElementPos; 2], painter: Painter<'_>, color: Color32) {
    let endpoints = endpoints.map(ElementPos::into_pos);

    let stroke = Stroke::new(2.0, color);

    let l = endpoints[1] - endpoints[0];
    let length = l.length();

    let d = 0.5 * (length - RESISTOR_WIDTH) * l.normalized();

    painter.line([endpoints[0], endpoints[0] + d], stroke);
    painter.line([endpoints[1], endpoints[1] - d], stroke);

    let f = 0.5 * RESISTOR_HEIGHT * l.normalized().rot90();

    painter.render(PathShape {
        points: vec![
            endpoints[0] + d + f,
            endpoints[0] + d - f,
            endpoints[1] - d - f,
            endpoints[1] - d + f,
        ],
        closed: true,
        fill: color,
        stroke: stroke.into(),
    });
}
