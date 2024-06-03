use egui_tiles::{Container, Linear, LinearDir};

enum Pane {
    Settings,
    Text(String),
}

struct TreeBehavior {}

impl egui_tiles::Behavior<Pane> for TreeBehavior {
    fn tab_title_for_pane(&mut self, pane: &Pane) -> egui::WidgetText {
        match pane {
            Pane::Settings => "Settings".into(),
            Pane::Text(text) => text.clone().into(),
        }
    }

    fn pane_ui(
        &mut self,
        ui: &mut egui::Ui,
        _tile_id: egui_tiles::TileId,
        pane: &mut Pane,
    ) -> egui_tiles::UiResponse {
        match pane {
            Pane::Settings => {
                ui.label("settings");
            }
            Pane::Text(text) => {
                ui.text_edit_singleline(text);
            }
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
    let settings = tiles.insert_pane(Pane::Settings);
    let text = tiles.insert_pane(Pane::Text("Hello".to_owned()));

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
            let mut behavior = TreeBehavior {};
            tree.ui(&mut behavior, ui);
        });
    })
}
