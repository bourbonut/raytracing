#[cfg(test)]
mod rtengine_tests {
    use glam::Vec3A;
    use ndarray::Array2;
    use raytracing::engine::Material;
    use raytracing::engine::RTEngine;
    use raytracing::engine::Sphere;
    const WIDTH: usize = 900;
    const HEIGHT: usize = 600;

    fn engine_for_test() -> RTEngine {
        let ratio: f32 = WIDTH as f32 / HEIGHT as f32;
        let screen = (-1., 1. / ratio, 1., -1. / ratio);
        let mut pixels = Array2::<Vec3A>::default((HEIGHT, WIDTH));

        let step_height: f32 = (screen.3 - screen.1) / (HEIGHT as f32);
        let step_width: f32 = (screen.2 - screen.0) / (WIDTH as f32);
        for i in 0..HEIGHT {
            let y: f32 = screen.1 + (i as f32) * step_height;
            for j in 0..WIDTH {
                let x: f32 = screen.0 + (j as f32) * step_width;
                pixels[[i, j]] = Vec3A::new(x, y, 0.);
            }
        }

        let red_sphere: Sphere = Sphere {
            center: Vec3A::new(-0.2, 0., -1.),
            radius: 0.7,
        };
        let violet_sphere: Sphere = Sphere {
            center: Vec3A::new(0.1, -0.3, 0.),
            radius: 0.1,
        };
        let green_sphere: Sphere = Sphere {
            center: Vec3A::new(-0.3, 0., 0.),
            radius: 0.15,
        };
        let plane: Sphere = Sphere {
            center: Vec3A::new(0., -9000., 0.),
            radius: 9000. - 0.7,
        };

        let red_material: Material = Material {
            ambiant: Vec3A::new(0.1, 0., 0.),
            diffuse: Vec3A::new(0.7, 0., 0.),
            specular: Vec3A::new(1., 1., 1.),
            shininess: 100.,
            reflection: 0.5,
        };
        let violet_material: Material = Material {
            ambiant: Vec3A::new(0.1, 0., 0.1),
            diffuse: Vec3A::new(0.7, 0., 0.7),
            specular: Vec3A::new(1., 1., 1.),
            shininess: 100.,
            reflection: 0.5,
        };
        let green_material: Material = Material {
            ambiant: Vec3A::new(0., 0.1, 0.),
            diffuse: Vec3A::new(0., 0.6, 0.),
            specular: Vec3A::new(1., 1., 1.),
            shininess: 100.,
            reflection: 0.5,
        };
        let plane_material: Material = Material {
            ambiant: Vec3A::new(0.1, 0.1, 0.1),
            diffuse: Vec3A::new(0.6, 0.6, 0.6),
            specular: Vec3A::new(1., 1., 1.),
            shininess: 100.,
            reflection: 0.5,
        };

        let all_objects: Vec<Sphere> = vec![red_sphere, violet_sphere, green_sphere, plane];
        let materials: Vec<Material> = vec![
            red_material,
            violet_material,
            green_material,
            plane_material,
        ];

        RTEngine {
            pos_camera: Vec3A::new(0., 0., 1.),
            pos_pixels: pixels,
            pos_light: Vec3A::new(5., 5., 5.),
            objects: all_objects,
            material: materials,
        }
    }

    #[test]
    fn basic_instanciation() {
        let rte = engine_for_test();
        assert_eq!(rte.pos_camera, Vec3A::new(0., 0., 1.));
        assert_eq!(rte.pos_pixels.len(), WIDTH * HEIGHT);
    }

    #[test]
    // Size of array of pixels should be the same as the size
    // of pos_pixels inputs after computing the path tracing algorithm.
    //
    // Meaning that if you have a screen of x*y pixels, you have x*y colors.
    // pt = Path tracing
    fn pt_size_of_output() {
        let mut rte = engine_for_test();
        let c = rte.path_tracing();
        assert_eq!(c.len(), rte.pos_pixels.len());
    }
}
