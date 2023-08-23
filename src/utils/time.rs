use std::str::FromStr;

use chrono::prelude::*;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug)]
pub struct ParseTimeError;

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Serialize, ToSchema, PartialEq)]
#[serde(untagged)]
pub enum TibiaTime {
    #[schema(example = "2007-11-28T18:26:00+00:00")]
    YMDHMS(String),
    #[schema(example = "2007-11-28")]
    YMD(String),
    #[schema(example = "2007-11")]
    YM(String),
}

impl Default for TibiaTime {
    fn default() -> Self {
        Self::YMDHMS("".to_string())
    }
}

impl TryFrom<&str> for TibiaTime {
    type Error = ParseTimeError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        // Player record dates
        // Total are in CET (UTC+1)
        // World are in CEST (UTC+2)
        if let Ok(naive_date) = NaiveDateTime::parse_from_str(s, "%b %d %Y, %H:%M:%S %Z") {
            let offset = if s.contains("CET") {
                FixedOffset::east_opt(3600).unwrap()
            } else if s.contains("CEST") {
                FixedOffset::east_opt(7200).unwrap()
            } else {
                return Err(ParseTimeError);
            };

            let date_time = DateTime::<FixedOffset>::from_local(naive_date, offset);

            let utc_time = date_time.with_timezone(&Utc);

            return Ok(Self::YMDHMS(utc_time.to_rfc3339()));
        }

        // BattleEye dates (unknown timezone)
        if let Ok(naive_date) = NaiveDate::parse_from_str(s, "%B %d, %Y") {
            let formatted = naive_date.format("%Y-%m-%d").to_string();
            return Ok(Self::YMD(formatted));
        }

        // OLD: Created dates (unknown timezone)
        if let Ok(naive_date) = NaiveDate::parse_from_str(&format!("{s}/01"), "%m/%y/%d")
            .map(|t| t.and_hms_opt(0, 0, 0).unwrap())
        {
            let formatted = naive_date.format("%Y-%m").to_string();
            return Ok(Self::YM(formatted));
        }

        // NEW: Created dates (unknown timezone)
        if let Ok(naive_date) = NaiveDate::parse_from_str(&format!("{s} 01"), "%B %Y %d")
            .map(|t| t.and_hms_opt(0, 0, 0).unwrap())
        {
            let formatted = naive_date.format("%Y-%m").to_string();
            return Ok(Self::YM(formatted));
        }

        Err(ParseTimeError)
    }
}

impl FromStr for TibiaTime {
    type Err = ParseTimeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.try_into()
    }
}

impl ToString for TibiaTime {
    fn to_string(&self) -> String {
        match self {
            TibiaTime::YMDHMS(s) | TibiaTime::YMD(s) | TibiaTime::YM(s) => s.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_can_parse_record_date_time_cet() {
        let input = "Nov 28 2007, 19:26:00 CET";
        let expected = "2007-11-28T18:26:00+00:00";

        let output = input.parse::<TibiaTime>().unwrap().to_string();
        assert_eq!(expected, output);
    }

    #[test]
    fn it_can_parse_record_date_time_cest() {
        let input = "Nov 28 2007, 19:26:00 CEST";
        let expected = "2007-11-28T17:26:00+00:00";

        let output = input.parse::<TibiaTime>().unwrap().to_string();
        assert_eq!(expected, output);
    }

    #[test]
    fn it_can_parse_battle_eye_date() {
        let input = "August 29, 2017";
        let expected = "2017-08-29";

        let output = input.parse::<TibiaTime>().unwrap().to_string();
        assert_eq!(expected, output);
    }

    #[test]
    fn it_can_parse_old_created_date() {
        let input = "10/20";
        let expected = "2020-10";

        let output = input.parse::<TibiaTime>().unwrap().to_string();
        assert_eq!(expected, output);
    }

    #[test]
    fn it_can_parse_new_created_date() {
        let input = "October 2020";
        let expected = "2020-10";

        let output = input.parse::<TibiaTime>().unwrap().to_string();
        assert_eq!(expected, output);
    }
}
