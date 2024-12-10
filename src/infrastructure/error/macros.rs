/// A macro for error handling with automatic logging at the INFO level.
///
/// This macro combines error conversion and logging in a concise way.
/// It supports optional context message and span information.
///
/// # Arguments
/// * `$expr` - The expression that may return a Result
/// * `$context` - (Optional) A string message providing context about the error
/// * `$field = $value` - (Optional) Additional key-value pairs for span fields
///
/// # Examples
///
/// ```rust
/// // Simple usage
/// let result = try_with_log!(fetch_data());
///
/// // With context message
/// let result = try_with_log!(fetch_data(), "Failed to fetch user data");
///
/// // With context and span fields
/// let result = try_with_log!(
///     fetch_data(),
///     "Failed to fetch user data",
///     user_id = user.id,
///     attempt = retry_count
/// );
/// ```
#[macro_export]
macro_rules! try_with_log {
    // Basic case: just the expression
    ($expr:expr) => {
        match $expr {
            Ok(val) => val,
            Err(e) => {
                let err: $crate::infrastructure::AppError = e.into();
                tracing::info!(error = %err, "Operation failed");
                return Err(err);
            }
        }
    };

    // With context message
    ($expr:expr, $context:expr) => {
        match $expr {
            Ok(val) => val,
            Err(e) => {
                let mut err: $crate::infrastructure::AppError = e.into();
                err.set_context($context.into());
                tracing::info!(
                    error = %err,
                    context = $context,
                    "Operation failed"
                );
                return Err(err);
            }
        }
    };

    // With context message and additional span fields
    ($expr:expr, $context:expr, $($field:tt = $value:expr),+) => {
        match $expr {
            Ok(val) => val,
            Err(e) => {
                let mut err: $crate::infrastructure::AppError = e.into();
                err.set_context($context.into());
                tracing::info!(
                    error = %err,
                    context = $context,
                    $($field = ?$value,)*
                    "Operation failed"
                );
                return Err(err);
            }
        }
    };
}

/// A macro for error handling with warning level logging.
///
/// Similar to `try_with_log!` but logs at WARN level instead of INFO.
/// Useful for handling non-critical errors that should be monitored.
///
/// # Arguments
/// * `$expr` - The expression that may return a Result
/// * `$context` - (Optional) A string message providing context about the error
/// * `$field = $value` - (Optional) Additional key-value pairs for span fields
///
/// # Examples
///
/// ```rust
/// // Simple usage
/// let result = try_with_warn!(parse_config());
///
/// // With context
/// let result = try_with_warn!(
///     parse_config(),
///     "Config parsing failed, using defaults"
/// );
///
/// // With context and span fields
/// let result = try_with_warn!(
///     parse_config(),
///     "Config parsing failed",
///     config_path = path,
///     fallback = true
/// );
/// ```
#[macro_export]
macro_rules! try_with_warn {
    ($expr:expr) => {
        match $expr {
            Ok(val) => val,
            Err(e) => {
                let err: $crate::infrastructure::AppError = e.into();
                tracing::warn!(error = ?err, "Operation failed");
                return Err(err);
            }
        }
    };

    ($expr:expr, $context:expr) => {
        match $expr {
            Ok(val) => val,
            Err(e) => {
                let mut err: $crate::infrastructure::AppError = e.into();
                err.set_context($context.into());
                tracing::warn!(
                    error = ?err,
                    context = $context,
                    "Operation failed"
                );
                return Err(err);
            }
        }
    };

    ($expr:expr, $context:expr, $($field:tt = $value:expr),+) => {
        match $expr {
            Ok(val) => val,
            Err(e) => {
                let mut err: $crate::infrastructure::AppError = e.into();
                err.set_context($context.into());
                tracing::warn!(
                    error = ?err,
                    context = $context,
                    $($field = ?$value,)*
                    "Operation failed"
                );
                return Err(err);
            }
        }
    };
}

/// A macro for error handling with debug level logging.
///
/// Similar to `try_with_log!` but logs at DEBUG level.
/// Useful for detailed troubleshooting and development.
///
/// # Arguments
/// * `$expr` - The expression that may return a Result
/// * `$context` - (Optional) A string message providing context about the error
/// * `$field = $value` - (Optional) Additional key-value pairs for span fields
///
/// # Examples
///
/// ```rust
/// // Simple usage
/// let result = try_with_debug!(validate_input());
///
/// // With context
/// let result = try_with_debug!(
///     validate_input(),
///     "Input validation failed"
/// );
///
/// // With context and span fields
/// let result = try_with_debug!(
///     validate_input(),
///     "Input validation failed",
///     input_length = input.len(),
///     input_type = typename
/// );
/// ```
#[macro_export]
macro_rules! try_with_debug {
    ($expr:expr) => {
        match $expr {
            Ok(val) => val,
            Err(e) => {
                let err: $crate::infrastructure::AppError = e.into();
                tracing::debug!(error = ?err, "Operation failed");
                return Err(err);
            }
        }
    };

    ($expr:expr, $context:expr) => {
        match $expr {
            Ok(val) => val,
            Err(e) => {
                let mut err: $crate::infrastructure::AppError = e.into();
                err.set_context($context.into());
                tracing::debug!(
                    error = ?err,
                    context = $context,
                    "Operation failed"
                );
                return Err(err);
            }
        }
    };

    ($expr:expr, $context:expr, $($field:tt = $value:expr),+) => {
        match $expr {
            Ok(val) => val,
            Err(e) => {
                let mut err: $crate::infrastructure::AppError = e.into();
                err.set_context($context.into());
                tracing::debug!(
                    error = ?err,
                    context = $context,
                    $($field = ?$value,)*
                    "Operation failed"
                );
                return Err(err);
            }
        }
    };
}

/// A macro for retrying asynchronous operations with configurable retry attempts
///
/// # Arguments
/// * `$expr` - The async expression that may return a Result
/// * `max_attempts` - Maximum number of retry attempts
/// * `context` - (Optional) A context message for error logging
///
/// # Examples
///
/// ```rust
/// // Basic retry
/// let result = try_with_retry!(fetch_data(), max_attempts = 3);
///
/// // With context
/// let result = try_with_retry!(
///     fetch_data(),
///     max_attempts = 3,
///     context = "Failed to fetch data"
/// );
/// ```
#[macro_export]
macro_rules! try_with_retry {
    // Async retry without context
    ($expr:expr, max_attempts = $max:expr) => {{
        use tokio::time::{sleep, Duration};

        let result = async {
            let mut last_error = None;

            for attempt in 0..$max {
                match $expr {
                    Ok(val) => return Ok(val),
                    Err(e) => {
                        last_error = Some(e);
                        if attempt < $max - 1 {
                            sleep(Duration::from_millis(100 * (attempt + 1) as u64)).await;
                        }
                    }
                }
            }

            Err(last_error.unwrap())
        }.await;

        result
    }};

    // Async retry with context
    ($expr:expr, max_attempts = $max:expr, context = $context:expr) => {{
        use tokio::time::{sleep, Duration};

        let result = async {
            let mut last_error = None;

            for attempt in 0..$max {
                match $expr {
                    Ok(val) => return Ok(val),
                    Err(e) => {
                        let mut err: $crate::infrastructure::error::AppError = e.into();
                        err.set_context($context.into());
                        tracing::info!(
                            error = %err,
                            context = $context,
                            attempt = attempt + 1,
                            "Retry operation failed"
                        );
                        last_error = Some(err);

                        if attempt < $max - 1 {
                            sleep(Duration::from_millis(100 * (attempt + 1) as u64)).await;
                        }
                    }
                }
            }

            Err(last_error.unwrap())
        }.await;

        result
    }};
}
