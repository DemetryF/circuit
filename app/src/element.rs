mod current_source;
mod resistor;
mod wire;

use std::borrow::{Borrow, BorrowMut};
use std::marker::PhantomData;

use egui::{Color32, Pos2, Rect, Rounding, Vec2};

use circuit::Conductor;

use crate::utils::Painter;

pub use current_source::render_current_source;
pub use resistor::render_resistor;
pub use wire::render_wire;

const CHARGE_VALUE: f32 = 1.0;
const CHARGE_DISTANCE: f32 = 20.0;
const CHARGE_SIZE: f32 = 3.0;

pub const HIGHLIGHTED_COLOR: Color32 = Color32::from_rgb(67, 197, 240);

pub struct Element<'data> {
    pub conductor: Box<dyn ElementTrait>,

    shift: f32,

    lt: PhantomData<&'data ()>,
}

const SENSABLE_DIST: f32 = 10.0;

impl<'data> Element<'data> {
    pub fn new(conductor: Box<dyn ElementTrait>) -> Self {
        Self {
            conductor,
            shift: 0.0,
            lt: PhantomData,
        }
    }

    fn render_charges(&self, endpoints: [ElementPos; 2], painter: Painter<'_>) {
        let endpoints = endpoints.map(ElementPos::into_pos);

        let dist = endpoints[1] - endpoints[0];
        let length = dist.length();

        let charges_count = (length / CHARGE_DISTANCE).floor() as usize;

        let dir = dist.normalized();

        for n in 0..charges_count {
            let pos = endpoints[0] + (self.shift + n as f32 * CHARGE_DISTANCE) * dir;

            let rect = Rect::from_min_size(
                pos - Vec2::splat(CHARGE_SIZE / 2.0),
                Vec2::splat(CHARGE_SIZE),
            );

            painter.rect_filled(rect, Rounding::ZERO, Color32::YELLOW);
        }
    }

    pub fn includes(&self, endpoints: [ElementPos; 2], point: Pos2) -> bool {
        let endpoints = endpoints.map(ElementPos::into_pos);

        let min_y = f32::min(endpoints[0].y, endpoints[1].y);
        let max_y = f32::max(endpoints[0].y, endpoints[1].y);

        if endpoints[0].x == endpoints[1].x
            && (endpoints[0].x - point.x).abs() <= SENSABLE_DIST
            && min_y - SENSABLE_DIST < point.y
            && point.y < max_y + SENSABLE_DIST
        {
            return true;
        }

        let min_x = f32::min(endpoints[0].x, endpoints[1].x);
        let max_x = f32::max(endpoints[0].x, endpoints[1].x);

        if endpoints[0].y == endpoints[1].y
            && (endpoints[0].y - point.y).abs() <= SENSABLE_DIST
            && min_x - SENSABLE_DIST < point.x
            && point.x < max_x + SENSABLE_DIST
        {
            return true;
        }

        let k1 = (endpoints[0].y - endpoints[1].y) / (endpoints[0].x - endpoints[1].x);
        let k2 = -1.0 / k1;

        let intersection_x =
            (k1 * endpoints[0].x - endpoints[0].y + k2 * point.x + point.y) / (k1 + k2);

        let intersection_y = k1 * (intersection_x - endpoints[0].x) + endpoints[0].y;

        let angle = f32::atan2(
            endpoints[0].y - endpoints[1].y,
            endpoints[0].x - endpoints[1].x,
        );

        let (sin, cos) = angle.sin_cos();

        let protrusion_x = cos.abs() * SENSABLE_DIST;
        let protrusion_y = sin.abs() * SENSABLE_DIST;

        min_x - protrusion_x < intersection_x
            && intersection_x < max_x + protrusion_x
            && min_y - protrusion_y < intersection_y
            && intersection_y < max_y + protrusion_y
            && (intersection_x - point.x).hypot(intersection_y - point.y) <= SENSABLE_DIST
    }
}

pub trait ElementTrait: Conductor + Render {}
impl<T: Conductor + Render> ElementTrait for T {}

impl<'data> Borrow<dyn Conductor + 'data> for Element<'data> {
    fn borrow(&self) -> &(dyn Conductor + 'data) {
        self
    }
}

impl<'data> BorrowMut<dyn Conductor + 'data> for Element<'data> {
    fn borrow_mut(&mut self) -> &mut (dyn Conductor + 'data) {
        self
    }
}

impl<'data> Conductor for Element<'data> {
    fn zap(&mut self, amperage: f32, delta_time: f32) {
        self.conductor.zap(amperage, delta_time);

        let delta = amperage * CHARGE_DISTANCE / CHARGE_VALUE * delta_time;

        self.shift += delta;
        self.shift = self.shift.rem_euclid(CHARGE_DISTANCE);
    }

    fn emf(&self) -> f32 {
        self.conductor.emf()
    }

    fn resistance(&self) -> f32 {
        self.conductor.resistance()
    }
}

pub trait Render {
    fn render(&self, endpoints: [ElementPos; 2], painter: Painter);
    fn render_highlighted(&self, endpoints: [ElementPos; 2], painter: Painter);
}

impl<'data> Render for Element<'data> {
    fn render(&self, endpoints: [ElementPos; 2], painter: Painter) {
        self.conductor.render(endpoints, painter);

        self.render_charges(endpoints, painter);
    }

    fn render_highlighted(&self, endpoints: [ElementPos; 2], painter: Painter) {
        self.conductor.render_highlighted(endpoints, painter);

        self.render_charges(endpoints, painter);
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Default)]
pub struct ElementPos {
    pub x: isize,
    pub y: isize,
}

pub const CELL_SIZE: f32 = 20.0;

impl ElementPos {
    pub fn from_pos(pos: impl Into<Pos2>) -> Self {
        let pos = pos.into();

        Self {
            x: (pos.x / CELL_SIZE).round() as isize,
            y: (pos.y / CELL_SIZE).round() as isize,
        }
    }

    pub fn into_pos(self) -> Pos2 {
        Pos2::new((self.x as f32) * CELL_SIZE, (self.y as f32) * CELL_SIZE)
    }
}
