use std::{
    fmt::Display,
    time::{SystemTime, UNIX_EPOCH},
};

use chrono::{DateTime, Local, LocalResult, NaiveDateTime, TimeZone};
use derive_more::{AsMut, AsRef, Deref, DerefMut};
use serde::Serialize;

fn system_time_to_date_time(time: SystemTime) -> LocalResult<DateTime<Local>> {
    let (secs, nano_secs) = time
        .duration_since(UNIX_EPOCH)
        .map(|duration| {
            (
                duration
                    .as_secs()
                    .try_into()
                    .expect("time within reasonable range"),
                duration.subsec_nanos(),
            )
        })
        .expect("time not to have gone backwards");

    Local.timestamp_opt(secs, nano_secs)
}

#[derive(Debug, Copy, Clone, AsRef, AsMut, Deref, DerefMut)]
pub struct NicerTime(DateTime<Local>);

impl From<SystemTime> for NicerTime {
    fn from(time: SystemTime) -> Self {
        Self(system_time_to_date_time(time).unwrap())
    }
}

impl Display for NicerTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.format("%Y-%m-%d %H:%M:%S").fmt(f)
    }
}

impl Serialize for NicerTime {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.to_string().serialize(serializer)
    }
}

#[derive(Debug, Copy, Clone, AsRef, AsMut, Deref, DerefMut)]
pub struct NicerNaiveTime(NaiveDateTime);

impl From<NaiveDateTime> for NicerNaiveTime {
    fn from(time: NaiveDateTime) -> Self {
        Self(time)
    }
}

impl Display for NicerNaiveTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Serialize for NicerNaiveTime {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.to_string().serialize(serializer)
    }
}
