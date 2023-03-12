/// 自制的简单日志库，只用到了 `log` 和 `chrono`
#[cfg(feature = "chrono")]
#[cfg_attr(docsrs, doc(cfg(feature = "chrono")))]
pub mod slog;