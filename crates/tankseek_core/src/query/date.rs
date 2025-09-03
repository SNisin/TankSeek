use crate::query::query_parser::*;
impl From<&str> for QueryDate {
    fn from(s: &str) -> Self {
        use chrono::{Datelike, Local, NaiveDate, NaiveTime, TimeZone};
        
        let s = s.to_lowercase();
        
        // Handle "unknown" keyword
        if s == "unknown" {
            return QueryDate::Unknown;
        }
        
        // Handle weekdays
        match s.as_str() {
            "sunday" | "sun" => return QueryDate::Weekday(Weekday::Sunday),
            "monday" | "mon" => return QueryDate::Weekday(Weekday::Monday),
            "tuesday" | "tue" => return QueryDate::Weekday(Weekday::Tuesday),
            "wednesday" | "wed" => return QueryDate::Weekday(Weekday::Wednesday),
            "thursday" | "thu" => return QueryDate::Weekday(Weekday::Thursday),
            "friday" | "fri" => return QueryDate::Weekday(Weekday::Friday),
            "saturday" | "sat" => return QueryDate::Weekday(Weekday::Saturday),
            _ => {}
        }
        
        // Handle months
        match s.as_str() {
            "january" | "jan" => return QueryDate::Month(Month::January),
            "february" | "feb" => return QueryDate::Month(Month::February),
            "march" | "mar" => return QueryDate::Month(Month::March),
            "april" | "apr" => return QueryDate::Month(Month::April),
            "may" => return QueryDate::Month(Month::May),
            "june" | "jun" => return QueryDate::Month(Month::June),
            "july" | "jul" => return QueryDate::Month(Month::July),
            "august" | "aug" => return QueryDate::Month(Month::August),
            "september" | "sep" => return QueryDate::Month(Month::September),
            "october" | "oct" => return QueryDate::Month(Month::October),
            "november" | "nov" => return QueryDate::Month(Month::November),
            "december" | "dec" => return QueryDate::Month(Month::December),
            _ => {}
        }
        
        // Helper function to create timestamp range from start and end dates
        let date_range_to_timestamps = |start_date: NaiveDate, end_date: NaiveDate| -> (i64, i64) {
            let start_datetime = start_date.and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap());
            let end_datetime = end_date.and_time(NaiveTime::from_hms_opt(23, 59, 59).unwrap());
            
            let start_timestamp = Local.from_local_datetime(&start_datetime)
                .single()
                .map(|dt| dt.timestamp())
                .unwrap_or(0);
            let end_timestamp = Local.from_local_datetime(&end_datetime)
                .single()
                .map(|dt| dt.timestamp())
                .unwrap_or(0);
                
            (start_timestamp, end_timestamp)
        };
        
        // Handle special date constants
        let today = Local::now().date_naive();
        match s.as_str() {
            "today" => {
                let (start, end) = date_range_to_timestamps(today, today);
                return QueryDate::Range(start, end);
            },
            "yesterday" => {
                let yesterday = today - chrono::Duration::days(1);
                let (start, end) = date_range_to_timestamps(yesterday, yesterday);
                return QueryDate::Range(start, end);
            },
            // Week-based constants
            "lastweek" | "last week" | "pastweek" | "past week" | "prevweek" | "prev week" => {
                let days_since_monday = today.weekday().num_days_from_monday();
                let last_monday = today - chrono::Duration::days((days_since_monday + 7) as i64);
                let last_sunday = last_monday + chrono::Duration::days(6);
                let (start, end) = date_range_to_timestamps(last_monday, last_sunday);
                return QueryDate::Range(start, end);
            },
            "thisweek" | "this week" | "currentweek" | "current week" => {
                let days_since_monday = today.weekday().num_days_from_monday();
                let this_monday = today - chrono::Duration::days(days_since_monday as i64);
                let this_sunday = this_monday + chrono::Duration::days(6);
                let (start, end) = date_range_to_timestamps(this_monday, this_sunday);
                return QueryDate::Range(start, end);
            },
            "nextweek" | "next week" | "comingweek" | "coming week" => {
                let days_since_monday = today.weekday().num_days_from_monday();
                let next_monday = today + chrono::Duration::days((7 - days_since_monday) as i64);
                let next_sunday = next_monday + chrono::Duration::days(6);
                let (start, end) = date_range_to_timestamps(next_monday, next_sunday);
                return QueryDate::Range(start, end);
            },
            // Month-based constants
            "lastmonth" | "last month" | "pastmonth" | "past month" | "prevmonth" | "prev month" => {
                let last_month = if today.month() == 1 {
                    NaiveDate::from_ymd_opt(today.year() - 1, 12, 1).unwrap()
                } else {
                    NaiveDate::from_ymd_opt(today.year(), today.month() - 1, 1).unwrap()
                };
                let last_month_end = last_month + chrono::Duration::days(
                    chrono::NaiveDate::from_ymd_opt(
                        if last_month.month() == 12 { last_month.year() + 1 } else { last_month.year() },
                        if last_month.month() == 12 { 1 } else { last_month.month() + 1 },
                        1
                    ).unwrap().signed_duration_since(last_month).num_days() - 1
                );
                let (start, end) = date_range_to_timestamps(last_month, last_month_end);
                return QueryDate::Range(start, end);
            },
            "thismonth" | "this month" | "currentmonth" | "current month" => {
                let this_month_start = NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap();
                let next_month = if today.month() == 12 {
                    NaiveDate::from_ymd_opt(today.year() + 1, 1, 1).unwrap()
                } else {
                    NaiveDate::from_ymd_opt(today.year(), today.month() + 1, 1).unwrap()
                };
                let this_month_end = next_month - chrono::Duration::days(1);
                let (start, end) = date_range_to_timestamps(this_month_start, this_month_end);
                return QueryDate::Range(start, end);
            },
            "nextmonth" | "next month" | "comingmonth" | "coming month" => {
                let next_month_start = if today.month() == 12 {
                    NaiveDate::from_ymd_opt(today.year() + 1, 1, 1).unwrap()
                } else {
                    NaiveDate::from_ymd_opt(today.year(), today.month() + 1, 1).unwrap()
                };
                let month_after_next = if next_month_start.month() == 12 {
                    NaiveDate::from_ymd_opt(next_month_start.year() + 1, 1, 1).unwrap()
                } else {
                    NaiveDate::from_ymd_opt(next_month_start.year(), next_month_start.month() + 1, 1).unwrap()
                };
                let next_month_end = month_after_next - chrono::Duration::days(1);
                let (start, end) = date_range_to_timestamps(next_month_start, next_month_end);
                return QueryDate::Range(start, end);
            },
            // Year-based constants
            "lastyear" | "last year" | "pastyear" | "past year" | "prevyear" | "prev year" => {
                let last_year_start = NaiveDate::from_ymd_opt(today.year() - 1, 1, 1).unwrap();
                let last_year_end = NaiveDate::from_ymd_opt(today.year() - 1, 12, 31).unwrap();
                let (start, end) = date_range_to_timestamps(last_year_start, last_year_end);
                return QueryDate::Range(start, end);
            },
            "thisyear" | "this year" | "currentyear" | "current year" => {
                let this_year_start = NaiveDate::from_ymd_opt(today.year(), 1, 1).unwrap();
                let this_year_end = NaiveDate::from_ymd_opt(today.year(), 12, 31).unwrap();
                let (start, end) = date_range_to_timestamps(this_year_start, this_year_end);
                return QueryDate::Range(start, end);
            },
            "nextyear" | "next year" | "comingyear" | "coming year" => {
                let next_year_start = NaiveDate::from_ymd_opt(today.year() + 1, 1, 1).unwrap();
                let next_year_end = NaiveDate::from_ymd_opt(today.year() + 1, 12, 31).unwrap();
                let (start, end) = date_range_to_timestamps(next_year_start, next_year_end);
                return QueryDate::Range(start, end);
            },
            _ => {}
        }
        
        // Handle numeric relative dates like "last3days", "next2weeks", etc.
        if let Some(captures) = regex::Regex::new(r"^(last|past|prev|next|coming)(\d+)(years?|months?|weeks?|days?|hours?|minutes?|mins?|seconds?|secs?)$").unwrap().captures(&s) {
            let direction = &captures[1];
            let amount = captures[2].parse::<i64>().unwrap_or(1);
            let unit = &captures[3];
            
            let is_backwards = matches!(direction, "last" | "past" | "prev");
            let duration_amount = if is_backwards { -amount } else { amount };
            
            let (start_date, end_date) = match unit {
                "year" | "years" => {
                    let target_year = today.year() + duration_amount as i32;
                    let year_start = NaiveDate::from_ymd_opt(target_year, 1, 1).unwrap_or(today);
                    let year_end = NaiveDate::from_ymd_opt(target_year, 12, 31).unwrap_or(today);
                    if is_backwards {
                        // For "last X years", include from X years ago until today
                        let start = NaiveDate::from_ymd_opt(today.year() + duration_amount as i32, 1, 1).unwrap_or(today);
                        (start, today)
                    } else {
                        (year_start, year_end)
                    }
                },
                "month" | "months" => {
                    if is_backwards {
                        // For "last X months", go back X months from today
                        let months_ago = today - chrono::Duration::days(amount * 30); // Approximation
                        (months_ago, today)
                    } else {
                        // For "next X months", go forward X months from today
                        let months_ahead = today + chrono::Duration::days(amount * 30); // Approximation
                        (today, months_ahead)
                    }
                },
                "week" | "weeks" => {
                    if is_backwards {
                        let weeks_ago = today - chrono::Duration::weeks(amount);
                        (weeks_ago, today)
                    } else {
                        let weeks_ahead = today + chrono::Duration::weeks(amount);
                        (today, weeks_ahead)
                    }
                },
                "day" | "days" => {
                    if is_backwards {
                        let days_ago = today - chrono::Duration::days(amount);
                        (days_ago, today)
                    } else {
                        let days_ahead = today + chrono::Duration::days(amount);
                        (today, days_ahead)
                    }
                },
                "hour" | "hours" => {
                    // For hours, minutes, seconds - use current time as base
                    let now = Local::now();
                    if is_backwards {
                        let hours_ago = now - chrono::Duration::hours(amount);
                        return QueryDate::Range(hours_ago.timestamp(), now.timestamp());
                    } else {
                        let hours_ahead = now + chrono::Duration::hours(amount);
                        return QueryDate::Range(now.timestamp(), hours_ahead.timestamp());
                    }
                },
                "minute" | "minutes" | "min" | "mins" => {
                    let now = Local::now();
                    if is_backwards {
                        let minutes_ago = now - chrono::Duration::minutes(amount);
                        return QueryDate::Range(minutes_ago.timestamp(), now.timestamp());
                    } else {
                        let minutes_ahead = now + chrono::Duration::minutes(amount);
                        return QueryDate::Range(now.timestamp(), minutes_ahead.timestamp());
                    }
                },
                "second" | "seconds" | "sec" | "secs" => {
                    let now = Local::now();
                    if is_backwards {
                        let seconds_ago = now - chrono::Duration::seconds(amount);
                        return QueryDate::Range(seconds_ago.timestamp(), now.timestamp());
                    } else {
                        let seconds_ahead = now + chrono::Duration::seconds(amount);
                        return QueryDate::Range(now.timestamp(), seconds_ahead.timestamp());
                    }
                },
                _ => (today, today), // Fallback
            };
            
            let (start, end) = date_range_to_timestamps(start_date, end_date);
            return QueryDate::Range(start, end);
        }
        
        // Try to parse as year only (4 digits)
        if let Ok(year) = s.parse::<i32>() {
            if year >= 1970 && year <= 9999 {
                if let Some(start_date) = NaiveDate::from_ymd_opt(year, 1, 1) {
                    if let Some(end_date) = NaiveDate::from_ymd_opt(year, 12, 31) {
                        let (start, end) = date_range_to_timestamps(start_date, end_date);
                        return QueryDate::Range(start, end);
                    }
                }
            }
        }
        
        // Try various date formats using chrono's parsing
        let date_formats = [
            "%Y-%m-%d",        // 2023-12-25
            "%Y-%m-%d %H:%M:%S", // 2023-12-25 12:30:45 (will use date part only)
            "%m/%d/%Y",        // 12/25/2023
            "%d/%m/%Y",        // 25/12/2023
        ];
        
        for format in &date_formats {
            if let Ok(parsed_date) = NaiveDate::parse_from_str(&s, format) {
                let (start, end) = date_range_to_timestamps(parsed_date, parsed_date);
                return QueryDate::Range(start, end);
            }
        }
        
        // Try to parse MM/YYYY or YYYY/MM format
        if let Some(captures) = regex::Regex::new(r"^(\d{1,4})/(\d{1,4})$").unwrap().captures(&s) {
            if let (Ok(first), Ok(second)) = (captures[1].parse::<u32>(), captures[2].parse::<u32>()) {
                let (year, month) = if first >= 1970 && first <= 9999 && second >= 1 && second <= 12 {
                    (first as i32, second)
                } else if second >= 1970 && second <= 9999 && first >= 1 && first <= 12 {
                    (second as i32, first)
                } else {
                    return QueryDate::Range(0, 0);
                };
                
                if let Some(start_date) = NaiveDate::from_ymd_opt(year, month, 1) {
                    // Get last day of month
                    let next_month = if month == 12 { 
                        NaiveDate::from_ymd_opt(year + 1, 1, 1)
                    } else { 
                        NaiveDate::from_ymd_opt(year, month + 1, 1)
                    };
                    
                    if let Some(next_month_date) = next_month {
                        let end_date = next_month_date - chrono::Duration::days(1);
                        let (start, end) = date_range_to_timestamps(start_date, end_date);
                        return QueryDate::Range(start, end);
                    }
                }
            }
        }
        
        // If all parsing fails, return Range(0, 0)
        QueryDate::Range(0, 0)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weekday_parsing() {
        // Test full weekday names
        assert_eq!(QueryDate::from("sunday"), QueryDate::Weekday(Weekday::Sunday));
        assert_eq!(QueryDate::from("monday"), QueryDate::Weekday(Weekday::Monday));
        assert_eq!(QueryDate::from("tuesday"), QueryDate::Weekday(Weekday::Tuesday));
        assert_eq!(QueryDate::from("wednesday"), QueryDate::Weekday(Weekday::Wednesday));
        assert_eq!(QueryDate::from("thursday"), QueryDate::Weekday(Weekday::Thursday));
        assert_eq!(QueryDate::from("friday"), QueryDate::Weekday(Weekday::Friday));
        assert_eq!(QueryDate::from("saturday"), QueryDate::Weekday(Weekday::Saturday));

        // Test abbreviated weekday names
        assert_eq!(QueryDate::from("sun"), QueryDate::Weekday(Weekday::Sunday));
        assert_eq!(QueryDate::from("mon"), QueryDate::Weekday(Weekday::Monday));
        assert_eq!(QueryDate::from("tue"), QueryDate::Weekday(Weekday::Tuesday));
        assert_eq!(QueryDate::from("wed"), QueryDate::Weekday(Weekday::Wednesday));
        assert_eq!(QueryDate::from("thu"), QueryDate::Weekday(Weekday::Thursday));
        assert_eq!(QueryDate::from("fri"), QueryDate::Weekday(Weekday::Friday));
        assert_eq!(QueryDate::from("sat"), QueryDate::Weekday(Weekday::Saturday));

        // Test case insensitivity
        assert_eq!(QueryDate::from("SUNDAY"), QueryDate::Weekday(Weekday::Sunday));
        assert_eq!(QueryDate::from("SUN"), QueryDate::Weekday(Weekday::Sunday));
        assert_eq!(QueryDate::from("Monday"), QueryDate::Weekday(Weekday::Monday));
    }

    #[test]
    fn test_month_parsing() {
        // Test full month names
        assert_eq!(QueryDate::from("january"), QueryDate::Month(Month::January));
        assert_eq!(QueryDate::from("february"), QueryDate::Month(Month::February));
        assert_eq!(QueryDate::from("march"), QueryDate::Month(Month::March));
        assert_eq!(QueryDate::from("april"), QueryDate::Month(Month::April));
        assert_eq!(QueryDate::from("may"), QueryDate::Month(Month::May));
        assert_eq!(QueryDate::from("june"), QueryDate::Month(Month::June));
        assert_eq!(QueryDate::from("july"), QueryDate::Month(Month::July));
        assert_eq!(QueryDate::from("august"), QueryDate::Month(Month::August));
        assert_eq!(QueryDate::from("september"), QueryDate::Month(Month::September));
        assert_eq!(QueryDate::from("october"), QueryDate::Month(Month::October));
        assert_eq!(QueryDate::from("november"), QueryDate::Month(Month::November));
        assert_eq!(QueryDate::from("december"), QueryDate::Month(Month::December));

        // Test abbreviated month names
        assert_eq!(QueryDate::from("jan"), QueryDate::Month(Month::January));
        assert_eq!(QueryDate::from("feb"), QueryDate::Month(Month::February));
        assert_eq!(QueryDate::from("mar"), QueryDate::Month(Month::March));
        assert_eq!(QueryDate::from("apr"), QueryDate::Month(Month::April));
        assert_eq!(QueryDate::from("jun"), QueryDate::Month(Month::June));
        assert_eq!(QueryDate::from("jul"), QueryDate::Month(Month::July));
        assert_eq!(QueryDate::from("aug"), QueryDate::Month(Month::August));
        assert_eq!(QueryDate::from("sep"), QueryDate::Month(Month::September));
        assert_eq!(QueryDate::from("oct"), QueryDate::Month(Month::October));
        assert_eq!(QueryDate::from("nov"), QueryDate::Month(Month::November));
        assert_eq!(QueryDate::from("dec"), QueryDate::Month(Month::December));

        // Test case insensitivity
        assert_eq!(QueryDate::from("JANUARY"), QueryDate::Month(Month::January));
        assert_eq!(QueryDate::from("JAN"), QueryDate::Month(Month::January));
        assert_eq!(QueryDate::from("December"), QueryDate::Month(Month::December));
    }

    #[test]
    fn test_special_keywords() {
        // Test unknown keyword
        assert_eq!(QueryDate::from("unknown"), QueryDate::Unknown);
        assert_eq!(QueryDate::from("UNKNOWN"), QueryDate::Unknown);
        assert_eq!(QueryDate::from("Unknown"), QueryDate::Unknown);

        // Test today and yesterday (these will return ranges, so we just check the variant)
        match QueryDate::from("today") {
            QueryDate::Range(start, end) => {
                assert!(start > 0);
                assert!(end > start);
                // Should be a 24-hour range
                assert_eq!(end - start, 86399); // 23:59:59 in seconds
            }
            _ => panic!("Expected Range for 'today'"),
        }

        match QueryDate::from("yesterday") {
            QueryDate::Range(start, end) => {
                assert!(start > 0);
                assert!(end > start);
                // Should be a 24-hour range
                assert_eq!(end - start, 86399); // 23:59:59 in seconds
            }
            _ => panic!("Expected Range for 'yesterday'"),
        }
    }

    #[test]
    fn test_year_parsing() {
        // Test valid years
        match QueryDate::from("2023") {
            QueryDate::Range(start, end) => {
                assert!(start > 0);
                assert!(end > start);
                // Should span the entire year - just check it's roughly a year
                let duration = end - start;
                // Should be around 365 days in seconds (31,536,000) plus end-of-day seconds
                assert!(duration > 31_500_000 && duration < 32_000_000);
            }
            _ => panic!("Expected Range for year '2023'"),
        }

        // Test leap year
        match QueryDate::from("2024") {
            QueryDate::Range(start, end) => {
                assert!(start > 0);
                assert!(end > start);
                // Should span the entire leap year - just check it's roughly a leap year
                let duration = end - start;
                // Should be around 366 days in seconds (31,622,400) plus end-of-day seconds
                assert!(duration > 31_600_000 && duration < 32_000_000);
            }
            _ => panic!("Expected Range for leap year '2024'"),
        }

        // Test invalid years
        assert_eq!(QueryDate::from("1969"), QueryDate::Range(0, 0)); // Before Unix epoch
        assert_eq!(QueryDate::from("10000"), QueryDate::Range(0, 0)); // Too far in future
        assert_eq!(QueryDate::from("abc"), QueryDate::Range(0, 0)); // Not a number
    }

    #[test]
    fn test_iso_date_parsing() {
        // Test valid ISO dates
        match QueryDate::from("2023-12-25") {
            QueryDate::Range(start, end) => {
                assert!(start > 0);
                assert!(end > start);
                // Should be a 24-hour range
                assert_eq!(end - start, 86399); // 23:59:59 in seconds
            }
            _ => panic!("Expected Range for ISO date '2023-12-25'"),
        }

        // Test various valid formats
        match QueryDate::from("2023-01-01") {
            QueryDate::Range(_, _) => {} // Should parse successfully
            _ => panic!("Expected Range for ISO date '2023-01-01'"),
        }

        match QueryDate::from("2023-2-5") {
            QueryDate::Range(_, _) => {} // Should parse successfully with single digits
            _ => panic!("Expected Range for ISO date '2023-2-5'"),
        }

        // Test invalid ISO dates
        assert_eq!(QueryDate::from("2023-13-01"), QueryDate::Range(0, 0)); // Invalid month
        assert_eq!(QueryDate::from("2023-12-32"), QueryDate::Range(0, 0)); // Invalid day
        assert_eq!(QueryDate::from("2023-00-01"), QueryDate::Range(0, 0)); // Invalid month
        assert_eq!(QueryDate::from("2023-12-00"), QueryDate::Range(0, 0)); // Invalid day
    }

    #[test]
    fn test_slash_date_parsing() {
        // Test MM/DD/YYYY format
        match QueryDate::from("12/25/2023") {
            QueryDate::Range(start, end) => {
                assert!(start > 0);
                assert!(end > start);
                assert_eq!(end - start, 86399); // 24-hour range
            }
            _ => panic!("Expected Range for slash date '12/25/2023'"),
        }

        // Test DD/MM/YYYY format (should also work for ambiguous cases)
        match QueryDate::from("25/12/2023") {
            QueryDate::Range(start, end) => {
                assert!(start > 0);
                assert!(end > start);
                assert_eq!(end - start, 86399); // 24-hour range
            }
            _ => panic!("Expected Range for slash date '25/12/2023'"),
        }

        // Test invalid slash dates
        assert_eq!(QueryDate::from("13/32/2023"), QueryDate::Range(0, 0)); // Both invalid
        assert_eq!(QueryDate::from("00/01/2023"), QueryDate::Range(0, 0)); // Invalid month/day
    }

    #[test]
    fn test_month_year_parsing() {
        // Test MM/YYYY format
        match QueryDate::from("12/2023") {
            QueryDate::Range(start, end) => {
                assert!(start > 0);
                assert!(end > start);
                // Should span the entire month of December - just check it's roughly a month
                let duration = end - start;
                // Should be around 31 days in seconds (2,678,400) 
                assert!(duration > 2_600_000 && duration < 2_800_000);
            }
            _ => panic!("Expected Range for month/year '12/2023'"),
        }

        // Test YYYY/MM format
        match QueryDate::from("2023/12") {
            QueryDate::Range(start, end) => {
                assert!(start > 0);
                assert!(end > start);
                // Should span the entire month of December
                let duration = end - start;
                assert!(duration > 2_600_000 && duration < 2_800_000);
            }
            _ => panic!("Expected Range for year/month '2023/12'"),
        }

        // Test February in leap year
        match QueryDate::from("2/2024") {
            QueryDate::Range(start, end) => {
                assert!(start > 0);
                assert!(end > start);
                // Should span February in leap year (29 days)
                let duration = end - start;
                // Should be around 29 days in seconds (2,505,600)
                assert!(duration > 2_400_000 && duration < 2_600_000);
            }
            _ => panic!("Expected Range for leap February '2/2024'"),
        }

        // Test February in regular year
        match QueryDate::from("2/2023") {
            QueryDate::Range(start, end) => {
                assert!(start > 0);
                assert!(end > start);
                // Should span February in regular year (28 days)
                let duration = end - start;
                // Should be around 28 days in seconds (2,419,200)
                assert!(duration > 2_300_000 && duration < 2_500_000);
            }
            _ => panic!("Expected Range for regular February '2/2023'"),
        }

        // Test invalid month/year combinations
        assert_eq!(QueryDate::from("13/2023"), QueryDate::Range(0, 0)); // Invalid month
        assert_eq!(QueryDate::from("2023/13"), QueryDate::Range(0, 0)); // Invalid month
        assert_eq!(QueryDate::from("0/2023"), QueryDate::Range(0, 0)); // Invalid month
    }

    #[test]
    fn test_error_cases() {
        // Test completely invalid inputs
        assert_eq!(QueryDate::from(""), QueryDate::Range(0, 0));
        assert_eq!(QueryDate::from("invalid"), QueryDate::Range(0, 0));
        assert_eq!(QueryDate::from("123abc"), QueryDate::Range(0, 0));
        assert_eq!(QueryDate::from("not-a-date"), QueryDate::Range(0, 0));
        assert_eq!(QueryDate::from("2023-abc-def"), QueryDate::Range(0, 0));
        assert_eq!(QueryDate::from("abc/def/ghi"), QueryDate::Range(0, 0));

        // Test edge cases
        assert_eq!(QueryDate::from("   "), QueryDate::Range(0, 0)); // Whitespace only
        assert_eq!(QueryDate::from("2023-"), QueryDate::Range(0, 0)); // Incomplete format
        assert_eq!(QueryDate::from("/2023"), QueryDate::Range(0, 0)); // Incomplete format
        assert_eq!(QueryDate::from("2023/"), QueryDate::Range(0, 0)); // Incomplete format
    }

    #[test]
    fn test_case_insensitivity() {
        // Test that all parsing is case insensitive
        assert_eq!(QueryDate::from("SUNDAY"), QueryDate::from("sunday"));
        assert_eq!(QueryDate::from("January"), QueryDate::from("january"));
        assert_eq!(QueryDate::from("UNKNOWN"), QueryDate::from("unknown"));
        assert_eq!(QueryDate::from("TODAY"), QueryDate::from("today"));
        assert_eq!(QueryDate::from("YESTERDAY"), QueryDate::from("yesterday"));
    }

    #[test]
    fn test_relative_date_constants() {
        // Test week-based constants
        match QueryDate::from("lastweek") {
            QueryDate::Range(start, end) => {
                assert!(start > 0);
                assert!(end > start);
                // Should be approximately 7 days
                let duration = end - start;
                assert!(duration > 6 * 86400 && duration < 8 * 86400);
            }
            _ => panic!("Expected Range for 'lastweek'"),
        }

        match QueryDate::from("thisweek") {
            QueryDate::Range(start, end) => {
                assert!(start > 0);
                assert!(end > start);
                // Should be approximately 7 days
                let duration = end - start;
                assert!(duration > 6 * 86400 && duration < 8 * 86400);
            }
            _ => panic!("Expected Range for 'thisweek'"),
        }

        match QueryDate::from("nextweek") {
            QueryDate::Range(start, end) => {
                assert!(start > 0);
                assert!(end > start);
                // Should be approximately 7 days
                let duration = end - start;
                assert!(duration > 6 * 86400 && duration < 8 * 86400);
            }
            _ => panic!("Expected Range for 'nextweek'"),
        }

        // Test month-based constants
        match QueryDate::from("lastmonth") {
            QueryDate::Range(start, end) => {
                assert!(start > 0);
                assert!(end > start);
                // Should be roughly a month (28-31 days)
                let duration = end - start;
                assert!(duration > 27 * 86400 && duration < 32 * 86400);
            }
            _ => panic!("Expected Range for 'lastmonth'"),
        }

        match QueryDate::from("thismonth") {
            QueryDate::Range(start, end) => {
                assert!(start > 0);
                assert!(end > start);
                // Should be roughly a month
                let duration = end - start;
                assert!(duration > 27 * 86400 && duration < 32 * 86400);
            }
            _ => panic!("Expected Range for 'thismonth'"),
        }

        match QueryDate::from("nextmonth") {
            QueryDate::Range(start, end) => {
                assert!(start > 0);
                assert!(end > start);
                // Should be roughly a month
                let duration = end - start;
                assert!(duration > 27 * 86400 && duration < 32 * 86400);
            }
            _ => panic!("Expected Range for 'nextmonth'"),
        }

        // Test year-based constants
        match QueryDate::from("lastyear") {
            QueryDate::Range(start, end) => {
                assert!(start > 0);
                assert!(end > start);
                // Should be roughly a year (365-366 days)
                let duration = end - start;
                assert!(duration > 364 * 86400 && duration < 367 * 86400);
            }
            _ => panic!("Expected Range for 'lastyear'"),
        }

        match QueryDate::from("thisyear") {
            QueryDate::Range(start, end) => {
                assert!(start > 0);
                assert!(end > start);
                // Should be roughly a year
                let duration = end - start;
                assert!(duration > 364 * 86400 && duration < 367 * 86400);
            }
            _ => panic!("Expected Range for 'thisyear'"),
        }

        match QueryDate::from("nextyear") {
            QueryDate::Range(start, end) => {
                assert!(start > 0);
                assert!(end > start);
                // Should be roughly a year
                let duration = end - start;
                assert!(duration > 364 * 86400 && duration < 367 * 86400);
            }
            _ => panic!("Expected Range for 'nextyear'"),
        }
    }

    #[test]
    fn test_alternative_formats() {
        // Test alternative spellings with spaces
        match QueryDate::from("last week") {
            QueryDate::Range(_, _) => {} // Should parse successfully
            _ => panic!("Expected Range for 'last week'"),
        }

        match QueryDate::from("this month") {
            QueryDate::Range(_, _) => {} // Should parse successfully
            _ => panic!("Expected Range for 'this month'"),
        }

        match QueryDate::from("next year") {
            QueryDate::Range(_, _) => {} // Should parse successfully
            _ => panic!("Expected Range for 'next year'"),
        }

        // Test alternative words
        match QueryDate::from("pastweek") {
            QueryDate::Range(_, _) => {} // Should parse successfully
            _ => panic!("Expected Range for 'pastweek'"),
        }

        match QueryDate::from("currentmonth") {
            QueryDate::Range(_, _) => {} // Should parse successfully
            _ => panic!("Expected Range for 'currentmonth'"),
        }

        match QueryDate::from("comingyear") {
            QueryDate::Range(_, _) => {} // Should parse successfully
            _ => panic!("Expected Range for 'comingyear'"),
        }
    }

    #[test]
    fn test_numeric_relative_dates() {
        // Test last/past X days
        match QueryDate::from("last3days") {
            QueryDate::Range(start, end) => {
                assert!(start > 0);
                assert!(end > start);
                // Should be roughly 3 days
                let duration = end - start;
                assert!(duration > 2 * 86400 && duration < 4 * 86400);
            }
            _ => panic!("Expected Range for 'last3days'"),
        }

        match QueryDate::from("past7days") {
            QueryDate::Range(start, end) => {
                assert!(start > 0);
                assert!(end > start);
                // Should be roughly 7 days
                let duration = end - start;
                assert!(duration > 6 * 86400 && duration < 8 * 86400);
            }
            _ => panic!("Expected Range for 'past7days'"),
        }

        // Test next/coming X weeks
        match QueryDate::from("next2weeks") {
            QueryDate::Range(start, end) => {
                assert!(start > 0);
                assert!(end > start);
                // Should be roughly 14 days
                let duration = end - start;
                assert!(duration > 13 * 86400 && duration < 15 * 86400);
            }
            _ => panic!("Expected Range for 'next2weeks'"),
        }

        match QueryDate::from("coming4weeks") {
            QueryDate::Range(start, end) => {
                assert!(start > 0);
                assert!(end > start);
                // Should be roughly 28 days
                let duration = end - start;
                assert!(duration > 27 * 86400 && duration < 29 * 86400);
            }
            _ => panic!("Expected Range for 'coming4weeks'"),
        }

        // Test time-based units (hours, minutes, seconds)
        match QueryDate::from("last2hours") {
            QueryDate::Range(start, end) => {
                assert!(start > 0);
                assert!(end > start);
                // Should be roughly 2 hours
                let duration = end - start;
                assert!(duration > 7000 && duration < 7300); // ~2 hours in seconds
            }
            _ => panic!("Expected Range for 'last2hours'"),
        }

        match QueryDate::from("next30minutes") {
            QueryDate::Range(start, end) => {
                assert!(start > 0);
                assert!(end > start);
                // Should be roughly 30 minutes
                let duration = end - start;
                assert!(duration > 1700 && duration < 1900); // ~30 minutes in seconds
            }
            _ => panic!("Expected Range for 'next30minutes'"),
        }

        match QueryDate::from("last120seconds") {
            QueryDate::Range(start, end) => {
                assert!(start > 0);
                assert!(end > start);
                // Should be roughly 120 seconds
                let duration = end - start;
                assert!(duration > 110 && duration < 130); // ~120 seconds
            }
            _ => panic!("Expected Range for 'last120seconds'"),
        }
    }

    #[test]
    fn test_abbreviated_time_units() {
        // Test abbreviated forms
        match QueryDate::from("last5mins") {
            QueryDate::Range(start, end) => {
                assert!(start > 0);
                assert!(end > start);
                let duration = end - start;
                assert!(duration > 290 && duration < 310); // ~5 minutes
            }
            _ => panic!("Expected Range for 'last5mins'"),
        }

        match QueryDate::from("next10secs") {
            QueryDate::Range(start, end) => {
                assert!(start > 0);
                assert!(end > start);
                let duration = end - start;
                assert!(duration > 8 && duration < 12); // ~10 seconds
            }
            _ => panic!("Expected Range for 'next10secs'"),
        }

        // Test singular vs plural forms
        match QueryDate::from("last1day") {
            QueryDate::Range(start, end) => {
                assert!(start > 0);
                assert!(end > start);
                let duration = end - start;
                // "last1day" creates a range from 1 day ago until today, so roughly 1-2 days
                assert!(duration > 80000 && duration < 200000); // Between ~22 hours and ~55 hours
            }
            _ => panic!("Expected Range for 'last1day'"),
        }

        match QueryDate::from("next1week") {
            QueryDate::Range(start, end) => {
                assert!(start > 0);
                assert!(end > start);
                let duration = end - start;
                assert!(duration > 6 * 86400 && duration < 8 * 86400); // ~1 week
            }
            _ => panic!("Expected Range for 'next1week'"),
        }
    }

    #[test]
    fn test_demo_examples() {
        // Demonstrate some practical examples
        let examples = [
            "today",
            "yesterday", 
            "lastweek",
            "thismonth",
            "nextyear",
            "last3days",
            "next2weeks",
            "past5hours",
            "coming30minutes",
            "prev10seconds",
            "monday",
            "january",
            "unknown"
        ];

        for example in &examples {
            let result = QueryDate::from(*example);
            match result {
                QueryDate::Range(start, end) => {
                    println!("{}: Range({}, {}) - duration: {} seconds", 
                            example, start, end, end - start);
                    assert!(start >= 0);
                    assert!(end >= start);
                }
                QueryDate::Weekday(day) => {
                    println!("{}: Weekday({:?})", example, day);
                }
                QueryDate::Month(month) => {
                    println!("{}: Month({:?})", example, month);
                }
                QueryDate::Unknown => {
                    println!("{}: Unknown", example);
                }
            }
        }
    }

    #[test]
    fn test_specific_requirements() {
        // Test that "unknown" keyword returns Unknown variant
        assert_eq!(QueryDate::from("unknown"), QueryDate::Unknown);
        
        // Test that weekdays return Weekday variant
        assert_eq!(QueryDate::from("monday"), QueryDate::Weekday(Weekday::Monday));
        
        // Test that months return Month variant
        assert_eq!(QueryDate::from("january"), QueryDate::Month(Month::January));
        
        // Test that date parsing returns Range with valid timestamps
        match QueryDate::from("2023-01-01") {
            QueryDate::Range(start, end) => {
                assert!(start > 0);
                assert!(end > start);
            }
            _ => panic!("Expected Range for valid date"),
        }
        
        // Test that parsing errors return Range(0, 0)
        assert_eq!(QueryDate::from("invalid_input"), QueryDate::Range(0, 0));
        assert_eq!(QueryDate::from("not-a-date"), QueryDate::Range(0, 0));
        assert_eq!(QueryDate::from("2023-13-45"), QueryDate::Range(0, 0)); // Invalid date
        assert_eq!(QueryDate::from(""), QueryDate::Range(0, 0)); // Empty string
    }
}
