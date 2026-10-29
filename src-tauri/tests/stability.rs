use rvnflow_lib::{
    domain::{
        settings::Settings,
        timer::{Session, Status, Timer},
    },
    error::invalid,
    infrastructure::{notifications, storage::Storage},
    services::repository::Repository,
    state::AppData,
};
use std::sync::{Arc, mpsc};
use std::time::Duration;

#[test]
fn reentrant_snapshot_during_mutation_and_slow_save_does_not_deadlock() {
    let repository = Arc::new(Repository::new(AppData::default()));
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let writer = repository.clone();
    let thread = std::thread::spawn(move || {
        writer.transact(
            |data| {
                // Reproduces the old native callback's read of state during a mutation.
                assert_eq!(writer.snapshot()?.revision, data.revision);
                data.settings.focus_minutes = 30;
                Ok(())
            },
            |_| {
                entered_tx.send(()).unwrap();
                release_rx.recv().unwrap();
                Ok(())
            },
        )
    });
    entered_rx
        .recv_timeout(Duration::from_secs(2))
        .expect("mutation reentered a held state lock");
    assert_eq!(repository.snapshot().unwrap().settings.focus_minutes, 25);
    release_tx.send(()).unwrap();
    thread.join().unwrap().unwrap();
    assert_eq!(repository.snapshot().unwrap().settings.focus_minutes, 30);
}
#[test]
fn failed_persistence_does_not_publish_candidate_data() {
    let repository = Repository::new(AppData::default());
    assert!(
        repository
            .transact(
                |data| {
                    data.settings.focus_minutes = 30;
                    Ok(())
                },
                |_| Err(invalid("disk full"))
            )
            .is_err()
    );
    assert_eq!(repository.snapshot().unwrap().settings.focus_minutes, 25);
    assert_eq!(repository.snapshot().unwrap().revision, 0);
}
#[test]
fn concurrent_writers_preserve_both_updates() {
    let repository = Arc::new(Repository::new(AppData::default()));
    let handles: Vec<_> = (0..8)
        .map(|_| {
            let repository = repository.clone();
            std::thread::spawn(move || {
                repository.transact(
                    |data| {
                        data.timer.completed_focus += 1;
                        Ok(())
                    },
                    |_| Ok(()),
                )
            })
        })
        .collect();
    for handle in handles {
        handle.join().unwrap().unwrap();
    }
    assert_eq!(repository.snapshot().unwrap().timer.completed_focus, 8);
}
#[test]
fn four_fifty_minute_sessions_run_automatically_with_ten_minute_breaks() {
    let settings = Settings {
        focus_minutes: 50,
        short_break_minutes: 10,
        sessions_per_cycle: 4,
        repeat_routine: false,
        ..Settings::default()
    };
    let mut timer = Timer::new(&settings);
    let mut now = 0;
    timer.start(now, now, &settings);
    for focus in 1..=4 {
        assert_eq!(timer.session, Session::Focus);
        assert_eq!(timer.status, Status::Running);
        assert_eq!(timer.remaining(now), 3_000_000);
        now += 3_000_000;
        assert_eq!(timer.advance(now, now, &settings), Some(Session::Focus));
        assert_eq!(timer.completed_focus, focus);
        assert_eq!(timer.advance(now, now, &settings), None);
        if focus < 4 {
            assert_eq!(timer.session, Session::ShortBreak);
            assert_eq!(timer.status, Status::Running);
            assert_eq!(timer.remaining(now), 600_000);
            now += 600_000;
            assert_eq!(
                timer.advance(now, now, &settings),
                Some(Session::ShortBreak)
            );
        }
    }
    assert_eq!(now, 13_800_000);
    assert_eq!(timer.status, Status::Completed);
    assert!(timer.routine_complete);
    assert_eq!(
        timer.advance(now + 99_000_000, now + 99_000_000, &settings),
        None
    );
    timer.start(now, now, &settings);
    assert_eq!(timer.session, Session::Focus);
    assert_eq!(timer.status, Status::Running);
    assert_eq!(timer.routine_completed_focus, 0);
    assert_eq!(timer.completed_focus, 4);
}
#[test]
fn long_break_every_two_replaces_short_break_independently_of_four_session_goal() {
    let settings = Settings {
        focus_minutes: 50,
        short_break_minutes: 10,
        long_break_minutes: 30,
        sessions_per_cycle: 4,
        repeat_routine: false,
        long_break_enabled: true,
        long_break_interval: 2,
        ..Settings::default()
    };
    let mut timer = Timer::new(&settings);
    let mut now = 0;
    timer.start(now, now, &settings);
    for expected in [Session::ShortBreak, Session::LongBreak, Session::ShortBreak] {
        now += 3_000_000;
        timer.advance(now, now, &settings);
        assert_eq!(timer.session, expected);
        assert_eq!(timer.status, Status::Running);
        let rest = if expected == Session::LongBreak {
            1_800_000
        } else {
            600_000
        };
        assert_eq!(timer.remaining(now), rest);
        now += rest;
        assert_eq!(timer.advance(now, now, &settings), Some(expected));
        assert_eq!(timer.session, Session::Focus);
        assert_eq!(timer.status, Status::Running);
    }
    now += 3_000_000;
    timer.advance(now, now, &settings);
    assert!(timer.routine_complete);
    assert_eq!(timer.completed_focus, 4);
}
#[test]
fn automatic_transitions_anchor_to_wake_and_preserve_current_task() {
    let settings = Settings::default();
    let mut timer = Timer::new(&settings);
    timer.task_id = Some("current-task".into());
    timer.start(0, 0, &settings);
    assert!(timer.advance(100_000_000, 100_000_000, &settings).is_some());
    assert_eq!(timer.session, Session::ShortBreak);
    assert_eq!(timer.status, Status::Running);
    assert_eq!(timer.remaining(100_000_000), 300_000);
    assert_eq!(timer.advance(100_000_000, 100_000_000, &settings), None);
    timer.advance(100_300_000, 100_300_000, &settings);
    assert_eq!(timer.status, Status::Running);
    assert_eq!(timer.session, Session::Focus);
    assert_eq!(timer.remaining(100_300_000), 1_500_000);
    assert_eq!(timer.task_id.as_deref(), Some("current-task"));
}
#[test]
fn long_break_can_be_disabled_or_scheduled_every_four_sessions() {
    for enabled in [false, true] {
        let settings = Settings {
            sessions_per_cycle: 8,
            long_break_enabled: enabled,
            long_break_interval: 4,
            ..Settings::default()
        };
        let mut timer = Timer::new(&settings);
        let mut now = 0;
        timer.start(now, now, &settings);
        for focus in 1..=4 {
            now += 1_500_000;
            timer.advance(now, now, &settings);
            assert_eq!(
                timer.session,
                if enabled && focus == 4 {
                    Session::LongBreak
                } else {
                    Session::ShortBreak
                }
            );
            now += timer.duration_ms;
            timer.advance(now, now, &settings);
        }
    }
}
#[test]
fn repeating_routines_preserve_long_break_cadence_and_never_stop() {
    let settings = Settings {
        sessions_per_cycle: 4,
        long_break_enabled: true,
        long_break_interval: 3,
        ..Settings::default()
    };
    let mut timer = Timer::new(&settings);
    let mut now = 0;
    timer.start(now, now, &settings);
    for focus in 1..=9 {
        now += 1_500_000;
        timer.advance(now, now, &settings);
        assert_eq!(
            timer.session,
            if focus % 3 == 0 {
                Session::LongBreak
            } else {
                Session::ShortBreak
            }
        );
        assert_eq!(timer.status, Status::Running);
        assert!(!timer.routine_complete);
        now += timer.duration_ms;
        timer.advance(now, now, &settings);
        assert_eq!(timer.session, Session::Focus);
    }
}
#[test]
fn v01_upgrade_preserves_tasks_paused_timer_and_explicit_notification_preference() {
    let dir = tempfile::tempdir().unwrap();
    let store = Storage::new(dir.path().into()).unwrap();
    let mut data = AppData::default();
    let list = data.tasks.active_list_id.clone();
    data.tasks.create(&list, "Existing task", 1).unwrap();
    data.timer.start(0, 0, &data.settings);
    data.timer.pause(2000);
    data.settings.notifications = false;
    let mut json = serde_json::to_value(&data).unwrap();
    json["schemaVersion"] = 1.into();
    json["settings"]
        .as_object_mut()
        .unwrap()
        .remove("autoStartBreaks");
    json["settings"]
        .as_object_mut()
        .unwrap()
        .remove("autoStartFocus");
    std::fs::write(
        dir.path().join("app-state.json"),
        serde_json::to_vec(&json).unwrap(),
    )
    .unwrap();
    let upgraded = store.load().unwrap();
    assert_eq!(upgraded.schema_version, 3);
    assert_eq!(upgraded.tasks.tasks[0].title, "Existing task");
    assert_eq!(upgraded.timer.status, Status::Paused);
    assert_eq!(upgraded.timer.remaining_ms, 1_498_000);
    assert!(!upgraded.settings.notifications);
    assert!(upgraded.settings.repeat_routine);
    store.save(&upgraded).unwrap();
    assert_eq!(store.load().unwrap().tasks.tasks.len(), 1);
}
#[test]
fn contextual_notification_uses_real_duration_and_next_session() {
    let settings = Settings {
        short_break_minutes: 7,
        long_break_minutes: 22,
        ..Settings::default()
    };
    let mut completion = rvnflow_lib::state::Completion {
        finished_at: 1,
        session: Session::Focus,
        task_id: None,
        next_session: Some(Session::ShortBreak),
    };
    assert!(
        notifications::content(&completion, &settings)
            .1
            .contains("7-minute")
    );
    completion.next_session = Some(Session::LongBreak);
    assert!(
        notifications::content(&completion, &settings)
            .1
            .contains("22-minute")
    );
    completion.session = Session::LongBreak;
    assert_eq!(
        notifications::content(&completion, &settings).0,
        "Break finished!"
    );
}

#[test]
fn legacy_completed_and_running_timers_recover_without_duplicate_credit() {
    for completed in [false, true] {
        let dir = tempfile::tempdir().unwrap();
        let storage = Storage::new(dir.path().into()).unwrap();
        let mut data = AppData::default();
        data.timer.start(0, 0, &data.settings);
        if completed {
            data.timer.complete(1_500_000);
        }
        let mut raw = serde_json::to_value(&data).unwrap();
        raw["schemaVersion"] = 1.into();
        std::fs::write(
            dir.path().join("app-state.json"),
            serde_json::to_vec(&raw).unwrap(),
        )
        .unwrap();
        let mut recovered = storage.load().unwrap();
        assert!(dir.path().join("app-state.v1.json").exists());
        if !completed {
            recovered.timer.recover(100, 10_000_000);
            assert_eq!(
                recovered
                    .timer
                    .advance(100, 10_000_000, &recovered.settings),
                Some(Session::Focus)
            );
        }
        assert_eq!(recovered.timer.completed_focus, 1);
        assert_eq!(recovered.timer.session, Session::ShortBreak);
        assert_eq!(
            recovered
                .timer
                .advance(100, 10_000_000, &recovered.settings),
            None
        );
        assert_eq!(recovered.timer.completed_focus, 1);
    }
}

#[test]
fn v02_manual_start_flags_do_not_disable_automatic_routines_after_upgrade() {
    let dir = tempfile::tempdir().unwrap();
    let storage = Storage::new(dir.path().into()).unwrap();
    let mut data = AppData::default();
    data.settings.focus_minutes = 50;
    data.settings.short_break_minutes = 10;
    let list = data.tasks.active_list_id.clone();
    data.tasks.create(&list, "Preserved task", 1).unwrap();
    data.timer = Timer::new(&data.settings);
    data.timer.start(0, 0, &data.settings);
    let mut json = serde_json::to_value(&data).unwrap();
    json["schemaVersion"] = 2.into();
    let settings = json["settings"].as_object_mut().unwrap();
    for key in ["longBreakEnabled", "longBreakInterval", "repeatRoutine"] {
        settings.remove(key);
    }
    settings.insert("autoStartBreaks".into(), false.into());
    settings.insert("autoStartFocus".into(), false.into());
    std::fs::write(
        dir.path().join("app-state.json"),
        serde_json::to_vec(&json).unwrap(),
    )
    .unwrap();
    let mut upgraded = storage.load().unwrap();
    assert_eq!(upgraded.tasks.tasks[0].title, "Preserved task");
    assert_eq!(upgraded.schema_version, 3);
    assert!(dir.path().join("app-state.v2.json").exists());
    upgraded.timer.recover(0, 0);
    upgraded
        .timer
        .advance(3_000_000, 3_000_000, &upgraded.settings);
    assert_eq!(upgraded.timer.status, Status::Running);
    assert_eq!(upgraded.timer.session, Session::ShortBreak);
    assert_eq!(upgraded.timer.remaining(3_000_000), 600_000);
    assert!(
        serde_json::to_value(&upgraded.settings)
            .unwrap()
            .get("autoStartBreaks")
            .is_none()
    );
}

#[test]
fn completed_legacy_four_session_routine_does_not_gain_an_extra_focus_session() {
    let dir = tempfile::tempdir().unwrap();
    let storage = Storage::new(dir.path().into()).unwrap();
    let mut data = AppData::default();
    data.settings.repeat_routine = false;
    data.timer.start(0, 0, &data.settings);
    data.timer.complete(1_500_000);
    data.timer.completed_focus = 4;
    let mut raw = serde_json::to_value(&data).unwrap();
    raw["schemaVersion"] = 2.into();
    std::fs::write(
        dir.path().join("app-state.json"),
        serde_json::to_vec(&raw).unwrap(),
    )
    .unwrap();
    let mut upgraded = storage.load().unwrap();
    assert!(upgraded.timer.routine_complete);
    assert_eq!(upgraded.timer.routine_completed_focus, 4);
    assert_eq!(
        upgraded
            .timer
            .advance(9_000_000, 9_000_000, &upgraded.settings),
        None
    );
    assert_eq!(upgraded.timer.completed_focus, 4);
}

#[test]
fn migrated_completed_cycle_starts_its_long_break_without_extra_credit() {
    let dir = tempfile::tempdir().unwrap();
    let storage = Storage::new(dir.path().into()).unwrap();
    let mut data = AppData::default();
    data.settings.long_break_enabled = true;
    data.settings.long_break_interval = 4;
    data.timer.start(0, 0, &data.settings);
    data.timer.complete(1_500_000);
    data.timer.completed_focus = 4;
    let mut raw = serde_json::to_value(&data).unwrap();
    raw["schemaVersion"] = 2.into();
    std::fs::write(
        dir.path().join("app-state.json"),
        serde_json::to_vec(&raw).unwrap(),
    )
    .unwrap();
    let upgraded = storage.load().unwrap();
    assert_eq!(upgraded.timer.session, Session::LongBreak);
    assert_eq!(upgraded.timer.status, Status::Running);
    assert_eq!(upgraded.timer.completed_focus, 4);
}

#[test]
fn migrated_prepared_break_runs_but_explicitly_paused_break_stays_paused() {
    for paused in [false, true] {
        let dir = tempfile::tempdir().unwrap();
        let storage = Storage::new(dir.path().into()).unwrap();
        let mut data = AppData::default();
        data.timer.completed_focus = 1;
        data.timer.select(Session::ShortBreak, &data.settings);
        if paused {
            data.timer.status = Status::Paused;
        }
        data.history.push(rvnflow_lib::state::Completion {
            finished_at: 1,
            session: Session::Focus,
            task_id: None,
            next_session: Some(Session::ShortBreak),
        });
        let mut raw = serde_json::to_value(&data).unwrap();
        raw["schemaVersion"] = 2.into();
        std::fs::write(
            dir.path().join("app-state.json"),
            serde_json::to_vec(&raw).unwrap(),
        )
        .unwrap();
        let upgraded = storage.load().unwrap();
        assert_eq!(
            upgraded.timer.status,
            if paused {
                Status::Paused
            } else {
                Status::Running
            }
        );
        assert_eq!(upgraded.history.len(), 1);
        assert_eq!(upgraded.timer.completed_focus, 1);
    }
}
