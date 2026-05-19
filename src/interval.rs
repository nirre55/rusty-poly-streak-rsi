use anyhow::Result;
use chrono::Duration;

pub fn parse_interval_duration(interval: &str) -> Result<Duration> {
    if interval.len() < 2 {
        anyhow::bail!("intervalle invalide: {}", interval);
    }
    let (value, unit) = interval.split_at(interval.len() - 1);
    let value: i64 = value.parse()?;
    match unit {
        "m" => Ok(Duration::minutes(value)),
        "h" => Ok(Duration::hours(value)),
        "d" => Ok(Duration::days(value)),
        _ => anyhow::bail!("unité d'intervalle non supportée: {}", interval),
    }
}

#[cfg(test)]
mod tests {
    use super::parse_interval_duration;
    use chrono::Duration;

    #[test]
    fn parses_minutes_hours_and_days() {
        assert_eq!(parse_interval_duration("5m").unwrap(), Duration::minutes(5));
        assert_eq!(parse_interval_duration("2h").unwrap(), Duration::hours(2));
        assert_eq!(parse_interval_duration("1d").unwrap(), Duration::days(1));
    }

    #[test]
    fn rejects_invalid_interval() {
        assert!(parse_interval_duration("").is_err());
        assert!(parse_interval_duration("m").is_err());
        assert!(parse_interval_duration("5x").is_err());
    }
}
