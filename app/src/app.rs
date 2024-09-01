use std::cell::Cell;
use std::collections::HashSet;
use std::marker::PhantomData;

use egui::emath::TSTransform;
use egui::{Button, Frame, Id, Key, Margin, Sense, Shape};
use egui::{Color32, Pos2, Vec2};

use circuit::circuit::ElementId;
use circuit::default_conductors::*;
use circuit::Circuit;

use crate::element::{render_current_source, render_resistor, render_wire};
use crate::element::{Element, ElementPos, ElementTrait, ElementType, Render, CELL_SIZE};
use crate::utils::{Painter, WidgetsGallery};

#[derive(Default)]
pub struct App<'data> {
    circuit: Circuit<'data, Element<'data>, ElementPos>,

    adding: Cell<Option<Adding>>,
    moving: Option<Moving>,

    transform: TSTransform,

    selected: HashSet<ElementId>,
    hovered: Option<(ElementId, Option<usize>)>,

    lt: PhantomData<&'data ()>,
}

impl<'data> eframe::App for App<'data> {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();

        let delta_time = ctx.input(|state| state.stable_dt);
        let mouse_pos = ctx.input(|state| state.pointer.hover_pos());

        self.circuit.update(delta_time);

        self.update_moving(ctx);
        self.update_zoom(ctx);
        self.update_selected(ctx);

        let central_panel = egui::CentralPanel::default().frame(Frame {
            inner_margin: Margin::ZERO,
            outer_margin: Margin::ZERO,
            fill: Color32::from_gray(27),
            ..Default::default()
        });

        central_panel.show(ctx, |ui| {
            let top_left = self.transform.inverse().mul_pos(Pos2::ZERO);
            let top_left = ElementPos::from_pos(top_left);

            let right_bottom = self
                .transform
                .inverse()
                .mul_pos(ui.available_size().to_pos2());
            let right_bottom = ElementPos::from_pos(right_bottom);

            let (response, painter) =
                ui.allocate_painter(ui.available_size(), Sense::click_and_drag());

            let painter = Painter::new(&painter, self.transform);

            for x in (top_left.x - 1)..=right_bottom.x {
                for y in (top_left.y - 1)..=right_bottom.y {
                    let pos = ElementPos { x, y };
                    let pos = pos.into_pos();

                    painter.render(Shape::circle_filled(
                        pos,
                        CELL_SIZE / 10.0,
                        Color32::from_gray(40),
                    ));
                }
            }

            if let Some(mouse_pos) = self.mouse_ivec(ctx) {
                painter.render(Shape::circle_filled(
                    mouse_pos.into_pos(),
                    CELL_SIZE / 7.0,
                    Color32::from_gray(60),
                ));

                if let Some(Adding {
                    element,
                    first: Some(first),
                }) = self.adding.get()
                {
                    let endpoints = [first, mouse_pos];

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

            self.hovered = None;

            for (idx, element) in self.circuit.iter() {
                let endpoints = self.circuit.endpoints(idx);

                let mut highlighted = self.selected.contains(&idx);

                if let Some(mouse_pos) = mouse_pos {
                    if element.includes(endpoints, self.transform.inverse() * mouse_pos)
                        && self.adding.get().is_none()
                    {
                        highlighted = true;

                        let endpoints = endpoints.map(|point| self.transform * point.into_pos());

                        let endpoint = endpoints
                            .into_iter()
                            .enumerate()
                            .find(|&(_, pos)| (mouse_pos - pos).length() < 5.0)
                            .map(|(idx, _)| idx);

                        self.hovered = self.hovered.or(Some((idx, endpoint)));

                        let pressed_shift = ctx.input(|state| state.modifiers.shift);

                        if response.clicked() && pressed_shift {
                            if self.selected.contains(&idx) {
                                self.selected.remove(&idx);
                            } else {
                                self.selected.insert(idx);
                            }
                        } else if response.clicked() {
                            self.selected.clear();
                            self.selected.insert(idx);
                        }
                    }
                }

                if highlighted {
                    element.render_highlighted(endpoints, painter);
                } else {
                    element.render(endpoints, painter);
                }
            }

            if response.clicked() {
                let pos = response.interact_pointer_pos().unwrap();
                let pos = self.transform.inverse().mul_pos(pos);

                let pos = ElementPos::from_pos(pos);

                if let Some(adding) = self.adding.get_mut() {
                    if let Some(first) = adding.first {
                        let second = pos;

                        let endpoints = [first, second];
                        let &mut adding = adding;
                        let conductor = self.create_element(adding);

                        self.circuit.add(endpoints, Element::new(conductor));

                        self.adding.set(None);
                    } else {
                        adding.first = Some(pos);
                    }
                }
            }
        });

        self.elements_panel(ctx);
    }
}

impl<'data> App<'data> {
    fn update_moving(&mut self, ctx: &egui::Context) {
        let (pressed, released, mouse_pos) = ctx.input(|state| {
            (
                state.pointer.primary_pressed(),
                state.pointer.primary_released(),
                state.pointer.hover_pos(),
            )
        });

        if pressed {
            self.moving = Some({
                let origin = mouse_pos.unwrap().to_vec2();

                if let Some((id, endpoint)) = self.hovered {
                    let endpoints = self.circuit.endpoints(id);

                    if let Some(endpoint_idx) = endpoint {
                        Moving {
                            object: MovingObject::ElementEndpoint {
                                id,
                                endpoint_idx,
                                old_value: endpoints[endpoint_idx],
                            },
                            origin,
                        }
                    } else {
                        Moving {
                            object: MovingObject::Element {
                                id,
                                old_endpoints: endpoints,
                            },
                            origin,
                        }
                    }
                } else {
                    Moving {
                        object: MovingObject::View {
                            old_translation: self.transform.translation,
                        },
                        origin,
                    }
                }
            });
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
                        .map(|point| ElementPos::from_pos(point));

                    if old_endpoints != new_endpoints {
                        self.circuit.change(id, new_endpoints);
                    }
                }
                MovingObject::ElementEndpoint {
                    id,
                    endpoint_idx,
                    old_value,
                } => {
                    let new_point =
                        self.transform.inverse() * (self.transform * old_value.into_pos() + delta);

                    let new_point = ElementPos::from_pos(new_point);

                    let mut endpoints = self.circuit.endpoints(id);
                    endpoints[endpoint_idx] = new_point;

                    self.circuit.change(id, endpoints)
                }
            }
        }
    }

    fn update_zoom(&mut self, ctx: &egui::Context) {
        if let Some(real_mouse_pos) = ctx.input(|state| state.pointer.hover_pos()) {
            let delta_scale = ctx.input(|state| state.zoom_delta());

            if delta_scale != 1.0 {
                let new_real_mouse_pos =
                    (real_mouse_pos - self.transform.translation) / delta_scale;

                let delta = new_real_mouse_pos - real_mouse_pos + self.transform.translation;

                self.transform.translation += delta;
                self.transform.scaling *= delta_scale;
            }
        }
    }

    fn update_selected(&mut self, ctx: &egui::Context) {
        let pressed_delete = ctx.input(|state| state.keys_down.contains(&Key::Delete));

        if pressed_delete {
            for &idx in self.selected.iter() {
                self.circuit.remove(idx)
            }

            self.selected.clear();
        }

        let pressed_esc = ctx.input(|state| state.keys_down.contains(&Key::Escape));

        if pressed_esc {
            self.selected.clear();
        }
    }

    fn elements_panel(&mut self, ctx: &egui::Context) {
        let screen_rect = ctx.screen_rect();

        let max_panel_size = (screen_rect.height() - screen_rect.width()).abs();

        let min_coord = f32::min(screen_rect.width(), screen_rect.height());

        let max_element_size = Vec2::splat(min_coord / 8.0);
        let min_element_size = Vec2::splat(min_coord / 12.0);

        if ctx.screen_rect().width() > ctx.screen_rect().height() {
            egui::SidePanel::right(Id::new("id"))
                .resizable(true)
                .max_width(max_panel_size)
                .show(ctx, |ui| {
                    self.element_buttons(ui, max_element_size, min_element_size)
                });
        } else {
            egui::TopBottomPanel::bottom(Id::new("id"))
                .resizable(true)
                .max_height(max_panel_size)
                .show(ctx, |ui| {
                    self.element_buttons(ui, max_element_size, min_element_size)
                });
        }
    }

    fn element_buttons(&mut self, ui: &mut egui::Ui, max_size: Vec2, min_size: Vec2) {
        let vec = vec![
            (Button::new("wire").min_size(min_size), ElementType::Wire),
            (
                Button::new("resistor").min_size(min_size),
                ElementType::Resistor,
            ),
            (
                Button::new("current source").min_size(min_size),
                ElementType::CurrentSource,
            ),
        ];

        let adding = &self.adding;

        let widgets = vec.into_iter().map(|(button, element)| {
            let react = Box::new(move |response: egui::Response| {
                if response.clicked() {
                    adding.set(Some(Adding {
                        element,
                        first: None,
                    }))
                }
            });

            (button, react as Box<dyn FnOnce(egui::Response)>)
        });

        WidgetsGallery { max_size, widgets }.show(ui);
    }

    fn create_element(&self, adding: Adding) -> Box<dyn ElementTrait> {
        match adding.element {
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

    fn mouse_ivec(&self, ctx: &egui::Context) -> Option<ElementPos> {
        ctx.input(|state| state.pointer.hover_pos())
            .map(|pos| self.transform.inverse() * pos)
            .map(|pos| ElementPos::from_pos(pos))
    }
}

#[derive(Clone, Copy)]
pub struct Adding {
    pub element: ElementType,
    pub first: Option<ElementPos>,
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
        id: ElementId,
        endpoint_idx: usize,
        old_value: ElementPos,
    },
}
