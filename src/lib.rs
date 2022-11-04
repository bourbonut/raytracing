pub mod engine;
pub mod utils;
pub mod mesh;
pub mod path;

use std::time::Instant;
use glam::Vec3A;
use image;
use ndarray::Array2;

use crate::engine::Material;
use crate::engine::RTEngine;
// use crate::engine::Sphere;

use std::fs::File;
use crate::mesh::Mesh;

// pub fn run_lib() {
//     let width = 1920;
//     let height = 1080;
//     let ratio: f32 = width as f32 / height as f32;
//     let screen = (-1., 1. / ratio, 1., -1. / ratio);
//     let mut pixels = Array2::<Vec3A>::default((height, width));

//     let step_height: f32 = (screen.3 - screen.1) / ((height - 1) as f32);
//     let step_width: f32 = (screen.2 - screen.0) / ((width - 1) as f32);
//     for i in 0..height {
//         let y: f32 = screen.1 + (i as f32) * step_height;
//         for j in 0..width {
//             let x: f32 = screen.0 + (j as f32) * step_width;
//             pixels[[i, j]] = Vec3A::new(x, y, 0.);
//         }
//     }

//     let red_sphere = Sphere {
//         center: Vec3A::new(-0., 0., -1.),
//         radius: 0.7,
//     };

//     let red_material = Material {
//         ambiant: Vec3A::new(0.1, 0., 0.),
//         diffuse: Vec3A::new(0.7, 0., 0.),
//         specular: Vec3A::new(1., 1., 1.),
//         shininess: 100.,
//         reflection: 0.5,
//     };

//     let all_objects: Vec<Sphere> = vec![red_sphere]; //, violet_sphere, green_sphere, plane];
//     let materials: Vec<Material> = vec![red_material];

//     let mut rte = RTEngine {
//         pos_camera: Vec3A::new(0., 0., 1.),
//         pos_pixels: pixels,
//         pos_light: Vec3A::new(5., 5., 5.),
//         objects: all_objects,
//         material: materials,
//     };
//     let pixels: Array2<Vec3A> = rte.path_tracing();
//     let mut imgbuf = image::ImageBuffer::new(width as u32, height as u32);

//     for ((y, x), pixel) in pixels.indexed_iter() {
//         let pixel_on_img = imgbuf.get_pixel_mut(x as u32, y as u32);
//         let image::Rgb(_data) = *pixel_on_img; // _data is the current color of the pixel
//         let rgb = pixel.to_array();
//         *pixel_on_img = image::Rgb([rgb[0] as u8, rgb[1] as u8, rgb[2] as u8]);
//     }
//     imgbuf.save("output.png").unwrap();
// }

pub fn small_test() {
    let file = File::open("cube.stl");
    match file{
        Ok(mut x) => {
            let now = Instant::now();
            let mesh = Mesh::new(&mut x);
            let new_now = Instant::now();
            println!("initialization duration = {:?}", new_now - now);
            // let ray_origin = Vec3A::new(0., 0., 0.3);
            // let ray_direction = Vec3A::new(0., -0.01, -0.31);
            let ray_origin = Vec3A::new(0.047127932, -0.04722385, 0.05000995);
            let ray_direction = Vec3A::new(0.5763695, 0.5793987, 0.576277);
            match mesh {
                Ok(m) => { 
                    let now = Instant::now();
                    let x = m.intersect(ray_origin, ray_direction);
                    let new_now = Instant::now();
                    println!("intersection duration = {:?}", new_now - now);
                    match x {
                            Some(x) => {
                                let a = x[0];
                                let b = x[1];
                                println!("Intersection point = {:?}, normal = {:?}", a, b);
                            }
                            None => { println!("No intersection found."); }
                        }
                    println!("Ok");
                    }
                Err(_) => { println!("Error for mesh"); }
            }
        }
        Err(_) => {
            println!("Error");
        }
    }
}

pub fn cube_raytracing() {
    // let width = 192 * 2;
    // let height = 108 * 2;
    let width = 1920;
    let height = 1080;
    let ratio: f32 = width as f32 / height as f32;
    let screen = (-1., 1. / ratio, 1., -1. / ratio);
    let mut pixels = Array2::<Vec3A>::default((height, width));

    let step_height: f32 = (screen.3 - screen.1) / ((height - 1) as f32);
    let step_width: f32 = (screen.2 - screen.0) / ((width - 1) as f32);
    for i in 0..height {
        let y: f32 = screen.1 + (i as f32) * step_height;
        for j in 0..width {
            let x: f32 = screen.0 + (j as f32) * step_width;
            pixels[[i, j]] = Vec3A::new(x, y, 0.);
        }
    }

    // let file = File::open("cube.stl");
    let file = File::open("cube.stl");
    match file {
        Ok(mut x) => {
            let mesh = Mesh::new(&mut x);
            match mesh {
                Ok(cube) => {
                    let white_material = Material {
                        ambiant: Vec3A::new(0.1, 0.1, 0.1),
                        diffuse: Vec3A::new(0.7, 0.7, 0.7),
                        specular: Vec3A::new(1., 1., 1.),
                        shininess: 100.,
                        reflection: 0.5,
                    };

                    let all_objects: Vec<Mesh> = vec![cube];
                    let materials: Vec<Material> = vec![white_material];
                    let mut rte = RTEngine {
                        pos_camera: Vec3A::new(-0.5, 0.2, 1.).normalize(),
                        // pos_camera: Vec3A::Z,
                        pos_pixels: pixels,
                        pos_light: Vec3A::new(18., 18., 18.),
                        objects: all_objects,
                        material: materials,
                    };
                    let pixels: Array2<Vec3A> = rte.path_tracing();
                    let mut imgbuf = image::ImageBuffer::new(width as u32, height as u32);

                    for ((y, x), pixel) in pixels.indexed_iter() {
                        let pixel_on_img = imgbuf.get_pixel_mut(x as u32, y as u32);
                        let image::Rgb(_data) = *pixel_on_img; // _data is the current color of the pixel
                        let rgb = pixel.to_array();
                        *pixel_on_img = image::Rgb([rgb[0] as u8, rgb[1] as u8, rgb[2] as u8]);
                    }
                    imgbuf.save("output.png").unwrap();
                }
                Err(_) => { println!("Problem with the cube file") }
            }
        }
        Err(_) => { println!("Impossible to open the cube file") }
    }
}
