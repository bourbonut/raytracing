use glam::{Vec3A, Mat3A};

#[allow(non_snake_case)]
pub fn barycentric_coordinates(target:Vec3A, A:Vec3A, B:Vec3A, C:Vec3A) -> [f32; 2]{
    let AC = A - C;
    let BC = B - C;
    let TC = target - C;
    let M = Mat3A::from_cols(AC, BC, AC.cross(BC));
    let sol = M.inverse() * TC;
    [sol.x, sol.y]
}

pub fn intersect_plane(point:Vec3A, normal:Vec3A, ray_origin:Vec3A, ray_direction:Vec3A) -> Option<Vec3A> {
    let denom = ray_direction.dot(normal);
    if denom == 0.0 {
        None
    } else {
        Some(ray_origin + ((point - ray_origin).dot(normal) / denom) * ray_direction)
    }
}

#[allow(non_snake_case)]
pub fn intersect_triangle(P1:Vec3A, P2:Vec3A, P3:Vec3A, ray_origin:Vec3A, ray_direction:Vec3A) -> Option<Vec3A> {
    let normal = (P2 - P1).cross(P3 - P1);
    let I = intersect_plane(P1, normal, ray_origin, ray_direction);
    match I {
        Some(x) => {
            let sol = barycentric_coordinates(x, P1, P2, P3);
            let u = sol[0];
            let v = sol[1];
            if u + v <= 1. && 0. <= u && u <= 1. && 0. <= v && v <= 1. {
                I
            } else {
                None
            }
        }
        None => None
    }
}
