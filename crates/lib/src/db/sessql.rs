use spyland_core::{Session, State};

/// A database representation for [`Session`].
///
/// Used to convert and store [`Session`] data in SQLite.
pub struct SessionSql {
    /// Start time in seconds
    pub start: i64,
    /// End time in seconds
    pub end: i64,

    /// Is active session
    ///
    /// This field determines some other fields:
    /// - [`SessionSql::app_id`] will only have a value ([`Some`]) if this field equals `true`.
    /// - [`SessionSql::workspace`] will never have a value ([`None`]) if this field equals `false`.
    ///
    /// See more documentation for these fields.
    pub is_active: bool,

    /// Application identifier.
    ///
    /// Only [`Some`] if this is an active session.
    /// See [`SessionSql::is_active`]
    pub app_id: Option<String>,
    /// Workspace number.
    ///
    /// Unlike [`SessionSql::app_id`], it may be [`None`] even if [`SessionSql::is_active`] equals
    /// `true`, because of some compositors may not have workspaces at all (see [`State`]).
    pub workspace: Option<i64>,
}

impl From<Session> for SessionSql {
    fn from(session: Session) -> Self {
        let is_active: bool;
        let app_id: Option<String>;
        let workspace: Option<i64>;

        if let State::Active {
            app_id: a,
            workspace: w,
        } = session.state
        {
            is_active = true;
            app_id = Some(a);
            workspace = w.map(|i| i as i64);
        } else {
            is_active = false;
            app_id = None;
            workspace = None;
        }

        Self {
            start: session.start as i64,
            end: session.end as i64,

            is_active,

            app_id,
            workspace,
        }
    }
}

impl From<SessionSql> for Session {
    fn from(value: SessionSql) -> Self {
        if value.is_active {
            Self {
                start: value.start as u64,
                end: value.end as u64,

                state: State::Active {
                    app_id: value.app_id.unwrap(),
                    workspace: value.workspace.map(|i| i as i32),
                },
            }
        } else {
            Self {
                start: value.start as u64,
                end: value.end as u64,

                state: State::Idle,
            }
        }
    }
}
