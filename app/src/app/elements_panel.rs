use egui::{Button, Id, Vec2};

use crate::utils::WidgetsGallery;

use super::{field::Adding, state::AppState};

#[derive(Default)]
pub struct ElementsPanel;

impl ElementsPanel {
    pub fn show<'app, 'data>(&mut self, state: &mut AppState<'app, 'data>) {
        let screen_rect = state.ctx.screen_rect();

        let max_panel_size = (screen_rect.height() - screen_rect.width()).abs();

        let min_coord = f32::min(screen_rect.width(), screen_rect.height());

        let max_size = Vec2::splat(min_coord / 8.0);
        let min_size = Vec2::splat(min_coord / 12.0);

        if state.ctx.screen_rect().width() > state.ctx.screen_rect().height() {
            egui::SidePanel::right(Id::new("elements_panel"))
                .resizable(true)
                .max_width(max_panel_size)
                .show(state.ctx, |ui| {
                    self.element_buttons(state, ui, max_size, min_size)
                });
        } else {
            egui::TopBottomPanel::bottom(Id::new("id"))
                .resizable(true)
                .max_height(max_panel_size)
                .show(state.ctx, |ui| {
                    self.element_buttons(state, ui, max_size, min_size)
                });
        }
    }

    fn element_buttons(
        &mut self,
        state: &AppState,
        ui: &mut egui::Ui,
        max_size: Vec2,
        min_size: Vec2,
    ) {
        let buttons = vec![
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

        let adding = &state.adding;

        let buttons = buttons.into_iter().map(|(button, element)| {
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

        WidgetsGallery {
            max_size,
            widgets: buttons,
        }
        .show(ui);
    }
}

#[derive(Clone, Copy)]
pub enum ElementType {
    CurrentSource,
    Wire,
    Resistor,
}
