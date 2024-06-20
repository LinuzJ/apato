use anyhow::anyhow;
use anyhow::Result;
use nalgebra::DMatrix;
use std::cmp::Ordering;
use std::{sync::Arc, vec};

use crate::{
    config::Config, interest_rate::interest_rate_client, models::apartment::InsertableApartment,
};

pub async fn get_estimated_irr(
    config: &Arc<Config>,
    apartment: InsertableApartment,
) -> Result<f64> {
    /*
       Calculate yield here
       - Get rent for similar apartments close by
       - Get interest rate from Nordea
       - Calculate
    */

    let interest_rate_result = interest_rate_client::get_interest_rate(config).await;
    let interest_rate = match interest_rate_result {
        Ok(r) => r,
        Err(_e) => {
            return Err(anyhow!(
                "Failed to fetch interest for apartment {}",
                apartment.card_id,
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
    let irr: f64 = irr(yearly_cash_flows).unwrap_or_default() * 100.0;

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

pub fn pmt(interest_rate: f64, periods: f64, pv: f64) -> f64 {
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

pub fn future_value(interest_rate: f64, periods: f64, c: f64, pv: f64) -> f64 {
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
pub fn interest_payment_for_period(
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
pub fn principal_payment_for_period(
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

pub fn valuation_increase(config: &Arc<Config>, price: f64, year: u32) -> f64 {
    let growth_rate = config.estimated_yearly_apartment_price_increase as f64 / 100.0;
    let this_year = price * (1.0 + growth_rate).powf(year as f64);
    let last_year = price * (1.0 + growth_rate).powf(year as f64 - 1.0);
    this_year - last_year
}

pub fn irr(cash_flow: Vec<f64>) -> Option<f64> {
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
pub fn get_roots(coeffs: Vec<f64>) -> Vec<f64> {
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
pub fn companion_matrix(polynomial: &Vec<f64>) -> DMatrix<f64> {
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

pub fn reverse_matrix(matrix: &DMatrix<f64>) -> DMatrix<f64> {
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
