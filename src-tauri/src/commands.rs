use crate::{
    domain::{
        settings::Settings,
        timer::{Session, Status},
    },
    error::{AppError, Result, invalid},
    infrastructure::clock::{now_ms, wall_ms},
    services::{
        startup::{self, Startup},
        window,
    },
    state::AppData,
};
use tauri::{Manager, State};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_dialog::DialogExt;
type Backend<'a> = State<'a, Startup>;

#[tauri::command(async)]
pub fn get_app_snapshot(backend: Backend<'_>) -> Result<AppData> {
    backend.ready()?.snapshot()
}

#[tauri::command(async)]
pub fn timer_action(
    backend: Backend<'_>,
    action: String,
    session: Option<Session>,
) -> Result<AppData> {
    let engine = backend.ready()?;
    engine.update("timer:changed", |d| {
        match action.as_str() {
            "start" | "resume" => d.timer.start(now_ms(), wall_ms(), &d.settings),
            "pause" => d.timer.pause(now_ms()),
            "reset" => {
                d.timer.routine_completed_focus = 0;
                d.timer.focus_since_long_break = 0;
                d.timer.select(Session::Focus, &d.settings);
            }
            "skip" => d.timer.skip(now_ms(), wall_ms(), &d.settings),
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
#[tauri::command(async)]
pub fn task_action(backend: Backend<'_>, change: TaskAction) -> Result<AppData> {
    let engine = backend.ready()?;
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
#[tauri::command(async)]
pub fn settings_update(backend: Backend<'_>, settings: Settings) -> Result<AppData> {
    let engine = backend.ready()?;
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
        Ok(())
    });
    if let Ok(data) = &result
        && let Err(error) = window::apply(&engine.app, data, false)
    {
        let _ = engine.update("settings:changed", |d| {
            d.settings = previous.settings.clone();
            if d.timer.status == Status::Idle {
                d.timer.select(d.timer.session, &d.settings);
            }
            Ok(())
        });
        let _ = if previous.settings.start_with_windows {
            launcher.enable()
        } else {
            launcher.disable()
        };
        let _ = window::apply(&engine.app, &previous, false);
        return Err(error);
    }
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
#[tauri::command(async)]
pub fn window_set_mode(backend: Backend<'_>, expanded: bool) -> Result<AppData> {
    let engine = backend.ready()?;
    let previous = engine.snapshot()?;
    let data = engine.update("window:changed", |d| {
        d.window.expanded = expanded;
        Ok(())
    })?;
    if let Err(error) = window::apply(&engine.app, &data, false) {
        let _ = engine.update("window:changed", |d| {
            d.window.expanded = previous.window.expanded;
            Ok(())
        });
        let _ = window::apply(&engine.app, &previous, false);
        return Err(error);
    }
    Ok(data)
}
#[tauri::command(async)]
pub fn window_close(backend: Backend<'_>) -> Result<()> {
    let engine = backend.ready()?;
    if let Some(window) = engine.app.get_webview_window("main") {
        window
            .close()
            .map_err(|e| AppError::Native(e.to_string()))?;
    }
    Ok(())
}
#[tauri::command(async)]
pub fn audio_preview(backend: Backend<'_>, kind: String, settings: Option<Settings>) -> Result<()> {
    let engine = backend.ready()?;
    let d = engine.snapshot()?;
    let settings = settings.unwrap_or(d.settings);
    settings.validate()?;
    match kind.as_str() {
        "alarm" => engine
            .audio
            .play(&settings.alarm, settings.alarm_volume, false),
        "tick" => engine
            .audio
            .play(&settings.tick, settings.tick_volume, false),
        _ => Err(invalid("Unknown audio preview.")),
    }
}
#[tauri::command]
pub async fn audio_import_custom(backend: Backend<'_>) -> Result<AppData> {
    let engine = backend.ready()?;
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
                Ok(())
            })
        } else {
            engine.snapshot()
        }
    })
    .await
    .map_err(|e| AppError::Native(e.to_string()))?
}

#[tauri::command(async)]
pub fn window_minimize(backend: Backend<'_>) -> Result<()> {
    let engine = backend.ready()?;
    if let Some(window) = engine.app.get_webview_window("main") {
        window
            .set_skip_taskbar(false)
            .map_err(|e| AppError::Native(e.to_string()))?;
        window
            .minimize()
            .map_err(|e| AppError::Native(e.to_string()))?;
    }
    Ok(())
}
#[tauri::command(async)]
pub fn window_hide(backend: Backend<'_>) -> Result<()> {
    let engine = backend.ready()?;
    engine.persist_position()?;
    window::hide(&engine.app)
}
#[tauri::command(async)]
pub fn app_quit(backend: Backend<'_>) -> Result<()> {
    let engine = backend.ready()?;
    engine.persist_position()?;
    engine.update("window:changed", |_| Ok(()))?;
    engine.app.exit(0);
    Ok(())
}
#[tauri::command]
pub fn get_startup_status(backend: Backend<'_>) -> startup::StartupStatus {
    backend.status()
}
#[tauri::command(async)]
pub fn frontend_ready() {
    crate::infrastructure::diagnostics::record("frontend", "ready");
}
#[tauri::command]
pub fn startup_retry(app: tauri::AppHandle) -> Result<()> {
    startup::launch(
        app.clone(),
        app.path()
            .app_data_dir()
            .map_err(|e| AppError::Native(e.to_string()))?,
    );
    Ok(())
}
