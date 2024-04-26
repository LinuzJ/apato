use anyhow::anyhow;
use anyhow::Result;
use nalgebra::DMatrix;
use std::cmp::Ordering;
use std::{sync::Arc, vec};

use crate::{
    config::Config, interest_rate::interest_rate_client, models::apartment::InsertableApartment,
    oikotie::oikotie::Oikotie,
};

pub async fn get_estimated_irr(
    config: &Arc<Config>,
    mut apartment: InsertableApartment,
    mut oikotie: Oikotie,
) -> Result<f64> {
    /*
       Calculate yield here
       - Get rent for similar apartments close by
       - Get interest rate from Nordea
       - Calculate
    */

    let estimated_rent = oikotie.get_estimated_rent(&apartment).await?;

    apartment.rent = Some(estimated_rent);

    let interest_rate_result = interest_rate_client::get_interest_rate(config).await;
    let interest_rate = match interest_rate_result {
        Ok(r) => r,
        Err(_e) => {
            return Err(anyhow!(
                "Failed to fetch interest for apartment {} in watchlist {}",
                apartment.card_id,
                apartment.watchlist_id
            ));
        }
    };

    let price: f64 = apartment.price.unwrap().into();
    let rent: f64 = apartment.rent.unwrap().into();
    let additional_cost: f64 = apartment.additional_costs.unwrap().into();

    let irr = calculate_irr(config, price, rent, additional_cost, interest_rate);

    Ok(irr)
}

pub fn calculate_irr(
    config: &Arc<Config>,
    price: f64,
    rent: f64,
    additional_cost: f64,
    interest_rate: f64,
) -> f64 {
    let loan: f64 = price + config.avg_renovation_costs as f64;
    let down_payment_amount: f64 = (config.down_payment_percentage as f64 / 100.0) * loan;
    let initial_principal: f64 = loan - down_payment_amount;

    let mut yearly_cash_flows: Vec<f64> = vec![];
    yearly_cash_flows.push(-down_payment_amount);

    // Calculate cash flows for each year
    for year in 1..(config.loan_duration_years + 1) {
        let income = get_rent(config, year, rent);
        let vacancy = get_vacancy_cost(config, income / 12.0);
        let depreciation = get_depreciation(config);
        let fixed_costs = -additional_cost * 12.0;

        let ebit = income + vacancy + fixed_costs + depreciation;

        let taxes = -ebit * (config.tax as f64 / 100.0);
        let depreciation_add = -depreciation;
        let interest_payment = interest_payment_for_period(
            interest_rate / 100.0,
            year as f64,
            config.loan_duration_years as f64,
            initial_principal,
        );
        let principal_payment = principal_payment_for_period(
            interest_rate / 100.0,
            year as f64,
            config.loan_duration_years as f64,
            initial_principal,
        );
        let fcf = ebit + taxes + depreciation_add + interest_payment + principal_payment;

        let apartment_value_increase = valuation_increase(config, price, year);

        let fcfe = fcf + apartment_value_increase + (-principal_payment);

        yearly_cash_flows.push(fcfe);
    }
    let irr: f64 = _irr(yearly_cash_flows).unwrap_or_default() * 100.0;

    // Make sure the value is within reasonable limits
    if !(-50.0..=50.0).contains(&irr) {
        return 0.0;
    }

    irr
}

fn get_rent(config: &Arc<Config>, year: u32, rent: f64) -> f64 {
    let increase = (config.avg_estimated_rent_increase_per_year as f64) / 100.0;
    let multiplier = (1.0 + increase).powf(year as f64 - 1.0);
    (rent * multiplier) * 12.0
}

fn get_vacancy_cost(config: &Arc<Config>, rent: f64) -> f64 {
    let vacancies = config.avg_vacant_month_per_year as f64;
    let rent_missed = rent * vacancies;
    -rent_missed
}

fn get_depreciation(config: &Arc<Config>) -> f64 {
    -(config.avg_renovation_costs as f64 / config.loan_duration_years as f64)
}

fn pmt(interest_rate: f64, periods: f64, pv: f64) -> f64 {
    // Calculate the fixed yearly payment for a loan or investment.

    // Arguments:
    // interest_rate -- the yearly interest rate.
    // periods -- the total number of payment periods (in years).
    // pv   -- the present value, or the total amount of the loan or investment.
    // fv   -- the future value, or the remaining balance after all payments have been made (default 0).

    // Returns:
    // The fixed yearly payment amount.
    if interest_rate == 0.0 {
        -pv / periods
    } else {
        (interest_rate / (1.0 - (1.0 + interest_rate).powf(-periods))) * -pv
    }
}

fn future_value(interest_rate: f64, periods: f64, c: f64, pv: f64) -> f64 {
    // Calculate the future value of an investment or loan after a specified number of periods.

    // Arguments:
    // interest_rate -- the yearly interest rate.
    // periods -- the period number to calculate the future value for.
    // c    -- the fixed yearly payment.
    // pv   -- the present value, or the total amount of the loan or investment.

    // Returns:
    // The future value after the specified number of periods.
    -(c * ((1.0 + interest_rate).powf(periods) - 1.0) / interest_rate
        + pv * (1.0 + interest_rate).powf(periods))
}

// Calculates the amount of interest that should be payed at a specific period.
fn interest_payment_for_period(
    interest_rate: f64,
    period: f64,
    total_periods: f64,
    present_value: f64,
) -> f64 {
    // Calculates the amount of a payment goes to interest on the loan principal at period {period} / {total_period}

    // Arguments:
    // interest_rate -- the yearly interest rate.
    // period -- current period
    // total_period -- total periods
    // present_value -- the present value, or the total amount of the loan.
    let total_payment = pmt(interest_rate, total_periods, present_value);
    let future_pv = future_value(interest_rate, period - 1.0, total_payment, present_value);

    future_pv * interest_rate
}

// Calculates the amount of principal that should be payed at a specific period.
fn principal_payment_for_period(
    interest_rate: f64,
    period: f64,
    total_periods: f64,
    present_value: f64,
) -> f64 {
    // Calculates the amount of a payment goes to pay back the loan principal at period {period} / {total_period}

    // Arguments:
    // interest_rate -- the yearly interest rate.
    // period -- current period
    // total_period -- total periods
    // present_value -- the present value, or the total amount of the loan.
    let total_payment = pmt(interest_rate, total_periods, present_value);
    let interest_part =
        interest_payment_for_period(interest_rate, period, total_periods, present_value);
    total_payment - interest_part
}

fn valuation_increase(config: &Arc<Config>, price: f64, year: u32) -> f64 {
    let growth_rate = config.estimated_yearly_apartment_price_increase as f64 / 100.0;
    let this_year = price * (1.0 + growth_rate).powf(year as f64);
    let last_year = price * (1.0 + growth_rate).powf(year as f64 - 1.0);
    this_year - last_year
}

fn _irr(cash_flow: Vec<f64>) -> Option<f64> {
    // Calculate the inner rate of return for the given cashflow.
    // Arguments:
    // cash_flow -- Vector with yearly cash-flow, including the year 0 investment.

    let all_roots = get_roots(cash_flow);
    let mut potential_roots: Vec<f64> = vec![];

    for root in all_roots {
        if root >= -1.0 {
            potential_roots.push(root);
        }
    }

    // If no real or valid roots
    if potential_roots.is_empty() {
        return None;
    }

    // If one root
    if potential_roots.len() == 1 {
        return Some(potential_roots[0] - 1.0);
    }

    // If many roots -> choose most valid
    let abs_root: Vec<(f64, f64)> = potential_roots
        .iter()
        .map(|r| (r.to_owned(), r.abs()))
        .collect();
    let min_root = abs_root.iter().min_by(|x, y| x.1.total_cmp(&y.1)).unwrap();
    Some(min_root.0 - 1.0)
}

// https://math.mit.edu/~edelman/publications/polynomial_roots.pdf
// https://web.mit.edu/18.06/www/Spring17/Eigenvalue-Polynomials.pdf
// Find the roots of the polynomial given.
// Roots are the eigenvalues of the companion matrix of the polynomial.
fn get_roots(coeffs: Vec<f64>) -> Vec<f64> {
    let n = coeffs.len() - 1;

    match n.cmp(&1) {
        Ordering::Less => vec![],
        Ordering::Equal => vec![-coeffs[0] / coeffs[1]],
        Ordering::Greater => {
            let mut companion_matrix = companion_matrix(&coeffs);
            // Reverse matrix to minimize error of eigenvalue roots
            companion_matrix = reverse_matrix(&companion_matrix);
            let eigenvalues_complx = companion_matrix.complex_eigenvalues();
            let eigenvalues = eigenvalues_complx.data.as_vec();

            let mut real_roots: Vec<f64> = vec![];

            for r in eigenvalues {
                if r.im == 0.0 {
                    real_roots.push(r.re);
                }
            }

            _map_domain(&real_roots, (-1.0, 1.0), (-1.0, 1.0))
        }
    }
}

// Generate the companion matrix of the polynomial given
fn companion_matrix(polynomial: &Vec<f64>) -> DMatrix<f64> {
    let n = polynomial.len() - 1;

    match n.cmp(&1) {
        Ordering::Less => {
            panic!("polynomial must have maximum degree of at least 1.");
        }
        Ordering::Equal => {
            let mut companion = DMatrix::<f64>::zeros(1, 1);
            companion[(0, 0)] = -polynomial[0] / polynomial[1];
            companion
        }
        Ordering::Greater => {
            let mut companion = DMatrix::<f64>::zeros(n, n);

            // Set the diagonal flags for which poly degree
            for i in 0..(n - 1) {
                companion[(i + 1, i)] = 1.0;
            }

            // Set the coeff of the polynomial in the last column
            for i in 0..n {
                companion[(i, n - 1)] = -polynomial[i] / polynomial[n];
            }

            companion
        }
    }
}

fn reverse_matrix(matrix: &DMatrix<f64>) -> DMatrix<f64> {
    let mut reversed = DMatrix::<f64>::zeros(matrix.nrows(), matrix.ncols());

    for i in 0..matrix.nrows() {
        for j in 0..matrix.ncols() {
            reversed[(i, j)] = matrix[(matrix.nrows() - 1 - i, matrix.ncols() - 1 - j)];
        }
    }

    reversed
}

// TODO: Figure why inverse is needed
// https://github.com/numpy/numpy/blob/1c8b03bf2c87f081eea211a5061e423285c548af/numpy/polynomial/polyutils.py#L286
fn _map_domain(x: &Vec<f64>, _old: (f64, f64), _new: (f64, f64)) -> Vec<f64> {
    // let off = new.0 - ((new.1 - new.0) / (old.1 - old.0)) * old.0;
    // let scl = (new.1 - new.0) / (old.1 - old.0);

    // TODO: figure out why roots are inverse :D
    return x.iter().map(|&val| 1.0 / val).collect();
}

#[cfg(test)]
mod matrix_tests {
    use nalgebra::dmatrix;

    use crate::producer::calculations::{companion_matrix, reverse_matrix};

    use super::get_roots;

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
    fn _companion_matrix_test_1() {
        let input = vec![1.0, 2.0, 3.0];

        let expected = dmatrix![0.0, -0.3333333333333333;
                                1.0,  -0.6666666666666666;];

        let matrix = companion_matrix(&input);

        assert_eq!(matrix, expected)
    }

    #[test]
    fn _companion_matrix_test_2() {
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
    fn _companion_matrix_test_3() {
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

#[cfg(test)]
mod yield_calculations {

    use crate::config;

    use super::*;

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
        let irr_raw = _irr(cash_flow).unwrap();
        let irr = (irr_raw * 1000000.0).round() / 1000000.0;
        assert_eq!(irr, 0.156152)
    }

    #[test]
    fn test_irr_2() {
        let cash_flow: Vec<f64> = vec![-100000.0, 20000.0, 50000.0, 20000.0];
        let irr_raw = _irr(cash_flow).unwrap();
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
        let irr_raw = _irr(cash_flow).unwrap();
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
