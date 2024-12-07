#[macro_export]
macro_rules! config_set_env {
    ($settings:expr, $env:literal, $target:expr) => {
        match $crate::infrastructure::config::utils::parse_env($env) {
            Ok(value) => $target = value,
            Err(e) => return Err(e),
        }
    };
}

#[macro_export]
macro_rules! config_set_string {
    ($settings:expr, $env:literal, $target:expr) => {
        match $crate::infrastructure::config::utils::get_env_string($env) {
            Ok(value) => $target = value,
            Err(e) => return Err(e),
        }
    };
}

#[macro_export]
macro_rules! config_validate {
    ($cond:expr, $msg:expr) => {
        if !$cond {
            return Err($crate::infrastructure::AppError::Infrastructure(
                $crate::infrastructure::InfrastructureError::new(
                    $crate::infrastructure::InfrastructureErrorKind::Config,
                    $msg,
                    None,
                ),
            ));
        }
    };
}
