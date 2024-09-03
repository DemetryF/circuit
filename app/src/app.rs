mod elements_panel;
mod field;
mod state;

use std::cell::Cell;

use elements_panel::ElementsPanel;
use state::AppState;

use circuit::Circuit;

use crate::element::{Element, ElementPos};
use field::{Adding, Field};

#[derive(Default)]
pub struct App<'data> {
    circuit: Circuit<'data, Element<'data>, ElementPos>,

    field: Field,
    elements_panel: ElementsPanel,

    adding: Cell<Option<Adding>>,
}

impl<'data> eframe::App for App<'data> {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();

        let delta_time = ctx.input(|state| state.stable_dt);

        self.circuit.update(delta_time);

        let mut state = AppState {
            ctx,
            circuit: &mut self.circuit,
            adding: &self.adding,
        };

        self.field.show(&mut state);
        self.elements_panel.show(&mut state);
    }
}
