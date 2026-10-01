use bollard::Docker;
use chrono::{DateTime, Local, Utc};
use color_eyre::{Report, Result, eyre::eyre};

pub fn driver_connector() -> Result<Docker> {
    Ok(Docker::connect_with_local_defaults()?)
}

pub fn parse_timestamp_from_epoch(epoch: i64) -> Result<String, Report> {
    let date: String = DateTime::<Utc>::from_timestamp(epoch, 0)
        .ok_or_else(|| eyre!("Invaid timestamp: {epoch}"))?
        .with_timezone(&Local)
        .format("%d-%m-%Y")
        .to_string();
    Ok(date)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_timestamp() {
        let epoch: i64 = i64::MAX;
        let result: Result<String, Report> = parse_timestamp_from_epoch(epoch);

        assert!(result.is_err());
    }

    #[test]
    fn test_valid_timestamp() {
        let epoch: i64 = 1_788_826_315;
        let result: String = parse_timestamp_from_epoch(epoch).unwrap();
        assert_eq!(result.len(), 10);
        assert_eq!(&result[2..3], "-");
        assert_eq!(&result[5..6], "-");
    }
}
