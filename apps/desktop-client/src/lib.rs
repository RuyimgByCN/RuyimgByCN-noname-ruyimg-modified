//! Desktop shell host bridge for noname-next.
//!
//! Phase 10: stable API surface exposed by the Tauri 2 shell to
//! the Web UI. Content packages never receive raw filesystem or network access —
//! only these capability-gated operations.

#![deny(missing_docs)]

use content_runtime::{
    import_extensions, ActiveCatalogPage, ContentLock, ExtensionImportReport, InstallOptions,
    InstallResult, PackageSignature, PackageStore, RuntimeError,
};
use diagnostics::{DiagnosticBuilder, DiagnosticPack, FaultCode};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Shell host error.
#[derive(Debug, Error)]
pub enum ShellError {
    /// Content runtime.
    #[error(transparent)]
    Runtime(#[from] RuntimeError),
    /// Diagnostics.
    #[error("diag: {0}")]
    Diag(String),
    /// Policy denial.
    #[error("denied: {0}")]
    Denied(String),
}

/// Capability flags the shell grants to the UI layer (never to content packages).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellCapabilities {
    /// Install / activate content packages.
    pub manage_content: bool,
    /// Export redacted diagnostics.
    pub export_diagnostics: bool,
    /// Allow unsigned packages (dev builds only).
    pub allow_unsigned_packages: bool,
}

impl Default for ShellCapabilities {
    fn default() -> Self {
        Self {
            manage_content: true,
            export_diagnostics: true,
            allow_unsigned_packages: cfg!(debug_assertions),
        }
    }
}

/// Platform label for artifact retention.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformInfo {
    /// OS family: macos / windows / linux / android / ios / web.
    pub os: String,
    /// Arch: aarch64 / x86_64 / …
    pub arch: String,
    /// Shell id: tauri2 / webview / headless.
    pub shell: String,
}

/// Canonical active lock plus dependency-ordered original package JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentBundle {
    /// Canonical active content lock.
    pub content_lock: ContentLock,
    /// Installed package JSON in dependency order.
    pub packages: Vec<String>,
}

impl PlatformInfo {
    /// Detect best-effort from std (desktop).
    pub fn detect_desktop() -> Self {
        Self::detect_desktop_for_shell("host-bridge")
    }

    /// Detect the desktop platform and record a concrete shell id.
    pub fn detect_desktop_for_shell(shell: impl Into<String>) -> Self {
        Self {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            shell: shell.into(),
        }
    }

    /// Combined platform key.
    pub fn platform_key(&self) -> String {
        format!("{}-{}-{}", self.os, self.arch, self.shell)
    }
}

/// In-process desktop host: package store + diagnostics under a data directory.
pub struct DesktopHost {
    store: PackageStore,
    caps: ShellCapabilities,
    platform: PlatformInfo,
    /// When true, content lock is frozen (live match) — no activate/install.
    match_frozen: bool,
}

impl DesktopHost {
    /// Open host data dir (`…/content` used as package store root).
    pub fn open(data_dir: impl AsRef<Path>, caps: ShellCapabilities) -> Result<Self, ShellError> {
        Self::open_for_shell(data_dir, caps, "host-bridge")
    }

    /// Open the host for a concrete shell implementation such as `tauri2`.
    pub fn open_for_shell(
        data_dir: impl AsRef<Path>,
        caps: ShellCapabilities,
        shell: impl Into<String>,
    ) -> Result<Self, ShellError> {
        let data_dir = data_dir.as_ref();
        let root = data_dir.join("content");
        let store = PackageStore::open(root)?;
        std::fs::create_dir_all(data_dir.join("extensions")).map_err(RuntimeError::from)?;
        import_extensions(
            &store,
            &data_dir.join("extensions"),
            &InstallOptions {
                activate: true,
                allow_unsigned: caps.allow_unsigned_packages,
                verify_key: std::env::var("CONTENT_VERIFY_KEY").ok(),
                host: content_runtime::tooling_host_versions(),
            },
        )?;
        Ok(Self {
            store,
            caps,
            platform: PlatformInfo::detect_desktop_for_shell(shell),
            match_frozen: false,
        })
    }

    /// Package store.
    pub fn store(&self) -> &PackageStore {
        &self.store
    }

    /// Platform info.
    pub fn platform(&self) -> &PlatformInfo {
        &self.platform
    }

    /// Freeze content changes for the duration of a live match.
    pub fn freeze_for_match(&mut self) {
        self.match_frozen = true;
    }

    /// Unfreeze after match ends (does not auto-upgrade mid-match).
    pub fn unfreeze_after_match(&mut self) {
        self.match_frozen = false;
    }

    /// Whether content mutations are blocked.
    pub fn is_match_frozen(&self) -> bool {
        self.match_frozen
    }

    fn ensure_manage(&self) -> Result<(), ShellError> {
        if !self.caps.manage_content {
            return Err(ShellError::Denied("manage_content".into()));
        }
        if self.match_frozen {
            return Err(ShellError::Denied(
                "content frozen during live match".into(),
            ));
        }
        Ok(())
    }

    /// Install package (shell command target).
    pub fn install_package(
        &self,
        package_bytes: &[u8],
        signature: Option<&PackageSignature>,
        verify_key: Option<String>,
        activate: bool,
    ) -> Result<InstallResult, ShellError> {
        self.ensure_manage()?;
        let opts = InstallOptions {
            activate,
            allow_unsigned: self.caps.allow_unsigned_packages && signature.is_none(),
            verify_key,
            host: content_runtime::tooling_host_versions(),
        };
        Ok(self.store.install(package_bytes, signature, &opts)?)
    }

    /// Rescan a drop-in extension directory and atomically activate valid packages.
    pub fn import_extension_directory(
        &self,
        root: &Path,
        verify_key: Option<String>,
    ) -> Result<ExtensionImportReport, ShellError> {
        self.ensure_manage()?;
        Ok(import_extensions(
            &self.store,
            root,
            &InstallOptions {
                activate: true,
                allow_unsigned: self.caps.allow_unsigned_packages,
                verify_key,
                host: content_runtime::tooling_host_versions(),
            },
        )?)
    }

    /// Resolve a verified asset path for an active extension package.
    pub fn resolve_active_asset(
        &self,
        namespace: &str,
        package_id: &str,
        asset_id: &str,
    ) -> Result<PathBuf, ShellError> {
        Ok(self
            .store
            .resolve_active_asset(namespace, package_id, asset_id)?)
    }

    /// Return one bounded page of the active presentation catalog.
    pub fn content_catalog_page(
        &self,
        offset: Option<u32>,
        limit: Option<u32>,
    ) -> Result<ActiveCatalogPage, ShellError> {
        let (lock, catalog) = self.store.load_active_catalog()?;
        Ok(catalog.page(lock.lock_hash, offset, limit))
    }

    /// Return the canonical lock and dependency-ordered original package JSON.
    pub fn content_bundle(&self) -> Result<ContentBundle, ShellError> {
        let (content_lock, packages) = self.store.load_active_packages()?;
        let packages = packages
            .iter()
            .map(|package| {
                self.store
                    .load_active_package_bytes(
                        &package.manifest.namespace,
                        &package.manifest.package_id,
                    )
                    .and_then(|bytes| {
                        String::from_utf8(bytes).map_err(|_| {
                            RuntimeError::Invalid("installed package JSON is not UTF-8".into())
                        })
                    })
            })
            .collect::<Result<_, _>>()?;
        Ok(ContentBundle {
            content_lock,
            packages,
        })
    }

    /// Resolve an active asset only when it is inside the app-data content root.
    pub fn resolve_content_asset(
        &self,
        namespace: &str,
        package_id: &str,
        asset_id: &str,
    ) -> Result<PathBuf, ShellError> {
        let root = self
            .store
            .root()
            .canonicalize()
            .map_err(RuntimeError::from)?;
        let path = self.resolve_active_asset(namespace, package_id, asset_id)?;
        if !path.starts_with(&root) {
            return Err(ShellError::Denied(
                "asset is outside the app-data content root".into(),
            ));
        }
        Ok(path)
    }

    /// Rollback active set.
    pub fn rollback(&self) -> Result<(), ShellError> {
        self.ensure_manage()?;
        self.store.rollback()?;
        Ok(())
    }

    /// Content lock hash for UI / match start.
    pub fn content_lock_hash(&self) -> Result<String, ShellError> {
        Ok(self.store.current_lock()?.lock_hash)
    }

    /// Build redacted diagnostic pack.
    pub fn build_diagnostic(
        &self,
        fault: FaultCode,
        message: &str,
        match_id: Option<&str>,
        revision: Option<u64>,
    ) -> Result<DiagnosticPack, ShellError> {
        if !self.caps.export_diagnostics {
            return Err(ShellError::Denied("export_diagnostics".into()));
        }
        let mut b = DiagnosticBuilder::new()
            .fault(fault)
            .message(message)
            .engine_build_id(engine_api::ENGINE_BUILD_ID)
            .content_lock_hash(
                self.store
                    .current_lock()
                    .map(|l| l.lock_hash)
                    .unwrap_or_default(),
            );
        if let Some(id) = match_id {
            b = b.match_id(id);
        }
        if let Some(r) = revision {
            b = b.current_revision(r);
        }
        b.note(format!("platform={}", self.platform.platform_key()))
            .build()
            .map_err(|e| ShellError::Diag(e.to_string()))
    }

    /// Default data directory suggestion (not created).
    pub fn default_data_dir() -> PathBuf {
        dirs_hint().join("noname-next")
    }
}

fn dirs_hint() -> PathBuf {
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home).join("Library/Application Support");
    }
    PathBuf::from(".")
}

/// Mobile platform shell adaptation (architecture §29.2).
///
/// Android / iOS: reuse [`DesktopHost`] API with a platform-specific data
/// directory and WebView UI. WASM Worker support is shell validation, not a
/// second rules engine. This module defines:
///
/// - Safe area insets (notch / home indicator)
/// - Touch target sizing (§30 accessibility: minimum 44pt)
/// - WebView performance verification checklist
/// - Background / foreground lifecycle policy
/// - Platform capability detection
pub mod mobile {
    use serde::{Deserialize, Serialize};

    /// Placeholder platform id for mobile shells.
    pub const SHELL_ID: &str = "mobile-webview";

    /// Minimum touch target size in CSS pixels (§30 accessibility: 44pt).
    pub const MIN_TOUCH_TARGET_PX: u32 = 44;

    // ---- Safe Area (§29.2: 触摸、刘海和安全区域) ----

    /// Safe area insets for notch / home indicator / rounded corners.
    ///
    /// The WebView reports these as CSS `env(safe-area-inset-*)` values.
    /// The UI layer applies them as padding to avoid clipping content under
    /// notches, home indicators, or rounded corners.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
    pub struct SafeAreaInsets {
        /// Top inset (notch / status bar).
        pub top: u32,
        /// Bottom inset (home indicator).
        pub bottom: u32,
        /// Left inset (landscape notch).
        pub left: u32,
        /// Right inset (landscape notch).
        pub right: u32,
    }

    impl SafeAreaInsets {
        /// No insets (desktop / no-notch device).
        pub const NONE: Self = Self {
            top: 0,
            bottom: 0,
            left: 0,
            right: 0,
        };

        /// Typical iPhone with Dynamic Island / notch (portrait).
        pub const IPHONE_NOTCH: Self = Self {
            top: 59,
            bottom: 34,
            left: 0,
            right: 0,
        };

        /// Typical Android with punch-hole camera (portrait).
        pub const ANDROID_PUNCH_HOLE: Self = Self {
            top: 48,
            bottom: 24,
            left: 0,
            right: 0,
        };

        /// Render as CSS `env()` values for the UI.
        pub fn to_css_vars(&self) -> String {
            format!(
                "--safe-area-top: {}px; --safe-area-bottom: {}px; --safe-area-left: {}px; --safe-area-right: {}px;",
                self.top, self.bottom, self.left, self.right
            )
        }
    }

    // ---- Touch Target (§29.2: 触摸 + §30: 可访问性) ----

    /// Touch target size recommendation.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub struct TouchTarget {
        /// Width in CSS pixels.
        pub width: u32,
        /// Height in CSS pixels.
        pub height: u32,
    }

    impl TouchTarget {
        /// Minimum accessible touch target (44×44pt per §30).
        pub const MIN: Self = Self {
            width: MIN_TOUCH_TARGET_PX,
            height: MIN_TOUCH_TARGET_PX,
        };

        /// Comfortable touch target (48×48pt).
        pub const COMFORTABLE: Self = Self {
            width: 48,
            height: 48,
        };

        /// Card tap target (72×100 for card images).
        pub const CARD: Self = Self {
            width: 72,
            height: 100,
        };

        /// Whether this target meets the minimum size.
        pub fn meets_minimum(&self) -> bool {
            self.width >= MIN_TOUCH_TARGET_PX && self.height >= MIN_TOUCH_TARGET_PX
        }
    }

    // ---- WebView Performance Verification (§29.2) ----

    /// One item in the mobile verification checklist (§29.2).
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct VerificationItem {
        /// Item id.
        pub id: String,
        /// Human-readable description.
        pub description: String,
        /// Whether this item passed verification.
        pub passed: Option<bool>,
        /// Notes (failure reason, measurements, etc.).
        #[serde(default)]
        pub notes: Option<String>,
    }

    /// Full mobile platform verification checklist (§29.2).
    #[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
    pub struct MobileVerificationChecklist {
        /// Items.
        pub items: Vec<VerificationItem>,
    }

    impl MobileVerificationChecklist {
        /// Create the standard §29.2 checklist for a platform.
        pub fn for_platform(platform: MobilePlatform) -> Self {
            let items = match platform {
                MobilePlatform::Android => vec![
                    VerificationItem {
                        id: "android-webview-perf".into(),
                        description:
                            "Android WebView renders battle UI at ≥30fps on mid-range device".into(),
                        passed: None,
                        notes: None,
                    },
                    VerificationItem {
                        id: "android-wasm-worker".into(),
                        description: "WASM Worker spawns and communicates via postMessage".into(),
                        passed: None,
                        notes: None,
                    },
                    VerificationItem {
                        id: "android-background-recovery".into(),
                        description: "App restores match state after background→foreground".into(),
                        passed: None,
                        notes: None,
                    },
                    VerificationItem {
                        id: "android-touch-targets".into(),
                        description: "All interactive elements ≥44px touch targets".into(),
                        passed: None,
                        notes: None,
                    },
                    VerificationItem {
                        id: "android-safe-area".into(),
                        description:
                            "Content not clipped under punch-hole camera or navigation bar".into(),
                        passed: None,
                        notes: None,
                    },
                    VerificationItem {
                        id: "android-low-end-textures".into(),
                        description: "Texture budget ≤128MB on 2GB RAM devices".into(),
                        passed: None,
                        notes: None,
                    },
                ],
                MobilePlatform::Ios => vec![
                    VerificationItem {
                        id: "ios-webview-memory".into(),
                        description: "iOS WKWebView stays under 512MB memory pressure limit".into(),
                        passed: None,
                        notes: None,
                    },
                    VerificationItem {
                        id: "ios-wasm-worker".into(),
                        description: "WASM Worker spawns in Safari WKWebView".into(),
                        passed: None,
                        notes: None,
                    },
                    VerificationItem {
                        id: "ios-background-recovery".into(),
                        description: "App restores match state after background→foreground".into(),
                        passed: None,
                        notes: None,
                    },
                    VerificationItem {
                        id: "ios-touch-targets".into(),
                        description: "All interactive elements ≥44pt touch targets".into(),
                        passed: None,
                        notes: None,
                    },
                    VerificationItem {
                        id: "ios-notch-safe-area".into(),
                        description: "Content not clipped under Dynamic Island / notch".into(),
                        passed: None,
                        notes: None,
                    },
                    VerificationItem {
                        id: "ios-low-memory-warning".into(),
                        description:
                            "App responds to UIApplication memory warning by flushing cache".into(),
                        passed: None,
                        notes: None,
                    },
                ],
            };
            Self { items }
        }

        /// Mark an item as passed or failed.
        pub fn set_result(&mut self, id: &str, passed: bool, notes: Option<String>) {
            if let Some(item) = self.items.iter_mut().find(|i| i.id == id) {
                item.passed = Some(passed);
                item.notes = notes;
            }
        }

        /// Whether all items have been verified and passed.
        pub fn all_passed(&self) -> bool {
            !self.items.is_empty() && self.items.iter().all(|i| i.passed == Some(true))
        }

        /// Whether any item failed.
        pub fn has_failures(&self) -> bool {
            self.items.iter().any(|i| i.passed == Some(false))
        }

        /// Render as text report.
        pub fn render(&self) -> String {
            let mut out = String::new();
            for item in &self.items {
                let status = match item.passed {
                    Some(true) => "✓",
                    Some(false) => "✗",
                    None => "?",
                };
                out.push_str(&format!(
                    "  {status} {:<30} {}\n",
                    item.id, item.description
                ));
                if let Some(ref notes) = item.notes {
                    out.push_str(&format!("     notes: {notes}\n"));
                }
            }
            if self.all_passed() {
                out.push_str("result=PASS\n");
            } else if self.has_failures() {
                out.push_str("result=FAIL\n");
            } else {
                out.push_str("result=PENDING\n");
            }
            out
        }
    }

    // ---- Background / Foreground Lifecycle (§29.2: 后台与恢复) ----

    /// Background lifecycle policy (§29.2: 移动端后台恢复).
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    #[repr(u8)]
    pub enum BackgroundPolicy {
        /// App is in foreground — full resources.
        Foreground = 0,
        /// App went to background — pause animations, flush state.
        Background = 1,
        /// App received low-memory warning — flush caches, release textures.
        LowMemory = 2,
        /// App is being terminated — persist state urgently.
        Terminating = 3,
    }

    impl BackgroundPolicy {
        /// What the app should do on this transition.
        pub fn actions(&self) -> &'static [&'static str] {
            match self {
                BackgroundPolicy::Foreground => &["resume animations", "reload textures"],
                BackgroundPolicy::Background => &[
                    "pause animations",
                    "flush pending state to WAL",
                    "release non-critical textures",
                ],
                BackgroundPolicy::LowMemory => &[
                    "flush all caches",
                    "release all textures",
                    "reduce DOM nodes to minimum",
                    "switch to low-memory resource budget",
                ],
                BackgroundPolicy::Terminating => {
                    &["persist match state to WAL immediately", "save settings"]
                }
            }
        }
    }

    // ---- Platform Detection ----

    /// Mobile platform identifier.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    #[repr(u8)]
    pub enum MobilePlatform {
        /// Android.
        Android = 0,
        /// iOS / iPadOS.
        Ios = 1,
    }

    impl MobilePlatform {
        /// Label.
        pub fn label(self) -> &'static str {
            match self {
                MobilePlatform::Android => "android",
                MobilePlatform::Ios => "ios",
            }
        }

        /// Default safe area insets for this platform.
        pub fn default_safe_area(self) -> SafeAreaInsets {
            match self {
                MobilePlatform::Android => SafeAreaInsets::ANDROID_PUNCH_HOLE,
                MobilePlatform::Ios => SafeAreaInsets::IPHONE_NOTCH,
            }
        }

        /// Default memory budget (MB) for this platform.
        pub fn default_memory_budget_mb(self) -> u32 {
            match self {
                MobilePlatform::Android => 256,
                MobilePlatform::Ios => 512,
            }
        }

        /// Detect platform from user agent string (best-effort).
        pub fn from_user_agent(ua: &str) -> Option<Self> {
            if ua.contains("Android") {
                Some(MobilePlatform::Android)
            } else if ua.contains("iPhone") || ua.contains("iPad") || ua.contains("iOS") {
                Some(MobilePlatform::Ios)
            } else {
                None
            }
        }
    }

    /// Mobile shell configuration.
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct MobileShellConfig {
        /// Target platform.
        pub platform: MobilePlatform,
        /// Safe area insets (reported by WebView).
        pub safe_area: SafeAreaInsets,
        /// Whether the WebView supports WASM Workers.
        pub wasm_worker_supported: bool,
        /// Memory budget in MB.
        pub memory_budget_mb: u32,
        /// Whether background recovery is enabled.
        pub background_recovery: bool,
    }

    impl MobileShellConfig {
        /// Default config for a platform.
        pub fn for_platform(platform: MobilePlatform) -> Self {
            Self {
                platform,
                safe_area: platform.default_safe_area(),
                wasm_worker_supported: true,
                memory_budget_mb: platform.default_memory_budget_mb(),
                background_recovery: true,
            }
        }

        /// Generate the CSS variables for the WebView.
        pub fn css_vars(&self) -> String {
            format!(
                "{} --touch-min: {}px; --mem-budget: {}MB;",
                self.safe_area.to_css_vars(),
                MIN_TOUCH_TARGET_PX,
                self.memory_budget_mb
            )
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn safe_area_css_vars() {
            let sa = SafeAreaInsets::IPHONE_NOTCH;
            let css = sa.to_css_vars();
            assert!(css.contains("--safe-area-top: 59px"));
            assert!(css.contains("--safe-area-bottom: 34px"));
        }

        #[test]
        fn touch_target_minimum() {
            assert!(TouchTarget::MIN.meets_minimum());
            assert!(TouchTarget::COMFORTABLE.meets_minimum());
            assert!(!TouchTarget {
                width: 30,
                height: 30
            }
            .meets_minimum());
        }

        #[test]
        fn android_checklist_has_6_items() {
            let checklist = MobileVerificationChecklist::for_platform(MobilePlatform::Android);
            assert_eq!(checklist.items.len(), 6);
            assert!(checklist
                .items
                .iter()
                .any(|i| i.id == "android-webview-perf"));
            assert!(checklist.items.iter().any(|i| i.id == "android-safe-area"));
        }

        #[test]
        fn ios_checklist_has_6_items() {
            let checklist = MobileVerificationChecklist::for_platform(MobilePlatform::Ios);
            assert_eq!(checklist.items.len(), 6);
            assert!(checklist.items.iter().any(|i| i.id == "ios-webview-memory"));
            assert!(checklist
                .items
                .iter()
                .any(|i| i.id == "ios-notch-safe-area"));
        }

        #[test]
        fn checklist_all_passed() {
            let mut checklist = MobileVerificationChecklist::for_platform(MobilePlatform::Android);
            assert!(!checklist.all_passed());
            for item in &checklist.items.clone() {
                checklist.set_result(&item.id, true, None);
            }
            assert!(checklist.all_passed());
        }

        #[test]
        fn checklist_has_failure() {
            let mut checklist = MobileVerificationChecklist::for_platform(MobilePlatform::Ios);
            checklist.set_result("ios-webview-memory", false, Some("OOM crash".into()));
            assert!(checklist.has_failures());
            assert!(!checklist.all_passed());
            let rendered = checklist.render();
            assert!(rendered.contains("FAIL"));
            assert!(rendered.contains("OOM crash"));
        }

        #[test]
        fn background_policy_actions() {
            assert!(!BackgroundPolicy::LowMemory.actions().is_empty());
            assert!(BackgroundPolicy::Terminating
                .actions()
                .contains(&"persist match state to WAL immediately"));
        }

        #[test]
        fn user_agent_detection() {
            assert_eq!(
                MobilePlatform::from_user_agent("Mozilla/5.0 (Linux; Android 13)"),
                Some(MobilePlatform::Android)
            );
            assert_eq!(
                MobilePlatform::from_user_agent("Mozilla/5.0 (iPhone; CPU iPhone OS 17)"),
                Some(MobilePlatform::Ios)
            );
            assert_eq!(
                MobilePlatform::from_user_agent("Mozilla/5.0 (Macintosh)"),
                None
            );
        }

        #[test]
        fn shell_config_css_vars() {
            let config = MobileShellConfig::for_platform(MobilePlatform::Ios);
            let css = config.css_vars();
            assert!(css.contains("--safe-area-top"));
            assert!(css.contains("--touch-min"));
            assert!(css.contains("--mem-budget"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use content_runtime::sign_package;
    use content_schema::{ContentPackage, PackageManifest};
    use rule_vm::BytecodeModule;

    fn pkg() -> Vec<u8> {
        let mut p = ContentPackage {
            manifest: PackageManifest::new("core", "std", "1.0.0"),
            modules: vec![BytecodeModule {
                name: "m".into(),
                code: vec![1, 2, 3],
                local_count: 0,
                max_stack: 8,
                entry_pc: 0,
                logic_hash: [0u8; 32],
                caps_mask: 0,
            }],
            skill_index: vec![],
            character_index: vec![],
            skin_index: Default::default(),
            card_index: vec![],
            trigger_index: vec![],
            modifier_index: vec![],
            active_index: vec![],
            view_as_index: vec![],
            translations: Default::default(),
            asset_manifest: None,
            asset_paths: Default::default(),
            compiler_version: "t".into(),
        };
        p.modules[0].logic_hash = *blake3::hash(&p.modules[0].code).as_bytes();
        p.manifest.logic_hash = p.compute_logic_hash();
        p.to_json_bytes().unwrap()
    }

    #[test]
    fn freeze_blocks_install() {
        let dir = tempfile::tempdir().unwrap();
        let mut host = DesktopHost::open(
            dir.path(),
            ShellCapabilities {
                manage_content: true,
                export_diagnostics: true,
                allow_unsigned_packages: true,
            },
        )
        .unwrap();
        let bytes = pkg();
        host.install_package(&bytes, None, None, true).unwrap();
        host.freeze_for_match();
        let err = host.install_package(&bytes, None, None, true).unwrap_err();
        assert!(matches!(err, ShellError::Denied(_)));
        host.unfreeze_after_match();
        // same version already installed — ok path for re-install same bytes
        host.install_package(&bytes, None, None, true).unwrap();
    }

    #[test]
    fn signed_install_via_host() {
        let dir = tempfile::tempdir().unwrap();
        let host = DesktopHost::open(
            dir.path(),
            ShellCapabilities {
                manage_content: true,
                export_diagnostics: true,
                allow_unsigned_packages: false,
            },
        )
        .unwrap();
        let bytes = pkg();
        let sig = sign_package(&bytes, "k", "dev").unwrap();
        host.install_package(&bytes, Some(&sig), Some("k".into()), true)
            .unwrap();
        assert!(!host.content_lock_hash().unwrap().is_empty());
        let bundle = host.content_bundle().unwrap();
        assert_eq!(bundle.content_lock.packages.len(), 1);
        assert_eq!(bundle.packages, vec![String::from_utf8(bytes).unwrap()]);
        assert_eq!(
            host.content_catalog_page(None, Some(u32::MAX))
                .unwrap()
                .limit,
            content_runtime::MAX_CATALOG_PAGE_LIMIT
        );
        let pack = host
            .build_diagnostic(FaultCode::Other, "ok", Some("m"), Some(1))
            .unwrap();
        pack.assert_no_hidden_leaks().unwrap();
    }
}
