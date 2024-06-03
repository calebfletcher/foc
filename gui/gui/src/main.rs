use egui::{Align, Layout};
use egui_tiles::{Container, Linear, LinearDir};

enum Pane {
    Connections,
    Graph,
}

struct Behaviour {}

impl egui_tiles::Behavior<Pane> for Behaviour {
    fn tab_title_for_pane(&mut self, pane: &Pane) -> egui::WidgetText {
        match pane {
            Pane::Connections => "Connections".into(),
            Pane::Graph => "Graph".into(),
        }
    }

    fn pane_ui(
        &mut self,
        ui: &mut egui::Ui,
        _tile_id: egui_tiles::TileId,
        pane: &mut Pane,
    ) -> egui_tiles::UiResponse {
        match pane {
            Pane::Connections => {
                let display_connection = |ui: &mut egui::Ui, connection: &str| {
                    ui.horizontal(|ui| {
                        ui.label(connection);
                        ui.allocate_ui_with_layout(
                            ui.available_size_before_wrap()
                                - egui::Vec2 {
                                    x: ui.spacing().item_spacing.x,
                                    y: 0.,
                                },
                            Layout::right_to_left(Align::TOP),
                            |ui| {
                                if ui.button("Connect").clicked() {
                                    println!("connect");
                                }
                            },
                        );
                    });
                };

                ui.label("RTT");
                ui.group(|ui| {
                    // TODO: Read list of RTT devices
                    display_connection(ui, "Device 0");
                    display_connection(ui, "Device 1");
                    display_connection(ui, "Device 2");
                });
                ui.label("USB");
                ui.group(|ui| {
                    // TODO: Read list of USB devices
                    display_connection(ui, "Device 0");
                });
            }
            Pane::Graph => {}
        }

        Default::default()
    }
}

fn main() -> Result<(), eframe::Error> {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1280.0, 720.0]),
        follow_system_theme: false,
        ..Default::default()
    };

    // Setup tiles
    let mut tiles = egui_tiles::Tiles::default();
    let settings = tiles.insert_pane(Pane::Connections);
    let text = tiles.insert_pane(Pane::Graph);

    // Setup initial tile layout
    let mut inner = Linear {
        children: vec![settings, text],
        dir: LinearDir::Horizontal,
        ..Default::default()
    };
    inner.shares.set_share(settings, 0.2);
    inner.shares.set_share(text, 0.8);
    let root = tiles.insert_container(Container::Linear(inner));

    let mut tree = egui_tiles::Tree::new("tree", root, tiles);

    eframe::run_simple_native("FOC Remote Tuner", options, move |ctx, _frame| {
        ctx.send_viewport_cmd(egui::ViewportCommand::SetTheme(egui::SystemTheme::Dark));
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut behavior = Behaviour {};
            tree.ui(&mut behavior, ui);
        });
        ctx.request_repaint();
    })
}
