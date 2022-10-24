use crate::utils::intersect_triangle;
use stl::read_stl;
use glam::{Vec3A, Vec2};
use std::fs::File;
use std::io::Result;

type Tree = Vec<(usize, f32)>;

#[derive(Default, Clone)]
pub struct Triangle{
    pub p1: Vec3A,
    pub p2: Vec3A,
    pub p3: Vec3A,
    pub normal: Vec3A,
    pub center: Vec3A,
}

#[derive(Default, Clone)]
pub struct Mesh{
    pub triangles: Vec<Triangle>,
    pub center: Vec3A,
    pub tree: Tree,
}

fn convert(array: [f32; 3]) -> Vec3A {
    Vec3A::new(array[0], array[1], array[2])
}

fn semi_spherical_coordinates(v: &Vec3A) -> Vec2{
    let rho = v.length();
    if rho == 0. { Vec2::new(0., 0.) }
    else {
        let delta = v.z / rho; // without asin
        let theta = if v.x != 0. {v.y / v.x} else { v.y.signum() } ; // without atan2
        Vec2::new(delta, theta)
    }
}

impl Mesh {
    pub fn new(file: &mut File) -> Result<Self> {
        let binary = read_stl(file);        
        match binary {
            Ok(x) => {
                // Initialization of triangles
                let mut triangles: Vec<Triangle> = Vec::new();
                let mut mesh_center = Vec3A::new(0., 0., 0.);
                for t in x.triangles.iter(){
                    let p1 = convert(t.v1);
                    let p2 = convert(t.v2);
                    let p3 = convert(t.v3);
                    let center = (p1 + p2 + p3) / 3.;
                    mesh_center += center;
                    triangles.push(Triangle { p1, p2, p3, normal: convert(t.normal), center})
                }
                mesh_center /= x.triangles.len() as f32;

                // Initialization of tree
                let mut tree: Tree = Vec::new();
                for (index, triangle) in triangles.iter().enumerate(){
                    let v = triangle.center - mesh_center;
                    let spherical_loc = semi_spherical_coordinates(&v);
                    let radius = spherical_loc.length();
                    tree.push((index, radius));
                }

                tree.sort_by(|a, b| (&a.1).partial_cmp(&b.1).unwrap());

                Ok(Mesh { triangles, center: mesh_center, tree})
            }
            Err(x) => return Err(x)
        }
    }


    fn _smallest_spherical_intersection(&self, ray_origin: Vec3A, ray_direction: Vec3A) -> Option<Vec3A> {
        let i = ray_origin - ray_direction.dot(ray_origin - self.center) / ray_direction.length_squared() * ray_direction;
        if (i - ray_origin).dot(ray_direction) > 0. { Some(i) } else { None }
    }

    fn _biggest_spherical_intersection(&self, ray_origin: Vec3A, ray_direction: Vec3A) -> Option<Vec3A> {
        let oa = ray_origin - self.center;
        if let Some(last) = self.tree.last() { 
            let greatest_radius = last.1;
            let a = ray_direction.length_squared();
            let b = 2. * oa.dot(ray_direction);
            let c = oa.length_squared() - greatest_radius * greatest_radius;
            let delta = b * b - 4. * a * c;
            if delta > 0. {
                let sdelta = delta.sqrt();
                let t1 = (-b - sdelta) / (2. * a);
                let t2 = (-b + sdelta) / (2. * a);
                Some(ray_origin + t1.min(t2) * ray_direction)
            } else if delta == 0. {
                let t = -b / (2. * a);
                Some(ray_origin + t * ray_direction)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn _search(&self, target: f32) -> usize {
        let length = self.tree.len();
        if self.tree[0].1 > target {
            0
        }
        else if self.tree[length - 1].1 <= target {
            length - 1
        }
        else {
            let mut d = 4;
            let mut middle = length / 2;
            while !(self.tree[middle - 1].1 <= target && target <= self.tree[middle].1){
                let a = length / d;
                if self.tree[middle - 1].1 == self.tree[middle].1 {
                    break;
                } else if target < self.tree[middle].1 {
                    middle -= if a != 0 { a } else {1};
                } else {
                    middle += if a != 0 { a } else {1};
                }
                d = 2 * d.min(length);
            }
            middle
        }
    }

    fn _get_candidates(&self, rmin: f32, rmax:f32) -> Vec<usize>{
        let start = self._search(rmin);
        let end = self._search(rmax);
        // let mut indices: Vec<usize> = Vec::new();
        // for i in start..end {
        //     indices.push(self.tree[i].0);
        // }
        // indices
        self.tree[start..end].iter().map(|x| x.0).collect()
    }

    pub fn intersect(&self, ray_origin: Vec3A, ray_direction: Vec3A) -> Option<[Vec3A; 2]> {
        match self._smallest_spherical_intersection(ray_origin, ray_direction) {
            Some(imin) => {
                match self._biggest_spherical_intersection(ray_origin, ray_direction) {
                    Some(imax) => {
                        let smin = semi_spherical_coordinates(&(imin - self.center));
                        let smax = semi_spherical_coordinates(&(imax - self.center));
                        let middle = 0.5 * (smin + smax);
                        let circle_radius = (middle - smin).length();
                        let radius = middle.length();
                        let rmin = radius - circle_radius;
                        let rmax = radius + circle_radius;
                        let candidates = self._get_candidates(rmin, rmax);
                        let mut result: Option<[Vec3A; 2]> = None;
                        for index in candidates.iter() {
                            let triangle = &self.triangles[*index];
                            match intersect_triangle(triangle.p1, triangle.p2, triangle.p3, ray_origin, ray_direction) {
                                Some(sol) => { result = Some([sol, triangle.normal]); break; }
                                None => { continue; }
                            }
                        }
                        result
                    }
                    None => None
                }
            }
            None => None
        }
    }
}
