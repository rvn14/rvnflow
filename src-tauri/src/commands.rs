use crate::{
    domain::{
        settings::Settings,
        timer::{Session, Status},
    },
    error::{AppError, Result, invalid},
    infrastructure::clock::{now_ms, wall_ms},
    services::{engine::Engine, window},
    state::AppData,
};
use std::sync::Arc;
use tauri::{Manager, State};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_dialog::DialogExt;
type Backend<'a> = State<'a, Arc<Engine>>;

#[tauri::command]
pub fn get_app_snapshot(engine: Backend<'_>) -> Result<AppData> {
    engine.snapshot()
}

#[tauri::command]
pub fn timer_action(
    engine: Backend<'_>,
    action: String,
    session: Option<Session>,
) -> Result<AppData> {
    engine.update("timer:changed", |d| {
        match action.as_str() {
            "start" | "resume" => d.timer.start(now_ms(), wall_ms(), &d.settings),
            "pause" => d.timer.pause(now_ms()),
            "reset" => d.timer.select(d.timer.session, &d.settings),
            "skip" => d.timer.skip(&d.settings),
            "select" => d.timer.select(
                session.ok_or_else(|| invalid("Choose a session."))?,
                &d.settings,
            ),
            _ => return Err(invalid("Unknown timer action.")),
        }
        Ok(())
    })
}
#[derive(serde::Deserialize)]
#[serde(tag = "action", rename_all = "camelCase")]
pub enum TaskAction {
    Create {
        #[serde(rename = "listId")]
        list_id: String,
        title: String,
    },
    Update {
        id: String,
        title: String,
    },
    Toggle {
        id: String,
    },
    Delete {
        id: String,
    },
    Reorder {
        #[serde(rename = "listId")]
        list_id: String,
        ids: Vec<String>,
    },
    CreateList {
        name: String,
    },
    RenameList {
        id: String,
        name: String,
    },
    DeleteList {
        id: String,
    },
    SelectList {
        id: String,
    },
    Associate {
        id: Option<String>,
    },
}
#[tauri::command]
pub fn task_action(engine: Backend<'_>, change: TaskAction) -> Result<AppData> {
    engine.update("tasks:changed", |d| {
        match change {
            TaskAction::Create { list_id, title } => {
                d.tasks.create(&list_id, &title, wall_ms())?;
            }
            TaskAction::Update { id, title } => {
                d.tasks.update(&id, Some(&title), false, wall_ms())?
            }
            TaskAction::Toggle { id } => d.tasks.update(&id, None, true, wall_ms())?,
            TaskAction::Delete { id } => {
                d.tasks.delete(&id)?;
                if d.timer.task_id.as_ref() == Some(&id) {
                    d.timer.task_id = None;
                }
            }
            TaskAction::Reorder { list_id, ids } => d.tasks.reorder(&list_id, &ids)?,
            TaskAction::CreateList { name } => {
                d.tasks.create_list(&name)?;
            }
            TaskAction::RenameList { id, name } => d.tasks.rename_list(&id, &name)?,
            TaskAction::DeleteList { id } => {
                d.tasks.delete_list(&id)?;
                if d.timer
                    .task_id
                    .as_ref()
                    .is_some_and(|id| !d.tasks.tasks.iter().any(|t| &t.id == id))
                {
                    d.timer.task_id = None;
                }
            }
            TaskAction::SelectList { id } => {
                if !d.tasks.lists.iter().any(|l| l.id == id) {
                    return Err(invalid("List not found."));
                }
                d.tasks.active_list_id = id;
            }
            TaskAction::Associate { id } => {
                if id
                    .as_ref()
                    .is_some_and(|id| !d.tasks.tasks.iter().any(|t| &t.id == id))
                {
                    return Err(invalid("Task not found."));
                }
                d.timer.task_id = id;
            }
        }
        Ok(())
    })
}
#[tauri::command]
pub fn settings_update(engine: Backend<'_>, settings: Settings) -> Result<AppData> {
    settings.validate()?;
    let previous = engine.snapshot()?;
    if settings.alarm == "custom" && !previous.custom_audio {
        return Err(invalid("Import a custom sound first."));
    }
    let launcher = engine.app.autolaunch();
    if settings.start_with_windows != previous.settings.start_with_windows {
        if settings.start_with_windows {
            launcher.enable()
        } else {
            launcher.disable()
        }
        .map_err(|e| AppError::Native(e.to_string()))?;
    }
    let result = engine.update("settings:changed", |d| {
        d.settings = settings;
        if d.timer.status == Status::Idle {
            d.timer.select(d.timer.session, &d.settings);
        }
        window::apply(&engine.app, d, false)?;
        Ok(())
    });
    if result.is_err() {
        let _ = if previous.settings.start_with_windows {
            launcher.enable()
        } else {
            launcher.disable()
        };
        let _ = window::apply(&engine.app, &previous, false);
    }
    result
}
#[tauri::command]
pub fn window_set_mode(engine: Backend<'_>, expanded: bool) -> Result<AppData> {
    engine.update("window:changed", |d| {
        d.window.expanded = expanded;
        window::apply(&engine.app, d, false)
    })
}
#[tauri::command]
pub fn window_close(engine: Backend<'_>) -> Result<()> {
    if let Some(window) = engine.app.get_webview_window("main") {
        window
            .close()
            .map_err(|e| AppError::Native(e.to_string()))?;
    }
    Ok(())
}
#[tauri::command]
pub fn audio_preview(engine: Backend<'_>, kind: String) -> Result<()> {
    let d = engine.snapshot()?;
    match kind.as_str() {
        "alarm" => engine
            .audio
            .play(&d.settings.alarm, d.settings.alarm_volume, false),
        "tick" => engine
            .audio
            .play(&d.settings.tick, d.settings.tick_volume, false),
        _ => Err(invalid("Unknown audio preview.")),
    }
}
#[tauri::command]
pub async fn audio_import_custom(engine: Backend<'_>) -> Result<AppData> {
    let engine = engine.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let file = engine
            .app
            .dialog()
            .file()
            .add_filter("Alarm sound", &["wav", "mp3"])
            .blocking_pick_file();
        if let Some(file) = file {
            let path = file.into_path().map_err(|e| invalid(e.to_string()))?;
            engine
                .audio
                .import(&path, &engine.storage.directory.join("custom-audio"))?;
            engine.update("settings:changed", |d| {
                d.custom_audio = true;
                d.settings.alarm = "custom".into();
                Ok(())
            })
        } else {
            engine.snapshot()
        }
    })
    .await
    .map_err(|e| AppError::Native(e.to_string()))?
}
