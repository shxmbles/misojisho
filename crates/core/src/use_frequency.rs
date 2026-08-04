#[derive(Debug, PartialEq)]
pub enum UseFrequency {
    Ichi(u8),
    News(u8),
    LoanWord(u8),
    NewsFrequency(u8),
}

impl UseFrequency {
    /// Returns a rank where lower = higher priority.
    /// Ichi tags are most reliable for everyday usage; NewsFrequency(nf) is least.
    pub fn priority(&self) -> u8 {
        match self {
            UseFrequency::Ichi(1) => 1,
            UseFrequency::Ichi(2) => 2,
            UseFrequency::News(1) => 3,
            UseFrequency::News(2) => 4,
            UseFrequency::LoanWord(1) => 5,
            UseFrequency::LoanWord(2) => 6,
            UseFrequency::NewsFrequency(n) => 6 + *n as u8,
            _ => u8::MAX,
        }
    }

    pub fn from_priority_code(value: &str) -> Option<Self> {
        match value {
            "ichi1" => Some(UseFrequency::Ichi(1)),
            "news1" => Some(UseFrequency::News(1)),
            "ichi2" => Some(UseFrequency::Ichi(2)),
            "news2" => Some(UseFrequency::News(2)),
            "gai1" => Some(UseFrequency::LoanWord(1)),
            "gai2" => Some(UseFrequency::LoanWord(2)),
            _ => value
                .strip_prefix("nf")
                .and_then(|n| n.parse::<u8>().ok())
                .map(UseFrequency::NewsFrequency),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::u8;

    use super::*;

    #[test]
    fn should_map_ichi1_to_highest_priority_u8() {
        let actual = UseFrequency::priority(&UseFrequency::Ichi(1));

        assert_eq!(actual, 1)
    }

    #[test]
    fn should_map_ichi2_to_priority_u8() {
        let actual = UseFrequency::priority(&UseFrequency::Ichi(2));

        assert_eq!(actual, 2)
    }

    #[test]
    fn should_map_news1_to_priority_u8() {
        let actual = UseFrequency::priority(&UseFrequency::News(1));

        assert_eq!(actual, 3)
    }

    #[test]
    fn should_map_news2_to_priority_u8() {
        let actual = UseFrequency::priority(&UseFrequency::News(2));

        assert_eq!(actual, 4)
    }

    #[test]
    fn should_map_loan_word1_to_priority_u8() {
        let actual = UseFrequency::priority(&UseFrequency::LoanWord(1));

        assert_eq!(actual, 5)
    }

    #[test]
    fn should_map_loan_word2_to_priority_u8() {
        let actual = UseFrequency::priority(&UseFrequency::LoanWord(2));

        assert_eq!(actual, 6)
    }

    #[test]
    fn should_map_nf_to_priority_u8() {
        let actual = UseFrequency::priority(&UseFrequency::NewsFrequency(1));

        assert_eq!(actual, 7)
    }

    #[test]
    fn should_map_out_of_range_to_max_u8() {
        let actual = UseFrequency::priority(&UseFrequency::Ichi(5));

        assert_eq!(actual, u8::MAX)
    }

    #[test]
    fn should_map_str_to_ichi1() {
        let actual = UseFrequency::from_priority_code("ichi1");

        assert_eq!(actual, Some(UseFrequency::Ichi(1)))
    }

    #[test]
    fn should_map_str_to_ichi2() {
        let actual = UseFrequency::from_priority_code("ichi2");

        assert_eq!(actual, Some(UseFrequency::Ichi(2)))
    }

    #[test]
    fn should_map_str_to_news1() {
        let actual = UseFrequency::from_priority_code("news1");

        assert_eq!(actual, Some(UseFrequency::News(1)))
    }

    #[test]
    fn should_map_str_to_news2() {
        let actual = UseFrequency::from_priority_code("news2");

        assert_eq!(actual, Some(UseFrequency::News(2)))
    }

    #[test]
    fn should_map_str_to_loan_word1() {
        let actual = UseFrequency::from_priority_code("gai1");

        assert_eq!(actual, Some(UseFrequency::LoanWord(1)))
    }

    #[test]
    fn should_map_str_to_loan_word2() {
        let actual = UseFrequency::from_priority_code("gai2");

        assert_eq!(actual, Some(UseFrequency::LoanWord(2)))
    }

    #[test]
    fn should_map_str_to_nf_number_frequency() {
        let actual = UseFrequency::from_priority_code("nf30");

        assert_eq!(actual, Some(UseFrequency::NewsFrequency(30)))
    }

    #[test]
    fn should_not_map_str_to_nf_number_frequency_when_number_is_not_u8() {
        let actual = UseFrequency::from_priority_code("nf30.2");

        assert_eq!(actual, None)
    }

    #[test]
    fn should_return_none_when_no_matched_value() {
        let actual = UseFrequency::from_priority_code("Viktor Gyökeres");

        assert_eq!(actual, None)
    }
}
