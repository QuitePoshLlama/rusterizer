use raylib::prelude::*;
use crate::transform::Transform;

pub struct Camera {
    pub fov: f32,
    pub camera_speed: f32,
    pub mouse_sensitivity: f32,
    pub transform: Transform,
}

impl Camera {
    pub fn camera_update(&mut self, r1: &RaylibHandle) {
        let mouse_delta = r1.get_mouse_delta();
        // Update yaw & pitch if clicking
        if r1.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
            self.transform.yaw   -= mouse_delta.x * self.mouse_sensitivity;
            self.transform.pitch += mouse_delta.y * self.mouse_sensitivity;
            // Clamp pitch so camera can't flip upside-down
            self.transform.pitch = self.transform.pitch.clamp(-85.0f32.to_radians(), 85.0f32.to_radians());
        }
        let (right, _up, forward) = self.transform.get_basis_vectors();

        if r1.is_key_down(KeyboardKey::KEY_W) {self.transform.posistion += forward}
        if r1.is_key_down(KeyboardKey::KEY_A) {self.transform.posistion -= right}
        if r1.is_key_down(KeyboardKey::KEY_S) {self.transform.posistion -= forward}
        if r1.is_key_down(KeyboardKey::KEY_D) {self.transform.posistion += right}
        if r1.is_key_down(KeyboardKey::KEY_SPACE) {self.transform.posistion.y -= self.camera_speed}
        if r1.is_key_down(KeyboardKey::KEY_LEFT_SHIFT) {self.transform.posistion.y += self.camera_speed}
        self.fov -= r1.get_mouse_wheel_move()/100.0;
        // Clamp fov so camera can't flip inside-out
        self.fov = self.fov.clamp(1.0_f32.to_radians(), 170.0_f32.to_radians());
    }
}