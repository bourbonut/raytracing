#[cfg(test)]
mod path_tests{
    use glam::Vec3A;
    use raytracing::path::Path;
    
    #[test]
    fn initialization() {
        let path = Path::new([0, 0, 1], Vec3A::new(0.5, 0.3, 1.5), [1, 1, 0], Vec3A::new(1.7, 1.7, 0.4), 1., 2, 2, 2);
        assert_eq!(path.current_key, [0, 0, 1]);
    }

    // #[test]
    // fn min_value() {
    //     let path = Path::new(4, Vec3A::new(0.5, 0.3, 1.5), 3, Vec3A::new(1.7, 1.7, 0.4), 1., 2, 2, 2);
    //     assert_eq!(path.along(), 5. / 12.);
    // }

    #[test]
    fn find() {
        let mut path = Path::new([0, 0, 1], Vec3A::new(0.5, 0.3, 1.5), [1, 1, 0], Vec3A::new(1.7, 1.7, 0.4), 1., 2, 2, 2);
        assert_eq!(path.next(), true);
        assert_eq!(path.current_key, [0, 0, 1]);
        assert_eq!(path.next(), true);
        assert_eq!(path.current_key, [1, 0, 1]);
        assert_eq!(path.next(), true);
        assert_eq!(path.current_key, [1, 0, 0]);
        assert_eq!(path.next(), true);
        assert_eq!(path.current_key, [1, 1, 0]);
        assert_eq!(path.next(), false);
    }

    #[test]
    fn find_2() {
        let mut path = Path::new([0, 0, 1], Vec3A::new(0.09397406, 0.09448958, 0.19999996), [1, 0, 0], Vec3A::new(0.20000002, 0.050145037, 0.009423651), 0.1, 2, 2, 2);
        assert_eq!(path.next(), true);
        assert_eq!(path.current_key, [0, 0, 1]);
        assert_eq!(path.next(), true);
        assert_eq!(path.current_key, [1, 0, 1]);
        assert_eq!(path.next(), true);
        assert_eq!(path.current_key, [1, 0, 0]);
        assert_eq!(path.next(), false);
    }
    
    #[test]
    fn find_3() {
        let mut path = Path::new([0, 1, 1], Vec3A::new(0.027154043, 0.1777035, 0.20000002), [0, 1, 0], Vec3A::new(0.010966055, 0.19497094, -2.2351742e-8), 0.1, 2, 2, 2);
        assert_eq!(path.next(), true);
        assert_eq!(path.current_key, [0, 1, 1]);
        assert_eq!(path.next(), true);
        assert_eq!(path.current_key, [0, 1, 0]);
        assert_eq!(path.next(), false);
    }
}
