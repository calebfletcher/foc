use egui::{vec2, Color32, Pos2, Sense, Stroke, Vec2};

#[derive(Debug)]
pub struct VectorPlot<'a> {
    settable: bool,
    arrows: bool,
    size: f32,
    fixed_magnitude: bool,
    value: &'a mut Pos2,
}

impl<'a> VectorPlot<'a> {
    pub fn new(value: &'a mut Pos2) -> Self {
        Self {
            settable: true,
            arrows: true,
            size: 100.,
            fixed_magnitude: false,
            value,
        }
    }
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    pub fn settable(mut self, enabled: bool) -> Self {
        self.settable = enabled;
        self
    }

    pub fn arrows(mut self, enabled: bool) -> Self {
        self.arrows = enabled;
        self
    }

    pub fn fixed_magnitude(mut self, enabled: bool) -> Self {
        self.fixed_magnitude = enabled;
        self
    }
}

impl egui::Widget for VectorPlot<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let (rot_frame_resp, painter) =
            ui.allocate_painter(Vec2::splat(self.size * 2. + 16.), Sense::click_and_drag());
        let center = rot_frame_resp.rect.center();

        if self.arrows {
            painter.arrow(
                center,
                vec2(self.size, 0.),
                Stroke::new(2., Color32::DARK_GRAY),
            );
            painter.arrow(
                center,
                vec2(0., -self.size),
                Stroke::new(2., Color32::DARK_GRAY),
            );
        }
        painter.circle_stroke(center, self.size, (2., Color32::LIGHT_GRAY));

        let dq_to_screen = vec2(self.size, -self.size);

        if self.settable && rot_frame_resp.is_pointer_button_down_on() {
            let screen_pos = rot_frame_resp.interact_pointer_pos().unwrap();
            let raw_dq_pos = (screen_pos - center) / dq_to_screen;
            let dq_pos = if self.fixed_magnitude || raw_dq_pos.length() > 1. {
                raw_dq_pos.normalized()
            } else {
                raw_dq_pos
            };
            *self.value = dq_pos.to_pos2();
        }

        painter.circle(
            center + self.value.to_vec2() * dq_to_screen,
            5.,
            Color32::GRAY,
            Stroke::NONE,
        );

        rot_frame_resp
    }
}

#[derive(Debug)]
pub struct AnglePlot<'a> {
    settable: bool,
    size: f32,
    value_rad: &'a mut f32,
}

impl<'a> AnglePlot<'a> {
    pub fn new(value_rad: &'a mut f32) -> Self {
        Self {
            settable: true,
            size: 100.,
            value_rad,
        }
    }
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    pub fn settable(mut self, enabled: bool) -> Self {
        self.settable = enabled;
        self
    }
}

impl egui::Widget for AnglePlot<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let (rot_frame_resp, painter) =
            ui.allocate_painter(Vec2::splat(self.size * 2. + 16.), Sense::click_and_drag());
        let center = rot_frame_resp.rect.center();

        painter.circle_stroke(center, self.size, (2., Color32::LIGHT_GRAY));

        let dq_to_screen = vec2(self.size, -self.size);

        if self.settable && rot_frame_resp.is_pointer_button_down_on() {
            let screen_pos = rot_frame_resp.interact_pointer_pos().unwrap();
            let raw_dq_pos = (screen_pos - center) / dq_to_screen;
            let dq_pos = raw_dq_pos.normalized();
            *self.value_rad = dq_pos.angle();
        }

        //if let Some(rotating_setpoint) = self.value_rad {
        painter.circle(
            center + Vec2::angled(*self.value_rad) * dq_to_screen,
            5.,
            Color32::GRAY,
            Stroke::NONE,
        );
        //}

        rot_frame_resp
    }
}

#[derive(Debug)]
pub struct ThreePhaseArrowPlot<'a> {
    size: f32,
    value: &'a mut [f32; 3],
}

impl<'a> ThreePhaseArrowPlot<'a> {
    pub fn new(value: &'a mut [f32; 3]) -> Self {
        Self { size: 100., value }
    }
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }
}

impl egui::Widget for ThreePhaseArrowPlot<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let (rot_frame_resp, painter) =
            ui.allocate_painter(Vec2::splat(self.size * 2. + 16.), Sense::click_and_drag());
        let center = rot_frame_resp.rect.center();

        painter.arrow(
            center,
            self.value[0] * self.size * Vec2::angled(0.),
            Stroke::new(2., Color32::DARK_GRAY),
        );
        painter.arrow(
            center,
            self.value[1] * self.size * Vec2::angled(-120f32.to_radians()),
            Stroke::new(2., Color32::DARK_GRAY),
        );
        painter.arrow(
            center,
            self.value[2] * self.size * Vec2::angled(120f32.to_radians()),
            Stroke::new(2., Color32::DARK_GRAY),
        );

        painter.circle_stroke(center, self.size, (2., Color32::LIGHT_GRAY));

        rot_frame_resp
    }
}
