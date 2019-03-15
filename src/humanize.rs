pub fn duration(duration: &chrono::Duration) -> String {
    let min = duration.num_minutes();
    if min < 1 {
        "less than a minute".to_string()
    } else if min < 60 {
        format!("{} minute(s)", min).to_string()
    } else {
        let hour = (min / 60) as i64;
        let min = min - hour * 60;
        format!("{} hour(s) and {} minute(s)", hour, min).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_convert() {
        assert_eq!(
            "less than a minute",
            duration(&chrono::Duration::seconds(42))
        );
        assert_eq!("1 minute(s)", duration(&chrono::Duration::seconds(75)));
        assert_eq!("2 minute(s)", duration(&chrono::Duration::seconds(125)));
        assert_eq!(
            "1 hour(s) and 1 minute(s)",
            duration(&chrono::Duration::seconds(3672))
        );
    }
}
