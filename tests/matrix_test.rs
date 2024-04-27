mod matrix_tests {
    use apato::consumer::calculations::{self, companion_matrix, get_roots, reverse_matrix};
    use nalgebra::dmatrix;
    #[test]
    fn roots_test_1() {
        let input = vec![-2.0, 1.0]; //
        let expected: Vec<f64> = vec![2.0];
        let roots = get_roots(input);
        let mut i = 0;
        while i < expected.len() {
            let expected_rounded: f64 = (expected[i] * 100.0).round() / 100.0;
            let real_rounded: f64 = (roots[i] * 100.0).round() / 100.0;

            assert_eq!(expected_rounded, real_rounded);
            i += 1;
        }
    }

    #[test]
    fn test_root_2() {
        let coeffs = vec![1.0, -5.0, 6.0]; // x^2 - 5x + 6 = (x - 2)(x - 3)
        let expected: Vec<f64> = vec![2.0, 3.0]; // Roots: x = 2, 3
        let roots = get_roots(coeffs);
        let mut i = 0;
        while i < (vec![2.0, 3.0]).len() {
            let expected_rounded: f64 = (expected[i] * 100.0).round() / 100.0;
            let real_rounded: f64 = (roots[i] * 100.0).round() / 100.0;

            assert_eq!(expected_rounded, real_rounded);
            i += 1;
        }
    }

    #[test]
    fn test_root_3() {
        let coeffs = vec![1.0, -6.0, 11.0, -6.0]; // x^3 - 6x^2 + 11x - 6 = (x - 1)(x - 2)(x - 3)
        let expected: Vec<f64> = vec![1.0, 2.0, 3.0]; // Roots: x = 1, 2, 3
        let roots = get_roots(coeffs);
        let mut i = 0;
        while i < expected.len() {
            let expected_rounded: f64 = (expected[i] * 100.0).round() / 100.0;
            let real_rounded: f64 = (roots[i] * 100.0).round() / 100.0;

            assert_eq!(expected_rounded, real_rounded);
            i += 1;
        }
    }

    #[test]
    fn test_root_4() {
        let coeffs = vec![1.0, 2.0, 3.0]; // x^2 + 2x + 3
        let expected: Vec<f64> = Vec::new(); // No real roots
        let roots = get_roots(coeffs);
        let mut i = 0;
        while i < expected.len() {
            let expected_rounded: f64 = (expected[i] * 100.0).round() / 100.0;
            let real_rounded: f64 = (roots[i] * 100.0).round() / 100.0;

            assert_eq!(expected_rounded, real_rounded);
            i += 1;
        }
    }

    #[test]
    fn companion_matrix_test_1() {
        let input = vec![1.0, 2.0, 3.0];

        let expected = dmatrix![0.0, -0.3333333333333333;
                                1.0,  -0.6666666666666666;];

        let matrix = companion_matrix(&input);

        assert_eq!(matrix, expected)
    }

    #[test]
    fn companion_matrix_test_2() {
        let input = vec![-2.0, 3.0, 4.0, -5.0, 1.0];

        let expected = dmatrix![
            0.0, 0.0, 0.0, 2.0;
            1.0, 0.0, 0.0, -3.0;
            0.0, 1.0, 0.0, -4.0;
            0.0, 0.0, 1.0, 5.0;
        ];

        let matrix = companion_matrix(&input);

        assert_eq!(matrix, expected)
    }

    #[test]
    fn companion_matrix_test_3() {
        let input = vec![1.0, -6.0, 11.0, -6.0];

        let expected = dmatrix![
            0.0, 0.0, 0.16666666666666666;
            1.0, 0.0, -1.0;
            0.0, 1.0, 1.8333333333333333;
        ];

        let matrix = companion_matrix(&input);

        assert_eq!(matrix, expected)
    }

    #[test]
    fn test_reverse_matrix_1() {
        let input = dmatrix![
            0.0, 0.0, 0.0, 2.0;
            1.0, 0.0, 0.0, -3.0;
            0.0, 1.0, 0.0, -4.0;
            0.0, 0.0, 1.0, 5.0;
        ];
        let expected = dmatrix![
            5.0, 1.0, 0.0, 0.0;
            -4.0, 0.0, 1.0, 0.0;
            -3.0, 0.0, 0.0, 1.0;
            2.0, 0.0, 0.0, 0.0;
        ];
        let matrix = reverse_matrix(&input);

        assert_eq!(matrix, expected)
    }
}
