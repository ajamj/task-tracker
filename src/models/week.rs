//! Week range calculations with ISO-8601 Monday start.

use chrono::{Datelike, Duration, NaiveDate};
use regex::Regex;
use serde::Serialize;

/// A week range with Monday start and Sunday end.
#[derive(Debug, Clone, Serialize)]
pub struct WeekRange {
    /// Monday (start of week).
    pub start: NaiveDate,
    /// Sunday (end of week).
    pub end: NaiveDate,
    /// ISO week string (e.g., "2026-W13").
    pub iso_week: String,
    /// Year.
    pub year: i32,
    /// Week number.
    pub week: u32,
}

impl WeekRange {
    /// Create a WeekRange from a date.
    ///
    /// The week starts on Monday (ISO-8601).
    ///
    /// # Example
    ///
    /// ```
    /// use chrono::NaiveDate;
    /// let date = NaiveDate::from_ymd_opt(2026, 3, 28).unwrap(); // Saturday
    /// let week = WeekRange::from_date(date);
    /// assert_eq!(week.start, NaiveDate::from_ymd_opt(2026, 3, 23).unwrap()); // Monday
    /// assert_eq!(week.end, NaiveDate::from_ymd_opt(2026, 3, 29).unwrap()); // Sunday
    /// assert_eq!(week.iso_week, "2026-W13");
    /// ```
    pub fn from_date(date: NaiveDate) -> Self {
        let weekday = date.weekday();
        let days_since_monday = weekday.num_days_from_monday();
        let start = date - Duration::days(days_since_monday as i64);
        let end = start + Duration::days(6);

        Self {
            start,
            end,
            iso_week: format!("{}-W{:02}", date.year(), date.iso_week().week()),
            year: date.year(),
            week: date.iso_week().week(),
        }
    }

    /// Create a WeekRange from an ISO week string (e.g., "2026-W13").
    ///
    /// # Example
    ///
    /// ```
    /// let week = WeekRange::from_iso_string("2026-W13").unwrap();
    /// assert_eq!(week.year, 2026);
    /// assert_eq!(week.week, 13);
    /// ```
    pub fn from_iso_string(iso: &str) -> Option<Self> {
        let re = Regex::new(r"^(\d{4})-W(\d{2})$").ok()?;
        let caps = re.captures(iso)?;
        let year: i32 = caps.get(1)?.as_str().parse().ok()?;
        let week: u32 = caps.get(2)?.as_str().parse().ok()?;

        // Find Monday of that week
        // ISO week 1 contains January 4th
        let jan4 = NaiveDate::from_ymd_opt(year, 1, 4)?;
        let mut start =
            jan4 - Duration::days(jan4.weekday().num_days_from_monday() as i64);
        start = start + Duration::days((week - 1) as i64 * 7);

        Some(Self::from_date(start))
    }

    /// Check if a date is within this week range.
    pub fn contains(&self, date: NaiveDate) -> bool {
        date >= self.start && date <= self.end
    }

    /// Get all days in this week range.
    pub fn days(&self) -> Vec<NaiveDate> {
        let mut days = Vec::new();
        let mut current = self.start;
        while current <= self.end {
            days.push(current);
            current = current + Duration::days(1);
        }
        days
    }

    /// Get the current week range.
    pub fn current() -> Self {
        Self::from_date(chrono::Local::now().date_naive())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_week_range_from_date_saturday() {
        let date = NaiveDate::from_ymd_opt(2026, 3, 28).unwrap(); // Saturday
        let week = WeekRange::from_date(date);

        assert_eq!(week.start, NaiveDate::from_ymd_opt(2026, 3, 23).unwrap()); // Monday
        assert_eq!(week.end, NaiveDate::from_ymd_opt(2026, 3, 29).unwrap()); // Sunday
        assert_eq!(week.iso_week, "2026-W13");
        assert_eq!(week.year, 2026);
        assert_eq!(week.week, 13);
    }

    #[test]
    fn test_week_range_from_date_monday() {
        let date = NaiveDate::from_ymd_opt(2026, 3, 23).unwrap(); // Monday
        let week = WeekRange::from_date(date);

        assert_eq!(week.start, NaiveDate::from_ymd_opt(2026, 3, 23).unwrap());
        assert_eq!(week.end, NaiveDate::from_ymd_opt(2026, 3, 29).unwrap());
        assert_eq!(week.iso_week, "2026-W13");
    }

    #[test]
    fn test_week_range_from_date_sunday() {
        let date = NaiveDate::from_ymd_opt(2026, 3, 29).unwrap(); // Sunday
        let week = WeekRange::from_date(date);

        assert_eq!(week.start, NaiveDate::from_ymd_opt(2026, 3, 23).unwrap());
        assert_eq!(week.end, NaiveDate::from_ymd_opt(2026, 3, 29).unwrap());
        assert_eq!(week.iso_week, "2026-W13");
    }

    #[test]
    fn test_week_range_from_iso_string() {
        let week = WeekRange::from_iso_string("2026-W13").unwrap();

        assert_eq!(week.year, 2026);
        assert_eq!(week.week, 13);
        assert_eq!(week.start, NaiveDate::from_ymd_opt(2026, 3, 23).unwrap());
        assert_eq!(week.end, NaiveDate::from_ymd_opt(2026, 3, 29).unwrap());
    }

    #[test]
    fn test_week_range_from_iso_string_invalid() {
        assert!(WeekRange::from_iso_string("invalid").is_none());
        assert!(WeekRange::from_iso_string("2026-13").is_none());
        assert!(WeekRange::from_iso_string("W13").is_none());
    }

    #[test]
    fn test_week_range_contains() {
        let week = WeekRange::from_iso_string("2026-W13").unwrap();

        assert!(week.contains(NaiveDate::from_ymd_opt(2026, 3, 23).unwrap())); // Monday
        assert!(week.contains(NaiveDate::from_ymd_opt(2026, 3, 28).unwrap())); // Saturday
        assert!(week.contains(NaiveDate::from_ymd_opt(2026, 3, 29).unwrap())); // Sunday
        assert!(!week.contains(NaiveDate::from_ymd_opt(2026, 3, 22).unwrap())); // Previous Sunday
        assert!(!week.contains(NaiveDate::from_ymd_opt(2026, 3, 30).unwrap())); // Next Monday
    }

    #[test]
    fn test_week_range_days() {
        let week = WeekRange::from_iso_string("2026-W13").unwrap();
        let days = week.days();

        assert_eq!(days.len(), 7);
        assert_eq!(days[0], NaiveDate::from_ymd_opt(2026, 3, 23).unwrap()); // Monday
        assert_eq!(days[6], NaiveDate::from_ymd_opt(2026, 3, 29).unwrap()); // Sunday
    }

    #[test]
    fn test_week_range_year_boundary() {
        // Test week that spans year boundary
        let date = NaiveDate::from_ymd_opt(2025, 12, 31).unwrap();
        let week = WeekRange::from_date(date);

        // Week should start on Monday Dec 29, 2025
        assert_eq!(week.start, NaiveDate::from_ymd_opt(2025, 12, 29).unwrap());
        assert_eq!(week.end, NaiveDate::from_ymd_opt(2026, 1, 4).unwrap());
    }

    #[test]
    fn test_week_range_iso_week_1() {
        // ISO week 1 of 2026
        let week = WeekRange::from_iso_string("2026-W01").unwrap();

        assert_eq!(week.year, 2026);
        assert_eq!(week.week, 1);
        // ISO week 1 of 2026 starts on Dec 29, 2025
        assert_eq!(week.start, NaiveDate::from_ymd_opt(2025, 12, 29).unwrap());
    }
}
