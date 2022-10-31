use crate::mesh::Mesh;
// use crate::utils::color::RGBColor;
// use crate::utils::geometry::Line;
// use crate::utils::geometry::Point;

use glam::{Vec3A, Vec3, Affine3A, Quat};
use ndarray::Array2;

// Avoid recurrent algorithm
// const MAX_DEPTH: i32 = 10;

const AMBIANT_LIGHT: Vec3A = Vec3A::ONE;
const DIFFUSE_LIGHT: Vec3A = Vec3A::ONE;
const SPECULAR_LIGHT: Vec3A = Vec3A::ONE;

pub struct RTEngine {
    pub pos_camera: Vec3A,
    pub pos_pixels: Array2<Vec3A>,
    pub pos_light: Vec3A,
    pub objects: Vec<Mesh>,
    pub material: Vec<Material>,
}

#[derive(Default, Copy, Clone)]
pub struct Sphere {
    pub center: Vec3A,
    pub radius: f32,
}

#[derive(Default, Copy, Clone)]
pub struct Material {
    pub ambiant: Vec3A,
    pub diffuse: Vec3A,
    pub specular: Vec3A,
    pub shininess: f32,
    pub reflection: f32,
}

impl RTEngine {
    /// The simplest ray tracing algorithm : path tracing
    /// Return an array with all colors
    pub fn path_tracing(&mut self) -> Array2<Vec3A> {
        let width: usize = self.pos_pixels.shape()[0];
        let height: usize = self.pos_pixels.shape()[1];

        let mut colors = Array2::<Vec3A>::default((width, height));
        for ((i, j), pixel) in self.pos_pixels.indexed_iter() {
            colors[[i, j]] = self._color_contribution(*pixel, 1);
        }

        return colors;
    }

    /// Return the color contribution on a pixel
    ///
    /// # Arguments
    ///
    /// `pixel` - (`Vec3A`) the position of the pixel
    /// `max_depth` - (`i32`) the maximum number of reflection and refraction
    fn _color_contribution(&self, pixel: Vec3A, max_depth: i32) -> Vec3A {
        let mut origin = self.pos_camera;
        let mut direction: Vec3A = (pixel - origin).normalize();
        // let color = RGBColor {
        //     ..Default::default()
        // }; // TODO : set background color

        let mut color = Vec3A::new(0., 0., 0.);

        let mut reflection = 1.;

        for _ in 0..max_depth {
            // println!("origin = {:?}, direction = {:?}", origin, direction);
            let (target_index, _, point_normal): (i32, f32, [Vec3A; 2]) =
                self._nearest_intersected_object(origin, direction);
            if target_index <= -1 {
                break;
            }
            // Object and material given the ray
            // let nearest_object: Mesh = self.objects[target_index as usize];
            let material: Material = self.material[target_index as usize];

            // Intersection computation
            let intersection = point_normal[0];
            println!("intersection = {:?}", intersection);
            println!("origin = {:?}, direction = {:?}", origin, direction);
            let normal_to_surface = point_normal[1].normalize();
            let shifted_point = intersection + 1e-5 * normal_to_surface;
            let intersection_to_light = (self.pos_light - shifted_point).normalize();

            let (_, min_distance, _): (i32, f32, [Vec3A; 2]) =
                self._nearest_intersected_object(shifted_point, intersection_to_light);
            let intersection_to_light_distance = (self.pos_light - intersection).length();
            // println!("min_distance = {:?}, intersection_to_light = {:?}", (shifted_point - point_normal[0]).length(), intersection_to_light_distance);
            if min_distance < intersection_to_light_distance {
                break;
            }

            let mut illumination: Vec3A = Vec3A::new(0., 0., 0.);

            // Ambiant contribution
            illumination += material.ambiant * AMBIANT_LIGHT;

            // Diffuse contribution
            illumination +=
                material.diffuse * DIFFUSE_LIGHT * intersection_to_light.dot(normal_to_surface);

            // Specular contribution
            let intersection_to_camera = (self.pos_camera - intersection).normalize();
            let h = (intersection_to_light + intersection_to_camera).normalize();
            illumination += material.specular
                * SPECULAR_LIGHT
                * normal_to_surface.dot(h).powf(material.shininess * 0.25);

            // Reflection
            color += reflection * illumination;
            reflection *= material.reflection;

            // New origin and direction
            origin = shifted_point.clone();
            direction = reflected(direction, normal_to_surface);
        }
        return clip(255. * color, 0., 255.);
    }

    fn _change_reference(reference: &Vec3A) -> [Affine3A; 2] {
        let translation = if (*reference - Vec3A::Z).length() == 0. { 
            Affine3A::IDENTITY 
        } else { 
            Affine3A::from_translation(Vec3::from(*reference) - Vec3::Z)
        };
        let rotation = if reference.cross(Vec3A::Z).length() == 0. {
            Affine3A::IDENTITY 
        } else {
            let mag = reference.length();
            let angle = if mag != 0. {(reference.dot(Vec3A::Z) / mag).acos()} else {0}
            let axis = Vec3A::Z.cross(reference);
            let quat = Quat::from_axis_angle(axis, angle);
            Affine3A::from_quat(quat);
        }
        [translation, rotation]
    }

    /// Return the index and the distance of
    /// the nearest intersected object of the collection
    ///
    /// # Arguments
    ///
    /// `ray_origin` - (`Vec3A`) origin of the ray
    /// `ray_direction` - (`Vec3A`) direction of the ray
    fn _nearest_intersected_object(&self, ray_origin: Vec3A, ray_direction: Vec3A) -> (i32, f32, [Vec3A; 2]) {
        let mut results : Vec<Option<[Vec3A; 2]>> = Vec::new();
        for obj in self.objects.iter() {
            results.push(obj.intersect(ray_origin, ray_direction));
        }
        let mut nearest_object: i32 = -1;
        let mut min_distance: f32 = std::f32::INFINITY;
        let mut sol : [Vec3A; 2] = [Vec3A::ONE, Vec3A::ONE];
        for (index, result) in results.iter().enumerate() {
            match result {
                Some(point_normal) => {
                    let distance = (ray_origin - point_normal[0]).length();
                    if distance < min_distance {
                        min_distance = distance;
                        nearest_object = index as i32;
                        sol = *point_normal;
                    }
                }
                None => { continue; }
            }
        }
        (nearest_object, min_distance, sol)
    }
}

/// Return the intersection distance on a sphere
/// between the ray origin and the intersection point
///
/// # Arguments
///
/// `center` - (`Vec3A`) center of the sphere
/// `radius` - (`f32`) radius of the sphere
/// `ray_origin` - (`Vec3A`) origin of the ray
/// `ray_direction` - (`Vec3A`) direction of the ray
#[allow(dead_code)]
fn sphere_intersect(center: Vec3A, radius: f32, ray_origin: Vec3A, ray_direction: Vec3A) -> f32 {
    let b = ray_direction.dot(ray_origin - center) * 2.;
    let c = (ray_origin - center).length_squared() - radius * radius;
    let delta = b * b - 4. * c;
    let mut result = -1.;
    if delta > 0. {
        let s = delta.sqrt();
        let t1 = (-b + s) * 0.5;
        let t2 = (-b - s) * 0.5;
        result = t1.min(t2);
    }
    result
}

/// Return the reflection ray according to
/// R = I - 2 (N.I) N where I is the incidence ray
/// and N is the surface normal
///
/// # Arguments
///
/// * `vector` - (`Vec3A`) the incidence ray
/// * `axis` - (`Vec3A`) the surface normal
fn reflected(vector: Vec3A, axis: Vec3A) -> Vec3A {
    vector - 2. * vector.dot(axis) * axis
}

/// Return the transmitted ray according to
/// T = eta_it I + (eta_it cos(theta_i) - sqrt(1 + eta_it^2 * (cos(theta_i)^2 - 1))) N
/// where I is the incidence ray and N is the surface normal
///
/// # Arguments
///
/// * `vector` - (`Vec3A`) the incidence ray
/// * `axis` - (`Vec3A`) the surface normal
/// * `eta_i` - (`f32`) eta_i the index of refraction of medium i respectively
/// * `eta_t` - (`f32`) eta_t the index of refraction of medium t respectively
#[allow(dead_code)]
fn transmitted(vector: Vec3A, axis: Vec3A, eta_i: f32, eta_t: f32) -> Vec3A {
    let etait: f32 = eta_i / eta_t;
    let cosi: f32 = vector.angle_between(axis);
    let before_root: f32 = 1. + etait * etait * (cosi * cosi - 1.);
    let mut result: Vec3A = Vec3A::new(0., 0., 0.);
    if before_root >= 0. {
        result = etait * vector + (etait * cosi - before_root.sqrt()) * axis;
    }
    result
}

/// Return a vector where values are clipped between `a_min` and `a_max`
///
/// # Arguments
///
/// * `vector` - (`Vec3A`) the input
/// * `a_min` - (`f32`) the lower limit
/// * `a_max` - (`f32`) the upper limit
fn clip(vector: Vec3A, a_min: f32, a_max: f32) -> Vec3A {
    let one: Vec3A = Vec3A::new(1., 1., 1.);
    vector.min(one * a_max).max(one * a_min)
}
