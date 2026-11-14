use super::{algorithm::*, *};
use jiff::Timestamp;
use uuid::Uuid;

fn instant(s: &str) -> i64 {
    s.parse::<Timestamp>().unwrap().as_millisecond()
}
fn prior(now: i64, rating: u8, plan: &ReviewPlan) -> RecallReviewEvent {
    let next = compute(None, rating, now, plan).unwrap();
    RecallReviewEvent {
        contract_version: 1,
        id: Uuid::new_v4(),
        attempt_id: Uuid::new_v4(),
        session_id: Uuid::new_v4(),
        unit_id: Uuid::new_v4().to_string(),
        activity_revision_id: Uuid::new_v4().to_string(),
        prior_event_id: None,
        generation: plan.generation,
        plan_revision: plan.revision,
        algorithm: ALGORITHM.into(),
        parameters: fsrs::DEFAULT_PARAMETERS.to_vec(),
        parameter_sha256: parameter_hash(),
        retention_basis_points: plan.retention_basis_points,
        time_zone: plan.time_zone.clone(),
        time_zone_rules: rules(),
        day_start_hour: plan.day_start_hour,
        reviewed_at_unix_ms: now,
        study_day: study_day(now, &plan.time_zone, plan.day_start_hour).unwrap(),
        rating,
        memory: next.memory,
        raw_interval_days: next.raw_interval_days,
        interval_days: next.interval_days,
        due_day: next.due_day,
    }
}
fn close(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() <= 0.00002_f32.max(expected.abs() * 0.00001),
        "{actual} != {expected}"
    );
}

#[test]
fn review_fsrs_seven_matches_version_pinned_upstream_initial_states() {
    // Independent numerical fixture from fsrs 7.0.0 inference.rs documentation.
    let expected = [
        (0.1104, 6.1686, 0.08832, 3.8275626e-5),
        (2.2395, 5.261278, 1.7916001, 0.5974598),
        (3.9221, 3.5307243, 3.13768, 4.7777247),
        (11.7841, 1.0, 9.427279, 53.869392),
    ];
    let now = instant("2026-11-14T12:00:00Z");
    for (index, (s, d, fast, interval)) in expected.into_iter().enumerate() {
        let result = compute(None, index as u8 + 1, now, &ReviewPlan::default()).unwrap();
        close(result.memory.stability, s);
        close(result.memory.difficulty, d);
        close(result.memory.stability_fast, fast);
        close(result.raw_interval_days, interval);
        assert_eq!(result.interval_days, [1, 1, 5, 54][index]);
        assert_eq!(
            result.due_day,
            study_day(now, "Etc/UTC", 4).unwrap() + result.interval_days as i32
        );
    }
    assert_eq!(
        fsrs::DEFAULT_PARAMETERS,
        [
            0.1104, 2.2395, 3.9221, 11.7841, 6.1686, 0.6457, 3.6807, 1.9795, 0.0, 1.3826, 0.7024,
            0.5999, 0.8146, 0.6398, 1.0, 1.3207, 0.6707, 3.8668, 0.4416, 0.0934, 1.8631, 0.6162,
            1.0869, 0.1567, 0.0801, 0.2421, 0.9464, 0.1433, 0.7145, 0.0, 0.5667, 0.3734, 0.5333,
            0.3048
        ]
    );
    assert_eq!(parameter_hash().len(), 64);
}

#[test]
fn review_study_days_use_pinned_zone_rules_across_dst_gap_and_fold() {
    let day = |time| study_day(instant(time), "Europe/Kyiv", 4).unwrap();
    assert_eq!(
        day_label(day("2026-03-29T00:59:59Z")).unwrap(),
        "2026-03-28"
    );
    assert_eq!(
        day_label(day("2026-03-29T01:00:00Z")).unwrap(),
        "2026-03-29"
    );
    assert_eq!(day("2026-10-25T00:30:00Z"), day("2026-10-25T01:30:00Z"));
    assert_eq!(day("2026-10-25T02:00:00Z"), day("2026-10-25T01:30:00Z") + 1);
    assert_eq!(
        day_label(day("2026-10-25T02:00:00Z") + 1).unwrap(),
        "2026-10-26"
    );
    assert!(rules().contains("jiff-tzdb=0.1.9"));
}

#[test]
fn review_clock_rollback_is_rejected_but_same_day_and_long_gaps_are_explicit() {
    let now = instant("2026-11-14T12:00:00Z");
    let plan = ReviewPlan::default();
    let p = prior(now, 3, &plan);
    assert_eq!(
        compute(Some(&p), 3, now - 1, &plan).err().unwrap().code,
        ErrorCode::ClockRollback
    );
    for elapsed in [0, 1000, 86_400_000, 30 * 365 * 86_400_000_i64] {
        let result = compute(Some(&p), 3, now + elapsed, &plan).unwrap();
        assert!(result.memory.stability.is_finite());
        assert!(result.memory.stability_fast.is_finite());
        assert!(result.interval_days >= 1);
    }
    for rating in [0, 5, u8::MAX] {
        assert!(compute(None, rating, now, &plan).is_err());
    }
    assert!(compute(None, 3, -1, &plan).is_err());
    assert!(study_day(now, "../private-zone", 4).is_err());
    assert!(study_day(now, "Etc/UTC", 24).is_err());
}
