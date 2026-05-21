use chrono::{DateTime, Utc};

pub fn format_age(created: &DateTime<Utc>) -> String {
    let now = Utc::now();
    let duration = now.signed_duration_since(*created);

    let seconds = duration.num_seconds();
    if seconds < 0 {
        return "just now".to_string();
    }

    let days = duration.num_days();
    if days > 0 {
        return format!("{days}d");
    }

    let hours = duration.num_hours();
    if hours > 0 {
        return format!("{hours}h");
    }

    let minutes = duration.num_minutes();
    if minutes > 0 {
        return format!("{minutes}m");
    }

    if seconds > 0 {
        return format!("{seconds}s");
    }

    "just now".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_format_age_days() {
        let now = Utc::now();
        let created = now - Duration::days(3);
        assert_eq!(format_age(&created), "3d");
    }

    #[test]
    fn test_format_age_hours() {
        let now = Utc::now();
        let created = now - Duration::hours(5);
        assert_eq!(format_age(&created), "5h");
    }

    #[test]
    fn test_format_age_minutes() {
        let now = Utc::now();
        let created = now - Duration::minutes(45);
        assert_eq!(format_age(&created), "45m");
    }

    #[test]
    fn test_format_age_seconds() {
        let now = Utc::now();
        let created = now - Duration::seconds(30);
        assert_eq!(format_age(&created), "30s");
    }

    #[test]
    fn test_format_age_just_now() {
        let now = Utc::now();
        assert_eq!(format_age(&now), "just now");
    }

    #[test]
    fn test_format_age_future() {
        let now = Utc::now();
        let future = now + Duration::hours(1);
        assert_eq!(format_age(&future), "just now");
    }
}
