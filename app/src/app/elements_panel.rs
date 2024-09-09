use std::cell::Cell;

use egui::{Button, Id, Sense, Vec2};

use super::{action::Action, Context};
use crate::utils::WidgetsGallery;

#[derive(Default)]
pub struct ElementsPanel;

impl ElementsPanel {
    pub fn show(&mut self, ctx: Context, action: &mut Action) {
        let screen_rect = ctx.0.screen_rect();

        let max_panel_size = (screen_rect.height() - screen_rect.width()).abs();

        let min_coord = f32::min(screen_rect.width(), screen_rect.height());

        let max_size = Vec2::splat(min_coord / 8.0);
        let adding = Cell::new(None);

        if ctx.0.screen_rect().width() > ctx.0.screen_rect().height() {
            egui::SidePanel::right(Id::new("elements_panel"))
                .resizable(true)
                .max_width(max_panel_size)
                .show(ctx.0, |ui| {
                    WidgetsGallery {
                        max_size,
                        widgets: self.buttons(&adding),
                    }
                    .show(ui);

                    ui.allocate_painter(ui.available_size(), Sense::click_and_drag())
                });
        } else {
            egui::TopBottomPanel::bottom(Id::new("elements_panel"))
                .resizable(true)
                .max_height(max_panel_size)
                .show(ctx.0, |ui| {
                    WidgetsGallery {
                        max_size,
                        widgets: self.buttons(&adding),
                    }
                    .show(ui);

                    ui.allocate_painter(ui.available_size(), Sense::click_and_drag())
                });
        }

        if let Some(ty) = adding.get() {
            action.try_init(|| Action::Adding {
                ty,
                first: None,
                second: None,
            });
        }
    }

    fn buttons<'frame>(
        &'frame mut self,
        adding_ty: &'frame Cell<Option<ElementType>>,
    ) -> impl Iterator<Item = (Button, Box<dyn FnOnce(egui::Response) + '_>)> + '_ {
        let buttons = [
            (Button::new("wire"), ElementType::Wire),
            (Button::new("resistor"), ElementType::Resistor),
            (Button::new("current source"), ElementType::CurrentSource),
        ]
        .into_iter();

        buttons.map(move |(button, ty)| {
            let react = Box::new(move |response: egui::Response| {
                if response.clicked() {
                    adding_ty.set(Some(ty));
                }
            });

            (button, react as Box<dyn FnOnce(egui::Response)>)
        })
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ElementType {
    CurrentSource,
    Wire,
    Resistor,
}
