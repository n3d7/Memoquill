use super::*;
use jiff::{Span, Timestamp, civil::Date, tz::TimeZone};
use sha2::{Digest, Sha256};

pub(super) const ALGORITHM: &str = "fsrs-7/fsrs-rs=7.0.0/daily-v1";
const MAX_TIME: i64 = 253_402_300_799_999;

pub(super) fn rules() -> String {
    format!(
        "jiff-tzdb=0.1.9/{}",
        jiff_tzdb::VERSION.unwrap_or("unknown")
    )
}
fn zone(name: &str) -> Result<TimeZone> {
    if name.len() > 128 {
        return Err(invalid());
    }
    // Load the pinned bundled TZif bytes explicitly; never consult host TZ data.
    let (canonical, bytes) = jiff_tzdb::get(name).ok_or_else(invalid)?;
    TimeZone::tzif(canonical, bytes).map_err(|_| invalid())
}
pub(super) fn valid_time(now: i64) -> Result<()> {
    if (0..=MAX_TIME).contains(&now) {
        Ok(())
    } else {
        Err(invalid())
    }
}
pub(super) struct DayClock {
    zone: TimeZone,
    start: u8,
}
impl DayClock {
    pub(super) fn new(name: &str, start: u8) -> Result<Self> {
        if start > 23 {
            return Err(invalid());
        }
        Ok(Self {
            zone: zone(name)?,
            start,
        })
    }
    pub(super) fn day(&self, now: i64) -> Result<i32> {
        valid_time(now)?;
        let zoned = Timestamp::from_millisecond(now)
            .map_err(|_| invalid())?
            .to_zoned(self.zone.clone());
        let date = if zoned.hour() < self.start as i8 {
            zoned
                .date()
                .checked_sub(Span::new().days(1))
                .map_err(|_| invalid())?
        } else {
            zoned.date()
        };
        Date::new(1970, 1, 1)
            .map_err(|_| invalid())?
            .until(date)
            .map(|s| s.get_days())
            .map_err(|_| invalid())
    }
}
pub(super) fn study_day(now: i64, name: &str, start: u8) -> Result<i32> {
    DayClock::new(name, start)?.day(now)
}
pub(super) fn day_label(day: i32) -> Result<String> {
    Date::new(1970, 1, 1)
        .map_err(|_| invalid())?
        .checked_add(Span::new().days(day))
        .map(|date| date.to_string())
        .map_err(|_| invalid())
}
pub(super) fn validate_plan(plan: &ReviewPlan) -> Result<()> {
    zone(&plan.time_zone)?;
    if plan.contract_version != 1
        || plan.revision < 0
        || plan.generation < 1
        || plan.revision > crate::workspace::types::MAX_REVISION
        || plan.generation > crate::workspace::types::MAX_REVISION
        || plan.day_start_hour > 23
        || plan.daily_new_limit > 100
        || plan.daily_review_limit > 500
        || !(7000..=9900).contains(&plan.retention_basis_points)
        || plan.reminders.len() > 64
    {
        return Err(invalid());
    }
    let mut ids = std::collections::HashSet::new();
    for reminder in &plan.reminders {
        if reminder.id.get_version_num() != 4
            || !ids.insert(reminder.id)
            || reminder.interval_days.is_some_and(|n| n == 0 || n > 3650)
            || reminder
                .completed_at_unix_ms
                .is_some_and(|n| valid_time(n).is_err())
            || day_label(reminder.due_day).is_err()
        {
            return Err(invalid());
        }
    }
    Ok(())
}
pub(super) fn parameter_hash() -> String {
    let mut hash = Sha256::new();
    hash.update(b"memoquill/fsrs7/parameters/v1\0");
    for value in fsrs::DEFAULT_PARAMETERS {
        hash.update(value.to_bits().to_le_bytes());
    }
    format!("{:x}", hash.finalize())
}
pub(super) struct Computed {
    pub memory: RecallMemory,
    pub raw_interval_days: f32,
    pub interval_days: u32,
    pub due_day: i32,
}
pub(super) fn compute(
    prior: Option<&RecallReviewEvent>,
    rating: u8,
    now: i64,
    plan: &ReviewPlan,
) -> Result<Computed> {
    validate_plan(plan)?;
    let today = study_day(now, &plan.time_zone, plan.day_start_hour)?;
    if !(1..=4).contains(&rating) {
        return Err(invalid());
    }
    let memory = prior.map(|p| fsrs::MemoryState {
        stability: p.memory.stability,
        difficulty: p.memory.difficulty,
        stability_fast: p.memory.stability_fast,
    });
    if let Some(p) = prior {
        if p.reviewed_at_unix_ms > now {
            return Err(AppError::new(ErrorCode::ClockRollback));
        }
        if ![
            p.memory.stability,
            p.memory.difficulty,
            p.memory.stability_fast,
        ]
        .iter()
        .all(|v| v.is_finite())
            || p.memory.stability <= 0.0
            || p.memory.stability_fast <= 0.0
            || !(1.0..=10.0).contains(&p.memory.difficulty)
        {
            return Err(corrupt());
        }
    }
    let elapsed = prior.map_or(0.0, |p| {
        ((now - p.reviewed_at_unix_ms) as f64 / 86_400_000.0) as f32
    });
    let model = fsrs::FSRS::new(&fsrs::DEFAULT_PARAMETERS).map_err(|_| invalid())?;
    let states = model
        .next_states_with_elapsed_days(
            memory,
            plan.retention_basis_points as f32 / 10000.0,
            elapsed,
        )
        .map_err(|_| invalid())?;
    let next = match rating {
        1 => states.again,
        2 => states.hard,
        3 => states.good,
        4 => states.easy,
        _ => unreachable!(),
    };
    if !next.interval.is_finite() || next.interval < 0.0 {
        return Err(corrupt());
    }
    let rounded = f64::from(next.interval).round().max(1.0);
    if rounded > i32::MAX as f64 {
        return Err(invalid());
    }
    let due = today.checked_add(rounded as i32).ok_or_else(invalid)?;
    day_label(due)?;
    Ok(Computed {
        memory: RecallMemory {
            stability: next.memory.stability,
            difficulty: next.memory.difficulty,
            stability_fast: next.memory.stability_fast,
        },
        raw_interval_days: next.interval,
        interval_days: rounded as u32,
        due_day: due,
    })
}
