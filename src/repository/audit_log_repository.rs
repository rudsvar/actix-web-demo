//! Functions for interacting with the audit log.

use sqlx::PgExecutor;

use crate::infra::{audit_log::AuditLogEntry, error::DbError};

/// Store a new user in the database.
pub(crate) async fn store_audit_event(
    conn: impl PgExecutor<'_>,
    audit_log_entry: &AuditLogEntry,
) -> Result<(), DbError> {
    let principal_data = audit_log_entry.principal_data();
    let audit_data = audit_log_entry.audit_data();
    let user_id = principal_data.user_id();
    let module = audit_data.module();
    let function = audit_data.function();
    let entity_id = audit_data.entity_id();
    let input = audit_data.input();
    let output = audit_data.output();
    sqlx::query!(
        r#"
        INSERT INTO audit_log (user_id, module, function, entity_id, input, output)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
        user_id,
        module,
        function,
        entity_id,
        input,
        output
    )
    .execute(conn)
    .await?;

    Ok(())
}
