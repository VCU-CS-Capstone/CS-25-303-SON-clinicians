use chrono::{Datelike, NaiveDate};

pub trait RandDate {
    fn random_date_max_now(&mut self) -> NaiveDate {
        self.random_date_with_range(
            &NaiveDate::from_ymd_opt(1900, 1, 1).unwrap(),
            &chrono::Local::now().date_naive(),
        )
    }
    fn random_date_in_year(&mut self, year: i32) -> NaiveDate {
        self.random_date_with_range(
            &NaiveDate::from_ymd_opt(year, 1, 1).unwrap(),
            &NaiveDate::from_ymd_opt(year, 12, 31).unwrap(),
        )
    }

    fn random_date_with_range(&mut self, min: &NaiveDate, max: &NaiveDate) -> NaiveDate;
}

impl<R> RandDate for R
where
    R: rand::Rng,
{
    fn random_date_with_range(&mut self, min: &NaiveDate, max: &NaiveDate) -> NaiveDate {
        let random_year = self.random_range(min.year()..=max.year());
        let random_month = if random_year == min.year() && random_year == max.year() {
            self.random_range(min.month()..=max.month())
        } else if random_year == min.year() {
            self.random_range(min.month()..=12)
        } else if random_year == max.year() {
            self.random_range(1..=max.month())
        } else {
            self.random_range(1..=12)
        };

        let day = if random_month == min.month() && random_year == min.year() {
            self.random_range(min.day()..=31)
        } else if random_month == max.month() && random_year == max.year() {
            self.random_range(1..=max.day())
        } else {
            self.random_range(1..=chrono_utils::days_in_month(random_year, random_month))
        };
        let Some(result) = NaiveDate::from_ymd_opt(random_year, random_month, day) else {
            // Just incase something is out of bounds just try again
            return self.random_date_with_range(min, max);
        };
        result
    }
}
mod chrono_utils {
    use chrono::NaiveDate;

    pub fn days_in_month(year: i32, month: u32) -> u32 {
        if month == 2 {
            if is_leap_year(year) { 29 } else { 28 }
        } else if month == 4 || month == 6 || month == 9 || month == 11 {
            30
        } else {
            31
        }
    }
    /// https://github.com/chronotope/chrono/issues/29
    pub fn is_leap_year(year: i32) -> bool {
        NaiveDate::from_ymd_opt(year, 2, 29).is_some()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    #[test]
    fn test_rand_date_with_range() {
        let mut rng = StdRng::seed_from_u64(0);
        let min = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
        let max = NaiveDate::from_ymd_opt(2020, 12, 31).unwrap();
        let date = rng.random_date_with_range(&min, &max);
        assert_eq!(date.year(), 2020);
        assert!(date.month() >= 1);
        assert!(date.month() <= 12);
    }

    #[test]
    fn test_random_date_in_year() {
        let mut rng = StdRng::seed_from_u64(0);
        let date = rng.random_date_in_year(2020);
        assert_eq!(date.year(), 2020);
        assert!(date.month() >= 1);
        assert!(date.month() <= 12);
    }
}
