use std::collections::HashMap;

use egui::{CentralPanel, Id, Sense};
use egui::{Color32, Margin, Pos2, Shape, Vec2};

use super::action::{Action, MovingObject};
use super::{AppState, Context, Hovered};
use crate::element::{ElementPos, Render};
use crate::element::{CELL_SIZE, SENSABLE_DIST};
use crate::utils::Painter;

#[derive(Default)]
pub struct Field;

impl Field {
    pub fn show(&mut self, state: &mut AppState, ctx: Context, action: &mut Action) {
        self.panel().show(ctx.0, move |ui| {
            let response = {
                let (_, rect) = ui.allocate_space(ui.available_size());

                ui.interact(rect, Id::new("field"), Sense::click_and_drag())
            };

            let painter = ctx.field_painter();
            let painter = Painter::new(&painter, state.transform);

            self.draw_grid(ctx, painter, ui.min_size());
            self.process_elements(state, ctx, painter);

            update_selected(state, ctx, &response);

            if response.drag_started_by(egui::PointerButton::Primary) {
                start_moving(ctx, action, state);
            }

            if response.drag_stopped_by(egui::PointerButton::Primary) {
                if let Action::Moving { .. } = action {
                    *action = Action::None;
                }
            }

            if response.drag_started_by(egui::PointerButton::Secondary) {
                let mouse_pos = ctx.mouse_pos().unwrap();

                action.try_init(|| Action::Selection {
                    mouse_pos,
                    origin: mouse_pos,
                });
            }

            if response.drag_stopped_by(egui::PointerButton::Secondary) {
                if let Action::Selection { .. } = action {
                    *action = Action::None;
                }
            }
        });
    }

    fn panel(&self) -> CentralPanel {
        egui::CentralPanel::default().frame(egui::Frame {
            inner_margin: Margin::ZERO,
            outer_margin: Margin::ZERO,
            fill: Color32::from_gray(27),
            ..Default::default()
        })
    }

    fn process_elements(&self, state: &mut AppState, ctx: Context, painter: Painter) {
        state.hovered = None;

        for (id, element) in state.circuit.iter() {
            let endpoints = state.circuit.endpoints(id);

            let mut highlighted = state.selected.contains(&id);

            if let Some(mouse_pos) = ctx.mouse_pos() {
                let grid_mouse_pos = state.transform.inverse() * mouse_pos;

                let hovered = element.includes(endpoints, grid_mouse_pos);

                highlighted = highlighted || hovered;

                if hovered {
                    if let Some(endpoint) = endpoints.into_iter().position(|point| {
                        point.to_pos().distance_sq(grid_mouse_pos) <= SENSABLE_DIST.powi(2) * 2.0
                    }) {
                        state.hovered = Some(Hovered {
                            id,
                            endpoint: Some(endpoint),
                        })
                    } else if state.hovered.is_none() {
                        state.hovered = Some(Hovered { id, endpoint: None })
                    }
                }
            }

            if highlighted {
                element.render_highlighted(endpoints, painter);
            } else {
                element.render(endpoints, painter);
            }
        }
    }

    fn draw_grid(&self, ctx: Context, painter: Painter, size: Vec2) {
        let top_left = painter.transform.inverse() * Pos2::ZERO;
        let top_left = ElementPos::from_pos(top_left);

        let right_bottom = painter.transform.inverse() * size.to_pos2();
        let right_bottom = ElementPos::from_pos(right_bottom);

        for x in (top_left.x - 1)..=right_bottom.x {
            for y in (top_left.y - 1)..=right_bottom.y {
                let pos = ElementPos { x, y }.to_pos();

                painter.render(Shape::circle_filled(
                    pos,
                    CELL_SIZE / 10.0,
                    Color32::from_gray(40),
                ));
            }
        }

        if let Some(mouse_pos) = ctx.mouse_pos() {
            let mouse_pos = ElementPos::from_pos(painter.transform.inverse() * mouse_pos);

            painter.render(Shape::circle_filled(
                mouse_pos.to_pos(),
                CELL_SIZE / 7.0,
                Color32::from_gray(60),
            ));
        }
    }
}

fn start_moving(ctx: Context<'_>, action: &mut Action, state: &mut AppState<'_>) {
    let mouse_pos = ctx.mouse_pos().unwrap();

    action.try_init(|| match state.hovered {
        Some(Hovered {
            id,
            endpoint: Some(endpoint),
        }) => {
            let origin_pos = state.circuit.endpoints(id)[endpoint];

            MovingObject::Endpoint {
                origin_pos,
                id,
                endpoint,
            }
            .into_moving(mouse_pos)
        }

        Some(Hovered { id, endpoint: None }) if state.selected.contains(&id) => {
            MovingObject::Elements {
                origin_endpoints: state
                    .selected
                    .iter()
                    .map(|&id| (id, state.circuit.endpoints(id)))
                    .collect(),
            }
            .into_moving(mouse_pos)
        }

        Some(Hovered { id, endpoint: None }) => {
            let origin_endpoints = state.circuit.endpoints(id);

            MovingObject::Elements {
                origin_endpoints: HashMap::from_iter([(id, origin_endpoints)]),
            }
            .into_moving(mouse_pos)
        }

        None => MovingObject::View {
            origin_translation: state.transform.translation,
        }
        .into_moving(mouse_pos),
    });
}

fn update_selected(state: &mut AppState<'_>, ctx: Context<'_>, response: &egui::Response) {
    if let Some(Hovered { id, .. }) = state.hovered {
        let pressed_shift = ctx.0.input(|state| state.modifiers.shift);

        if response.clicked() && pressed_shift {
            if state.selected.contains(&id) {
                state.selected.remove(&id);
            } else {
                state.selected.insert(id);
            }
        }

        if response.clicked() {
            state.selected.clear();

            if !state.selected.contains(&id) {
                state.selected.insert(id);
            }
        }

        if response.double_clicked() {
            state.settings = Some(id);
        }
    }
}
