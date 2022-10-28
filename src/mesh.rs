// use crate::utils::intersect_triangle;
use stl::read_stl;
use glam::Vec3A;
use std::fs::File;
use std::io::Result;
use rstar::RTree;
use std::collections::{HashMap, HashSet};

use crate::utils::intersect_triangle;

type Tree = HashMap<u32, RTree<[f32; 3]>>;

#[derive(Default, Copy, Clone)]
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
    pub rtree: Tree,
    pub radii: Vec<u32>,
    pub hashtable: HashMap<i32, Triangle>,
    pub max_radius: f32,
}

fn _to_triangle(triangle: &stl::Triangle) -> Triangle {        
    let convert = |array: [f32; 3]| Vec3A::new(array[0], array[1], array[2]);
    let p1 = convert(triangle.v1);
    let p2 = convert(triangle.v2);
    let p3 = convert(triangle.v3);
    let center = (p1 + p2 + p3) / 3.;
    Triangle { p1, p2, p3, normal: convert(triangle.normal), center}
}

fn hash_function(v: &Vec3A) -> i32 {
    let unique = |x, y, z| z / ((x + y) * (x + y) + x + y + 1.);
    (unique(v.x, v.y, v.z) * 100_000.) as i32
}

impl Mesh {
    pub fn new(file: &mut File) -> Result<Self> {
        let binary = read_stl(file);        
        match binary {
            Ok(x) => {
                // Initialization of triangles
                let mut triangles: Vec<Triangle> = Vec::new();
                let mut mesh_center:Vec3A = Vec3A::ZERO;
                let convert = |array: [f32; 3]| Vec3A::new(array[0], array[1], array[2]);
                for triangle in x.triangles.iter() {                    
                    let p1 = convert(triangle.v1);
                    let p2 = convert(triangle.v2);
                    let p3 = convert(triangle.v3);
                    let center = (p1 + p2 + p3) / 3.;
                    triangles.push(Triangle{p1, p2, p3, normal: convert(triangle.normal), center});
                }
                mesh_center /= x.triangles.len() as f32;
                let get_radii = |t:&Triangle| [t.p1, t.p2, t.p3].iter().map(|x| (*x - mesh_center).length()).fold(f32::NAN, f32::max);
                let max_radius = triangles.iter().map(get_radii).fold(f32::NAN, f32::max);

                // Initialization of tree
                let mut rtree: Tree = HashMap::new();
                let mut hs: HashSet<u32> = HashSet::new();
                for center in triangles.iter().map(|t| t.center){
                    let radius: u32 = ((center - mesh_center).length() * 100_000.) as u32;
                    hs.insert(radius);
                    let array = center.to_array();
                    if !rtree.contains_key(&radius){ rtree.insert(radius, RTree::new()); }
                    if let Some(x) = rtree.get_mut(&radius) { x.insert(array); }
                }

                let mut radii = Vec::from_iter(hs);
                radii.sort();
                
                let hashtable: HashMap<i32, Triangle> = triangles.iter().map(|t| (hash_function(&t.center), *t)).collect();
                Ok(Mesh { triangles, center: mesh_center, rtree, radii, hashtable, max_radius})
            }
            Err(x) => return Err(x)
        }
    }

    fn _smallest_spherical_intersection(&self, ray_origin: Vec3A, ray_direction: Vec3A) -> Option<Vec3A> {
        let i = ray_origin - ray_direction.dot(ray_origin - self.center) / ray_direction.length_squared() * ray_direction;
        if (i - ray_origin).dot(ray_direction) > 0. { Some(i) } else { None }
    }

    fn _spherical_intersection(&self, radius: f32, ray_origin: Vec3A, ray_direction: Vec3A) -> Option<Vec3A> {
        let oa = ray_origin - self.center;
        let a = ray_direction.length_squared();
        let b = 2. * oa.dot(ray_direction);
        let c = oa.length_squared() - radius * radius;
        let delta = b * b - 4. * a * c;
        // println!("ray_origin = {:?}", ray_origin);
        // println!("ray_direction = {:?}", ray_direction);
        // println!("radius = {:?}", radius);
        // println!("center = {:?}", self.center);
        // println!("a = {:?}", a);
        // println!("b = {:?}", b);
        // println!("c = {:?}", c);
        // println!("delta = {:?}", delta);
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
    }

    fn _search(&self, target: u32) -> usize {
        let length = self.radii.len();
        if self.radii[0] > target {
            0
        }
        else if self.radii[length - 1] <= target {
            length - 1
        }
        else {
            let mut d = 4;
            let mut middle = length / 2;
            while !(self.radii[middle - 1] <= target && target <= self.radii[middle]){
                let a = length / d;
                if self.radii[middle - 1] == self.radii[middle] {
                    break;
                } else if target < self.radii[middle] {
                    middle -= if a != 0 { a } else {1};
                } else {
                    middle += if a != 0 { a } else {1};
                }
                d = 2 * d.min(length);
            }
            middle
        }
    }

    pub fn intersect(&self, ray_origin: Vec3A, ray_direction: Vec3A) -> Option<[Vec3A; 2]> {
        match self._smallest_spherical_intersection(ray_origin, ray_direction) {
            Some(imin) => {
                // println!("imin = {:?}", imin);
                match self._spherical_intersection(self.max_radius, ray_origin, ray_direction) {
                    Some(imax) => {
                        // println!("imax = {:?}", imax);
                        let get_radius = |x: Vec3A| -> f32 { (x - self.center).length() };
                        let to_u32 = |x:f32| (x * 100_000.) as u32;
                        let to_f32 = |x:u32| x as f32 * 1e-6;
                        let min_radius = get_radius(imin);
                        let index_min = self._search(to_u32(min_radius));
                        let index_min = if index_min > 0 { 0.min(index_min - 1) } else { index_min };
                        let mut index = self._search(to_u32(get_radius(imax)));
                        let mut radius = self.radii[index];
                        let mut current_point = imax;
                        let mut result : Option<[Vec3A; 2]> = None;
                        let mut found = false;
                        while index >= index_min {
                            let candidates = self.rtree[&radius].nearest_neighbor_iter(&current_point.to_array());
                            for triangle in candidates.map(|x| self.hashtable[&hash_function(&Vec3A::from_array(*x))]) {
                                match intersect_triangle(triangle.p1, triangle.p2, triangle.p3, ray_origin, ray_direction) {
                                    Some(sol) => { result = Some([sol, triangle.normal]); found = true; break; }
                                    None => { continue; }
                                }
                            }
                            if found { break; }
                            if index == 0 { break; } else {index = index - 1};
                            radius = self.radii[index];
                            match self._spherical_intersection(to_f32(radius), ray_origin, ray_direction) {
                                Some(point) => {current_point = point }
                                None => { break; }
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
