//! Multiplatform surface: OS detection, backend URL defaults, and
//! platform-safe paths. One codebase for desktop (Windows/Linux/macOS),
//! Android (`cargo-apk` + `eframe` on `winit`) and iOS
//! (`cargo-xcodebuild`), with WebAssembly kept as a stub target.
//!
//! Design rule: everything platform-specific lives here behind `cfg!()` /
//! `std::env::consts::OS`, so `app`, `api`, `map` and `osm` stay portable.
//! No platform code touches the UI tree directly.

/// Running operating system (compile-time).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Windows,
    Linux,
    MacOs,
    Android,
    Ios,
    Web,
    Unknown,
}

impl Platform {
    pub fn current() -> Self {
        match std::env::consts::OS {
            "windows" => Self::Windows,
            "linux" => {
                // Android reports "android" via `cfg(target_os)` but
                // `std::env::consts::OS` is also "linux" on older toolchains:
                // prefer the cfg check.
                if cfg!(target_os = "android") {
                    Self::Android
                } else {
                    Self::Linux
                }
            }
            "macos" => Self::MacOs,
            "ios" => Self::Ios,
            "wasm" => Self::Web,
            _ => {
                if cfg!(target_os = "android") {
                    Self::Android
                } else if cfg!(target_os = "ios") {
                    Self::Ios
                } else if cfg!(target_arch = "wasm32") {
                    Self::Web
                } else {
                    Self::Unknown
                }
            }
        }
    }

    pub fn is_mobile(self) -> bool {
        matches!(self, Self::Android | Self::Ios)
    }

    pub fn needs_loopback_http_exception(self) -> bool {
        // ADR-0006: plain HTTP is only allowed for loopback backend URLs.
        // Mobile emulators reach the host via 10.0.2.2, which is also
        // loopback-adjacent for development — but releases are forced HTTPS.
        matches!(
            self,
            Self::Windows | Self::Linux | Self::MacOs | Self::Android
        )
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Windows => "Windows",
            Self::Linux => "Linux",
            Self::MacOs => "macOS",
            Self::Android => "Android",
            Self::Ios => "iOS",
            Self::Web => "Web (wasm)",
            Self::Unknown => "Unknown",
        }
    }
}

/// Default backend URL for local development on each platform.
/// Production builds MUST be overridden with `--backend https://…`.
pub fn default_backend_url() -> String {
    match Platform::current() {
        Platform::Android => {
            // Android emulator routes host loopback through 10.0.2.2.
            "http://10.0.2.2:8000".to_string()
        }
        _ => "http://localhost:8000".to_string(),
    }
}

/// Parent directory that is writable on every target (bookmarks, offline
/// packs, logs). Returns `None` on targets without a filesystem
/// (wasm / iOS app sandboxes) — callers must degrade gracefully.
pub fn app_data_dir() -> Option<std::path::PathBuf> {
    match Platform::current() {
        Platform::Android | Platform::Ios | Platform::Web => None,
        _ => home_dir().map(|h| h.join(".pinpoint")),
    }
}

fn home_dir() -> Option<std::path::PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .or_else(|| {
            std::env::var_os("HOMEDRIVE")
                .zip(std::env::var_os("HOMEPATH"))
                .map(|(d, p)| {
                    std::ffi::OsString::from(format!(
                        "{}{}",
                        d.to_string_lossy(),
                        p.to_string_lossy()
                    ))
                })
        })
        .map(std::path::PathBuf::from)
}

/// Human-readable build identity for About screens / diagnostics.
pub fn build_identity() -> String {
    format!(
        "Pinpoint {} on {} / {}",
        env!("CARGO_PKG_VERSION"),
        Platform::current().label(),
        std::env::consts::ARCH
    )
}

#[cfg(test)]
mod tests {
    use super::{default_backend_url, Platform};

    #[test]
    fn platform_current_never_panics() {
        let _ = Platform::current();
    }

    #[test]
    fn default_backend_is_http_loopback_for_dev() {
        let url = default_backend_url();
        assert!(
            url.starts_with("http://"),
            "dev default must be http: {url}"
        );
    }
}
