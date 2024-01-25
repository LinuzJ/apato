#[cfg(test)]
mod calculations_test {
    #[test]
    fn yield_calculations_work() {
        let price: i32 = 1000000;
        let rent: i32 = 1000;
        let additional_costs: i32 = 100;
        let interest_rate: f64 = 0.005;

        let yield_ = apato::calculate_rental_yield(price, rent, additional_costs, interest_rate);
        let yield_rounded: f64 = (yield_ * 100.0).round() / 100.0;
        assert_eq!(yield_rounded, 5.33)
    }
}
