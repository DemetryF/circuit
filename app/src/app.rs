mod action;
mod control_panel;
mod elements_panel;
mod field;

use std::collections::HashSet;

use action::Action;
use control_panel::ControlPanel;
use egui::{emath::TSTransform, InputState, LayerId, PointerButton, Pos2};

use circuit::{circuit::ElementId, Circuit};

use crate::element::{Element, ElementPos};
use crate::utils::Painter;

use elements_panel::ElementsPanel;
use field::Field;

#[derive(Default)]
pub struct App<'data> {
    field: Field,
    elements_panel: ElementsPanel,
    control_panel: ControlPanel,

    state: AppState<'data>,
    action: Action,
}

impl<'data> eframe::App for App<'data> {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();

        let ctx = Context(ctx);

        self.state.circuit.update(ctx.delta_time());
        self.state.update_zoom(ctx);

        let painter = ctx.0.layer_painter(LayerId::background());
        let painter = Painter::new(&painter, self.state.transform);

        self.field.show(&mut self.state, ctx, &mut self.action);
        self.elements_panel.show(ctx, &mut self.action);
        self.control_panel.show(&mut self.state, ctx);

        self.action.update(ctx);
        self.action.apply(&mut self.state);
        self.action.draw(ctx, painter);
    }
}

#[derive(Default)]
pub struct AppState<'data> {
    pub circuit: Circuit<'data, Element<'data>, ElementPos>,

    pub transform: TSTransform,
    pub settings: Option<ElementId>,
    pub selected: HashSet<ElementId>,
    pub hovered: Option<Hovered>,
}

impl<'data> AppState<'data> {
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
}

#[derive(Clone, Copy)]
pub struct Context<'frame>(pub &'frame egui::Context);

impl<'frame> Context<'frame> {
    pub fn delta_time(self) -> f32 {
        self.0.input(|state| state.stable_dt)
    }

    pub fn mouse_pos(self) -> Option<Pos2> {
        self.0.input(|state| state.pointer.hover_pos())
    }

    pub fn primary_released(self) -> bool {
        self.0.input(|state| state.pointer.primary_released())
    }

    pub fn primary_clicked(self) -> Option<Pos2> {
        self.0.input(|InputState { pointer, .. }| {
            pointer
                .button_clicked(PointerButton::Primary)
                .then(|| pointer.hover_pos().unwrap())
        })
    }

    pub fn field_painter(self) -> egui::Painter {
        self.0.layer_painter(LayerId::background())
    }
}

pub struct Hovered {
    pub id: ElementId,
    pub endpoint: Option<usize>,
}
