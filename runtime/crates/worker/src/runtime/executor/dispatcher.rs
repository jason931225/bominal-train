use sqlx::PgPool;

use super::{ExecutionError, ParsedProviderExecution};

pub(super) async fn resolve_execution_material(
    pool: &PgPool,
    provider: &str,
    parsed: &mut ParsedProviderExecution,
    operation_name: &str,
) -> Result<(), ExecutionError> {
    super::resolve_execution_material(pool, provider, parsed, operation_name).await
}
