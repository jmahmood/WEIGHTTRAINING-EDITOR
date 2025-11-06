use chrono::Utc;

/// Returns the current UTC timestamp formatted as RFC3339 (e.g., 2025-08-14T09:37:12Z)
pub fn now_utc_rfc3339() -> String {
    Utc::now().to_rfc3339()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{offset::TimeZone, DateTime};

    #[test]
    fn test_now_utc_rfc3339_is_utc() {
        let s = now_utc_rfc3339();
        // Parse and confirm UTC offset
        let dt: DateTime<chrono::FixedOffset> = DateTime::parse_from_rfc3339(&s).expect("valid RFC3339");
        assert_eq!(dt.offset().local_minus_utc(), 0);
        // Converting to Utc should succeed
        let _utc_dt = dt.with_timezone(&Utc);
    }
}
