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

/// A macro for error handling with automatic retries.
///
/// Attempts to execute an operation multiple times with exponential backoff
/// before giving up. Logs each retry attempt and final failure if all attempts fail.
///
/// # Arguments
/// * `$expr` - The expression that may return a Result
/// * `max_attempts` - (Optional) Maximum number of retry attempts (default: 3)
/// * `$context` - (Optional) A string message providing context about the error
/// * `$field = $value` - (Optional) Additional key-value pairs for span fields
///
/// # Examples
///
/// ```rust
/// // Simple usage with default 3 retries
/// let result = try_with_retry!(api_call());
///
/// // Specify max attempts
/// let result = try_with_retry!(api_call(), max_attempts = 5);
///
/// // With context and custom attempts
/// let result = try_with_retry!(
///     api_call(),
///     max_attempts = 5,
///     "API call failed",
///     endpoint = "/users",
///     payload_size = size
/// );
/// ```
///
/// # Behavior
/// - Uses exponential backoff between retries
/// - Logs each retry attempt with increasing attempt number
/// - Converts final error to AppError if all attempts fail
#[macro_export]
macro_rules! try_with_retry {
    // Basic case: just the expression
    ($expr:expr) => {
        try_with_retry!($expr, max_attempts = 3)
    };

    // With max attempts
    ($expr:expr, max_attempts = $max:expr) => {
        try_with_retry!($expr, max_attempts = $max, context = "Operation failed")
    };

    // With max attempts and context
    ($expr:expr, max_attempts = $max:expr, context = $context:expr) => {{
        let mut attempts = 0;
        loop {
            match $expr {
                Ok(val) => break Ok(val),
                Err(e) => {
                    let err: $crate::infrastructure::AppError = e.into();
                    attempts += 1;

                    if attempts >= $max {
                        tracing::error!(
                            error = %err,
                            context = $context,
                            attempts = attempts,
                            "Max retry attempts reached"
                        );
                        break Err(err);
                    }

                    if let Some(duration) = err.retry_after() {
                        tracing::warn!(
                            error = %err,
                            context = $context,
                            retry_after = ?duration,
                            attempts = attempts,
                            "Rate limited, waiting before retry"
                        );
                        tokio::time::sleep(duration).await;
                        continue;
                    }

                    if err.is_retryable() {
                        let backoff = std::time::Duration::from_secs(2u64.pow(attempts));
                        tracing::warn!(
                            error = %err,
                            context = $context,
                            backoff = ?backoff,
                            attempts = attempts,
                            "Retryable error, using exponential backoff"
                        );
                        tokio::time::sleep(backoff).await;
                        continue;
                    }

                    tracing::error!(
                        error = %err,
                        context = $context,
                        "Non-retryable error occurred"
                    );
                    break Err(err);
                }
            }
        }
    }};

    // With max attempts, context and additional fields
    ($expr:expr, max_attempts = $max:expr, context = $context:expr, $($field:tt = $value:expr),+) => {{
        let mut attempts = 0;
        loop {
            match $expr {
                Ok(val) => break Ok(val),
                Err(e) => {
                    let err: $crate::infrastructure::AppError = e.into();
                    attempts += 1;

                    if attempts >= $max {
                        tracing::error!(
                            error = %err,
                            context = $context,
                            attempts = attempts,
                            $($field = ?$value,)*
                            "Max retry attempts reached"
                        );
                        break Err(err);
                    }

                    if let Some(duration) = err.retry_after() {
                        tracing::warn!(
                            error = %err,
                            context = $context,
                            retry_after = ?duration,
                            attempts = attempts,
                            $($field = ?$value,)*
                            "Rate limited, waiting before retry"
                        );
                        tokio::time::sleep(duration).await;
                        continue;
                    }

                    if err.is_retryable() {
                        let backoff = std::time::Duration::from_secs(2u64.pow(attempts));
                        tracing::warn!(
                            error = %err,
                            context = $context,
                            backoff = ?backoff,
                            attempts = attempts,
                            $($field = ?$value,)*
                            "Retryable error, using exponential backoff"
                        );
                        tokio::time::sleep(backoff).await;
                        continue;
                    }

                    tracing::error!(
                        error = %err,
                        context = $context,
                        $($field = ?$value,)*
                        "Non-retryable error occurred"
                    );
                    break Err(err);
                }
            }
        }
    }};
}
