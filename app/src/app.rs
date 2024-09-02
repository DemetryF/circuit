mod context;
mod field;

use context::AppContext;
use egui::Vec2;
use egui::{Button, Id};

use circuit::Circuit;

use crate::element::{Element, ElementPos, ElementType};
use crate::utils::WidgetsGallery;
use field::{Adding, Field};

#[derive(Default)]
pub struct App<'data> {
    circuit: Circuit<'data, Element<'data>, ElementPos>,

    field: Field,
}

impl<'data> eframe::App for App<'data> {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();

        let delta_time = ctx.input(|state| state.stable_dt);

        self.circuit.update(delta_time);

        self.field.show(AppContext {
            egui_ctx: ctx,
            circuit: &mut self.circuit,
        });

        self.elements_panel(ctx);
    }
}

impl<'data> App<'data> {
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

        let adding = &self.field.adding;

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
}
