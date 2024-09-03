use std::cell::Cell;

use circuit::Circuit;
use egui::Pos2;

use crate::element::{Element, ElementPos};

use super::field::Adding;

pub struct AppState<'app, 'data> {
    pub ctx: &'app egui::Context,
    pub circuit: &'app mut Circuit<'data, Element<'data>, ElementPos>,
    pub adding: &'app Cell<Option<Adding>>,
}

impl<'app, 'data> AppState<'app, 'data> {
    pub fn mouse_pos(&self) -> Option<Pos2> {
        self.ctx.input(|state| state.pointer.hover_pos())
    }
}
