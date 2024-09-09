use std::collections::HashMap;

use egui::epaint::RectShape;
use egui::{Color32, Pos2, Rect, Rounding, Stroke, Vec2};

use circuit::circuit::ElementId;
use circuit::default_conductors::*;

use super::elements_panel::ElementType;
use super::{AppState, Context};
use crate::element::render::*;
use crate::element::{Element, ElementPos, ElementTrait, HIGHLIGHTED_COLOR};
use crate::utils::Painter;

#[derive(Default, PartialEq, Eq)]
pub enum Action {
    Moving {
        origin: Pos2,
        delta: Vec2,
        obj: MovingObject,
    },

    Adding {
        ty: ElementType,
        first: Option<Pos2>,
        second: Option<Pos2>,
    },

    Selection {
        origin: Pos2,
        mouse_pos: Pos2,
    },

    #[default]
    None,
}

impl Action {
    pub fn try_init(&mut self, get_action: impl FnOnce() -> Self) {
        if self == &Self::None {
            *self = get_action();
        }
    }

    pub fn update(&mut self, ctx: Context) {
        match self {
            Action::Moving { origin, delta, .. } => {
                *delta = ctx.mouse_pos().unwrap() - *origin;

                if ctx.primary_released() {
                    *self = Self::None;
                }
            }

            Action::Adding { first, second, .. } => {
                let Some(clickpoint) = ctx.primary_clicked() else {
                    return;
                };

                if ctx.0.interaction_snapshot(|i| i.clicked) != Some(egui::Id::new("field")) {
                    return;
                }

                if first.is_none() {
                    *first = Some(clickpoint);
                } else {
                    *second = Some(clickpoint);
                }
            }

            Action::Selection { mouse_pos, .. } => {
                *mouse_pos = ctx.mouse_pos().unwrap();
            }

            _ => {}
        }
    }

    pub fn apply(&mut self, state: &mut AppState) {
        match *self {
            Action::Moving {
                delta,
                obj: MovingObject::View { origin_translation },
                ..
            } => {
                state.transform.translation = origin_translation + delta;
            }

            Action::Moving {
                delta,
                obj:
                    MovingObject::Elements {
                        ref origin_endpoints,
                    },
                ..
            } => {
                // TODO: add check for overlapping existing endpoints

                for (&element, &origin_endpoints) in origin_endpoints {
                    let new_endpoints =
                        origin_endpoints.map(|point| point + delta / state.transform.scaling);

                    state.circuit.change(element, new_endpoints);
                }
            }

            Action::Moving {
                delta,
                obj:
                    MovingObject::Endpoint {
                        origin_pos,
                        id,
                        endpoint,
                    },
                ..
            } => {
                // TODO: add check for overlapping existing endpoints

                let mut new_endpoints = state.circuit.endpoints(id);

                new_endpoints[endpoint] = origin_pos + delta / state.transform.scaling;

                state.circuit.change(id, new_endpoints);
            }

            Action::Adding { ty, first, second } => {
                if let Some(endpoints) =
                    first.and_then(|first| second.map(|second| [first, second]))
                {
                    // TODO: add check for overlapping existing endpoints

                    let endpoints =
                        endpoints.map(|pos| ElementPos::from_pos(state.transform.inverse() * pos));

                    let conductor = match ty {
                        ElementType::CurrentSource => {
                            Box::new(CurrentSource::new(10.0, 0.0)) as Box<dyn ElementTrait>
                        }
                        ElementType::Resistor => {
                            Box::new(Resistor::new(5.0)) as Box<dyn ElementTrait>
                        }
                        ElementType::Wire => Box::new(Wire) as Box<dyn ElementTrait>,
                    };

                    state.circuit.add(endpoints, Element::new(conductor));
                }

                if first.is_some() && second.is_some() {
                    *self = Self::None
                }
            }

            Action::Selection { origin, mouse_pos } => {
                let rect = state.transform.inverse() * Rect::from_two_pos(origin, mouse_pos);

                for (id, _) in state.circuit.iter() {
                    let endpoints = state.circuit.endpoints(id).map(ElementPos::to_pos);

                    if rect.contains(endpoints[0]) && rect.contains(endpoints[1]) {
                        state.selected.insert(id);
                    } else {
                        state.selected.remove(&id);
                    }
                }
            }

            Action::None => {}
        }
    }

    pub fn draw(&self, ctx: Context, painter: Painter) {
        match *self {
            Action::Adding { ty, first, second } => {
                if let (Some(mouse_pos), Some(first), None) = (ctx.mouse_pos(), first, second) {
                    let endpoints = [first, mouse_pos]
                        .map(|point| painter.transform.inverse() * point)
                        .map(ElementPos::from_pos);

                    match ty {
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

            Action::Selection {
                origin,
                mouse_pos: mouse,
            } => {
                let rect = painter.transform.inverse() * Rect::from_two_pos(origin, mouse);

                let color = Color32::from_rgba_unmultiplied(
                    HIGHLIGHTED_COLOR.r(),
                    HIGHLIGHTED_COLOR.g(),
                    HIGHLIGHTED_COLOR.b(),
                    20,
                );

                painter.render(RectShape::new(
                    rect,
                    Rounding::ZERO,
                    color,
                    Stroke::new(2.0, HIGHLIGHTED_COLOR),
                ));
            }

            _ => {}
        }
    }
}

#[derive(PartialEq, Eq)]
pub enum MovingObject {
    View {
        origin_translation: Vec2,
    },
    Elements {
        origin_endpoints: HashMap<ElementId, [ElementPos; 2]>,
    },
    Endpoint {
        origin_pos: ElementPos,
        id: ElementId,
        endpoint: usize,
    },
}

impl MovingObject {
    pub fn into_moving(self, origin: Pos2) -> Action {
        Action::Moving {
            origin,
            delta: Vec2::ZERO,
            obj: self,
        }
    }
}
