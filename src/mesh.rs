use stl::read_stl;
use glam::Vec3A;
use std::fs::File;
use std::io::Result;

#[derive(Default, Clone)]
pub struct Triangle{
    pub points: Vec<Vec3A>,
    pub normal: Vec3A,
}

#[derive(Default, Clone)]
pub struct Mesh{
    pub triangles: Vec<Triangle>,
}

fn convert(array: &[f32; 3]) -> Vec3A {
    Vec3A::new(array[0], array[1], array[2])
}

impl Mesh {
    pub fn new(file: &mut File) -> Result<Self> {
        let binary = read_stl(file);        
        match binary {
            Ok(x) => {
                let mut triangles: Vec<Triangle> = Vec::new();
                for t in x.triangles.iter(){
                    let points: Vec<Vec3A> = vec![t.v1, t.v2, t.v3].iter().map(convert).collect();
                    triangles.push(Triangle { points, normal: convert(&t.normal) })
                }
                Ok(Mesh { triangles })
            }
            Err(x) => return Err(x)
        }
    }
}
