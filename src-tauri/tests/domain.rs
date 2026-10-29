use rvnflow_lib::{
    domain::{
        settings::Settings,
        task::Tasks,
        timer::{Session, Status, Timer},
    },
    infrastructure::storage::Storage,
    state::AppData,
};

#[test]
fn timer_pause_resume_reset_and_drift() {
    let settings = Settings::default();
    let mut timer = Timer::new(&settings);
    timer.start(10, 100, &settings);
    assert_eq!(timer.remaining(1010), 1_499_000);
    timer.pause(1010);
    assert_eq!(timer.status, Status::Paused);
    assert_eq!(timer.remaining(99_000), 1_499_000);
    timer.start(100_000, 200_000, &settings);
    assert_eq!(timer.remaining(101_000), 1_498_000);
    timer.select(Session::Focus, &settings);
    assert_eq!(timer.status, Status::Idle);
    assert_eq!(timer.remaining_ms, 1_500_000);
}
#[test]
fn completion_is_exactly_once_and_cycles_require_confirmation() {
    let settings = Settings::default();
    let mut timer = Timer::new(&settings);
    for index in 1..=4 {
        timer.select(Session::Focus, &settings);
        timer.start(0, 0, &settings);
        assert!(!timer.complete(1_499_999));
        assert!(timer.complete(1_500_000));
        assert!(!timer.complete(9_000_000));
        assert_eq!(timer.completed_focus, index);
        assert_eq!(timer.status, Status::Completed);
        timer.start(2_000_000, 2_000_000, &settings);
        assert_eq!(
            timer.session,
            if index == 4 {
                Session::LongBreak
            } else {
                Session::ShortBreak
            }
        );
    }
}
#[test]
fn sleep_clock_changes_and_restart_recovery() {
    let settings = Settings::default();
    let mut timer = Timer::new(&settings);
    timer.start(0, 10_000, &settings);
    // Live time uses the injected boot clock, never the wall deadline.
    timer.wall_deadline_ms = Some(9_999_999_999);
    assert_eq!(timer.remaining(1000), 1_499_000);
    assert!(timer.complete(3_600_000));
    assert_eq!(timer.completed_focus, 1);
    timer.select(Session::Focus, &settings);
    timer.start(0, 10_000, &settings);
    let mut recovered: Timer =
        serde_json::from_str(&serde_json::to_string(&timer).unwrap()).unwrap();
    recovered.recover(50, 610_000);
    assert_eq!(recovered.remaining(50), 900_000);
    recovered.recover(50, 0);
    assert!(recovered.remaining(50) <= 900_000);
    recovered.recover(50, 9_000_000);
    assert!(recovered.complete(50));
    assert!(!recovered.complete(51));
}
#[test]
fn skipped_focus_does_not_count() {
    let settings = Settings::default();
    let mut timer = Timer::new(&settings);
    timer.start(0, 0, &settings);
    timer.skip(&settings);
    assert_eq!(timer.completed_focus, 0);
    assert_eq!(timer.session, Session::ShortBreak);
    assert_eq!(timer.status, Status::Idle);
}
#[test]
fn tasks_crud_reordering_and_validation() {
    let mut tasks = Tasks::default();
    let list = tasks.active_list_id.clone();
    assert!(tasks.create(&list, " ", 0).is_err());
    assert!(tasks.create("missing", "Read", 0).is_err());
    let a = tasks.create(&list, " Read ", 1).unwrap();
    let b = tasks.create(&list, "Write", 2).unwrap();
    tasks.update(&a, Some("Read chapter 1"), true, 3).unwrap();
    assert_eq!(tasks.tasks[0].completed_at, Some(3));
    tasks.update(&a, None, true, 4).unwrap();
    assert_eq!(tasks.tasks[0].completed_at, None);
    assert!(tasks.reorder(&list, &[a.clone(), a.clone()]).is_err());
    tasks.reorder(&list, &[b.clone(), a.clone()]).unwrap();
    assert_eq!(tasks.tasks[0].order, 1);
    assert!(tasks.delete_list(&list).is_err());
    let other = tasks.create_list("Work").unwrap();
    tasks.rename_list(&other, "University").unwrap();
    tasks.delete(&b).unwrap();
    tasks.delete_list(&list).unwrap();
    assert!(tasks.tasks.is_empty());
    tasks.validate().unwrap();
}
#[test]
fn persistence_recovers_malformed_primary_without_losing_evidence() {
    let dir = tempfile::tempdir().unwrap();
    let store = Storage::new(dir.path().into()).unwrap();
    let mut data = AppData::default();
    let list = data.tasks.active_list_id.clone();
    data.tasks.create(&list, "Keep me", 1).unwrap();
    store.save(&data).unwrap();
    store.save(&data).unwrap();
    assert_eq!(store.load().unwrap().tasks.tasks.len(), 1);
    std::fs::write(dir.path().join("app-state.json"), "{broken").unwrap();
    let recovered = store.load().unwrap();
    assert!(recovered.recovery.is_some());
    assert_eq!(recovered.tasks.tasks[0].title, "Keep me");
    assert!(
        std::fs::read_dir(dir.path()).unwrap().any(|e| e
            .unwrap()
            .file_name()
            .to_string_lossy()
            .contains("corrupt"))
    );
}
#[test]
fn unreadable_data_without_backup_is_not_overwritten() {
    let dir = tempfile::tempdir().unwrap();
    let store = Storage::new(dir.path().into()).unwrap();
    std::fs::write(dir.path().join("app-state.json"), "do not delete").unwrap();
    assert!(store.load().is_err());
    assert_eq!(
        std::fs::read_to_string(dir.path().join("app-state.json")).unwrap(),
        "do not delete"
    );
}
#[test]
fn unknown_schema_is_not_downgraded() {
    let dir = tempfile::tempdir().unwrap();
    let store = Storage::new(dir.path().into()).unwrap();
    store.save(&AppData::default()).unwrap();
    store.save(&AppData::default()).unwrap();
    let data = AppData {
        schema_version: 42,
        ..AppData::default()
    };
    std::fs::write(
        dir.path().join("app-state.json"),
        serde_json::to_vec(&data).unwrap(),
    )
    .unwrap();
    assert!(store.load().is_err());
}
#[test]
fn settings_reject_invalid_ranges_and_nonfinite_volumes() {
    let mut s = Settings::default();
    s.validate().unwrap();
    s.focus_minutes = 0;
    assert!(s.validate().is_err());
    s.focus_minutes = 25;
    s.tick_volume = f32::NAN;
    assert!(s.validate().is_err());
}
