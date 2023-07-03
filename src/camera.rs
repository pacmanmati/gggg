use nalgebra::{Isometry3, Matrix4, Orthographic3, Perspective3, Point3, Vector3};

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, -1.0, 0.0, 0.0,
    0.0, 0.0, 1.0, 0.0,
    0.0, 0.0, 0.0, 1.0,
);

pub enum ProjectionType {
    Perspective {
        aspect: f32,
        fovy: f32,
        near: f32,
        far: f32,
    },
    Orthographic {
        left: f32,
        right: f32,
        top: f32,
        bottom: f32,
        near: f32,
        far: f32,
    },
}

pub struct Camera {
    pub eye: Point3<f32>,
    pub target: Point3<f32>,
    projection_type: ProjectionType,
}

impl Camera {
    /// Creates a new [`Camera`].
    pub fn new(eye: Point3<f32>, target: Point3<f32>, projection_type: ProjectionType) -> Self {
        Self {
            eye,
            target,
            projection_type,
        }
    }

    pub fn view(&self) -> Matrix4<f32> {
        Isometry3::look_at_rh(&self.eye, &self.target, &Vector3::y()).to_homogeneous()
    }

    pub fn projection(&self) -> Matrix4<f32> {
        match self.projection_type {
            ProjectionType::Perspective {
                aspect,
                fovy,
                near,
                far,
            } => Perspective3::new(aspect, fovy, near, far).into_inner(),

            ProjectionType::Orthographic {
                left,
                right,
                bottom,
                top,
                near,
                far,
            } => Orthographic3::new(left, right, bottom, top, near, far).into_inner(),
        }
    }

    pub fn view_projection(&self) -> Matrix4<f32> {
        self.projection() * self.view()
        // let eye = Point3::new(0.0, 0.0, 1.0);
        // let target = Point3::new(1.0, 0.0, 0.0);

        // let model = Isometry3::new(Vector3::x(), nalgebra::zero());
        // let projection = Perspective3::new(16.0 / 9.0, 3.14 / 2.0, 1.0, 1000.0);
        // let model_view_projection = projection.into_inner() * (view * model).to_homogeneous();
        // model_view_projection
    }

    pub fn model_view_projection(&self, model: Matrix4<f32>) -> Matrix4<f32> {
        OPENGL_TO_WGPU_MATRIX * self.projection() * (self.view() * model)
    }
}
