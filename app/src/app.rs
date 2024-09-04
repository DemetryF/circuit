mod control_panel;
mod elements_panel;
mod field;

use std::cell::Cell;

use control_panel::ControlPanel;
use egui::Pos2;

use circuit::{circuit::ElementId, Circuit};

use elements_panel::{ElementType, ElementsPanel};
use field::Field;

use crate::element::{Element, ElementPos};

#[derive(Default)]
pub struct App<'data> {
    field: Field,
    elements_panel: ElementsPanel,
    control_panel: ControlPanel,

    state: AppState<'data>,
}

impl<'data> eframe::App for App<'data> {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();

        let delta_time = ctx.input(|state| state.stable_dt);

        self.state.circuit.update(delta_time);

        let ctx = Context(ctx);

        self.field.show(&mut self.state, ctx);
        self.elements_panel.show(&mut self.state, ctx);
        self.control_panel.show(&mut self.state, ctx);
    }
}

#[derive(Default)]
pub struct AppState<'data> {
    pub circuit: Circuit<'data, Element<'data>, ElementPos>,
    pub adding: Cell<Option<Adding>>,
    pub settings: Option<ElementId>,
}

#[derive(Clone, Copy)]
pub struct Adding {
    pub ty: ElementType,
    pub first: Option<ElementPos>,
}

impl Adding {
    pub fn new(ty: ElementType) -> Self {
        Self { ty, first: None }
    }
}

#[derive(Clone, Copy)]
pub struct Context<'frame>(pub &'frame egui::Context);

impl<'frame> Context<'frame> {
    pub fn mouse_pos(self) -> Option<Pos2> {
        self.0.input(|state| state.pointer.hover_pos())
    }
}
