use egui::{Vec2, Widget};

pub struct WidgetsGallery<'ui, W, WS>
where
    W: Widget,
    WS: Iterator<Item = (W, Box<dyn FnOnce(egui::Response) + 'ui>)>,
{
    pub max_size: Vec2,
    pub widgets: WS,
}

impl<'ui, W, WS> WidgetsGallery<'ui, W, WS>
where
    W: Widget,
    WS: Iterator<Item = (W, Box<dyn FnOnce(egui::Response) + 'ui>)>,
{
    pub fn show(self, ui: &'ui mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            let num_columns = (ui.available_width() / self.max_size.x).ceil();
            let width = (ui.available_width()
                - (num_columns - 1.0) * ui.style().spacing.item_spacing.x)
                / num_columns;

            let num_columns = num_columns as usize;

            ui.columns(num_columns, |columns| {
                for (column, (widget, react)) in
                    (0..num_columns).into_iter().cycle().zip(self.widgets)
                {
                    let response = columns[column].add_sized(Vec2::splat(width), widget);

                    (react)(response)
                }
            });
        });
    }
}
