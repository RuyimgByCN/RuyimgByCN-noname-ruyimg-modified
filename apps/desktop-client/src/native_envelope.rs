//! Existing Worker envelope projection from the shared, observer-filtered session API.
use engine_native_wasm::NativeSession;
use serde_json::{json, Value};

fn parse(raw: String) -> Result<Value, String> {
    serde_json::from_str(&raw).map_err(|error| error.to_string())
}

fn card(view: &Value, id: &Value, owner: Value, zone: &str) -> Value {
    let face = &view["cards"][id.to_string()];
    json!({"id": id, "def": face["def"], "suit": face["suit"], "rank": face["rank"],
        "owner": owner, "zone": zone, "kind": face["kind"].as_str().unwrap_or("Other")})
}

fn snapshot(view: &Value, revision: u64) -> Result<Value, String> {
    let mut cards = Vec::new();
    let mut seats = Vec::new();
    for player in view["players"].as_array().into_iter().flatten() {
        for (key, zone) in [("equips", "Equip"), ("judge", "Judge")] {
            for id in player[key].as_array().into_iter().flatten() {
                cards.push(card(view, id, player["seat"].clone(), zone));
            }
        }
        let generals: [Option<engine_api::GeneralSlotView>; 2] = player.get("generals").map(|value| serde_json::from_value(value.clone())).transpose().map_err(|error| format!("generals: {error}"))?.unwrap_or_default();
        seats.push(json!({
            "player": player["seat"], "character_def": player["characterDef"].as_u64().unwrap_or(0),
            "generals": generals,
            "seat": player["seat"], "hp": player["hp"], "max_hp": player["maxHp"],
            "alive": player["alive"], "turned_over": player["turnedOver"],
            "hand_count": player["handCount"], "identity": player["identity"],
            "skills": player["skills"].as_array().into_iter().flatten().map(|name| json!({
                "id": 0, "def": 0, "enabled": true, "use_count": 0, "active": false,
                "view_as": false, "locked": false, "name": name,
            })).collect::<Vec<_>>()
        }));
    }
    let mut own = Vec::new();
    if let Some(player) = view["players"]
        .as_array()
        .and_then(|players| players.iter().find(|p| p["seat"] == view["seat"]))
    {
        for id in player["hand"]["own"].as_array().into_iter().flatten() {
            own.push(card(view, id, view["seat"].clone(), "Hand"));
        }
    }
    own.extend(cards);
    for (key, zone) in [("discard", "DiscardPile"), ("table", "Processing")] {
        for id in view[key].as_array().into_iter().flatten() {
            own.push(card(view, id, Value::Null, zone));
        }
    }
    let phase = match view["phase"].as_str() {
        Some("judge") => "Judge",
        Some("draw") => "Draw",
        Some("play") => "Play",
        Some("discard") => "Discard",
        Some("end") => "End",
        _ => "Idle",
    };
    Ok(json!({"revision": revision, "state_hash": "", "observer": view["seat"],
        "round": view["round"], "current_player": view["turn"], "phase": phase,
        "seats": seats, "known_cards": own, "outcome": view["outcome"]}))
}

fn option_label(option: &Value) -> String {
    if let Some(text) = option.as_str() {
        return text.into();
    }
    if option.is_number() {
        return option.to_string();
    }
    for key in ["card", "player", "tag", "kind"] {
        if !option[key].is_null() {
            return format!("{key}:{}", option[key]);
        }
    }
    "unknown".into()
}

fn choice(session: &NativeSession, observer: u8) -> Result<Value, String> {
    let raw = parse(session.open_choice_json(observer)?)?;
    if raw.is_null() || raw["hidden"] == true {
        return Ok(Value::Null);
    }
    let native_kind = raw.get("native_kind").filter(|value| !value.is_null()).map(|value| {
        value.as_str().and_then(engine_api::choice_kind_from_native).ok_or("invalid native_kind")
    }).transpose()?;
    let kind = native_kind.map(|kind| json!(kind)).unwrap_or_else(|| json!(raw["kind"].as_str().unwrap_or("Custom")));
    if raw.get("prompt").is_some_and(|value| !value.is_null() && !value.is_string()) { return Err("invalid prompt".into()); }
    let mut result = json!({"choice_id": raw["id"], "revision": session.revision(), "observer": observer, "kind": kind,
        "options": raw["options"].as_array().into_iter().flatten().enumerate().map(|(index, option)| {
            json!({"token": index, "label_key": option_label(option)})
        }).collect::<Vec<_>>()});
    for (source, target) in [
        ("prompt", "prompt"),
        ("native_kind", "native_kind"),
        ("min", "min_count"),
        ("max", "max_count"),
        ("allowCancel", "allow_cancel"),
    ] {
        if let Some(value) = raw.get(source) {
            result[target] = value.clone();
        }
    }
    Ok(result)
}

fn public_event(code: &Value) -> Value {
    let label = match code.as_u64() {
        Some(1) => "对局开始",
        Some(2) => "摸牌",
        Some(3) => "使用或响应牌",
        Some(4) => "伤害结算",
        Some(5) => "回复体力",
        Some(6) => "濒死状态变化",
        Some(7) => "玩家阵亡",
        Some(8) => "阶段或回合推进",
        Some(9) => "对局结束",
        Some(10) => "选择状态变化",
        _ => return json!({"PublicEvent": {"code": code, "label": format!("事件 {code}")}}),
    };
    json!({"PublicEvent": {"code": code, "label": label}})
}

pub(super) fn envelope(
    session: &NativeSession,
    observer: u8,
    cursor: &mut u32,
    result: Value,
) -> Result<Value, String> {
    let view = parse(session.view_json(observer)?)?;
    let legal = parse(session.legal_actions_json(observer)?)?;
    let choice = choice(session, observer)?;
    let mut events = Vec::new();
    if !result.is_null() {
        let end = session.log_len();
        let codes = parse(session.public_event_codes_json((*cursor).min(end)))?;
        events = codes
            .as_array()
            .into_iter()
            .flatten()
            .map(public_event)
            .collect();
        *cursor = end;
    }
    Ok(json!({"result": result, "events": events, "choice": choice,
        "view": snapshot(&view, session.revision())?, "legal_actions": legal["actions"],
        "hash_hex": session.state_hash_hex(), "revision": session.revision(), "finished": session.is_finished()}))
}
