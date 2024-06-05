use egui::{
    epaint::Shadow, pos2, vec2, Align, Color32, Layout, Pos2, Rect, Rounding, Sense, Stroke, Vec2,
};
use egui_tiles::{Container, Linear, LinearDir};

enum Pane {
    Connections,
    Graph,
}

struct Behaviour {
    rotating_setpoint: Option<Pos2>,
}

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
            Pane::Graph => {
                display_graph(ui, &mut self.rotating_setpoint);
            }
        }

        Default::default()
    }
}

fn display_graph(ui: &mut egui::Ui, rotating_setpoint: &mut Option<Pos2>) {
    // Draw rotating reference frame
    egui::Window::new("Rotating Reference Frame")
        .collapsible(false)
        .fixed_pos(pos2(400., 200.))
        .resizable(false)
        .frame(
            egui::Frame::window(ui.style())
                .shadow(Shadow::NONE)
                .rounding(Rounding::same(2.))
                .fill(ui.style().visuals.widgets.open.weak_bg_fill),
        )
        .show(ui.ctx(), |ui| {
            egui::Frame::none().inner_margin(16.).show(ui, |ui| {
                let size = 100.;
                let (rot_frame_resp, painter) =
                    ui.allocate_painter(Vec2::splat(size * 2. + 2.), Sense::click_and_drag());
                let center = rot_frame_resp.rect.center();

                painter.arrow(center, vec2(size, 0.), Stroke::new(2., Color32::WHITE));
                painter.arrow(center, vec2(0., -size), Stroke::new(2., Color32::WHITE));
                painter.circle_stroke(center, size, (2., Color32::LIGHT_GRAY));

                let dq_to_screen = vec2(size, -size);
                if rot_frame_resp.is_pointer_button_down_on() {
                    let screen_pos = rot_frame_resp.interact_pointer_pos().unwrap();
                    let dq_pos = ((screen_pos - center) / dq_to_screen).normalized();
                    *rotating_setpoint = Some(dq_pos.to_pos2());
                }

                if let Some(rotating_setpoint) = rotating_setpoint {
                    painter.circle(
                        center + rotating_setpoint.to_vec2() * dq_to_screen,
                        5.,
                        Color32::GRAY,
                        Stroke::NONE,
                    );
                }
            });
        });

    // let resp = ui.allocate_ui_at_rect(
    //     Rect::from_min_size(space.left_top() + vec2(300., 300.), vec2(200., 200.)),
    //     |ui| {},
    // );
    // let contents_rect = resp.response.rect;
    // let painter = ui.painter_at(contents_rect.expand(10.));
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

    let mut behavior = Behaviour {
        rotating_setpoint: None,
    };

    eframe::run_simple_native("FOC Remote Tuner", options, move |ctx, _frame| {
        ctx.send_viewport_cmd(egui::ViewportCommand::SetTheme(egui::SystemTheme::Dark));
        egui::CentralPanel::default().show(ctx, |ui| {
            tree.ui(&mut behavior, ui);
        });
        ctx.request_repaint();
    })
}
