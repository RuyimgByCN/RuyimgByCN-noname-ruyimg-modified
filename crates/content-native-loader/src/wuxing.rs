//! L06 `wuxing` (`wuxing-upstream-liuli`) runtime probability config parser (`assets/wuxing.json`).
//!
//! Upstream `archive/upstream/noname-liuli/noname/extension/wuxing/extension.js:42-59, 186-205`
//! gates card nature injection using `lib.config.wuxing_num_playpackconfig`.
//! Per `docs/architecture/native-rules-baseline.md`, the native asset defaults this rate to `0.3`
//! (`3000` basis points) while honoring explicit valid overrides in `0.0..=1.0`.

use engine_native::content::extension::{ExtensionPack, InitHook};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq)]
pub struct WuxingConfig {
    pub wuxing_num_playpackconfig: f64,
    pub nature_rate_bps: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WuxingConfigError {
    BadJson(String),
    BadField(String),
}

impl std::fmt::Display for WuxingConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WuxingConfigError::BadJson(e) => write!(f, "wuxing config JSON error: {e}"),
            WuxingConfigError::BadField(fld) => write!(f, "invalid wuxing config field: {fld}"),
        }
    }
}

fn parse_rate_value(val: &Value) -> Result<f64, WuxingConfigError> {
    let rate = if let Some(n) = val.as_f64() {
        n
    } else if let Some(s) = val.as_str() {
        s.trim()
            .parse::<f64>()
            .map_err(|_| WuxingConfigError::BadField("wuxing_num_playpackconfig".into()))?
    } else {
        return Err(WuxingConfigError::BadField(
            "wuxing_num_playpackconfig".into(),
        ));
    };
    if !rate.is_finite() || !(0.0..=1.0).contains(&rate) {
        return Err(WuxingConfigError::BadField(
            "wuxing_num_playpackconfig".into(),
        ));
    }
    Ok(rate)
}

/// Parse `assets/wuxing.json` and extract the configured card nature probability (`0.0..=1.0`).
/// Defaults to `0.3` (`3000` bps) when neither `wuxing_num_playpackconfig` nor legacy `nature_fraction` is specified.
pub fn parse_wuxing_config(json: &str) -> Result<WuxingConfig, WuxingConfigError> {
    let v: Value =
        serde_json::from_str(json).map_err(|e| WuxingConfigError::BadJson(e.to_string()))?;
    let obj = v
        .as_object()
        .ok_or_else(|| WuxingConfigError::BadField("root must be object".into()))?;
    let rate = if let Some(raw_rate) = obj.get("wuxing_num_playpackconfig") {
        parse_rate_value(raw_rate)?
    } else if let Some(frac) = obj.get("nature_fraction") {
        let num = frac
            .get("numerator")
            .and_then(|x| x.as_f64())
            .ok_or_else(|| WuxingConfigError::BadField("nature_fraction.numerator".into()))?;
        let den = frac
            .get("denominator")
            .and_then(|x| x.as_f64())
            .filter(|&d| d > 0.0)
            .ok_or_else(|| WuxingConfigError::BadField("nature_fraction.denominator".into()))?;
        let ratio = num / den;
        if !ratio.is_finite() || !(0.0..=1.0).contains(&ratio) {
            return Err(WuxingConfigError::BadField("nature_fraction".into()));
        }
        ratio
    } else {
        0.3
    };
    let nature_rate_bps = ((rate * 10_000.0).round() as u32).min(10_000) as u16;
    Ok(WuxingConfig {
        wuxing_num_playpackconfig: rate,
        nature_rate_bps,
    })
}

/// Apply parsed `wuxing.json` probability to an extension pack that declares `wuxing_natures`.
pub fn apply_wuxing_config(pack: &mut ExtensionPack, config_json: &str) -> Result<(), String> {
    if pack.init_hook.wuxing_natures_bps().is_none() {
        return Ok(());
    }
    let cfg = parse_wuxing_config(config_json).map_err(|e| e.to_string())?;
    pack.init_hook = InitHook::from_wuxing_bps(cfg.nature_rate_bps);
    Ok(())
}
