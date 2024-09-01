use egui::emath::TSTransform;
use egui::epaint::{PathStroke, RectShape};
use egui::{Color32, Pos2, Rect, Rounding, Shape, Stroke};

#[derive(Clone, Copy)]
pub struct Painter<'ui> {
    painter: &'ui egui::Painter,

    transform: TSTransform,
}

impl<'ui> Painter<'ui> {
    pub fn new(painter: &'ui egui::Painter, transform: TSTransform) -> Self {
        Self { painter, transform }
    }

    #[inline(always)]
    pub fn line(&self, points: [Pos2; 2], stroke: impl Into<PathStroke>) {
        let line = Shape::line_segment(points, stroke);

        self.render(line);
    }

    #[inline(always)]
    pub fn rect_filled(&self, rect: Rect, rounding: Rounding, fill_color: Color32) {
        let rect = RectShape::new(rect, rounding, fill_color, Stroke::NONE);

        self.render(rect);
    }

    #[inline(always)]
    pub fn render(&self, shape: impl Into<Shape>) {
        let mut shape = shape.into();

        shape.transform(self.transform);

        self.painter.add(shape);
    }
}
