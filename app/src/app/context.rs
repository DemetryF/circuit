use circuit::Circuit;
use egui::Pos2;

use crate::element::{Element, ElementPos};

pub struct AppContext<'app, 'data> {
    pub egui_ctx: &'app egui::Context,
    pub circuit: &'app mut Circuit<'data, Element<'data>, ElementPos>,
}

impl<'app, 'data> AppContext<'app, 'data> {
    pub fn mouse_pos(&self) -> Option<Pos2> {
        self.egui_ctx.input(|state| state.pointer.hover_pos())
    }
}
