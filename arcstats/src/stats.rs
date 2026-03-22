use anyhow::Result;
use chrono::{DateTime, Datelike, Duration, Timelike, Utc};
use chrono_tz::America::Chicago;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

use crate::loader::load_all_items_with_places;
use crate::models::Place;
use statsutils::DatePeriod;

const MARTIN_LUTHER_CHURCH: &str = "Martin Luther Church";
const ROLLOVER_HOUR: u32 = 4;

/// Checks if a place is a church based on Google place type or place name
fn is_church(place: &Place) -> bool {
    if place.name == MARTIN_LUTHER_CHURCH {
        return true;
    }

    if let Some(ref primary_type) = place.google_primary_type
        && primary_type == "church"
    {
        return true;
    }

    // Fallback: check if name contains "Church"
    place.name.contains("Church")
}

/// Checks if a visit time falls on a Sunday morning (4 AM–1 PM Chicago time).
/// Uses the same 4 AM rollover as the rest of the module.
fn is_sunday_morning(dt: DateTime<Utc>) -> bool {
    let dt_chicago = dt.with_timezone(&Chicago);
    let hour = dt_chicago.hour();

    // Apply 4 AM rollover: before 4 AM counts as previous day
    if hour < ROLLOVER_HOUR {
        return false; // Before rollover, so this is really Saturday night
    }

    // After rollover, check if it's Sunday and morning (before 1 PM)
    dt_chicago.weekday() == chrono::Weekday::Sun && hour < 13
}

/// Weekly statistics for church attendance
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct WeekStats {
    /// Week start date in YYYY-MM-DD format (Sunday)
    pub week_start: String,
    /// Time spent at church in minutes
    pub minutes: f64,
    /// Daily breakdown: [Sunday, Monday, Tuesday, Wednesday, Thursday, Friday, Saturday]
    /// Index 0 = Sunday, Index 6 = Saturday
    pub daily_minutes: Vec<f64>,
}

/// Statistics for a single place showing time spent
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PlaceStats {
    /// Name of the place
    pub place_name: String,
    /// Total hours spent at this place
    pub hours: f64,
}

/// Converts a UTC datetime to a week start date string (YYYY-MM-DD)
/// Applies 4 AM rollover and finds the most recent Sunday in Chicago timezone
fn get_week_start_for_datetime(dt: DateTime<Utc>) -> String {
    // Convert to Chicago timezone
    let dt_chicago = dt.with_timezone(&Chicago);

    // Apply 4 AM rollover: if before 4 AM, consider it part of previous day
    let adjusted_dt = if dt_chicago.hour() < ROLLOVER_HOUR {
        dt_chicago - Duration::hours(24)
    } else {
        dt_chicago
    };

    // Calculate days since last Sunday (0 if today is Sunday)
    let days_since_sunday = adjusted_dt.weekday().num_days_from_sunday();

    // Go back to the most recent Sunday
    let week_start = adjusted_dt - Duration::days(days_since_sunday as i64);

    // Format as YYYY-MM-DD
    week_start.format("%Y-%m-%d").to_string()
}

/// Gets day of week index (0=Sunday, 6=Saturday) for a datetime with 4 AM rollover
fn get_day_of_week_index(dt: DateTime<Utc>) -> usize {
    // Convert to Chicago timezone
    let dt_chicago = dt.with_timezone(&Chicago);

    // Apply 4 AM rollover: if before 4 AM, consider it part of previous day
    let adjusted_dt = if dt_chicago.hour() < ROLLOVER_HOUR {
        dt_chicago - Duration::hours(24)
    } else {
        dt_chicago
    };

    // Get day of week (0=Sunday, 6=Saturday)
    adjusted_dt.weekday().num_days_from_sunday() as usize
}

/// Gets church attendance statistics for the last 12 weeks
///
/// # Arguments
///
/// * `export_path` - Path to the Arc Timeline export directory containing places/, items/, and metadata.json
///
/// # Returns
///
/// A vector of 12 WeekStats, one for each week, in chronological order.
/// Weeks without church visits will have 0 minutes.
pub fn get_last_12_weeks_stats(export_path: &str) -> Result<Vec<WeekStats>> {
    // Get the period data for the last 12 weeks
    let period = DatePeriod::last_12_weeks()?;

    // Load all items with their associated places
    let items = load_all_items_with_places(export_path)?;

    // Filter for visits at any church and calculate duration in minutes for each visit
    let mut church_visits: Vec<(DateTime<Utc>, f64)> = Vec::new();

    for item_with_place in items {
        // Skip if not a visit
        if !item_with_place.item.base.is_visit {
            continue;
        }

        // Include church visits: Martin Luther Church any time,
        // other churches only on Sunday mornings
        if let Some(place) = &item_with_place.place
            && is_church(place)
        {
            let start = item_with_place.item.start_datetime();
            let is_martin_luther = place.name == MARTIN_LUTHER_CHURCH;

            if is_martin_luther || is_sunday_morning(start) {
                let duration_minutes = item_with_place.item.duration_seconds() / 60.0;
                church_visits.push((start, duration_minutes));
            }
        }
    }

    // Track both total and daily breakdown per week
    // HashMap<week_start, (total_minutes, [daily_minutes; 7])>
    let mut weekly_data: HashMap<String, (f64, [f64; 7])> = HashMap::new();

    for (visit_time, minutes) in church_visits {
        let week_start = get_week_start_for_datetime(visit_time);
        let day_index = get_day_of_week_index(visit_time);

        let entry = weekly_data.entry(week_start).or_insert((0.0, [0.0; 7]));
        entry.0 += minutes; // Total minutes
        entry.1[day_index] += minutes; // Daily breakdown
    }

    // Build results for all 12 weeks, filling gaps with 0 minutes
    let results = period.build_results(weekly_data, |date, (total, daily)| {
        // Validation: sum of daily should equal total (within rounding tolerance)
        debug_assert!(
            (daily.iter().sum::<f64>() - total).abs() < 0.1,
            "Daily sum doesn't match total for week {}: daily sum = {}, total = {}",
            date,
            daily.iter().sum::<f64>(),
            total
        );

        WeekStats {
            week_start: date,
            minutes: total,
            daily_minutes: daily.to_vec(),
        }
    });

    Ok(results)
}

/// Gets the top N places by total hours spent over the last 6 months
///
/// # Arguments
///
/// * `export_path` - Path to the Arc Timeline export directory containing places/, items/, and metadata.json
/// * `limit` - Maximum number of places to return (e.g., 10 for top 10)
///
/// # Returns
///
/// A vector of PlaceStats sorted by hours descending (most time first).
/// Excludes the place named "Home".
pub fn get_top_places_last_6_months(export_path: &str, limit: usize) -> Result<Vec<PlaceStats>> {
    const DAYS_IN_6_MONTHS: i64 = 182;

    // Calculate the cutoff date (6 months ago)
    let now = Utc::now();
    let cutoff_date = now - Duration::days(DAYS_IN_6_MONTHS);

    // Load all items with their associated places
    let items = load_all_items_with_places(export_path)?;

    // Collect visits with place names and durations
    let mut place_durations: HashMap<String, f64> = HashMap::new();

    for item_with_place in items {
        // Skip if not a visit
        if !item_with_place.item.base.is_visit {
            continue;
        }

        // Skip if no place
        let Some(place) = &item_with_place.place else {
            continue;
        };

        // Skip if place name is "Home"
        if place.name == "Home" {
            continue;
        }

        // Skip if visit is before cutoff date
        let visit_start = item_with_place.item.start_datetime();
        if visit_start < cutoff_date {
            continue;
        }

        // Calculate duration in hours
        let duration_hours = item_with_place.item.duration_seconds() / 3600.0;

        // Add to place total
        *place_durations.entry(place.name.clone()).or_insert(0.0) += duration_hours;
    }

    // Convert to vec of PlaceStats and sort by hours descending
    let mut place_stats: Vec<PlaceStats> = place_durations
        .into_iter()
        .map(|(place_name, hours)| PlaceStats { place_name, hours })
        .collect();

    place_stats.sort_by(|a, b| {
        b.hours
            .partial_cmp(&a.hours)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Take top N
    place_stats.truncate(limit);

    Ok(place_stats)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_week_stats_structure() {
        let stats = WeekStats {
            week_start: "2025-10-19".to_string(),
            minutes: 120.5,
            daily_minutes: vec![120.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        };

        assert_eq!(stats.week_start, "2025-10-19");
        assert_eq!(stats.minutes, 120.5);
        assert_eq!(stats.daily_minutes.len(), 7);
        assert_eq!(stats.daily_minutes[0], 120.5); // Sunday
    }
}
