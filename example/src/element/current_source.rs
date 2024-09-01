use egui::{Color32, Pos2, Stroke};

use circuit::default_conductors::CurrentSource;

use crate::utils::Painter;

use super::{ElementPos, Render, HIGHLIGHTED_COLOR};

const CURRENT_SOURCE_DISTANCE: f32 = 5.0;
const CURRENT_SOURCE_SIZE: f32 = 10.0;

impl Render for CurrentSource {
    fn render(&self, endpoints: [ElementPos; 2], painter: Painter) {
        render_current_source(endpoints, painter, Color32::GRAY);
    }

    fn render_highlighted(&self, endpoints: [ElementPos; 2], painter: Painter) {
        render_current_source(endpoints, painter, HIGHLIGHTED_COLOR);
    }
}

pub fn render_current_source(endpoints: [ElementPos; 2], painter: Painter<'_>, color: Color32) {
    let endpoints = endpoints.map(ElementPos::into_vec);

    let stroke = Stroke::new(2.0, color);

    let endpoints = endpoints.map(|pos| Pos2::new(pos.x as f32, pos.y as f32));

    let l = endpoints[1] - endpoints[0];
    let length = l.length();

    let d = 0.5 * (length - CURRENT_SOURCE_DISTANCE) * l.normalized();

    painter.line([endpoints[0], endpoints[0] + d], stroke);
    painter.line([endpoints[1], endpoints[1] - d], stroke);

    let f1 = 0.25 * CURRENT_SOURCE_SIZE * l.normalized().rot90();
    let f2 = 0.5 * CURRENT_SOURCE_SIZE * l.normalized().rot90();

    painter.line([endpoints[0] + d - f1, endpoints[0] + d + f1], stroke);
    painter.line([endpoints[1] - d - f2, endpoints[1] - d + f2], stroke);
}
