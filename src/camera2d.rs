use nalgebra as na;

pub type Point2 = na::Point2<f32>;
pub type Point3 = na::Point3<f32>;
pub type Point4 = na::Point4<f32>;
pub type Mat3 = na::Matrix3<f32>;
pub type Mat4 = na::Matrix4<f32>;
pub type Vec3 = na::Vector3<f32>;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScreenPoint {
    pub x: f32,
    pub y: f32,
}

pub fn ortho_lh_zo<N: na::RealField>(
    left: N,
    right: N,
    bottom: N,
    top: N,
    znear: N,
    zfar: N,
) -> na::Matrix4<N> {
    let one: N = N::one();
    let two: N = na::convert(2.0);
    let mut mat: na::Matrix4<N> = na::Matrix4::<N>::identity();

    mat[(0, 0)] = two / (right - left);
    mat[(0, 3)] = -(right + left) / (right - left);
    mat[(1, 1)] = two / (top - bottom);
    mat[(1, 3)] = -(top + bottom) / (top - bottom);
    mat[(2, 2)] = one / (zfar - znear);
    mat[(2, 3)] = -znear / (zfar - znear);
    mat
}

pub fn ortho(w: f32, h: f32) -> na::Matrix4<f32> {
    ortho_lh_zo(0.0, w, 0.0, h, 1.0, 100.0)
}

pub fn orthographic_projection2(width: f32, height: f32) -> nalgebra::Matrix4<f32> {
    // #[cfg_attr(rustfmt, rustfmt_skip)]
    nalgebra::Matrix4::new(
        2.0 / width as f32,
        0.0,
        0.0,
        0.0,
        0.0,
        2.0 / height as f32,
        0.0,
        0.0,
        0.0,
        0.0,
        1.0,
        0.0,
        -1.0,
        -1.0,
        0.0,
        1.0,
    )
    .transpose()
}

// func textureTranslation(from viewTranslation: CGPoint,
//     in textureView: MTKView) -> SIMD2<Float> {
//     let textureViewSize: SIMD2<Float> = .init(.init(textureView.frame.width),
//                             .init(textureView.frame.height))
//     var translation: SIMD2<Float> = .init(.init(viewTranslation.x),
//                     .init(viewTranslation.y))
//     translation /= textureViewSize // normalize

//     return translation
// }

fn texture_translation(view_translation: (f32, f32), outer_size: (f32, f32)) -> (f32, f32) {
    let translation = (
        view_translation.0 / outer_size.0,
        view_translation.1 / outer_size.1,
    );
    translation
}

// func texturePoint(from windowPoint: CGPoint,
//     in textureView: MTKView) -> SIMD2<Float> {
//     let textureViewSize: SIMD2<Float> = .init(.init(textureView.frame.width),
//                                     .init(textureView.frame.height))
//     var point: SIMD2<Float> = .init(.init(windowPoint.x),
//                         .init(windowPoint.y))
//     point /= textureViewSize // normalize
//     point *= 2               // convert
//     point -= 1               // to metal

//     let result: SIMD4<Float> = simd_float4x4(self.textureTransform.inversed)
//                 * .init(point.x, point.y, 0, 1)
//     return .init(result.x,
//     result.y)
// }
// converts it to ndc
fn texture_point(
    transform: &Mat4,
    window_point: ScreenPoint,
    outer_size: (f32, f32), // called texture size
) -> (f32, f32) {
    let mut point = (window_point.x, window_point.y);

    point = (point.0 / outer_size.0, point.1 / outer_size.1); // normalize
    point = (point.0 * 2.0, point.1 * 2.0); // convert
    point = (point.0 - 1.0, point.1 - 1.0); // to metal

    let point = Point4::new(point.0, point.1, 0.0, 1.0);

    let res = transform.try_inverse().unwrap() * point;
    (res.x, res.y)
}

// this is analog to orthogonal projection
// inner_size = texture_size
// outer_size = view_size
fn aspect_ratio_transform(inner_size: (f32, f32), outer_size: (f32, f32)) -> Mat4 {
    let inner_aspect_ratio = inner_size.0 / inner_size.1;
    let outer_aspect_ratio = outer_size.0 / outer_size.1;
    let scale_x = inner_aspect_ratio / outer_aspect_ratio;
    // println!("scale_x {:?}", scale_x);
    let scaling = Vec3::new(scale_x, 1.0, 1.0);
    Mat4::new_nonuniform_scaling(&scaling)
}

// inner size is the size of the image
// outer size is the size of the view

pub struct Camera2D {
    // view size
    outer_size: (f32, f32),
    transform: Mat4,
    scale: f32,
}

impl Camera2D {
    pub fn new(outer_size: (f32, f32)) -> Self {
        Self {
            outer_size,
            transform: Mat4::identity(),
            scale: 1.0,
        }
    }

    pub fn zoom_to(&mut self, zoom_point: ScreenPoint, scale: f32) {
        // let ScreenPoint { x, y } = zoom_point;

        let zp = texture_point(&self.transform, zoom_point, self.outer_size);
        println!("zoom_point: {:?}", zp);
        let scale = 1.0 + scale;
        self.scale *= scale;

        let shift = Vec3::new(zp.0, zp.1, 0.0);
        println!("shift: {:?}", shift);
        // we are using scale instead of self.scale since self.transform already is scaled by self.scale
        let scaling = {
            let s = scale as f32;
            Vec3::new(s, s, 1.0)
        };
        let to_point = Mat4::identity().append_translation(&shift);
        let from_point = Mat4::identity().append_translation(&-shift);
        let s = Mat4::identity().append_nonuniform_scaling(&scaling);

        // self.transform = self
        //     .transform
        //     .append_translation(&shift)
        //     .append_nonuniform_scaling(&scaling)
        //     .append_translation(&-shift);
        self.transform = self.transform * to_point * s * from_point;
        // self.transform = to_point * (s * (from_point * self.transform));
    }

    pub fn scroll(&mut self, delta: (f32, f32)) {
        // flip coordinates
        let delta = (delta.0, -delta.1);
        let translation = texture_translation(delta, self.outer_size);
        let shift = Vec3::new(translation.0 / self.scale, translation.1 / self.scale, 0.0);
        self.transform = self.transform.append_translation(&shift);
    }

    pub fn transform(&self) -> Mat4 {
        self.transform * orthographic_projection2(self.outer_size.0, self.outer_size.1)
    }
}
