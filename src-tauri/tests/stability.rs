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
fn focus_and_break_automatically_select_next_once() {
    let settings = Settings::default();
    let mut timer = Timer::new(&settings);
    for index in 1..=4 {
        timer.start(0, 0, &settings);
        assert_eq!(
            timer.advance(1_500_000, 1_500_000, &settings),
            Some(Session::Focus)
        );
        assert_eq!(timer.completed_focus, index);
        assert_eq!(timer.status, Status::Idle);
        assert_eq!(
            timer.session,
            if index == 4 {
                Session::LongBreak
            } else {
                Session::ShortBreak
            }
        );
        assert_eq!(
            timer.remaining_ms,
            Timer::duration(timer.session, &settings)
        );
        assert_eq!(timer.advance(9_000_000, 9_000_000, &settings), None);
        timer.start(10_000_000, 10_000_000, &settings);
        assert!(timer.advance(20_000_000, 20_000_000, &settings).is_some());
        assert_eq!(timer.session, Session::Focus);
        assert_eq!(timer.status, Status::Idle);
    }
}
#[test]
fn independent_automation_and_sleep_anchor_prevent_catchup_chains() {
    let mut settings = Settings {
        auto_start_breaks: true,
        ..Settings::default()
    };
    let mut timer = Timer::new(&settings);
    timer.task_id = Some("current-task".into());
    timer.start(0, 0, &settings);
    assert!(timer.advance(100_000_000, 100_000_000, &settings).is_some());
    assert_eq!(timer.session, Session::ShortBreak);
    assert_eq!(timer.status, Status::Running);
    assert_eq!(timer.remaining(100_000_000), 300_000);
    assert_eq!(timer.advance(100_000_000, 100_000_000, &settings), None);
    timer.advance(100_300_000, 100_300_000, &settings);
    assert_eq!(timer.status, Status::Idle);
    assert_eq!(timer.session, Session::Focus);
    settings.auto_start_focus = true;
    settings.auto_start_breaks = false;
    timer.select(Session::ShortBreak, &settings);
    timer.start(0, 0, &settings);
    timer.advance(300_000, 300_000, &settings);
    assert_eq!(timer.status, Status::Running);
    assert_eq!(timer.session, Session::Focus);
    assert_eq!(timer.task_id.as_deref(), Some("current-task"));
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
    assert_eq!(upgraded.schema_version, 2);
    assert_eq!(upgraded.tasks.tasks[0].title, "Existing task");
    assert_eq!(upgraded.timer.status, Status::Paused);
    assert_eq!(upgraded.timer.remaining_ms, 1_498_000);
    assert!(!upgraded.settings.notifications);
    assert!(!upgraded.settings.auto_start_breaks);
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
                .advance(100_000_000, 100_000_000, &recovered.settings),
            None
        );
        assert_eq!(recovered.timer.completed_focus, 1);
    }
}
