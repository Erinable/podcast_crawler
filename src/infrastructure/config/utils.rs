use crate::infrastructure::{AppError, AppResult, InfrastructureError, InfrastructureErrorKind};

pub fn parse_env<T: std::str::FromStr>(env_var: &str) -> AppResult<T> {
    let value = std::env::var(env_var).map_err(|e| {
        let infra_err: InfrastructureError = InfrastructureError::new(
            InfrastructureErrorKind::Config,
            format!("Failed to read environment variable: {}", env_var),
            Some(Box::new(e)),
        );
        AppError::from(infra_err)
    })?;

    value.parse().map_err(|_| {
        let infra_err: InfrastructureError = InfrastructureError::new(
            InfrastructureErrorKind::Config,
            format!(
                "Failed to parse environment variable: {} with value: {}",
                env_var, value
            ),
            None,
        );
        AppError::from(infra_err)
    })
}

pub fn get_env_string(env_var: &str) -> AppResult<String> {
    std::env::var(env_var).map_err(|e| {
        let infra_err: InfrastructureError = InfrastructureError::new(
            InfrastructureErrorKind::Config,
            format!("Failed to read environment variable: {}", env_var),
            Some(Box::new(e)),
        );
        AppError::from(infra_err)
    })
}

pub fn get_env_string_with_test(
    env_var: &str,
    test_env_var: &str,
    is_test: bool,
) -> AppResult<String> {
    let target_var = if is_test { test_env_var } else { env_var };
    std::env::var(target_var).map_err(|e| {
        let infra_err: InfrastructureError = InfrastructureError::new(
            InfrastructureErrorKind::Config,
            format!("Failed to read environment variable: {}", target_var),
            Some(Box::new(e)),
        );
        AppError::from(infra_err)
    })
}
