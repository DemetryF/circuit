use parse_int::parse;

use crate::element::Properties;

use super::{AppState, Context};

#[derive(Default)]
pub struct ControlPanel;

impl ControlPanel {
    pub fn show(&mut self, state: &mut AppState, ctx: Context) {
        let Some(element) = state.settings else {
            return;
        };

        let element = state.circuit.get_mut(element);

        egui::SidePanel::left("control").show(ctx.0, |ui| {
            let names = element.properties().into_iter();
            let values = element.properties_mut().into_iter();

            for (&name, value_ref) in Iterator::zip(names, values) {
                let mut value = value_ref.to_string();

                ui.label(name);
                let response = ui.text_edit_singleline(&mut value);

                if response.changed() {
                    if let Ok(value) = parse(&value) {
                        *value_ref = value
                    }
                }
            }
        });
    }
}
