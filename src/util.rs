pub mod time {
    use std::{fmt::Display, time::Duration};

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Time(pub u64, pub u64, pub u64);

    impl Display for Time {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let Time(h, m, s) = *self;

            if h > 0 {
                if m > 0 {
                    if s > 0 {
                        write!(f, "{h}시간 {m}분 {s}초")
                    } else {
                        write!(f, "{h}시간 {m}분")
                    }
                } else if s > 0 {
                    write!(f, "{h}시간 {s}초")
                } else {
                    write!(f, "{h}시간")
                }
            } else if m > 0 {
                if s > 0 {
                    write!(f, "{m}분 {s}초")
                } else {
                    write!(f, "{m}분")
                }
            } else if s > 0 {
                write!(f, "{s}초")
            } else {
                write!(f, "남은 시간 없음")
            }
        }
    }

    /// returns
    /// (hours, minutes, seconds)
    pub fn seperate_duration(duration: Duration) -> Time {
        let mut seconds = duration.as_secs();

        let hours = seconds / 3600;

        seconds -= hours * 3600;

        let minutes = seconds / 60;

        seconds -= minutes * 60;

        Time(hours, minutes, seconds)
    }

    pub fn parse_iso8601_duration(x: &str) -> Option<Duration> {
        // PT#H#M#S
        // P#DT#H#M#S

        let x = x.strip_prefix("PT").or(x.strip_prefix('P'))?;

        fn pos(x: Option<(u64, usize)>) -> usize {
            match x {
                Some((_, pos)) => pos,
                None => 0,
            }
        }

        fn value(x: Option<(u64, usize)>) -> u64 {
            match x {
                Some((r, _)) => r,
                None => 0,
            }
        }

        fn parse_dates(x: &str) -> Option<(u64, usize)> {
            let pos = x.chars().position(|c| c == 'D')?;

            x[..pos].parse().ok().map(|r| (r, pos + 2))
        }

        fn parse_hours(x: &str) -> Option<(u64, usize)> {
            let pos = x.chars().position(|c| c == 'H')?;

            x[..pos].parse().ok().map(|r| (r, pos + 1))
        }

        fn parse_minutes(x: &str) -> Option<(u64, usize)> {
            let pos = x.chars().position(|c| c == 'M')?;

            x[..pos].parse().ok().map(|r| (r, pos + 1))
        }

        fn parse_seconds(x: &str) -> Option<(u64, usize)> {
            let pos = x.chars().position(|c| c == 'S')?;

            x[..pos].parse().ok().map(|r| (r, pos + 1))
        }

        let dates = parse_dates(x);

        let x = &x[pos(dates)..];

        let hours = parse_hours(x);

        let x = &x[pos(hours)..];

        let minutes = parse_minutes(x);

        let x = &x[pos(minutes)..];

        let seconds = parse_seconds(x);

        Some(Duration::from_secs(
            value(seconds)
                + value(minutes) * 60
                + value(hours) * 60 * 60
                + value(dates) * 60 * 60 * 24,
        ))
    }

    #[test]
    fn test_parse_iso8601_duration() {
        let x = parse_iso8601_duration("P12DT22H45M23S").unwrap();

        assert_eq!(x, Duration::from_secs(1_036_800 + 79_200 + 2_700 + 23));

        let x = parse_iso8601_duration("PT17H33M").unwrap();

        assert_eq!(x, Duration::from_secs(61_200 + 1_980));
    }
}
