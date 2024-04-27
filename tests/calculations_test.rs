#[cfg(test)]
mod yield_calculations {
    use std::sync::Arc;

    use apato::{
        config,
        consumer::calculations::{
            calculate_irr, future_value, interest_payment_for_period, irr, pmt,
            principal_payment_for_period, valuation_increase,
        },
    };

    #[test]
    fn calculate_basic_yield_wip() {
        let config = Arc::new(config::create_test_config());
        let yield_ = calculate_irr(&config, 100000 as f64, 800 as f64, 200 as f64, 2.00);
        let yield_rounded = (yield_ * 1000.0).round() / 1000.0;
        assert_eq!(yield_rounded, 25.996)
    }

    #[test]
    fn test_irr_1() {
        let cash_flow: Vec<f64> = vec![-100000.0, 20000.0, 50000.0, 70000.0];
        let irr_raw = irr(cash_flow).unwrap();
        let irr = (irr_raw * 1000000.0).round() / 1000000.0;
        assert_eq!(irr, 0.156152)
    }

    #[test]
    fn test_irr_2() {
        let cash_flow: Vec<f64> = vec![-100000.0, 20000.0, 50000.0, 20000.0];
        let irr_raw = irr(cash_flow).unwrap();
        let irr = (irr_raw * 1000000.0).round() / 1000000.0;
        assert_eq!(irr, -0.051028)
    }

    #[test]
    fn test_irr_3() {
        let cash_flow: Vec<f64> = vec![
            -21000.00, 3790.00, 3914.05, 4039.87, 4167.47, 4296.90, 4428.19, 4561.35, 4696.42,
            4833.43, 4972.42, 5113.41, 5256.44, 5401.54, 5548.74, 5698.07, 5849.58, 6003.30,
            6159.26, 6317.50, 6478.06, 6640.97, 6806.27, 6974.01, 7144.22, 7316.94,
        ];
        let irr_raw = irr(cash_flow).unwrap();
        let irr = (irr_raw * 1000000.0).round() / 1000000.0;
        assert_eq!(irr, 0.207468)
    }

    #[test]
    fn test_interest_payment_period_1() {
        let interest_rate = 0.02;
        let period = 1.0;
        let total_periods = 25.0;
        let present_value = 84000.0;

        let result =
            interest_payment_for_period(interest_rate, period, total_periods, present_value);
        let result_rounded = (result * 10000.0).round() / 10000.0;
        assert_eq!(result_rounded, -1680.00)
    }

    #[test]
    fn test_interest_payment_period_2() {
        let interest_rate = 0.02;
        let period = 5.0;
        let total_periods = 25.0;
        let present_value = 84000.0;

        let result =
            interest_payment_for_period(interest_rate, period, total_periods, present_value);
        let result_rounded = (result * 10000.0).round() / 10000.0;
        assert_eq!(result_rounded, -1463.8203)
    }

    #[test]
    fn test_principal_payment_period_1() {
        let interest_rate = 0.02;
        let period = 5.0;
        let total_periods = 25.0;
        let present_value = 84000.0;

        let result =
            principal_payment_for_period(interest_rate, period, total_periods, present_value);
        let result_rounded = (result * 1000.0).round() / 1000.0;
        assert_eq!(result_rounded, -2838.697)
    }

    #[test]
    fn test_principal_payment_period_2() {
        let interest_rate = 0.02;
        let period = 1.0;
        let total_periods = 25.0;
        let present_value = 84000.0;

        let result =
            principal_payment_for_period(interest_rate, period, total_periods, present_value);
        let result_rounded = (result * 10000.0).round() / 10000.0;
        assert_eq!(result_rounded, -2622.5168)
    }

    #[test]
    fn test_principal_payment_period_3() {
        let interest_rate = 0.02;
        let period = 2.0;
        let total_periods = 25.0;
        let present_value = 84000.0;

        let result =
            principal_payment_for_period(interest_rate, period, total_periods, present_value);
        let result_rounded = (result * 1000.0).round() / 1000.0;
        assert_eq!(result_rounded, -2674.967)
    }

    #[test]
    fn test_principal_payment_for_period_1() {
        let rate = 0.02;
        let n = 25.0;
        let pv = 84000.0;
        let expected: Vec<f64> = vec![
            -2622.52, -2674.97, -2728.47, -2783.04, -2838.70, -2895.47, -2953.38, -3012.45,
            -3072.70, -3134.15, -3196.83, -3260.77, -3325.99, -3392.51, -3460.36, -3529.56,
            -3600.15, -3672.16, -3745.60, -3820.51, -3896.92, -3974.86, -4054.36, -4135.44,
            -4218.15,
        ];
        for i in 1..(n as u32) {
            let r = principal_payment_for_period(rate, i as f64, n, pv);
            let result_rounded = (r * 100.0).round() / 100.0;
            assert_eq!(result_rounded, expected[i as usize - 1]);
        }
    }

    #[test]
    fn test_future_value_1() {
        let rate = 0.02;
        let period = 2.0;
        let payment = 1234.0;
        let pv = 100000.0;
        let r = future_value(rate, period, payment, pv);
        let result_rounded = (r * 1000000.0).round() / 1000000.0;
        assert_eq!(result_rounded, -106532.68)
    }

    #[test]
    fn test_pmt_1() {
        let rate = 0.02;
        let periods = 20.0;
        let pv = 100000.0;
        let r = pmt(rate, periods, pv);
        let result_rounded = (r * 10000.0).round() / 10000.0;
        assert_eq!(result_rounded, -6115.6718)
    }

    #[test]
    fn test_apartment_value_increase_1() {
        let config = Arc::new(config::create_test_config());
        let price = 100000.0;
        let year = 2;
        let increase = valuation_increase(&config, price, year);
        assert_eq!(increase, 2040.0)
    }

    #[test]
    fn float_test() {
        let yield_: f64 = 0.20748;
        let yield_rounded = (yield_ * 10000.0).round() / 10000.0;
        assert_eq!(yield_rounded, 0.2075)
    }
}
