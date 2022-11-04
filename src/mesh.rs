use crate::path::Path;
use stl::read_stl;
use glam::Vec3A;
use std::fs::File;
use std::io::Result;
use std::slice::Iter;
// use rstar::RTree;
use std::collections::{HashMap, HashSet};

use crate::utils::{intersect_square, intersect_triangle};

// type Neighbors = Vec<HashSet<usize>>;

#[derive(Default, Copy, Clone)]
pub struct Triangle{
    pub p1: Vec3A,
    pub p2: Vec3A,
    pub p3: Vec3A,
    pub normal: Vec3A,
    pub center: Vec3A,
}

#[derive(Default, Clone)]
pub struct Package{
    pub bound_min: Vec3A,
    pub bound_max: Vec3A,
    pub bounds: [[Vec3A; 4]; 6],
}

#[derive(Default, Clone)]
pub struct Tree{
    pub keys: HashMap<[u32; 3], usize>,
    pub values: Vec<HashSet<usize>>,
}

#[derive(Default, Clone)]
pub struct Mesh{
    pub triangles: Vec<Triangle>,
    pub center: Vec3A,
    pub tree: Tree,
    pub package: Package,
    pub dx: u32,
    pub dy: u32,
    pub dz: u32,
    pub unit: f32,
}

impl Package {
    pub fn new(bound_min: Vec3A, bound_max:Vec3A) -> Self {
        let x_min = bound_min.x;
        let y_min = bound_min.y;
        let z_min = bound_min.z;
        let x_max = bound_max.x;
        let y_max = bound_max.y;
        let z_max = bound_max.z;
        let bounds = [
            [
                Vec3A::new(x_min, y_min, z_min),
                Vec3A::new(x_min, y_max, z_min),
                Vec3A::new(x_min, y_min, z_max),
                Vec3A::new(x_min, y_max, z_max)
            ],
            [
                Vec3A::new(x_max, y_min, z_min),
                Vec3A::new(x_max, y_max, z_min),
                Vec3A::new(x_max, y_min, z_max),
                Vec3A::new(x_max, y_max, z_max)
            ],
            [
                Vec3A::new(x_min, y_min, z_min),
                Vec3A::new(x_min, y_min, z_max),
                Vec3A::new(x_max, y_min, z_min),
                Vec3A::new(x_max, y_min, z_max),
            ],
            [
                Vec3A::new(x_min, y_max, z_min),
                Vec3A::new(x_min, y_max, z_max),
                Vec3A::new(x_max, y_max, z_min),
                Vec3A::new(x_max, y_max, z_max),
            ],
            [
                Vec3A::new(x_min, y_min, z_min),
                Vec3A::new(x_min, y_max, z_min),
                Vec3A::new(x_max, y_min, z_min),
                Vec3A::new(x_max, y_max, z_min),
            ],
            [
                Vec3A::new(x_min, y_min, z_max),
                Vec3A::new(x_min, y_max, z_max),
                Vec3A::new(x_max, y_min, z_max),
                Vec3A::new(x_max, y_max, z_max),
            ]
        ];
        Package { bound_min, bound_max, bounds }
    }
    
    pub fn iter(&self) -> Iter<[Vec3A; 4]> {
        self.bounds.iter()
    }
}

impl Tree {
    pub fn contains(&self, key: &[u32; 3], value: &usize) -> bool {
        self.values[self.keys[key]].contains(value)
    }

    pub fn contains_key(&self, key: &[u32; 3]) -> bool {
        self.keys.contains_key(key)
    }

    pub fn insert_key(&mut self, key:[u32; 3]) {
        let length = self.values.len();
        self.keys.insert(key, length);
        self.values.push(HashSet::new());
    }

    pub fn insert(&mut self, key: &[u32; 3], value: usize) -> bool {
        self.values[self.keys[key]].insert(value)
    }

    pub fn get(&self, key: &[u32; 3]) -> &HashSet<usize> {
        &self.values[self.keys[key]]
    }
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

                // Get the bounds of the mesh and the minimum distance
                let mut bound_min = Vec3A::ONE * f32::INFINITY;
                let mut bound_max = -Vec3A::ONE * f32::INFINITY;
                let mut unit = f32::INFINITY;
                for triangle in triangles.iter() {
                    bound_max = bound_max.max(triangle.p1).max(triangle.p2).max(triangle.p3);
                    bound_min = bound_min.min(triangle.p1).min(triangle.p2).min(triangle.p3);
                    let a = (triangle.p1 - triangle.p2).length();
                    let b = (triangle.p2 - triangle.p3).length();
                    let c = (triangle.p3 - triangle.p1).length();
                    unit = unit.min(a).min(b).min(c);
                }
                let dbound = bound_max - bound_min;
                let dx = dbound.x.abs().div_euclid(unit) as u32 + 1;
                let dy = dbound.y.abs().div_euclid(unit) as u32 + 1;
                let dz = dbound.z.abs().div_euclid(unit) as u32 + 1;

                let bound_min = bound_min - 0.5 * unit * Vec3A::ONE;
                let bound_max = bound_max + 0.5 * unit * Vec3A::ONE;

                let mut tree: Tree = Tree {keys: HashMap::new(), values: Vec::new()};
                // let mut neighbors: Neighbors = Vec::new();
                // for _ in 0..triangles.len() {
                //     neighbors.push(HashSet::new());
                // }
                let div = |x: f32| x.div_euclid(unit) as u32;
                for (index, triangle) in triangles.iter().enumerate() {
                    let mut keys: [[u32; 3]; 3] = [[0, 0, 0], [0, 0, 0], [0, 0, 0]];
                    let points = [triangle.p1, triangle.p2, triangle.p3];
                    for (i, point) in points.iter().enumerate() {
                        // println!("1");
                        let point = *point - bound_min;
                        let xdiv = div(point.x);
                        let ydiv = div(point.y);
                        let zdiv = div(point.z);
                        // println!("xdiv = {:?}", xdiv);
                        // println!("ydiv = {:?}", ydiv);
                        // println!("zdiv = {:?}", zdiv);
                        // println!("dx = {:?}", dx);
                        // println!("dy = {:?}", dy);
                        // println!("dz = {:?}", dz);
                        // println!("total = {:?}", dz * dy * dx);
                        keys[i] = [xdiv, ydiv, zdiv];
                        // println!("2");
                    }
                    // for key in keys.iter() {
                    //         if !tree.keys.contains_key(key) { tree.insert_key(*key);}
                    //         tree.insert(&key, index);
                    // }
                    let mut global_path = Path::new(keys[0], points[0] - bound_min, keys[1], points[1] - bound_min, unit, dx, dy, dz);
                    while global_path.next() {
                        let mut local_path = Path::new(keys[2], points[2] - bound_min, global_path.current_key, global_path.current_point, unit, dx, dy, dz);
                        while local_path.next() {
                            let key = local_path.current_key;
                            // println!("key = {:?}", key);
                            if !tree.keys.contains_key(&key) { tree.insert_key(key);}
                            tree.insert(&key, index);
                        }
                    }
                }

                // println!("tree.keys = {:?}", tree.keys);
                // println!("tree.values = {:?}", tree.values);
                // println!("unit = {:?}", unit);
                // println!("bound_min = {:?}", bound_min);
                // println!("bound_max = {:?}", bound_max);
                // println!("dx = {:?}", dx);
                // println!("dy = {:?}", dy);
                // println!("dz = {:?}", dz);
                // println!("total points = {:?}", 3 * triangles.len());
                // println!("total tree = {:?}", tree.values.iter().map(|x: &HashSet<usize>| -> usize { x.len() }).collect::<Vec<usize>>());
                let package = Package::new(bound_min, bound_max);
                Ok(Mesh { triangles, center: mesh_center, tree, package, dx, dy, dz, unit})
            }
            Err(x) => return Err(x)
        }
    }

    fn _div(&self, x: f32) -> u32 {
        x.div_euclid(self.unit) as u32
    }

    fn _compute_key(&self, point: Vec3A) -> [u32; 3] {
        // println!("point = {:?}", point);
        let point = point - self.package.bound_min;
        // println!("point = {:?}", point);
        let xdiv = self._div(point.x).min(self.dx - 1);
        let ydiv = self._div(point.y).min(self.dy - 1);
        let zdiv = self._div(point.z).min(self.dz - 1);
        // println!("xdiv = {:?}", xdiv);
        // println!("ydiv = {:?}", ydiv);
        // println!("zdiv = {:?}", zdiv);
        // println!("self.dx = {:?}", self.dx);
        // println!("self.dy = {:?}", self.dy);
        // println!("self.dz = {:?}", self.dz);
        // println!("self.package.bound_min = {:?}", self.package.bound_min);
        // println!("self.package.bound_max = {:?}", self.package.bound_max);
        [xdiv, ydiv, zdiv]
    }

    fn _hit_package(&self, ray_origin: Vec3A, ray_direction: Vec3A) -> Option<[([u32; 3], Vec3A); 2]> {
        let intersect = |x: &[Vec3A; 4]| -> Option<Vec3A> {intersect_square(x[0], x[1], x[2], x[3], ray_origin, ray_direction)};
        let compute_dist = |x: &Option<Vec3A>| -> f32 { if let Some(i) = x { (*i - ray_origin).length_squared() } else { f32::INFINITY }};
        let intersections = self.package.iter().map(intersect).filter(|x| x.is_some()).collect::<Vec<Option<Vec3A>>>();
        // println!("intersections = {:?}", intersections);
        // println!("dists = {:?}", intersections.iter().map(compute_dist).collect::<Vec<f32>>());
        if intersections.len() > 0 {
            let in_intersection = intersections.iter().min_by(|a, b| compute_dist(a).partial_cmp(&compute_dist(b)).unwrap());
            let out_intersection = intersections.iter().max_by(|a, b| compute_dist(a).partial_cmp(&compute_dist(b)).unwrap());
            if let Some(Some(in_point)) = in_intersection {
                if let Some(Some(out_point)) = out_intersection {
                        Some([(self._compute_key(*in_point), *in_point), (self._compute_key(*out_point), *out_point)]) 
                } else { None }
            } else { None }
        } else { None }
    }

    pub fn intersect(&self, ray_origin: Vec3A, ray_direction: Vec3A) -> Option<[Vec3A; 2]> {
        if let Some([(in_key, in_point), (out_key, out_point)]) = self._hit_package(ray_origin, ray_direction) {
            // println!("in_key = {:?}", in_key);
            // println!("out_key = {:?}", out_key);
            // println!("ray_origin = {:?}", ray_origin);
            // println!("ray_direction = {:?}", ray_direction);
            // println!("unit = {:?}", self.unit);
            // println!("tree.keys = {:?}", self.tree.keys);
            // println!("tree.values = {:?}", self.tree.values);
            
            // for DEBUG
            // for (index, triangle) in self.triangles.iter().enumerate() {
            //     if let Some(x) = intersect_triangle(triangle.p1, triangle.p2, triangle.p3, ray_origin, ray_direction) {
            //         println!("distance to ray_origin = {:?}", (x - ray_origin).length());
            //         println!("index of triangle = {:?}", index);
            //         println!("point of intersection = {:?}", x);
            //         // let normal = (triangle.p2 - triangle.p1).cross(triangle.p3 - triangle.p1);
            //         // let denom = normal.dot(ray_direction);
            //         // println!("t = {:?}", (triangle.p1 - ray_origin).dot(normal) / denom);
            //     } else { continue; }
            // }

            let mut point: Vec3A = Vec3A::ZERO;
            let mut normal: Vec3A = Vec3A::ZERO;
            let mut found = false;
            // println!("in_point = {:?}", in_point - self.package.bound_min);
            // println!("out_point = {:?}", out_point - self.package.bound_min);
            let mut path = Path::new(in_key, in_point - self.package.bound_min, out_key, out_point - self.package.bound_min, self.unit, self.dx, self.dy, self.dz);
            // println!("{:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}", in_key, in_point - self.package.bound_min, out_key, out_point - self.package.bound_min, self.unit, self.dx, self.dy, self.dz);
            while path.next() {
                let mut min_dist = f32::INFINITY;
                let key = path.current_key;
                // println!("key = {:?}", key);
                if self.tree.contains_key(&key) {
                    for index in self.tree.get(&key).iter() {
                        // println!("index = {:?}", index);
                        let triangle = self.triangles[*index];
                        let i2t = intersect_triangle(triangle.p1, triangle.p2, triangle.p3, ray_origin, ray_direction);
                        // println!("i2t = {:?}", i2t);
                        if let Some(x) = i2t {
                            // println!("x = {:?}", x);
                            let dist = (x - ray_origin).length_squared();
                            // println!("dist = {:?}", dist);
                            // println!("min_dist = {:?}", dist);
                            if min_dist > dist {
                                normal = triangle.normal;
                                point = x;
                                min_dist = dist;
                                found = true;
                            } else { continue; }
                        } else { continue; }
                    }
                    if found { break; }
                }
            }
            if found { Some([point, normal]) } else {
                // println!("origin = {:?}, direction = {:?}", ray_origin, ray_direction);
                None
            }
        } else { None }
    }
}
