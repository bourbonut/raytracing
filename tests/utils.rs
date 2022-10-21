#[cfg(test)]
mod utils_tests{
    use glam::Vec3A;
    use raytracing::utils::{intersect_plane, intersect_triangle};
    
    #[test]
    fn intersect_plane_test() {
        let point = Vec3A::new(0., 0., 0.);
        let normal = Vec3A::new(0., 0., 1.);
        
        let ro = Vec3A::new(1., 1., 1.);
        let rd = Vec3A::new(0., 0., -1.);
        let result = intersect_plane(point, normal, ro, rd);
        let real_value: Option<Vec3A> = Some(Vec3A::new(1., 1., 0.));
        assert_eq!(result, real_value);
    }

    #[test]
    fn intersect_triangle_test() {
        let p1 = Vec3A::new(-2., -1., 0.);
        let p2 = Vec3A::new(11., 7., 0.);
        let p3 = Vec3A::new(-2., 11., 0.);

        let ro = Vec3A::new(1., 1., 1.);
        let rd = Vec3A::new(0., 0., -1.);
        let result = intersect_triangle(p1, p2, p3, ro, rd);
        let real_value: Option<Vec3A> = Some(Vec3A::new(1., 1., 0.));
        assert_eq!(result, real_value);
    }
}
