//! Declarative accessibility schema (architecture plan §30).
//!
//! Accessibility information comes from the Observer View, not from GameState.
//! This crate defines:
//! - [`AccessibilityProfile`] — user preferences (reduce motion, high contrast,
//!   font scale, color-blind mode)
//! - [`AriaDescriptor`] — ARIA roles / labels / states for UI elements
//! - [`KeyboardAction`] — keyboard navigation actions and focus order
//! - [`StateSignal`] — non-color state signals (§30: 不只依赖颜色表达状态)
//!
//! The web client consumes these to render accessible UI. The rule core never
//! depends on this crate.

#![deny(missing_docs)]

use serde::{Deserialize, Serialize};

/// Accessibility profile version.
pub const A11Y_SCHEMA_VERSION: u32 = 1;

/// User accessibility preferences (§30).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccessibilityProfile {
    /// Schema version.
    pub schema_version: u32,
    /// Reduce motion / animations (§30: 减少动态效果).
    pub reduce_motion: bool,
    /// High contrast mode (§30: 高对比度).
    pub high_contrast: bool,
    /// Font scale multiplier (§30: 字号缩放). 1.0 = default.
    pub font_scale: f32,
    /// Color-blind mode (§30: 色觉友好提示).
    pub color_blind_mode: ColorBlindMode,
    /// Whether animation skip is enabled (§30: 动画跳过后仍可理解规则结果).
    pub skip_animations: bool,
    /// Screen reader verbosity level.
    pub screen_reader: ScreenReaderLevel,
}

impl Default for AccessibilityProfile {
    fn default() -> Self {
        Self {
            schema_version: A11Y_SCHEMA_VERSION,
            reduce_motion: false,
            high_contrast: false,
            font_scale: 1.0,
            color_blind_mode: ColorBlindMode::None,
            skip_animations: false,
            screen_reader: ScreenReaderLevel::Default,
        }
    }
}

/// Color-blind compensation mode (§30: 色觉友好).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[repr(u8)]
pub enum ColorBlindMode {
    /// No color-blind compensation.
    #[default]
    None = 0,
    /// Protanopia (red-blind).
    Protanopia = 1,
    /// Deuteranopia (green-blind).
    Deuteranopia = 2,
    /// Tritanopia (blue-blind).
    Tritanopia = 3,
    /// Achromatopsia (full color-blind).
    Achromatopsia = 4,
}

/// Screen reader verbosity (§30: 屏幕阅读器语义).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[repr(u8)]
pub enum ScreenReaderLevel {
    /// Screen reader off / minimal.
    Off = 0,
    /// Default verbosity.
    #[default]
    Default = 1,
    /// Verbose (announce all state changes).
    Verbose = 2,
}

/// ARIA descriptor for a UI element (§30: 屏幕阅读器语义).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AriaDescriptor {
    /// ARIA role (e.g. "button", "listbox", "option", "status").
    pub role: String,
    /// Accessible label (localized key).
    pub label_key: String,
    /// ARIA states (e.g. disabled, selected, expanded).
    #[serde(default)]
    pub states: Vec<AriaState>,
    /// Live region politeness (for status announcements).
    #[serde(default)]
    pub live: LiveRegion,
}

/// ARIA state key-value pair.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AriaState {
    /// State name (e.g. "aria-disabled", "aria-selected").
    pub name: String,
    /// State value (e.g. "true", "false").
    pub value: String,
}

/// ARIA live region politeness.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[repr(u8)]
pub enum LiveRegion {
    /// Not a live region.
    #[default]
    Off = 0,
    /// polite: announce when idle.
    Polite = 1,
    /// assertive: announce immediately.
    Assertive = 2,
}

/// Keyboard action for complete keyboard navigation (§30: 键盘完整操作).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyboardAction {
    /// Action id (e.g. "play-card", "next-phase", "confirm-choice").
    pub action_id: String,
    /// Key combination (e.g. "Enter", "Space", "Tab", "Escape", "1"-"8").
    pub key: String,
    /// Whether modifier keys are required.
    #[serde(default)]
    pub modifiers: KeyModifiers,
    /// Localized description key.
    pub description_key: String,
}

/// Keyboard modifier flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct KeyModifiers {
    /// Ctrl / Cmd.
    pub ctrl: bool,
    /// Shift.
    pub shift: bool,
    /// Alt / Option.
    pub alt: bool,
}

/// Non-color state signal (§30: 不只依赖颜色表达状态).
///
/// State must be conveyed through more than color alone. Each game state has
/// an associated icon + text label so color-blind users can distinguish it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum StateSignal {
    /// Player is alive (green + ✓ icon).
    Alive = 1,
    /// Player is dying (red + ⚠ icon + text "濒死").
    Dying = 2,
    /// Player is dead (gray + ✗ icon + text "阵亡").
    Dead = 3,
    /// Card is playable (blue + ▸ icon).
    CardPlayable = 4,
    /// Card is unplayable (gray + ✕ icon + text "不可用").
    CardUnplayable = 5,
    /// Current turn player (highlight + ► indicator).
    CurrentTurn = 6,
    /// Pending choice (pulsing border + ? icon + text "请选择").
    PendingChoice = 7,
}

impl StateSignal {
    /// Icon (non-color indicator) for this state.
    pub fn icon(self) -> &'static str {
        match self {
            StateSignal::Alive => "✓",
            StateSignal::Dying => "⚠",
            StateSignal::Dead => "✗",
            StateSignal::CardPlayable => "▸",
            StateSignal::CardUnplayable => "✕",
            StateSignal::CurrentTurn => "►",
            StateSignal::PendingChoice => "?",
        }
    }

    /// Whether this state has a text label (not color-only).
    pub fn has_text_label(self) -> bool {
        matches!(
            self,
            StateSignal::Dying
                | StateSignal::Dead
                | StateSignal::CardUnplayable
                | StateSignal::PendingChoice
        )
    }

    /// CSS color variable name (for high-contrast / color-blind themes).
    pub fn color_var(self) -> &'static str {
        match self {
            StateSignal::Alive | StateSignal::CardPlayable => "--color-positive",
            StateSignal::Dying | StateSignal::PendingChoice => "--color-warning",
            StateSignal::Dead | StateSignal::CardUnplayable => "--color-negative",
            StateSignal::CurrentTurn => "--color-accent",
        }
    }
}

/// Default keyboard action set for the battle UI (§30: 键盘完整操作).
///
/// Every interactive element must be reachable and operable via keyboard.
pub fn default_keyboard_actions() -> Vec<KeyboardAction> {
    vec![
        KeyboardAction {
            action_id: "select-card".into(),
            key: "Enter".into(),
            modifiers: KeyModifiers::default(),
            description_key: "a11y.action.select_card".into(),
        },
        KeyboardAction {
            action_id: "play-card".into(),
            key: "Space".into(),
            modifiers: KeyModifiers::default(),
            description_key: "a11y.action.play_card".into(),
        },
        KeyboardAction {
            action_id: "next-card".into(),
            key: "ArrowRight".into(),
            modifiers: KeyModifiers::default(),
            description_key: "a11y.action.next_card".into(),
        },
        KeyboardAction {
            action_id: "prev-card".into(),
            key: "ArrowLeft".into(),
            modifiers: KeyModifiers::default(),
            description_key: "a11y.action.prev_card".into(),
        },
        KeyboardAction {
            action_id: "next-player".into(),
            key: "Tab".into(),
            modifiers: KeyModifiers::default(),
            description_key: "a11y.action.next_player".into(),
        },
        KeyboardAction {
            action_id: "confirm-choice".into(),
            key: "Enter".into(),
            modifiers: KeyModifiers::default(),
            description_key: "a11y.action.confirm_choice".into(),
        },
        KeyboardAction {
            action_id: "cancel-choice".into(),
            key: "Escape".into(),
            modifiers: KeyModifiers::default(),
            description_key: "a11y.action.cancel_choice".into(),
        },
        KeyboardAction {
            action_id: "end-phase".into(),
            key: "e".into(),
            modifiers: KeyModifiers::default(),
            description_key: "a11y.action.end_phase".into(),
        },
        KeyboardAction {
            action_id: "skip-animation".into(),
            key: "s".into(),
            modifiers: KeyModifiers::default(),
            description_key: "a11y.action.skip_animation".into(),
        },
        KeyboardAction {
            action_id: "focus-hand".into(),
            key: "1".into(),
            modifiers: KeyModifiers::default(),
            description_key: "a11y.action.focus_hand".into(),
        },
        KeyboardAction {
            action_id: "focus-board".into(),
            key: "2".into(),
            modifiers: KeyModifiers::default(),
            description_key: "a11y.action.focus_board".into(),
        },
        KeyboardAction {
            action_id: "focus-log".into(),
            key: "3".into(),
            modifiers: KeyModifiers::default(),
            description_key: "a11y.action.focus_log".into(),
        },
    ]
}

/// Accessibility settings bundle for the client (serialized to localStorage).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccessibilitySettings {
    /// User profile.
    pub profile: AccessibilityProfile,
    /// Keyboard actions (may be remapped).
    pub keyboard_actions: Vec<KeyboardAction>,
}

impl Default for AccessibilitySettings {
    fn default() -> Self {
        Self {
            profile: AccessibilityProfile::default(),
            keyboard_actions: default_keyboard_actions(),
        }
    }
}

impl AccessibilitySettings {
    /// Serialize to JSON bytes (for localStorage / settings file).
    pub fn to_json_bytes(&self) -> Result<Vec<u8>, String> {
        serde_json::to_vec(self).map_err(|e| e.to_string())
    }

    /// Deserialize from JSON bytes.
    pub fn from_json_bytes(data: &[u8]) -> Result<Self, String> {
        serde_json::from_slice(data).map_err(|e| e.to_string())
    }
}

/// Validate that every StateSignal has a non-color indicator (§30 invariant:
/// 不只依赖颜色表达状态).
///
/// Returns `Ok(())` if all states have icons, or a list of states that fail.
pub fn validate_state_not_color_only() -> Result<(), Vec<StateSignal>> {
    let all = [
        StateSignal::Alive,
        StateSignal::Dying,
        StateSignal::Dead,
        StateSignal::CardPlayable,
        StateSignal::CardUnplayable,
        StateSignal::CurrentTurn,
        StateSignal::PendingChoice,
    ];
    let failing: Vec<StateSignal> = all
        .iter()
        .copied()
        .filter(|s| s.icon().is_empty())
        .collect();
    if failing.is_empty() {
        Ok(())
    } else {
        Err(failing)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_defaults_are_safe() {
        let p = AccessibilityProfile::default();
        assert_eq!(p.font_scale, 1.0);
        assert!(!p.reduce_motion);
        assert_eq!(p.color_blind_mode, ColorBlindMode::None);
    }

    #[test]
    fn settings_round_trip() {
        let s = AccessibilitySettings::default();
        let bytes = s.to_json_bytes().unwrap();
        let restored = AccessibilitySettings::from_json_bytes(&bytes).unwrap();
        assert_eq!(s, restored);
    }

    #[test]
    fn state_signals_have_icons() {
        // §30 invariant: every state has a non-color indicator.
        let result = validate_state_not_color_only();
        assert!(result.is_ok(), "all states must have non-color icons");
    }

    #[test]
    fn dying_and_dead_have_text_labels() {
        // Critical states must have text labels (not color-only).
        assert!(StateSignal::Dying.has_text_label());
        assert!(StateSignal::Dead.has_text_label());
        assert!(StateSignal::PendingChoice.has_text_label());
    }

    #[test]
    fn default_keyboard_actions_cover_core_actions() {
        let actions = default_keyboard_actions();
        let ids: Vec<&str> = actions.iter().map(|a| a.action_id.as_str()).collect();
        // Every core battle action must have a keyboard binding.
        assert!(ids.contains(&"play-card"));
        assert!(ids.contains(&"confirm-choice"));
        assert!(ids.contains(&"cancel-choice"));
        assert!(ids.contains(&"end-phase"));
        assert!(ids.contains(&"skip-animation"));
    }

    #[test]
    fn color_blind_modes_are_distinct() {
        assert_ne!(ColorBlindMode::Protanopia, ColorBlindMode::Deuteranopia);
        assert_ne!(ColorBlindMode::Deuteranopia, ColorBlindMode::Tritanopia);
        assert_ne!(ColorBlindMode::Tritanopia, ColorBlindMode::Achromatopsia);
    }

    #[test]
    fn profile_with_high_contrast_and_large_font() {
        let p = AccessibilityProfile {
            schema_version: A11Y_SCHEMA_VERSION,
            reduce_motion: true,
            high_contrast: true,
            font_scale: 1.5,
            color_blind_mode: ColorBlindMode::Deuteranopia,
            skip_animations: true,
            screen_reader: ScreenReaderLevel::Verbose,
        };
        let bytes = serde_json::to_vec(&p).unwrap();
        let restored: AccessibilityProfile = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(p, restored);
    }
}
