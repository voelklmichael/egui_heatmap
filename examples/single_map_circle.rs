#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// hide console window on Windows in release
use eframe::egui;
use egui_heatmap::{
    BitmapWidget, Color, ColorWithThickness, MapPosition, MultiBitmapWidgetSettings,
};

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1000.0, 800.0)),
        ..Default::default()
    };

    eframe::run_native(
        "Single-Map Circle",
        options,
        Box::new(|_cc| Box::new(MyApp::default())),
    )
}
struct MyApp {
    bitmap: BitmapWidget,
}

impl Default for MyApp {
    fn default() -> Self {
        let settings = MultiBitmapWidgetSettings {
            start_size: None,
            id: "test".to_owned(),
            boundary_between_data: ColorWithThickness {
                color: Color::DARK_GRAY,
                thickness: 10,
            },
            colorbar: Some((
                egui_heatmap::colors::Gradient::with_options(
                    &egui_heatmap::colors::ColorGradientOptions::StartCenterEnd {
                        start: egui::Color32::RED,
                        center: egui::Color32::DARK_GREEN,
                        end: egui::Color32::BLUE,
                        steps: 6,
                    },
                ),
                80,
                (-3.1235, 12.456),
            )),
            background: Color::BLACK,
            boundary_unselected: ColorWithThickness {
                color: Color::GRAY,
                thickness: 7,
            },
            boundary_selected: Color::WHITE,
            boundary_factor_min: 3,
        };

        Self {
            bitmap: BitmapWidget::with_settings(
                egui_heatmap::Data::<Color>::example_circle(
                    100,
                    100,
                    egui_heatmap::CoordinatePoint { x: 50, y: 50 },
                ),
                settings,
            ),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(
                egui::Layout::left_to_right(egui::Align::BOTTOM).with_cross_justify(true),
                |ui| {
                    ui.vertical(|ui| {
                        egui::scroll_area::ScrollArea::new([true, true]).show(ui, |ui| {
                            egui::Grid::new("grid").num_columns(2).show(ui, |ui| {
                                for i in 0..125 {
                                    ui.label("ae Row:");
                                    ui.label(&i.to_string());
                                    ui.end_row()
                                }
                            });
                        });
                    });
                    ui.with_layout(
                        egui::Layout::bottom_up(egui::Align::LEFT).with_cross_justify(true),
                        |ui| {
                            let problem = self.bitmap.problem().map_or_else(
                                || "no problems".to_string(),
                                |e| format!("Problem: {e:?}"),
                            );
                            ui.label(problem);
                            // mouse over text
                            let text = match self.bitmap.hover() {
                                MapPosition::NotHovering => "-----".to_owned(),
                                MapPosition::NoData(egui_heatmap::CoordinatePoint { x, y }) => {
                                    format!("no data at {x}|{y}")
                                }
                                MapPosition::Pixel(egui_heatmap::CoordinatePoint { x, y }) => {
                                    format!("{x}|{y}")
                                }
                                MapPosition::Colorbar(value) => {
                                    format!("Colorbar: {value:.5E}")
                                }
                            };
                            ui.label(text);
                            ui.label(
                                "Selected: ".to_owned()
                                    + &self
                                        .bitmap
                                        .selected()
                                        .into_iter()
                                        .map(|egui_heatmap::CoordinatePoint { x, y }| {
                                            format!("({x}|{y})")
                                        })
                                        .collect::<Vec<_>>()
                                        .join(", "),
                            );

                            self.bitmap.ui(ui, ctx);
                        },
                    );
                },
            );
        });
    }
}
