use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::rc::Rc;

/// Metadata about the Arc export
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    pub samples_completed: bool,
    pub export_mode: String,
    pub session_start_date: String,
    pub items_completed: bool,
    pub export_type: String,
    pub session_finish_date: String,
    pub stats: ExportStats,
    pub schema_version: String,
    pub places_completed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportStats {
    pub sample_count: u32,
    pub item_count: u32,
    pub place_count: u32,
}

/// A place/location from Arc
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Place {
    pub id: String,
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub radius_mean: f64,
    #[serde(rename = "radiusSD")]
    pub radius_sd: f64,
    pub visit_count: u32,
    pub visit_days: Option<u32>,
    pub last_saved: String,
    pub is_stale: bool,
    pub source: String,
    pub rtree_id: u32,
    pub seconds_from_gmt: Option<i32>,
    pub street_address: Option<String>,
    pub locality: Option<String>,
    pub country_code: Option<String>,
    pub google_place_id: Option<String>,
    pub google_primary_type: Option<String>,
    pub last_visit_date: Option<String>,
}

/// Item variant - either a visit or a trip
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ItemVariant {
    Visit(VisitDetails),
    Trip(TripDetails),
}

/// Timeline item with base data and variant-specific data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub base: BaseItem,
    #[serde(flatten)]
    pub variant: ItemVariant,
}

/// Base fields common to all items
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BaseItem {
    pub id: String,
    pub start_date: String,
    pub end_date: String,
    pub last_saved: String,
    pub source: String,
    pub source_version: Option<String>,
    pub is_visit: bool,
    pub deleted: bool,
    pub disabled: bool,
    pub locked: bool,
    pub samples_changed: Option<bool>,
    pub step_count: Option<u32>,
    pub active_energy_burned: Option<f64>,
    pub max_heart_rate: Option<f64>,
    pub average_heart_rate: Option<f64>,
    pub previous_item_id: Option<String>,
    pub next_item_id: Option<String>,
}

/// Details specific to visit items
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VisitDetails {
    pub item_id: String,
    pub place_id: Option<String>,
    pub latitude: f64,
    pub longitude: f64,
    pub radius_mean: f64,
    #[serde(rename = "radiusSD")]
    pub radius_sd: f64,
    pub confirmed_place: bool,
    pub uncertain_place: bool,
    pub last_saved: String,
    pub street_address: Option<String>,
}

/// Details specific to trip items
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TripDetails {
    pub item_id: String,
    pub distance: f64,
    pub speed: f64,
    pub classified_activity_type: Option<u32>,
    pub confirmed_activity_type: Option<u32>,
    pub uncertain_activity_type: bool,
    pub last_saved: String,
}

/// Parsed item with resolved place reference
#[derive(Debug, Clone)]
pub struct ItemWithPlace {
    pub item: Item,
    pub place: Option<Rc<Place>>,
}

impl Item {
    /// Check if this item is a visit
    pub fn is_visit(&self) -> bool {
        matches!(self.variant, ItemVariant::Visit(_))
    }

    /// Check if this item is a trip
    pub fn is_trip(&self) -> bool {
        matches!(self.variant, ItemVariant::Trip(_))
    }

    /// Get the place_id if this is a visit
    pub fn place_id(&self) -> Option<&str> {
        match &self.variant {
            ItemVariant::Visit(visit) => visit.place_id.as_deref(),
            ItemVariant::Trip(_) => None,
        }
    }

    /// Get the start date as DateTime
    pub fn start_datetime(&self) -> DateTime<Utc> {
        parse_iso8601_timestamp(&self.base.start_date).expect("Invalid start_date timestamp")
    }

    /// Get the end date as DateTime
    pub fn end_datetime(&self) -> DateTime<Utc> {
        parse_iso8601_timestamp(&self.base.end_date).expect("Invalid end_date timestamp")
    }

    /// Get the duration in seconds
    pub fn duration_seconds(&self) -> f64 {
        let start = self.start_datetime();
        let end = self.end_datetime();
        (end - start).num_milliseconds() as f64 / 1000.0
    }
}

impl Place {
    /// Get the last saved date as DateTime
    pub fn last_saved_datetime(&self) -> DateTime<Utc> {
        parse_iso8601_timestamp(&self.last_saved).expect("Invalid last_saved timestamp")
    }

    /// Get the last visit date as DateTime if available
    pub fn last_visit_datetime(&self) -> Option<DateTime<Utc>> {
        self.last_visit_date
            .as_ref()
            .map(|s| parse_iso8601_timestamp(s).expect("Invalid last_visit_date timestamp"))
    }
}

/// Parse ISO 8601 timestamp string to DateTime<Utc>
pub fn parse_iso8601_timestamp(timestamp: &str) -> Result<DateTime<Utc>, chrono::ParseError> {
    DateTime::parse_from_rfc3339(timestamp).map(|dt| dt.with_timezone(&Utc))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;

    #[test]
    fn test_iso8601_timestamp_parsing() {
        // Test parsing ISO 8601 timestamp from the new format
        let dt = parse_iso8601_timestamp("2025-12-20T22:20:04Z").expect("Failed to parse");

        // This should be December 20, 2025
        assert_eq!(dt.year(), 2025);
        assert_eq!(dt.month(), 12);
        assert_eq!(dt.day(), 20);
    }

    #[test]
    fn test_item_helpers() {
        let visit_item = Item {
            base: BaseItem {
                id: "test-id".to_string(),
                start_date: "2025-12-02T23:42:31Z".to_string(),
                end_date: "2025-12-02T23:58:15Z".to_string(),
                last_saved: "2025-12-03T02:10:04Z".to_string(),
                source: "LocoKit2".to_string(),
                source_version: Some("9.0.0".to_string()),
                is_visit: true,
                deleted: false,
                disabled: false,
                locked: false,
                samples_changed: Some(false),
                step_count: Some(252),
                active_energy_burned: Some(29.79),
                max_heart_rate: Some(133.0),
                average_heart_rate: Some(92.11),
                previous_item_id: None,
                next_item_id: Some("next-id".to_string()),
            },
            variant: ItemVariant::Visit(VisitDetails {
                item_id: "test-id".to_string(),
                place_id: Some("place-id".to_string()),
                latitude: 38.5,
                longitude: -90.4,
                radius_mean: 10.0,
                radius_sd: 5.42,
                confirmed_place: true,
                uncertain_place: false,
                last_saved: "2025-12-02T23:58:02Z".to_string(),
                street_address: Some("123 Main St".to_string()),
            }),
        };

        assert!(visit_item.is_visit());
        assert!(!visit_item.is_trip());
        assert_eq!(visit_item.place_id(), Some("place-id"));
        assert!(visit_item.duration_seconds() > 0.0);
    }
}
