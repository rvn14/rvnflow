use crate::{
    domain::{settings::Settings, timer::Session},
    state::Completion,
};
pub fn content(completion: &Completion, settings: &Settings) -> (String, String) {
    match completion.session {
        Session::Focus if completion.next_session.is_none() => (
            "Routine complete!".into(),
            format!(
                "You've completed all {} focus sessions. Take a moment to recharge.",
                settings.sessions_per_cycle
            ),
        ),
        Session::Focus if completion.next_session == Some(Session::LongBreak) => (
            "Time for a long break!".into(),
            format!(
                "Time to recharge. Your {}-minute long break has started.",
                settings.long_break_minutes
            ),
        ),
        Session::Focus => (
            "Focus session complete!".into(),
            format!(
                "Great work! Your {}-minute short break has started.",
                settings.short_break_minutes
            ),
        ),
        _ => (
            "Break finished!".into(),
            "Your next focus session has started. One thing at a time.".into(),
        ),
    }
}
