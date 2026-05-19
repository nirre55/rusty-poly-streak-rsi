use chrono::{DateTime, Datelike, Timelike, Utc, Weekday};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TradingFilterReason {
    ExcludedDay(String),
    ExcludedHour(u32),
}

pub fn trading_filter_reason(
    close_time: DateTime<Utc>,
    excluded_days: &[String],
    excluded_hours: &[(u32, u32)],
) -> Option<TradingFilterReason> {
    let day = weekday_key(close_time.weekday());
    if excluded_days.iter().any(|excluded| excluded == day) {
        return Some(TradingFilterReason::ExcludedDay(day.to_string()));
    }

    let hour = close_time.hour();
    if excluded_hours
        .iter()
        .any(|&(start, end)| hour >= start && hour < end)
    {
        return Some(TradingFilterReason::ExcludedHour(hour));
    }

    None
}

fn weekday_key(day: Weekday) -> &'static str {
    match day {
        Weekday::Mon => "mon",
        Weekday::Tue => "tue",
        Weekday::Wed => "wed",
        Weekday::Thu => "thu",
        Weekday::Fri => "fri",
        Weekday::Sat => "sat",
        Weekday::Sun => "sun",
    }
}

#[cfg(test)]
mod tests {
    use super::{trading_filter_reason, TradingFilterReason};
    use chrono::{TimeZone, Utc};

    #[test]
    fn returns_none_when_no_filters_match() {
        let close_time = Utc.with_ymd_and_hms(2026, 5, 19, 15, 0, 0).unwrap();
        assert_eq!(
            trading_filter_reason(close_time, &["sun".to_string()], &[(0, 9)]),
            None
        );
    }

    #[test]
    fn returns_excluded_day_when_weekday_matches() {
        let close_time = Utc.with_ymd_and_hms(2026, 5, 19, 15, 0, 0).unwrap();
        assert_eq!(
            trading_filter_reason(close_time, &["tue".to_string()], &[]),
            Some(TradingFilterReason::ExcludedDay("tue".to_string()))
        );
    }

    #[test]
    fn returns_excluded_hour_when_hour_is_inside_range() {
        let close_time = Utc.with_ymd_and_hms(2026, 5, 19, 8, 30, 0).unwrap();
        assert_eq!(
            trading_filter_reason(close_time, &[], &[(0, 9)]),
            Some(TradingFilterReason::ExcludedHour(8))
        );
    }

    #[test]
    fn hour_range_end_is_exclusive() {
        let close_time = Utc.with_ymd_and_hms(2026, 5, 19, 9, 0, 0).unwrap();
        assert_eq!(trading_filter_reason(close_time, &[], &[(0, 9)]), None);
    }
}
