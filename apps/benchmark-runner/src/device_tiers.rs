//! Device tier definitions and performance budget gates (architecture §38.1).
//!
//! The plan requires benchmarking against at least 6 device tiers:
//! - Low-end Android WebView
//! - Mainstream Android
//! - iPhone / iPad
//! - Desktop browser
//! - Desktop native
//! - Server baseline
//!
//! Each tier has performance budgets (§38.4). When running benchmarks, the
//! measured values are compared against the selected tier's budgets. A scenario
//! that exceeds its budget is flagged as a regression.

use serde::{Deserialize, Serialize};

/// All device tiers defined by §38.1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum DeviceTier {
    /// Low-end Android WebView (§38.1: 低端 Android WebView).
    /// Representative: Android 8+, 2GB RAM, WebView rendering.
    AndroidLowWebView = 0,
    /// Mainstream Android (§38.1: 主流 Android).
    /// Representative: Android 12+, 6GB RAM, Chrome.
    AndroidMainstream = 1,
    /// iPhone / iPad (§38.1: iPhone / iPad).
    /// Representative: A12+ chip, Safari/WebKit.
    AppleMobile = 2,
    /// Desktop browser (§38.1: 普通桌面浏览器).
    /// Representative: Chrome/Firefox on modern desktop.
    DesktopBrowser = 3,
    /// Desktop native (§38.1: 桌面 Native).
    /// Representative: Tauri/Rust native, no browser overhead.
    DesktopNative = 4,
    /// Server baseline (§38.1: 服务器基准机).
    /// Representative: dedicated server hardware.
    ServerBaseline = 5,
}

impl DeviceTier {
    /// All tiers in order from weakest to strongest.
    pub fn all() -> &'static [DeviceTier] {
        &[
            DeviceTier::AndroidLowWebView,
            DeviceTier::AndroidMainstream,
            DeviceTier::AppleMobile,
            DeviceTier::DesktopBrowser,
            DeviceTier::DesktopNative,
            DeviceTier::ServerBaseline,
        ]
    }

    /// Human-readable label.
    pub fn label(self) -> &'static str {
        match self {
            DeviceTier::AndroidLowWebView => "android-low-webview",
            DeviceTier::AndroidMainstream => "android-mainstream",
            DeviceTier::AppleMobile => "iphone-ipad",
            DeviceTier::DesktopBrowser => "desktop-browser",
            DeviceTier::DesktopNative => "desktop-native",
            DeviceTier::ServerBaseline => "server-baseline",
        }
    }

    /// Display name with hardware description.
    pub fn display_name(self) -> &'static str {
        match self {
            DeviceTier::AndroidLowWebView => "低端 Android WebView (2GB, Android 8+)",
            DeviceTier::AndroidMainstream => "主流 Android (6GB, Android 12+)",
            DeviceTier::AppleMobile => "iPhone / iPad (A12+, Safari)",
            DeviceTier::DesktopBrowser => "普通桌面浏览器 (Chrome)",
            DeviceTier::DesktopNative => "桌面 Native (Tauri/Rust)",
            DeviceTier::ServerBaseline => "服务器基准机",
        }
    }

    /// Parse a tier from its label string.
    pub fn from_label(s: &str) -> Option<Self> {
        match s {
            "android-low-webview" | "low" => Some(DeviceTier::AndroidLowWebView),
            "android-mainstream" | "android" => Some(DeviceTier::AndroidMainstream),
            "iphone-ipad" | "apple" | "ios" => Some(DeviceTier::AppleMobile),
            "desktop-browser" | "browser" => Some(DeviceTier::DesktopBrowser),
            "desktop-native" | "native" => Some(DeviceTier::DesktopNative),
            "server-baseline" | "server" => Some(DeviceTier::ServerBaseline),
            _ => None,
        }
    }
}

/// Performance budget for a tier (§38.4).
///
/// All values are in nanoseconds unless noted. A scenario exceeding its budget
/// is a regression.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TierBudget {
    /// Tier this budget applies to.
    pub tier: DeviceTier,
    /// StartGame P50 latency in microseconds.
    pub start_hash_p50_us: u64,
    /// Command resolution P95 latency in microseconds.
    pub command_p95_us: u64,
    /// Distance query P95 latency in microseconds.
    pub distance_query_p95_us: u64,
    /// Snapshot restore latency in microseconds.
    pub snapshot_restore_us: u64,
    /// Observer projection latency in microseconds.
    pub observer_projection_us: u64,
    /// Match completion (500-step identity match) in seconds.
    pub match_completion_secs: u64,
    /// Max main-thread frame budget in milliseconds (§38.4: <8ms target).
    pub frame_budget_ms: u64,
    /// Max image/GPU memory in MB.
    pub memory_budget_mb: u64,
}

impl TierBudget {
    /// Get the budget for a specific tier.
    pub fn for_tier(tier: DeviceTier) -> Self {
        match tier {
            DeviceTier::AndroidLowWebView => Self {
                tier,
                start_hash_p50_us: 2_000,      // 2ms
                command_p95_us: 8_000,         // 8ms
                distance_query_p95_us: 10_000, // 10ms
                snapshot_restore_us: 5_000,    // 5ms
                observer_projection_us: 3_000, // 3ms
                match_completion_secs: 30,
                frame_budget_ms: 16, // 60fps impossible; target 60fps skip
                memory_budget_mb: 128,
            },
            DeviceTier::AndroidMainstream => Self {
                tier,
                start_hash_p50_us: 1_000,
                command_p95_us: 4_000,
                distance_query_p95_us: 5_000,
                snapshot_restore_us: 2_000,
                observer_projection_us: 1_500,
                match_completion_secs: 15,
                frame_budget_ms: 16,
                memory_budget_mb: 256,
            },
            DeviceTier::AppleMobile => Self {
                tier,
                start_hash_p50_us: 800,
                command_p95_us: 3_000,
                distance_query_p95_us: 3_000,
                snapshot_restore_us: 1_500,
                observer_projection_us: 1_000,
                match_completion_secs: 10,
                frame_budget_ms: 16,
                memory_budget_mb: 256,
            },
            DeviceTier::DesktopBrowser => Self {
                tier,
                start_hash_p50_us: 500,
                command_p95_us: 2_000,
                distance_query_p95_us: 2_000,
                snapshot_restore_us: 1_000,
                observer_projection_us: 800,
                match_completion_secs: 8,
                frame_budget_ms: 8,
                memory_budget_mb: 512,
            },
            DeviceTier::DesktopNative => Self {
                tier,
                start_hash_p50_us: 300,
                command_p95_us: 1_000,
                distance_query_p95_us: 1_000,
                snapshot_restore_us: 500,
                observer_projection_us: 400,
                match_completion_secs: 5,
                frame_budget_ms: 8,
                memory_budget_mb: 1024,
            },
            DeviceTier::ServerBaseline => Self {
                tier,
                start_hash_p50_us: 100,
                command_p95_us: 500,
                distance_query_p95_us: 300,
                snapshot_restore_us: 200,
                observer_projection_us: 100,
                match_completion_secs: 3,
                frame_budget_ms: 8,
                memory_budget_mb: 2048,
            },
        }
    }
}

/// Result of checking a measured value against a tier budget.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BudgetCheck {
    /// Scenario name.
    pub scenario: String,
    /// Measured value (in the scenario's native unit).
    pub measured: u64,
    /// Budget limit.
    pub budget: u64,
    /// Unit (e.g. "us", "ms", "ns").
    pub unit: String,
    /// Whether the measurement is within budget.
    pub passed: bool,
}

impl BudgetCheck {
    /// Check a measured value against a budget.
    pub fn check(scenario: &str, measured: u64, budget: u64, unit: &str) -> Self {
        Self {
            scenario: scenario.into(),
            measured,
            budget,
            unit: unit.into(),
            passed: measured <= budget,
        }
    }

    /// Render as text.
    pub fn render(&self) -> String {
        let status = if self.passed { "✓" } else { "REGRESSION" };
        format!(
            "  {} {:<24} {} / {} {} ({}%)",
            status,
            self.scenario,
            self.measured,
            self.budget,
            self.unit,
            (self.measured as f64 / self.budget.max(1) as f64 * 100.0) as u64
        )
    }
}

/// A full tier report with all budget checks.
#[derive(Debug, Clone, Serialize)]
pub struct TierReport {
    /// Tier name.
    pub tier: String,
    /// Tier display name.
    pub display_name: String,
    /// Budget checks.
    pub checks: Vec<BudgetCheck>,
    /// Whether all checks passed.
    pub all_passed: bool,
}

/// Render a full tier report.
pub fn render_tier_report(report: &TierReport) -> String {
    let mut out = String::new();
    out.push_str(&format!("=== {} ===\n", report.display_name));
    out.push_str(&format!("tier={}\n", report.tier));
    if report.checks.is_empty() {
        out.push_str("  (no scenarios measured)\n");
    } else {
        for check in &report.checks {
            out.push_str(&format!("{}\n", check.render()));
        }
    }
    let status = if report.all_passed { "PASS" } else { "FAIL" };
    out.push_str(&format!("result={status}\n"));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_tiers_defined() {
        let tiers = DeviceTier::all();
        assert_eq!(tiers.len(), 6, "§38.1 requires 6 device tiers");
        // Each must have a unique label.
        let labels: Vec<&str> = tiers.iter().map(|t| t.label()).collect();
        let label_set: std::collections::HashSet<&str> = labels.iter().copied().collect();
        assert_eq!(labels.len(), label_set.len(), "tier labels must be unique");
    }

    #[test]
    fn tier_budgets_decrease_for_faster_tiers() {
        // Server should have stricter (lower) budgets than low-end Android.
        let low = TierBudget::for_tier(DeviceTier::AndroidLowWebView);
        let server = TierBudget::for_tier(DeviceTier::ServerBaseline);
        assert!(
            server.start_hash_p50_us < low.start_hash_p50_us,
            "server budget should be stricter than low-end Android"
        );
        assert!(
            server.command_p95_us < low.command_p95_us,
            "server command budget should be stricter"
        );
    }

    #[test]
    fn from_label_round_trips() {
        for tier in DeviceTier::all() {
            let label = tier.label();
            let parsed = DeviceTier::from_label(label);
            assert_eq!(parsed, Some(*tier), "label {label} should round-trip");
        }
    }

    #[test]
    fn budget_check_passes_within_limit() {
        let check = BudgetCheck::check("start-hash", 500, 1000, "us");
        assert!(check.passed);
    }

    #[test]
    fn budget_check_fails_over_limit() {
        let check = BudgetCheck::check("start-hash", 1500, 1000, "us");
        assert!(!check.passed);
    }

    #[test]
    fn tier_report_all_passed() {
        let report = TierReport {
            tier: "test".into(),
            display_name: "Test Tier".into(),
            checks: vec![BudgetCheck::check("a", 1, 10, "us")],
            all_passed: true,
        };
        let rendered = render_tier_report(&report);
        assert!(rendered.contains("PASS"));
        assert!(rendered.contains("Test Tier"));
    }

    #[test]
    fn frame_budget_within_plan_guidelines() {
        // §38.4: 普通帧脚本与渲染工作目标小于 8 ms
        for tier in DeviceTier::all() {
            let budget = TierBudget::for_tier(*tier);
            // Mobile tiers may need 16ms (60fps cap); desktop/server must target 8ms.
            if matches!(
                budget.tier,
                DeviceTier::DesktopBrowser | DeviceTier::DesktopNative | DeviceTier::ServerBaseline
            ) {
                assert!(
                    budget.frame_budget_ms <= 8,
                    "desktop/server frame budget must be <= 8ms"
                );
            }
        }
    }

    #[test]
    fn low_end_android_has_lowest_memory_budget() {
        let low = TierBudget::for_tier(DeviceTier::AndroidLowWebView);
        let desktop = TierBudget::for_tier(DeviceTier::DesktopNative);
        assert!(
            low.memory_budget_mb < desktop.memory_budget_mb,
            "low-end Android must have lower memory budget"
        );
    }
}
