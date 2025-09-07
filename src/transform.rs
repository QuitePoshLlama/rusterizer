use crate::point3d::Point3D;

pub struct Transform {
    pub yaw: f32,
    pub pitch: f32,
    pub posistion: Point3D,
}

fn transform_vector(ihat: Point3D, jhat: Point3D, khat: Point3D, v: Point3D) -> Point3D {
    ihat * v.x + jhat * v.y + khat * v.z
}

impl Transform {
    pub fn update_transform(&mut self, new_yaw: f32, new_pitch: f32, new_position: Point3D) {
        self.yaw = new_yaw;
        self.pitch = new_pitch;
        self.posistion = new_position;
    }
    pub fn get_basis_vectors(&self) -> (Point3D, Point3D, Point3D) {
        let ihat_yaw = Point3D { x: self.yaw.cos(), y: 0.0, z: self.yaw.sin() };
        let jhat_yaw = Point3D { x: 0.0, y: 1.0, z: 0.0 };
        let khat_yaw = Point3D { x: -self.yaw.sin(), y: 0.0, z: self.yaw.cos() };
        let ihat_pitch = Point3D { x: 1.0, y: 0.0, z: 0.0 };
        let jhat_pitch = Point3D { x: 0.0, y: self.pitch.cos(), z: -self.pitch.sin() };
        let khat_pitch = Point3D { x: 0.0, y: self.pitch.sin(), z: self.pitch.cos() };
        let ihat = transform_vector(ihat_yaw, jhat_yaw, khat_yaw, ihat_pitch);
        let jhat = transform_vector(ihat_yaw, jhat_yaw, khat_yaw, jhat_pitch);
        let khat = transform_vector(ihat_yaw, jhat_yaw, khat_yaw, khat_pitch);
        (ihat, jhat, khat)
    }

    pub fn get_inverse_basis_vectors(&self) -> (Point3D, Point3D, Point3D) {
        let (ihat, jhat, khat) = self.get_basis_vectors();
        let inv_ihat = Point3D{x: ihat.x, y: jhat.x, z: khat.x};
        let inv_jhat = Point3D{x: ihat.y, y: jhat.y, z: khat.y};
        let inv_khat = Point3D{x: ihat.z, y: jhat.z, z: khat.z};
        (inv_ihat, inv_jhat, inv_khat)
    }

    pub fn to_world_point(&self, point: Point3D) -> Point3D {
        let (ihat, jhat, khat) = self.get_basis_vectors();
        transform_vector(ihat, jhat, khat, point) + self.posistion
    }

    pub fn to_local_point(&self, worldpoint: Point3D) -> Point3D {
        let (ihat, jhat, khat) = self.get_inverse_basis_vectors();
        transform_vector(ihat, jhat, khat, worldpoint - self.posistion)
    }

    pub fn transform_direction(&self, dir: Point3D) -> Point3D {
        let (ihat, jhat, khat) = self.get_basis_vectors();
        transform_vector(ihat, jhat, khat, dir)
    }
}
