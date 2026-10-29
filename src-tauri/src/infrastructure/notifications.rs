use crate::{
    domain::{settings::Settings, timer::Session},
    state::Completion,
};
pub fn content(completion: &Completion, settings: &Settings) -> (String, String) {
    match completion.session {
        Session::Focus if completion.next_session == Some(Session::LongBreak) => (
            "Time for a long break!".into(),
            format!(
                "You've completed your focus cycle. Enjoy your {}-minute break.",
                settings.long_break_minutes
            ),
        ),
        Session::Focus => (
            "Focus session complete!".into(),
            format!(
                "Great work! Time for your {}-minute short break.",
                settings.short_break_minutes
            ),
        ),
        _ => (
            "Break finished!".into(),
            "Ready to focus again? Your next session is waiting.".into(),
        ),
    }
}
