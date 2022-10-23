use stl::read_stl;
use glam::{Vec3A, Vec2};
use std::fs::File;
use std::io::Result;

type Tree = Vec<(usize, f32)>;

#[derive(Default, Clone)]
pub struct Triangle{
    pub points: [Vec3A; 3],
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

fn compute_barycenter(points: &[Vec3A; 3] ) -> Vec3A {
    (points[0] + points[1] + points[2]) / 3.
}

fn semi_spherical_coordinates(v: &Vec3A) -> Vec2{
    let rho = v.length();
    let delta = v.z / rho; // without asin
    let theta = v.y / v.x; // without atan2
    Vec2::new(delta, theta)
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
                    let points: [Vec3A; 3] = [t.v1, t.v2, t.v3].map(convert);
                    let center = compute_barycenter(&points);
                    mesh_center += center;
                    triangles.push(Triangle { points, normal: convert(t.normal), center})
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
}
