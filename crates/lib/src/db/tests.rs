use spyland_core::{Session, State};
use sqlx::{SqlitePool, query};

use crate::db::{Db, SessionSql};

#[sqlx::test]
async fn create_test(pool: SqlitePool) {
    let db = Db { pool };

    db.create().await.unwrap();
}

#[sqlx::test]
async fn insert_test(pool: SqlitePool) {
    let db = Db { pool };

    db.create().await.unwrap();

    let session = Session {
        start: 1,
        end: 16,
        state: State::Active {
            app_id: "example_test_app_id".into(),
            workspace: None,
        },
    };

    let result = db.insert(session.into()).await.unwrap();

    assert_eq!(result.rows_affected(), 1);
}

#[sqlx::test]
async fn insert_integrity_test(pool: SqlitePool) {
    let db = Db { pool };

    db.create().await.unwrap();

    const START: u64 = 1;
    const END: u64 = 31;
    const APP_ID: &str = "steam";
    const WORKSPACE: i32 = 3;

    let session = Session {
        start: START,
        end: END,
        state: State::Active {
            app_id: APP_ID.into(),
            workspace: Some(WORKSPACE),
        },
    };

    let result = db.insert(session.into()).await.unwrap();

    assert_eq!(result.rows_affected(), 1);

    let result = query!("SELECT * FROM sessions")
        .fetch_one(&db.pool)
        .await
        .unwrap();

    assert_eq!(result.start, START as i64);
    assert_eq!(result.end, END as i64);
    assert!(result.is_active);
    assert_eq!(result.app_id, Some(APP_ID.into()));
    assert_eq!(result.workspace, Some(WORKSPACE as i64));
}

#[test]
fn session_mapping_test() {
    const START: u64 = 1;
    const END: u64 = 16;

    const APP_ID: &str = "example_test_app_id";
    const WORKSPACE: i32 = 2;

    let session = Session {
        start: START,
        end: END,

        state: State::Active {
            app_id: APP_ID.into(),
            workspace: Some(WORKSPACE),
        },
    };

    let session_sql: SessionSql = session.into();

    assert_eq!(session_sql.start, START as i64);
    assert_eq!(session_sql.end, END as i64);
    assert!(session_sql.is_active);
    assert_eq!(session_sql.app_id, Some(APP_ID.into()));
    assert_eq!(session_sql.workspace, Some(WORKSPACE as i64));

    let session2: Session = session_sql.into();

    assert_eq!(session2.start, START);
    assert_eq!(session2.end, END);
    assert!(matches!(
        session2.state,
        State::Active {
            app_id,
            workspace: Some(WORKSPACE),
        } if app_id == APP_ID
    ));
}

#[sqlx::test]
async fn update_by_rowid_test(pool: SqlitePool) {
    let db = Db { pool };

    db.create().await.unwrap();

    let session1 = Session {
        start: 1,
        end: 16,
        state: State::Active {
            app_id: "firefox".into(),
            workspace: Some(1),
        },
    };

    db.insert(session1.into()).await.unwrap();

    const UPDATED_END: i64 = 50;
    const UPDATED_APP_ID: &str = "chromium";

    let updated_session = SessionSql {
        start: 1,
        end: UPDATED_END,
        is_active: true,
        app_id: Some(UPDATED_APP_ID.into()),
        workspace: Some(1),
    };

    let result = db.update_by_rowid(1, updated_session).await.unwrap();
    assert_eq!(result.rows_affected(), 1);

    let sessions = db.query_all().await.unwrap();
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].end, UPDATED_END);
    assert_eq!(sessions[0].app_id, Some("chromium".into()));
}

#[sqlx::test]
async fn update_last_test(pool: SqlitePool) {
    let db = Db { pool };

    db.create().await.unwrap();

    const S1_END: u64 = 20;

    let session1 = Session {
        start: 1,
        end: S1_END,
        state: State::Active {
            app_id: "firefox".into(),
            workspace: None,
        },
    };

    const START: u64 = 20;
    const APP_ID: &str = "steam";
    const S2_END: u64 = 60;

    let session2 = Session {
        start: START,
        end: 40,
        state: State::Active {
            app_id: APP_ID.into(),
            workspace: Some(2),
        },
    };

    db.insert(session1.into()).await.unwrap();
    db.insert(session2.into()).await.unwrap();

    let updated_last = SessionSql {
        start: START as i64,
        end: S2_END as i64,
        is_active: true,
        app_id: Some(APP_ID.into()),
        workspace: Some(2),
    };

    let result = db.update_last(updated_last).await.unwrap();
    assert_eq!(result.rows_affected(), 1);

    let sessions = db.query_all().await.unwrap();
    assert_eq!(sessions.len(), 2);
    assert_eq!(sessions[0].end, S1_END as i64);
    assert_eq!(sessions[1].end, S2_END as i64);
}
