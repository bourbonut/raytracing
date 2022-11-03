#[cfg(test)]
mod path_tests{
    use glam::Vec3A;
    use raytracing::path::Path;
    
    #[test]
    fn initialization() {
        let path = Path::new(4, Vec3A::new(0.5, 0.3, 1.5), 3, Vec3A::new(1.7, 1.7, 0.4), 1., 2, 2, 2);
        assert_eq!(path.current_key, 4);
    }

    // #[test]
    // fn min_value() {
    //     let path = Path::new(4, Vec3A::new(0.5, 0.3, 1.5), 3, Vec3A::new(1.7, 1.7, 0.4), 1., 2, 2, 2);
    //     assert_eq!(path.along(), 5. / 12.);
    // }

    #[test]
    fn find() {
        let mut path = Path::new(4, Vec3A::new(0.5, 0.3, 1.5), 3, Vec3A::new(1.7, 1.7, 0.4), 1., 2, 2, 2);
        assert_eq!(path.current_key, 4);
        assert_eq!(path.next(), true);
        assert_eq!(path.current_key, 5);
        assert_eq!(path.next(), true);
        assert_eq!(path.current_key, 1);
        assert_eq!(path.next(), true);
        assert_eq!(path.current_key, 3);
        assert_eq!(path.next(), false);
    }
}
