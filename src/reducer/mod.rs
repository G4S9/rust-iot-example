use crate::{AppAction, AppState};

pub fn reducer(app_state: AppState, action: AppAction) -> AppState {
    match action {
        AppAction::SetSubCount(sub_count) => AppState {
            sub_count,
            ..app_state
        },
        AppAction::SetProvisioned(provisioned) => AppState {
            provisioned,
            ..app_state
        },
        AppAction::SetError(app_error) => AppState {
            error: Some(app_error),
            ..app_state
        },
    }
}
