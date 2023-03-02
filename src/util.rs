pub mod time {
    use std::{fmt::Display, time::Duration};

    #[derive(Debug, Clone, Copy)]
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
}
