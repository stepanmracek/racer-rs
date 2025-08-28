use crate::{
    controller::Controller,
    environment::{Action, Observation},
};
use macroquad::prelude::*;

#[derive(Default)]
pub struct TouchController {
    last_touch_pos: Option<Vec2>,
}

impl TouchController {
    fn controller_rect() -> Rect {
        let size = screen_width() / 2.0 - 50.0;
        Rect::new(
            screen_width() / 2.,
            screen_height() - size - 50.,
            size,
            size,
        )
    }
}

impl Controller for TouchController {
    fn control(&mut self, _observation: &Observation) -> Action {
        let mut steer = 0.0;
        let mut throttle = 0.0;
        let r = TouchController::controller_rect();
        if is_mouse_button_down(MouseButton::Left) {
            let touch_pos = Vec2::from(mouse_position()).clamp(r.point(), r.point() + r.size());
            self.last_touch_pos = Some(touch_pos);

            let control_vec = (touch_pos - r.center()) / (r.w * -0.5);
            steer = control_vec.x;
            throttle = control_vec.y;
        } else {
            self.last_touch_pos = None;
        }

        Action { steer, throttle }
    }

    fn draw(&self) {
        let r = TouchController::controller_rect();
        draw_rectangle_lines(r.x, r.y, r.w, r.h, 2.0, WHITE);
        draw_circle_lines(r.center().x, r.center().y, r.w / 10.0, 2.0, WHITE);
        if let Some(p) = self.last_touch_pos {
            draw_circle(p.x, p.y, r.w / 10.0, WHITE);
        }
    }
}
