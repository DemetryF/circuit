use std::collections::HashSet;

use egui::emath::TSTransform;
use egui::{CentralPanel, Frame, Key, Margin, Sense};
use egui::{Color32, Pos2, Shape, Vec2};

use circuit::circuit::ElementId;
use circuit::default_conductors::*;

use super::elements_panel::ElementType;
use super::{Adding, AppState, Context};
use crate::element::{render_current_source, render_resistor, render_wire};
use crate::element::{Element, ElementPos, ElementTrait, Render, CELL_SIZE};
use crate::utils::Painter;

#[derive(Default)]
pub struct Field {
    pub transform: TSTransform,

    pub moving: Option<Moving>,

    pub selected: HashSet<ElementId>,
    pub hovered: Option<Hovered>,
}

impl Field {
    pub fn show(&mut self, state: &mut AppState, ctx: Context) {
        self.panel().show(ctx.0, move |ui| {
            let (response, painter) =
                ui.allocate_painter(ui.available_size(), Sense::click_and_drag());

            let painter = Painter::new(&painter, self.transform);

            if response.clicked() {
                self.update_adding(state, ctx);
            }

            self.update_moving(state, ctx);
            self.update_selected(state, ctx);
            self.update_zoom(ctx);

            self.draw_grid(ctx, painter, ui.min_size());
            self.draw_elements(state, ctx, painter, &response);
            self.draw_adding(state, ctx, painter);
        });
    }

    fn panel(&self) -> CentralPanel {
        egui::CentralPanel::default().frame(Frame {
            inner_margin: Margin::ZERO,
            outer_margin: Margin::ZERO,
            fill: Color32::from_gray(27),
            ..Default::default()
        })
    }

    fn draw_grid(&self, ctx: Context, painter: Painter, size: Vec2) {
        let top_left = self.transform.inverse() * Pos2::ZERO;
        let top_left = ElementPos::from_pos(top_left);

        let right_bottom = self.transform.inverse() * size.to_pos2();
        let right_bottom = ElementPos::from_pos(right_bottom);

        for x in (top_left.x - 1)..=right_bottom.x {
            for y in (top_left.y - 1)..=right_bottom.y {
                let pos = ElementPos { x, y }.into_pos();

                painter.render(Shape::circle_filled(
                    pos,
                    CELL_SIZE / 10.0,
                    Color32::from_gray(40),
                ));
            }
        }

        if let Some(mouse_pos) = ctx.mouse_pos() {
            let mouse_pos = ElementPos::from_pos(self.transform.inverse() * mouse_pos);

            painter.render(Shape::circle_filled(
                mouse_pos.into_pos(),
                CELL_SIZE / 7.0,
                Color32::from_gray(60),
            ));
        }
    }

    fn draw_elements(
        &mut self,
        state: &AppState,
        ctx: Context,
        painter: Painter<'_>,
        response: &egui::Response,
    ) {
        self.hovered = None;

        for (id, element) in state.circuit.iter() {
            let endpoints = state.circuit.endpoints(id);

            let mut highlight = self.selected.contains(&id);

            if let Some(mouse_pos) = ctx.mouse_pos() {
                let grid_mouse_pos = self.transform.inverse() * mouse_pos;

                let is_hovered = element.includes(endpoints, grid_mouse_pos);

                if is_hovered && state.adding.get().is_none() {
                    highlight = true;

                    if self.hovered.is_none() {
                        self.hover(endpoints, grid_mouse_pos, id);
                    }

                    if response.clicked() {
                        self.select(ctx, id);
                    }
                }
            }

            if highlight {
                element.render_highlighted(endpoints, painter);
            } else {
                element.render(endpoints, painter);
            }
        }
    }

    fn hover(&mut self, endpoints: [ElementPos; 2], mouse_pos: Pos2, id: ElementId) {
        let endpoints = endpoints.map(|point| self.transform * point.into_pos());

        let endpoint_idx = endpoints
            .into_iter()
            .enumerate()
            .find(|&(_, pos)| (mouse_pos - pos).length() < 5.0)
            .map(|(idx, _)| idx);

        if let Some(endpoint_idx) = endpoint_idx {
            self.hovered = Some(Hovered::Endpoint {
                element: id,
                endpoint_idx,
            });
        } else {
            self.hovered = Some(Hovered::Element(id))
        }
    }

    fn select(&mut self, ctx: Context, id: ElementId) {
        let pressed_shift = ctx.0.input(|state| state.modifiers.shift);

        if pressed_shift {
            if self.selected.contains(&id) {
                self.selected.remove(&id);
            } else {
                self.selected.insert(id);
            }
        } else {
            self.selected.clear();
            self.selected.insert(id);
        }
    }

    fn draw_adding(&mut self, state: &AppState, ctx: Context, painter: Painter) {
        if let Some(mouse_pos) = ctx.mouse_pos() {
            if let Some(Adding {
                ty: element,
                first: Some(first),
            }) = state.adding.get()
            {
                let second = ElementPos::from_pos(self.transform.inverse() * mouse_pos);
                let endpoints = [first, second];

                match element {
                    ElementType::CurrentSource => {
                        render_current_source(endpoints, painter, Color32::DARK_GRAY);
                    }
                    ElementType::Wire => {
                        render_wire(endpoints, painter, Color32::DARK_GRAY);
                    }
                    ElementType::Resistor => {
                        render_resistor(endpoints, painter, Color32::DARK_GRAY);
                    }
                }
            }
        }
    }

    fn update_zoom(&mut self, ctx: Context) {
        if let Some(real_mouse_pos) = ctx.0.input(|state| state.pointer.hover_pos()) {
            let delta_scale = ctx.0.input(|state| state.zoom_delta());

            if delta_scale != 1.0 {
                let new_real_mouse_pos =
                    (real_mouse_pos - self.transform.translation) / delta_scale;

                let delta = new_real_mouse_pos - real_mouse_pos + self.transform.translation;

                self.transform.translation += delta;
                self.transform.scaling *= delta_scale;
            }
        }
    }

    fn update_moving(&mut self, state: &mut AppState, ctx: Context) {
        let (pressed, released, mouse_pos) = ctx.0.input(|state| {
            (
                state.pointer.primary_pressed(),
                state.pointer.primary_released(),
                state.pointer.hover_pos(),
            )
        });

        if pressed {
            let origin = mouse_pos.unwrap().to_vec2();

            let object = match self.hovered {
                Some(Hovered::Endpoint {
                    element,
                    endpoint_idx,
                }) => MovingObject::ElementEndpoint {
                    element,
                    endpoint_idx,
                    old_value: state.circuit.endpoints(element)[endpoint_idx],
                },

                Some(Hovered::Element(id)) => MovingObject::Element {
                    id,
                    old_endpoints: state.circuit.endpoints(id),
                },

                None => MovingObject::View {
                    old_translation: self.transform.translation,
                },
            };

            self.moving = Some(Moving { object, origin });
        } else if released {
            self.moving = None;
        }

        if let Some(moving) = self.moving {
            let delta = mouse_pos.unwrap().to_vec2() - moving.origin;

            match moving.object {
                MovingObject::View { old_translation } => {
                    self.transform.translation = old_translation + delta;
                }
                MovingObject::Element { id, old_endpoints } => {
                    let new_endpoints = old_endpoints
                        .map(|point| self.transform * point.into_pos())
                        .map(|point| point + delta)
                        .map(|point| self.transform.inverse() * point)
                        .map(ElementPos::from_pos);

                    state.circuit.change(id, new_endpoints);
                }
                MovingObject::ElementEndpoint {
                    element: id,
                    endpoint_idx,
                    old_value,
                } => {
                    let new_point =
                        self.transform.inverse() * (self.transform * old_value.into_pos() + delta);

                    let new_point = ElementPos::from_pos(new_point);

                    let mut endpoints = state.circuit.endpoints(id);
                    endpoints[endpoint_idx] = new_point;

                    state.circuit.change(id, endpoints)
                }
            }
        }
    }

    fn update_selected(&mut self, state: &mut AppState, ctx: Context) {
        let pressed_delete = ctx.0.input(|state| state.keys_down.contains(&Key::Delete));

        if pressed_delete {
            for &idx in self.selected.iter() {
                state.circuit.remove(idx)
            }

            self.selected.clear();
        }

        let pressed_esc = ctx.0.input(|state| state.keys_down.contains(&Key::Escape));

        if pressed_esc {
            self.selected.clear();
        }
    }

    fn update_adding(&mut self, state: &mut AppState, ctx: Context) {
        let pos = ctx.mouse_pos().unwrap();
        let pos = self.transform.inverse() * pos;

        let pos = ElementPos::from_pos(pos);

        if let Some(mut adding) = state.adding.get() {
            if let Some(first) = adding.first {
                let second = pos;

                let endpoints = [first, second];
                let conductor = self.create_element(adding);

                state.circuit.add(endpoints, Element::new(conductor));

                state.adding.set(None);
            } else {
                adding.first = Some(pos);

                state.adding.set(Some(adding));
            }
        }
    }

    fn create_element(&self, adding: Adding) -> Box<dyn ElementTrait> {
        match adding.ty {
            ElementType::CurrentSource => {
                let current_source = Box::new(CurrentSource {
                    resistance: 0.0,
                    emf: 10.0,
                });

                current_source as Box<dyn ElementTrait>
            }
            ElementType::Wire => {
                let wire = Box::new(Wire);

                wire as Box<dyn ElementTrait>
            }
            ElementType::Resistor => {
                let resistor = Box::new(Resistor { resistance: 5.0 });

                resistor as Box<dyn ElementTrait>
            }
        }
    }
}

#[derive(Clone, Copy)]
pub enum Hovered {
    Element(ElementId),
    Endpoint {
        element: ElementId,
        endpoint_idx: usize,
    },
}

#[derive(Clone, Copy)]
pub struct Moving {
    pub object: MovingObject,
    pub origin: Vec2,
}

#[derive(Clone, Copy)]
pub enum MovingObject {
    View {
        old_translation: Vec2,
    },
    Element {
        id: ElementId,
        old_endpoints: [ElementPos; 2],
    },
    ElementEndpoint {
        element: ElementId,
        endpoint_idx: usize,
        old_value: ElementPos,
    },
}
