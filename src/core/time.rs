//! Stamps: parsing and writing (TECHSPEC 3, 4.4).

use jiff::tz::TimeZone;

/// A stamp in the fixed offset written in the file.
pub type Stamp = jiff::Zoned;

/// Reads an RFC 3339 stamp with offset, or `YYYY-MM-DD HH:MM[:SS]` in `local`.
pub fn parse_stamp(text: &str, local: &TimeZone) -> Option<Stamp> {
    let _ = (text, local);
    todo!()
}

/// Writes `2026-10-08T10:31:05+03:00`, never the bracketed zone of `Display`.
pub fn format_stamp(stamp: &Stamp) -> String {
    let _ = stamp;
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use jiff::tz::offset;

    fn round(text: &str, local: &TimeZone) -> String {
        format_stamp(&parse_stamp(text, local).unwrap())
    }

    #[test]
    fn ts3_stamp_round_trips_byte_for_byte() {
        let s = "2026-10-08T10:31:05+03:00";
        for local in [TimeZone::UTC, TimeZone::fixed(offset(-7))] {
            assert_eq!(round(s, &local), s);
        }
    }

    #[test]
    fn ts3_round_trip_keeps_negative_and_zero_offsets() {
        for s in ["2026-01-31T23:59:59-05:30", "2026-10-08T08:31:05+00:00"] {
            assert_eq!(round(s, &TimeZone::fixed(offset(3))), s);
        }
    }

    #[test]
    fn ts3_format_never_appends_bracketed_zone() {
        let z = jiff::civil::date(2026, 10, 8)
            .at(10, 31, 5, 0)
            .to_zoned(TimeZone::fixed(offset(3)))
            .unwrap();
        assert!(!format_stamp(&z).contains('['));
    }

    #[test]
    fn ts4_4_local_stamp_without_seconds_uses_local_zone() {
        assert_eq!(
            round("2026-10-08 10:31", &TimeZone::fixed(offset(3))),
            "2026-10-08T10:31:00+03:00"
        );
    }

    #[test]
    fn ts4_4_local_stamp_with_seconds() {
        assert_eq!(
            round("2026-10-08 10:31:05", &TimeZone::UTC),
            "2026-10-08T10:31:05+00:00"
        );
    }

    #[test]
    fn ts4_4_z_offset_reads_as_utc() {
        assert_eq!(
            round("2026-10-08T07:31:05Z", &TimeZone::fixed(offset(3))),
            "2026-10-08T07:31:05+00:00"
        );
    }

    #[test]
    fn ts4_4_bad_stamp_is_none() {
        for s in [
            "yesterday",
            "2026-13-01T00:00:00+03:00",
            "2026-10-08T10:31:05",
            "",
        ] {
            assert!(parse_stamp(s, &TimeZone::UTC).is_none(), "{s}");
        }
    }
}
