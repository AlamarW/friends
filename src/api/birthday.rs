use anyhow::{anyhow, Result};
use chrono::{Datelike, Local, NaiveDate};
use std::collections::HashMap;

use crate::apps::AppletItem;

pub fn compute_birthday_items(profile: &HashMap<String, String>) -> Result<Vec<AppletItem>> {
    let raw = profile
        .get("birthday")
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| anyhow!("birthday field is required (YYYY-MM-DD)"))?;

    let dob = NaiveDate::parse_from_str(raw.trim(), "%Y-%m-%d")
        .map_err(|_| anyhow!("Cannot parse birthday '{}' — expected YYYY-MM-DD", raw.trim()))?;

    let today = Local::now().date_naive();
    let current_year = today.year();

    let this_year_bday = NaiveDate::from_ymd_opt(current_year, dob.month(), dob.day())
        .ok_or_else(|| anyhow!("Invalid birthday date (Feb 29 on non-leap year?)"))?;

    let (next_bday, upcoming_age) = if this_year_bday >= today {
        (this_year_bday, current_year - dob.year())
    } else {
        let next = NaiveDate::from_ymd_opt(current_year + 1, dob.month(), dob.day())
            .ok_or_else(|| anyhow!("Cannot compute next birthday"))?;
        (next, current_year + 1 - dob.year())
    };

    let days_until = (next_bday - today).num_days();
    let countdown = match days_until {
        0 => "Today!".to_string(),
        1 => "Tomorrow".to_string(),
        n => format!("In {n} days"),
    };

    Ok(vec![
        AppletItem {
            title: format!("Birthday: {} {} ({})", month_name(dob.month()), dob.day(), countdown),
            subtitle: Some(format!("Turning {} years old", upcoming_age)),
            url: None,
        },
        AppletItem {
            title: format!("Zodiac: {}", zodiac_sign(dob.month(), dob.day())),
            subtitle: Some(format!(
                "Born {} {}, {}",
                month_name(dob.month()),
                dob.day(),
                dob.year()
            )),
            url: None,
        },
    ])
}

fn zodiac_sign(month: u32, day: u32) -> &'static str {
    match (month, day) {
        (3, 21..=31) | (4, 1..=19) => "Aries",
        (4, 20..=30) | (5, 1..=20) => "Taurus",
        (5, 21..=31) | (6, 1..=20) => "Gemini",
        (6, 21..=30) | (7, 1..=22) => "Cancer",
        (7, 23..=31) | (8, 1..=22) => "Leo",
        (8, 23..=31) | (9, 1..=22) => "Virgo",
        (9, 23..=30) | (10, 1..=22) => "Libra",
        (10, 23..=31) | (11, 1..=21) => "Scorpio",
        (11, 22..=30) | (12, 1..=21) => "Sagittarius",
        (12, 22..=31) | (1, 1..=19) => "Capricorn",
        (1, 20..=31) | (2, 1..=18) => "Aquarius",
        (2, 19..=29) | (3, 1..=20) => "Pisces",
        _ => "Unknown",
    }
}

fn month_name(month: u32) -> &'static str {
    match month {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => "Unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile(birthday: &str) -> HashMap<String, String> {
        [("birthday".to_string(), birthday.to_string())].into()
    }

    #[test]
    fn test_zodiac_aries_march() {
        assert_eq!(zodiac_sign(3, 21), "Aries");
        assert_eq!(zodiac_sign(3, 20), "Pisces");
    }

    #[test]
    fn test_zodiac_aries_april() {
        assert_eq!(zodiac_sign(4, 1), "Aries");
        assert_eq!(zodiac_sign(4, 19), "Aries");
        assert_eq!(zodiac_sign(4, 20), "Taurus");
    }

    #[test]
    fn test_zodiac_capricorn_december() {
        assert_eq!(zodiac_sign(12, 22), "Capricorn");
        assert_eq!(zodiac_sign(12, 31), "Capricorn");
    }

    #[test]
    fn test_zodiac_capricorn_january() {
        assert_eq!(zodiac_sign(1, 1), "Capricorn");
        assert_eq!(zodiac_sign(1, 19), "Capricorn");
        assert_eq!(zodiac_sign(1, 20), "Aquarius");
    }

    #[test]
    fn test_zodiac_pisces_feb29() {
        assert_eq!(zodiac_sign(2, 29), "Pisces");
    }

    #[test]
    fn test_compute_returns_two_items() {
        let items = compute_birthday_items(&profile("1990-06-15")).unwrap();
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn test_compute_first_item_has_birthday_label() {
        let items = compute_birthday_items(&profile("1990-06-15")).unwrap();
        assert!(items[0].title.starts_with("Birthday:"), "got: {}", items[0].title);
    }

    #[test]
    fn test_compute_second_item_has_zodiac_label() {
        let items = compute_birthday_items(&profile("1990-06-15")).unwrap();
        assert!(items[1].title.starts_with("Zodiac:"), "got: {}", items[1].title);
    }

    #[test]
    fn test_compute_subtitle_mentions_age() {
        let items = compute_birthday_items(&profile("1990-06-15")).unwrap();
        assert!(
            items[0].subtitle.as_ref().unwrap().contains("years old"),
            "got: {:?}",
            items[0].subtitle
        );
    }

    #[test]
    fn test_compute_invalid_date_returns_error() {
        assert!(compute_birthday_items(&profile("not-a-date")).is_err());
    }

    #[test]
    fn test_compute_missing_birthday_returns_error() {
        assert!(compute_birthday_items(&HashMap::new()).is_err());
    }

    #[test]
    fn test_compute_empty_birthday_returns_error() {
        assert!(compute_birthday_items(&profile("")).is_err());
    }
}
