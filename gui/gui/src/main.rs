use std::{f32::consts::TAU, time::Instant};

use egui::{epaint::Shadow, pos2, vec2, Align, Layout, Pos2, Rounding};
use egui_tiles::{Container, Linear, LinearDir};
use fixed::types::I16F16;
use foc::pwm::Modulation as _;

mod connection;
mod widgets;

enum Pane {
    Connections,
    Graph,
}

struct Behaviour {
    motor_state: MotorState,
}

#[derive(Debug, PartialEq, Eq)]
enum Modulation {
    Square,
    Trapezoidal,
    Sinusoidal,
    SpaceVector,
}

struct MotorState {
    rotating_setpoint: Pos2,
    two_phase_setpoint: Pos2,
    three_phase_setpoint: [f32; 3],
    electrical_angle_rad: f32,
    angular_vel_radps: f32,
    last_time: Instant,
    modulation: Modulation,
}

impl MotorState {
    fn update(&mut self) {
        let current_time = Instant::now();
        let dt = current_time - self.last_time;
        self.last_time = current_time;

        let max_speed_rpm = 2200.;
        let no_load_voltage = 16.;

        let max_speed_radps = max_speed_rpm * TAU / 60.;
        let kv_si = max_speed_radps / no_load_voltage;
        let kt = 1. / kv_si;

        let dq = foc::park_clarke::RotatingReferenceFrame {
            d: I16F16::from_num(self.rotating_setpoint.x),
            q: I16F16::from_num(self.rotating_setpoint.y),
        };

        let torque_nm = kt * dq.q.to_num::<f32>();
        let inertia = 1.;

        let accel_ms2 = torque_nm / inertia;
        self.angular_vel_radps += accel_ms2 * dt.as_secs_f32();
        self.electrical_angle_rad += self.angular_vel_radps * dt.as_secs_f32();

        let sin_angle = I16F16::from_num(self.electrical_angle_rad.sin());
        let cos_angle = I16F16::from_num(self.electrical_angle_rad.cos());
        let two_phase_setpoint = foc::park_clarke::inverse_park(cos_angle, sin_angle, dq);
        self.two_phase_setpoint = pos2(
            two_phase_setpoint.beta.to_num(),
            two_phase_setpoint.alpha.to_num(),
        );

        self.three_phase_setpoint = match self.modulation {
            Modulation::Square => foc::pwm::Square::modulate(two_phase_setpoint),
            Modulation::Trapezoidal => foc::pwm::Trapezoidal::modulate(two_phase_setpoint),
            Modulation::Sinusoidal => foc::pwm::Sinusoidal::modulate(two_phase_setpoint),
            Modulation::SpaceVector => foc::pwm::SpaceVector::modulate(two_phase_setpoint),
        }
        .map(|v| v.to_num());
    }
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

                ui.label("USB");
                ui.group(|ui| {
                    // for device in connection::list_all() {
                    //     display_connection(
                    //         ui,
                    //         &format!("{} ({}:{})", device.product, device.bus, device.address),
                    //     );
                    // }
                });
            }
            Pane::Graph => {
                display_graph(ui, &mut self.motor_state);
            }
        }

        Default::default()
    }
}

fn display_graph(ui: &mut egui::Ui, state: &mut MotorState) {
    let window_frame = egui::Frame::window(ui.style())
        .shadow(Shadow::NONE)
        .rounding(Rounding::same(2.))
        .fill(ui.style().visuals.widgets.open.weak_bg_fill);

    // Draw rotating reference frame
    egui::Window::new("Rotating Frame")
        .collapsible(false)
        .fixed_pos(ui.min_rect().left_top() + vec2(100., 100.))
        .resizable(false)
        .frame(window_frame)
        .show(ui.ctx(), |ui| {
            ui.add(widgets::VectorPlot::new(&mut state.rotating_setpoint));
            ui.horizontal(|ui| {
                ui.label(format!("D: {:.3}", state.rotating_setpoint.x));
                ui.label(format!("Q: {:.3}", state.rotating_setpoint.y));
            });
        });

    // Draw two-phase stationary reference frame
    egui::Window::new("Two-Phase Stationary Frame")
        .collapsible(false)
        .fixed_pos(ui.min_rect().left_top() + vec2(400., 100.))
        .resizable(false)
        .frame(window_frame)
        .show(ui.ctx(), |ui| {
            ui.add(widgets::VectorPlot::new(&mut state.two_phase_setpoint));
            ui.horizontal(|ui| {
                ui.label(format!("Alpha: {:.3}", state.two_phase_setpoint.y));
                ui.label(format!("Beta: {:.3}", state.two_phase_setpoint.x));
            });
        });

    // Draw three-phase stationary reference frame
    egui::Window::new("Three-Phase Stationary Frame")
        .collapsible(false)
        .fixed_pos(ui.min_rect().left_top() + vec2(700., 100.))
        .resizable(false)
        .frame(window_frame)
        .show(ui.ctx(), |ui| {
            ui.add(widgets::ThreePhaseArrowPlot::new(
                &mut state.three_phase_setpoint,
            ));
            ui.horizontal(|ui| {
                ui.label(format!("A: {:.3}", state.three_phase_setpoint[0]));
                ui.label(format!("B: {:.3}", state.three_phase_setpoint[1]));
                ui.label(format!("C: {:.3}", state.three_phase_setpoint[2]));
            });
            ui.horizontal(|ui| {
                ui.selectable_value(&mut state.modulation, Modulation::Square, "Square");
                ui.selectable_value(&mut state.modulation, Modulation::Trapezoidal, "Trap");
                ui.selectable_value(&mut state.modulation, Modulation::Sinusoidal, "Sine");
                ui.selectable_value(&mut state.modulation, Modulation::SpaceVector, "SVPWM");
            });
        });

    // Draw electrical angle
    egui::Window::new("Electrical Angle")
        .collapsible(false)
        .fixed_pos(ui.min_rect().left_top() + vec2(400., 400.))
        .resizable(false)
        .frame(window_frame)
        .show(ui.ctx(), |ui| {
            ui.add(widgets::AnglePlot::new(&mut state.electrical_angle_rad));
            ui.horizontal(|ui| {
                ui.label(format!(
                    "Angle: {:.3}°",
                    state.electrical_angle_rad.to_degrees()
                ));
            });
        });
}

fn main() -> Result<(), eframe::Error> {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1280.0, 720.0]),
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
        motor_state: MotorState {
            rotating_setpoint: Pos2::ZERO,
            two_phase_setpoint: Pos2::ZERO,
            three_phase_setpoint: [0.; 3],
            electrical_angle_rad: 0.,
            angular_vel_radps: 0.,
            last_time: Instant::now(),
            modulation: Modulation::SpaceVector,
        },
    };

    eframe::run_simple_native("FOC Remote Tuner", options, move |ctx, _frame| {
        // Force dark theme
        ctx.send_viewport_cmd(egui::ViewportCommand::SetTheme(egui::SystemTheme::Dark));
        ctx.options_mut(|options| options.theme_preference = egui::ThemePreference::Dark);

        behavior.motor_state.update();

        egui::CentralPanel::default().show(ctx, |ui| {
            tree.ui(&mut behavior, ui);
        });
        ctx.request_repaint();
    })
}
