#![cfg_attr(docsrs, feature(doc_cfg))]

// =============std==================
/// 网络库相关
#[cfg(feature = "net")]
#[cfg_attr(docsrs, doc(cfg(feature = "net")))]
pub mod net;

/// 指针包装相关
#[cfg(feature = "ptr")]
#[cfg_attr(docsrs, doc(cfg(feature = "ptr")))]
pub mod ptr;

/// 提供线程池
#[cfg(feature = "thread")]
#[cfg_attr(docsrs, doc(cfg(feature = "thread")))]
pub mod thread;
// =============std==================
// =============3rd==================
/// 加密库相关
#[cfg(feature = "crypto")]
#[cfg_attr(docsrs, doc(cfg(feature = "crypto")))]
pub mod crypto;

/// 线性代数相关
#[cfg(feature = "linear")]
#[cfg_attr(docsrs, doc(cfg(feature = "linear")))]
pub mod linear;

/// 日志库
#[cfg(feature = "log")]
#[cfg_attr(docsrs, doc(cfg(feature = "log")))]
pub mod log;
// =============3rd==================