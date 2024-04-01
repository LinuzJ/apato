use log::error;
use nalgebra::DMatrix;
use std::{sync::Arc, vec};

use crate::{
    config::Config, db, interest_rate::interest_rate_client,
    models::apartment::InsertableApartment, oikotie::oikotie::Oikotie,
};

pub async fn process_apartment_calculations(
    config: &Arc<Config>,
    potential_apartments: Option<Vec<InsertableApartment>>,
    mut oikotie: Oikotie,
) {
    match potential_apartments {
        Some(apartments) => {
            for mut apartment in apartments {
                /*
                   Calculate yield here
                   - Get rent for similar apartments close by
                   - Get interest rate from Nordea
                   - Calculate
                */

                let estimated_rent = oikotie.get_estimated_rent(&apartment).await;

                match estimated_rent {
                    Ok(rent) => apartment.rent = Some(rent),
                    Err(e) => error!("{}", e),
                }

                let interest_rate = interest_rate_client::get_interest_rate(&config).await;

                let apartment_yield = match interest_rate {
                    Ok(interest_rate) => calculate_irr(
                        apartment.price.clone().unwrap(),
                        apartment.rent.unwrap(),
                        apartment.additional_costs.unwrap(),
                        interest_rate,
                    ),
                    Err(e) => {
                        // TODO better handling here
                        error!("Error while calculating rental yield: {}", e);
                        0.0
                    }
                };

                let insertable_apartment = InsertableApartment {
                    card_id: apartment.card_id,
                    location_id: apartment.location_id,
                    location_level: apartment.location_level,
                    location_name: apartment.location_name,
                    size: apartment.size,
                    rooms: apartment.rooms,
                    price: apartment.price,
                    additional_costs: apartment.additional_costs,
                    rent: apartment.rent,
                    estimated_yield: Some(apartment_yield),
                    watchlist_id: apartment.watchlist_id,
                };
                db::apartment::insert(&config, insertable_apartment);
            }
        }
        None => println!("No apartments added.."),
    }
}

pub fn calculate_irr(price: i32, rent: i32, additional_cost: i32, interest_rate: f64) -> f64 {
    // Calculate annual mortgage payment using the loan term and interest rate
    let annual_interest_rate = interest_rate / 100.0;
    let loan_term_months = 25 * 12;
    let monthly_interest_rate = annual_interest_rate / 12.0;
    let discount_factor = ((1.0 + monthly_interest_rate).powi(loan_term_months as i32) - 1.0)
        / (monthly_interest_rate * (1.0 + monthly_interest_rate).powi(loan_term_months as i32));

    let mortgage_payment = price as f64
        * (monthly_interest_rate
            / (1.0 - 1.0 / (1.0 + monthly_interest_rate).powi(loan_term_months as i32)))
        / discount_factor;

    // Calculate net cash flow (rent - mortgage payment)
    let net_cash_flow = rent as f64 - additional_cost as f64 - mortgage_payment;

    // Calculate rental yield (net cash flow / initial investment)
    let initial_investment = price as f64 * 0.2; // For simplicity for now, assume 20% down-payments
    let rental_yield = net_cash_flow / initial_investment;

    // Convert to yearly yield (multiply by 12)
    let yearly_yield = rental_yield * 12.0;

    yearly_yield * 100.0 // Convert to percentage
}

// TODO
pub fn calculate_irr_wip(
    config: Arc<Config>,
    price: f64,
    rent: f64,
    additional_cost: f64,
    interest_rate: f64,
) -> f64 {
    let down_payment_amount: f64 = (config.down_payment_percentage as f64 / 100.0) * price as f64;
    let loan: f64 = price as f64 + config.avg_renovation_costs as f64 - down_payment_amount;

    let mut yearly_cash_flows: Vec<f64> = vec![];
    yearly_cash_flows.push(-down_payment_amount);

    // Calculate cash flows for each year
    for year in 1..(config.loan_duration_years + 1) {
        let income = rent
            * (1.0 + (config.avg_estimated_rent_increase_per_year as f64) / 100.0)
                .powf(year as f64);
        let vacancy = -rent * (config.avg_vacant_month_per_year as f64 / 12.0);
        let depreciation =
            -(config.avg_renovation_costs as f64 / config.loan_duration_years as f64);

        let ebit = income + vacancy + (-additional_cost) + depreciation;

        let taxes = -ebit * (config.tax as f64 / 100.0);
        let depreciation_add = depreciation;
        let interest = -loan * (interest_rate / 100.0);
        let loan_principal_repayment = -(config.loan_duration_years as f64 * 12.0 - year as f64)
            / (config.loan_duration_years as f64 * 12.0)
            * loan;
        let fcf = ebit + taxes + depreciation_add + interest + loan_principal_repayment;
        yearly_cash_flows.push(fcf);
    }

    let irr: f64 = _irr(yearly_cash_flows).unwrap_or_default();

    return irr;
}

// TODO
// Calculates the amount of interest that should be payed at a specific period.
fn _interest_payment_for_period(
    interest_rate: f64,
    period: f64,
    total_periods: f64,
    present_value: f64,
) -> f64 {
    return -present_value * interest_rate * ((1.0 + interest_rate).powf(period - 1.0))
        / (((1.0 + interest_rate).powf(total_periods)) - 1.0);
}

// Calculates the amount of interest that should be payed at a specific period.
fn _principal_payment_for_period(
    interest_rate: f64,
    period: f64,
    total_periods: f64,
    present_value: f64,
) -> f64 {
    return -present_value * interest_rate / (((1.0 + interest_rate).powf(total_periods)) - 1.0);
}

fn _irr(cash_flow: Vec<f64>) -> Option<f64> {
    let all_roots = _roots(cash_flow);
    let mut potential_roots: Vec<f64> = vec![];

    for root in all_roots {
        if root >= -1.0 {
            potential_roots.push(root);
        }
    }

    // If no real or valid roots
    if potential_roots.len() == 0 {
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
    return Some(min_root.0 - 1.0);
}

// https://math.mit.edu/~edelman/publications/polynomial_roots.pdf
// https://web.mit.edu/18.06/www/Spring17/Eigenvalue-Polynomials.pdf
// Find the roots of the polynomial given.
// Roots are the eigenvalues of the companion matrix of the polynomial.
fn _roots(coeffs: Vec<f64>) -> Vec<f64> {
    let n = coeffs.len() - 1;

    if n < 1 {
        return vec![];
    } else if n == 1 {
        return vec![-coeffs[0] / coeffs[1]];
    }

    let mut _companion_matrix = _companion_matrix(&coeffs);
    // Reverse matrix to minimize error of eigenvalue roots
    _companion_matrix = _reverse_matrix(&_companion_matrix);
    let eigenvalues_complx = _companion_matrix.complex_eigenvalues();
    let eigenvalues = eigenvalues_complx.data.as_vec();

    let mut real_roots: Vec<f64> = vec![];

    for r in eigenvalues {
        if r.im == 0.0 {
            real_roots.push(r.re);
        }
    }

    let roots = _map_domain(&real_roots, (-1.0, 1.0), (-1.0, 1.0));
    return roots;
}

// Generate the companion matrix of the polynomial given
fn _companion_matrix(polynomial: &Vec<f64>) -> DMatrix<f64> {
    let n = polynomial.len() - 1;

    if n < 1 {
        panic!("polynomial must have maximum degree of at least 1.");
    } else if n == 1 {
        let mut companion = DMatrix::<f64>::zeros(1, 1);
        companion[(0, 0)] = -polynomial[0] / polynomial[1];
        return companion;
    }

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

fn _reverse_matrix(matrix: &DMatrix<f64>) -> DMatrix<f64> {
    let mut reversed = DMatrix::<f64>::zeros(matrix.nrows(), matrix.ncols());

    for i in 0..matrix.nrows() {
        for j in 0..matrix.ncols() {
            reversed[(i, j)] = matrix[(matrix.nrows() - 1 - i, matrix.ncols() - 1 - j)];
        }
    }

    reversed
}

// https://github.com/numpy/numpy/blob/1c8b03bf2c87f081eea211a5061e423285c548af/numpy/polynomial/polyutils.py#L286
fn _map_domain(x: &Vec<f64>, _old: (f64, f64), _new: (f64, f64)) -> Vec<f64> {
    // let off = new.0 - ((new.1 - new.0) / (old.1 - old.0)) * old.0;
    // let scl = (new.1 - new.0) / (old.1 - old.0);

    return x.iter().map(|&val| 1.0 / val).collect();
}

#[cfg(test)]
mod matrix_tests {
    use nalgebra::dmatrix;

    use crate::producer::calculations::{_companion_matrix, _reverse_matrix};

    use super::_roots;

    #[test]
    async fn roots_test_1() {
        let input = vec![-2.0, 1.0]; //
        let expected: Vec<f64> = vec![2.0];
        let roots = _roots(input);
        let mut i = 0;
        while i < expected.len() {
            let expected_rounded: f64 = (expected[i] * 100.0).round() / 100.0;
            let real_rounded: f64 = (roots[i] * 100.0).round() / 100.0;

            assert_eq!(expected_rounded, real_rounded);
            i += 1;
        }
    }

    #[test]
    async fn test_root_2() {
        let coeffs = vec![1.0, -5.0, 6.0]; // x^2 - 5x + 6 = (x - 2)(x - 3)
        let expected: Vec<f64> = vec![2.0, 3.0]; // Roots: x = 2, 3
        let roots = _roots(coeffs);
        let mut i = 0;
        while i < expected.len() {
            let expected_rounded: f64 = (expected[i] * 100.0).round() / 100.0;
            let real_rounded: f64 = (roots[i] * 100.0).round() / 100.0;

            assert_eq!(expected_rounded, real_rounded);
            i += 1;
        }
    }

    #[test]
    async fn test_root_3() {
        let coeffs = vec![1.0, -6.0, 11.0, -6.0]; // x^3 - 6x^2 + 11x - 6 = (x - 1)(x - 2)(x - 3)
        let expected: Vec<f64> = vec![1.0, 2.0, 3.0]; // Roots: x = 1, 2, 3
        let roots = _roots(coeffs);
        let mut i = 0;
        while i < expected.len() {
            let expected_rounded: f64 = (expected[i] * 100.0).round() / 100.0;
            let real_rounded: f64 = (roots[i] * 100.0).round() / 100.0;

            assert_eq!(expected_rounded, real_rounded);
            i += 1;
        }
    }

    #[test]
    async fn test_root_4() {
        let coeffs = vec![1.0, 2.0, 3.0]; // x^2 + 2x + 3
        let expected: Vec<f64> = Vec::new(); // No real roots
        let roots = _roots(coeffs);
        let mut i = 0;
        while i < expected.len() {
            let expected_rounded: f64 = (expected[i] * 100.0).round() / 100.0;
            let real_rounded: f64 = (roots[i] * 100.0).round() / 100.0;

            assert_eq!(expected_rounded, real_rounded);
            i += 1;
        }
    }

    #[test]
    async fn _companion_matrix_test_1() {
        let input = vec![1.0, 2.0, 3.0];

        let expected = dmatrix![0.0, -0.3333333333333333;
                                1.0,  -0.6666666666666666;];

        let matrix = _companion_matrix(&input);

        assert_eq!(matrix, expected)
    }

    #[test]
    async fn _companion_matrix_test_2() {
        let input = vec![-2.0, 3.0, 4.0, -5.0, 1.0];

        let expected = dmatrix![
            0.0, 0.0, 0.0, 2.0;
            1.0, 0.0, 0.0, -3.0;
            0.0, 1.0, 0.0, -4.0;
            0.0, 0.0, 1.0, 5.0;
        ];

        let matrix = _companion_matrix(&input);

        assert_eq!(matrix, expected)
    }

    #[test]
    async fn _companion_matrix_test_3() {
        let input = vec![1.0, -6.0, 11.0, -6.0];

        let expected = dmatrix![
            0.0, 0.0, 0.16666666666666666;
            1.0, 0.0, -1.0;
            0.0, 1.0, 1.8333333333333333;
        ];

        let matrix = _companion_matrix(&input);

        assert_eq!(matrix, expected)
    }

    #[test]
    async fn test_reverse_matrix_1() {
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
        let matrix = _reverse_matrix(&input);

        assert_eq!(matrix, expected)
    }
}

#[cfg(test)]
mod yield_calculations {

    use crate::config;

    use super::*;

    #[test]
    async fn calculate_basic_yield() {
        let yield_ = calculate_irr(100000, 800, 200, 2.00);
        let yield_rounded = (yield_ * 10000.0).round() / 10000.0;
        assert_eq!(yield_rounded, 0.2075)
    }

    #[test]
    async fn calculate_basic_yield_wip() {
        let config = Arc::new(config::create_test_config());
        let yield_ = calculate_irr_wip(config, 100000 as f64, 800 as f64, 200 as f64, 2.00);
        let yield_rounded = (yield_ * 10000.0).round() / 10000.0;
        assert_eq!(yield_rounded, 0.2075)
    }

    #[test]
    async fn test_irr_1() {
        let cash_flow: Vec<f64> = vec![-100000.0, 20000.0, 50000.0, 70000.0];
        let irr_raw = _irr(cash_flow).unwrap();
        let irr = (irr_raw * 1000000.0).round() / 1000000.0;
        assert_eq!(irr, 0.156152)
    }

    #[test]
    async fn test_irr_2() {
        let cash_flow: Vec<f64> = vec![-100000.0, 20000.0, 50000.0, 20000.0];
        let irr_raw = _irr(cash_flow).unwrap();
        let irr = (irr_raw * 1000000.0).round() / 1000000.0;
        assert_eq!(irr, -0.051028)
    }

    #[test]
    async fn test_irr_3() {
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
    async fn test_interest_payment_period_1() {
        let interest_rate = 0.02;
        let period = 1.0;
        let total_periods = 25.0;
        let present_value = 84000.0;

        let result =
            _interest_payment_for_period(interest_rate, period, total_periods, present_value);
        let result_rounded = (result * 10000.0).round() / 10000.0;
        assert_eq!(result_rounded, -1680.00)
    }

    // TODO
    #[test]
    async fn test_interest_payment_period_2() {
        let interest_rate = 0.02;
        let period = 1.0;
        let total_periods = 25.0;
        let present_value = 84000.0;

        let result =
            _interest_payment_for_period(interest_rate, period, total_periods, present_value);
        let result_rounded = (result * 10000.0).round() / 10000.0;
        assert_eq!(result_rounded, -1680.00)
    }

    #[test]
    async fn test_principal_payment_period_1() {
        let interest_rate = 0.02;
        let period = 1.0;
        let total_periods = 25.0;
        let present_value = 84000.0;

        let result =
            _principal_payment_for_period(interest_rate, period, total_periods, present_value);
        let result_rounded = (result * 10000.0).round() / 10000.0;
        assert_eq!(result_rounded, -1680.00)
    }

    #[test]
    async fn test_principal_payment_period_2() {
        let interest_rate = 0.02;
        let period = 1.0;
        let total_periods = 25.0;
        let present_value = 84000.0;

        let result =
            _principal_payment_for_period(interest_rate, period, total_periods, present_value);
        let result_rounded = (result * 10000.0).round() / 10000.0;
        assert_eq!(result_rounded, -1680.00)
    }

    #[test]
    async fn float_test() {
        let yield_: f64 = 0.20748;
        let yield_rounded = (yield_ * 10000.0).round() / 10000.0;
        assert_eq!(yield_rounded, 0.2075)
    }
}
