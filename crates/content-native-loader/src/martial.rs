//! L21 武皇争霸里仍会执行的决策。
//!
//! 联机在 `addMode` 之后被写成 `connect = false`。选将界面、回放、`Math.random`、
//! 定时器和 `extension/武皇/mightyFoe.js` 都不执行。双将扣血在 `askJoin` 里被注释。
//! 本模块不画界面，不播放，不写盘，不 `eval`，不睡眠。

use std::collections::BTreeSet;

use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MartialError {
    BadConfig(String),
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MartialConfig {
    pub source: String,
    pub upstream_path: String,
    pub upstream_commit: String,
    pub upstream_version: String,
    pub enable: bool,
    pub show: i64,
    pub battle: i64,
    pub hide: i64,
    pub double_character: bool,
    pub double_hp: String,
    pub free_choose: bool,
    pub change_choice: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Layout {
    pub show: i64,
    pub battle: i64,
    pub hide: i64,
    pub width: i64,
    pub num: i64,
    pub hidden: i64,
    pub height: i64,
    pub size: i64,
    pub ratio: f64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Seats {
    pub zhu: usize,
    pub fan: usize,
    pub side: u8,
    pub zhu_takes_friend: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DraftInput {
    pub names: Vec<String>,
    pub ranks: Vec<f64>,
    pub hidden: Vec<usize>,
    pub width: usize,
    pub first: u8,
    pub skip: bool,
    pub player_choices: Vec<String>,
    pub ai_indices: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DraftOutput {
    pub friend: Vec<String>,
    pub enemy: Vec<String>,
    pub hidden_ids: Vec<String>,
    pub side_start: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordBook {
    modes: Vec<ModeScore>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ModeScore {
    mode: String,
    zhu: Option<[u32; 2]>,
    fan: Option<[u32; 2]>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverOut {
    pub win: bool,
    pub coin: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeathQuery {
    pub backups_left: usize,
    pub dead_is_me: bool,
    pub me_dying: bool,
    pub me_has_name2: bool,
    pub dead_has_qiangdi: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeathOut {
    pub check_result: bool,
    pub try_mighty: bool,
    pub replace: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplaceOut {
    pub entrant: String,
    pub backups: Vec<String>,
    pub draw: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JoinOut {
    pub offered: bool,
    pub accepted: bool,
    pub name1: String,
    pub name2: Option<String>,
    pub backups: Vec<String>,
    pub hp_changed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MightyPair {
    pub name1: String,
    pub name2: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MightyOut {
    pub offered: bool,
    pub accepted: bool,
    pub check_result: bool,
    pub name1: Option<String>,
    pub name2: Option<String>,
    pub draw: u32,
    pub skill: bool,
    pub phase_skipped: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cleared {
    pub list_dropped: bool,
    pub friend: Vec<String>,
    pub enemy: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FreeOut {
    pub replaced: bool,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MightyScript {
    pub url: &'static str,
    pub ran: bool,
}

struct SeatCard {
    id: String,
    origin: usize,
    given_rank: f64,
}

pub fn parse_martial_config(text: &str) -> Result<MartialConfig, MartialError> {
    let value: Value = serde_json::from_str(text)
        .map_err(|err| MartialError::BadConfig(format!("json {err}")))?;
    let source = required_string(&value, "source")?;
    if source != "noname-liuli" {
        return Err(MartialError::BadConfig("source".into()));
    }
    let double_hp = match value.get("double_hp") {
        None => "hejiansan".to_string(),
        Some(Value::String(text)) if hp_rule(text) => text.clone(),
        _ => return Err(MartialError::BadConfig("double_hp".into())),
    };
    Ok(MartialConfig {
        source,
        upstream_path: required_string(&value, "upstream_path")?,
        upstream_commit: required_string(&value, "upstream_commit")?,
        upstream_version: required_string(&value, "upstream_version")?,
        enable: bool_field(&value, "enable", false)?,
        show: count_field(&value, "show_number", 5, true)?,
        battle: count_field(&value, "battle_number", 5, false)?,
        hide: count_field(&value, "hide_number", 2, true)?,
        double_character: bool_field(&value, "double_character", false)?,
        double_hp,
        free_choose: bool_field(&value, "free_choose", false)?,
        change_choice: bool_field(&value, "change_choice", false)?,
    })
}

pub fn layout_numbers(show: i64, battle: i64, hide: i64) -> Layout {
    let width = show.max(battle);
    Layout {
        show,
        battle,
        hide,
        width,
        num: battle,
        hidden: hide.min(width) * 2,
        height: 4,
        size: width * 4,
        ratio: 1.2,
    }
}

pub fn assign_seats(first: u8) -> Result<Seats, MartialError> {
    match first {
        0 => Ok(Seats {
            zhu: 1,
            fan: 0,
            side: 3,
            zhu_takes_friend: false,
        }),
        1 => Ok(Seats {
            zhu: 0,
            fan: 1,
            side: 1,
            zhu_takes_friend: true,
        }),
        _ => Err(MartialError::BadConfig("first".into())),
    }
}

pub fn filter_pool(ids: &[String], disabled: &BTreeSet<String>) -> Vec<String> {
    ids.iter()
        .filter(|id| !disabled.contains(id.as_str()))
        .cloned()
        .collect()
}

pub fn take_unique(ids: &[String], picks: &[usize]) -> Result<Vec<String>, MartialError> {
    let mut rest = ids.to_vec();
    let mut out = Vec::new();
    for pick in picks {
        if *pick >= rest.len() {
            return Err(MartialError::BadConfig("pick".into()));
        }
        out.push(rest.remove(*pick));
    }
    Ok(out)
}

pub fn ai_order(
    names: &[String],
    ranks: &[f64],
    hidden: &[usize],
) -> Result<Vec<String>, MartialError> {
    let cards = seat_cards(names, ranks, hidden)?;
    let hidden_set = hidden.iter().copied().collect();
    Ok(order_with_hidden(&cards, &hidden_set)
        .into_iter()
        .map(|card| card.id)
        .collect())
}

pub fn draft(input: &DraftInput) -> Result<DraftOutput, MartialError> {
    let cards = seat_cards(&input.names, &input.ranks, &input.hidden)?;
    if input.width == 0 || cards.len() != input.width * 2 {
        return Err(MartialError::BadConfig("width".into()));
    }
    let seats = assign_seats(input.first)?;
    let hidden_set: BTreeSet<usize> = input.hidden.iter().copied().collect();
    let ai = order_with_hidden(&cards, &hidden_set);
    let mut moved = vec![false; cards.len()];
    let mut player_at = 0usize;
    let mut ai_at = 0usize;
    let mut friend = Vec::new();
    let mut enemy = Vec::new();
    if input.skip {
        while enemy.len() < input.width {
            enemy.push(take_ai_at(&cards, &ai, &mut moved, 0)?);
        }
        while friend.len() < input.width {
            friend.push(take_ai_at(&cards, &ai, &mut moved, 0)?);
        }
    } else {
        let mut side = seats.side;
        while friend.len() + enemy.len() + 1 < input.width * 2 {
            if side < 2 {
                friend.push(take_player_at(input, &cards, &mut moved, &mut player_at)?);
            } else {
                let slot = next_ai_slot(input, &mut ai_at)?;
                enemy.push(take_ai_at(&cards, &ai, &mut moved, slot)?);
            }
            side = if side + 1 > 3 { 0 } else { side + 1 };
        }
        let left = (0..cards.len())
            .find(|origin| !moved[*origin])
            .ok_or_else(|| MartialError::BadConfig("left".into()))?;
        if side < 2 {
            friend.push(cards[left].id.clone());
        } else {
            enemy.push(cards[left].id.clone());
        }
    }
    let hidden_ids = cards
        .iter()
        .filter(|card| hidden_set.contains(&card.origin))
        .map(|card| card.id.clone())
        .collect();
    Ok(DraftOutput {
        friend,
        enemy,
        hidden_ids,
        side_start: seats.side,
    })
}

pub fn enemy_battle(
    enemy: &[String],
    width: usize,
    num: usize,
    bag_picks: &[usize],
    order: &[usize],
) -> Result<Vec<String>, MartialError> {
    if num > width || width > enemy.len() {
        return Err(MartialError::BadConfig("battle".into()));
    }
    let mut bag = Vec::new();
    for index in 0..width {
        for _ in 0..(width - index) {
            bag.push(index);
        }
    }
    let mut chosen = Vec::new();
    for step in 0..num {
        let pick = if bag_picks.is_empty() {
            0
        } else {
            *bag_picks
                .get(step)
                .ok_or_else(|| MartialError::BadConfig("bag".into()))?
        };
        if pick >= bag.len() {
            return Err(MartialError::BadConfig("bag".into()));
        }
        let slot = bag[pick];
        bag.retain(|item| *item != slot);
        chosen.push(enemy[slot].clone());
    }
    if order.is_empty() {
        return Ok(chosen);
    }
    if order.len() != chosen.len() || {
        let mut seen = vec![false; chosen.len()];
        order.iter().any(|index| {
            if *index >= seen.len() || seen[*index] {
                return true;
            }
            seen[*index] = true;
            false
        })
    } {
        return Err(MartialError::BadConfig("order".into()));
    }
    Ok(order.iter().map(|index| chosen[*index].clone()).collect())
}

pub fn battle_lists(first: u8, friend: &[String], enemy: &[String]) -> Result<(Vec<String>, Vec<String>), MartialError> {
    let seats = assign_seats(first)?;
    if seats.zhu_takes_friend {
        Ok((friend.to_vec(), enemy.to_vec()))
    } else {
        Ok((enemy.to_vec(), friend.to_vec()))
    }
}

pub fn opening_draw(identity: &str) -> u32 {
    if identity == "fan" {
        5
    } else {
        4
    }
}

impl RecordBook {
    pub fn new() -> Self {
        Self { modes: Vec::new() }
    }

    pub fn add(&mut self, mode: &str, identity: &str, win: Option<bool>) {
        let Some(win) = win else {
            return;
        };
        if identity != "zhu" && identity != "fan" {
            return;
        }
        let mode_score = if let Some(found) = self.modes.iter_mut().find(|item| item.mode == mode) {
            found
        } else {
            self.modes.push(ModeScore {
                mode: mode.to_string(),
                zhu: None,
                fan: None,
            });
            self.modes.last_mut().expect("just pushed")
        };
        let slot = if identity == "zhu" {
            &mut mode_score.zhu
        } else {
            &mut mode_score.fan
        };
        let pair = slot.get_or_insert([0, 0]);
        if win {
            pair[0] += 1;
        } else {
            pair[1] += 1;
        }
    }
}

impl Default for RecordBook {
    fn default() -> Self {
        Self::new()
    }
}

pub fn format_record(book: &RecordBook) -> String {
    let mut text = String::new();
    for mode in &book.modes {
        text.push_str(mode_label(&mode.mode));
        text.push_str("：<br>");
        for (identity, score) in [("zhu", mode.zhu), ("fan", mode.fan)] {
            if let Some([wins, losses]) = score {
                text.push_str(identity_label(identity));
                text.push_str("：");
                text.push_str(&wins.to_string());
                text.push_str("胜 ");
                text.push_str(&losses.to_string());
                text.push_str("负<br>");
            }
        }
    }
    text
}

pub fn check_result(me_alive: bool, enemy_qiangdi: bool, change_coin: bool) -> OverOut {
    if enemy_qiangdi {
        OverOut {
            win: true,
            coin: if me_alive && change_coin { 150 } else { 0 },
        }
    } else {
        OverOut {
            win: me_alive,
            coin: 0,
        }
    }
}

pub fn on_general_dead(query: &DeathQuery) -> DeathOut {
    if query.backups_left == 0 {
        DeathOut {
            check_result: query.dead_is_me
                || query.me_dying
                || !query.me_has_name2
                || query.dead_has_qiangdi,
            try_mighty: true,
            replace: false,
        }
    } else {
        DeathOut {
            check_result: false,
            try_mighty: false,
            replace: true,
        }
    }
}

pub fn replacement_draw(backups: &[String], cuorui: bool) -> Result<ReplaceOut, MartialError> {
    let mut rest = backups.to_vec();
    if rest.is_empty() {
        return Err(MartialError::BadConfig("backup".into()));
    }
    let entrant = rest.remove(0);
    let draw = if cuorui {
        2 + rest.len() as u32
    } else {
        4
    };
    Ok(ReplaceOut {
        entrant,
        backups: rest,
        draw,
    })
}

pub fn ask_join(
    double_character: bool,
    name1: &str,
    name2: Option<&str>,
    backups: &[String],
    accept: bool,
) -> JoinOut {
    let backups = backups.to_vec();
    if !double_character || backups.is_empty() || name2.is_some() {
        return JoinOut {
            offered: false,
            accepted: false,
            name1: name1.to_string(),
            name2: name2.map(str::to_string),
            backups,
            hp_changed: false,
        };
    }
    if !accept {
        return JoinOut {
            offered: true,
            accepted: false,
            name1: name1.to_string(),
            name2: None,
            backups,
            hp_changed: false,
        };
    }
    let mut rest = backups;
    let joined = rest.remove(0);
    JoinOut {
        offered: true,
        accepted: true,
        name1: name1.to_string(),
        name2: Some(joined),
        backups: rest,
        hp_changed: false,
    }
}

pub fn consider_mighty(
    pairs: &[MightyPair],
    known: &BTreeSet<String>,
    pick: usize,
    accept: bool,
    phase_using: bool,
) -> Result<MightyOut, MartialError> {
    let filtered: Vec<&MightyPair> = pairs
        .iter()
        .filter(|pair| {
            pair.name2
                .as_ref()
                .is_some_and(|name| known.contains(&pair.name1) && known.contains(name))
        })
        .collect();
    if filtered.is_empty() {
        return Ok(MightyOut {
            offered: false,
            accepted: false,
            check_result: true,
            name1: None,
            name2: None,
            draw: 0,
            skill: false,
            phase_skipped: false,
        });
    }
    let chosen = filtered
        .get(pick)
        .ok_or_else(|| MartialError::BadConfig("foe".into()))?;
    if !accept {
        return Ok(MightyOut {
            offered: true,
            accepted: false,
            check_result: true,
            name1: Some(chosen.name1.clone()),
            name2: chosen.name2.clone(),
            draw: 0,
            skill: false,
            phase_skipped: false,
        });
    }
    Ok(MightyOut {
        offered: true,
        accepted: true,
        check_result: false,
        name1: Some(chosen.name1.clone()),
        name2: chosen.name2.clone(),
        draw: 4,
        skill: true,
        phase_skipped: phase_using,
    })
}

pub fn attitude(same: bool) -> i32 {
    if same {
        10
    } else {
        -10
    }
}

pub fn has_zhu_skill() -> bool {
    false
}

pub fn video_name(name: &str, name2: Option<&str>, mode: &str, identity: &str) -> [String; 2] {
    let mut title = name.to_string();
    if let Some(second) = name2 {
        title.push('/');
        title.push_str(second);
    }
    [
        title,
        format!("{} - {}", mode_label(mode), identity_label(identity)),
    ]
}

pub fn double_hp_visible(double_character: bool) -> bool {
    double_character
}

pub fn reselect_pool(
    change_choice: bool,
    list2: &[String],
    friend: &[String],
    enemy: &[String],
) -> Result<Vec<String>, MartialError> {
    if !change_choice {
        return Err(MartialError::Unavailable);
    }
    let mut pool = list2.to_vec();
    pool.extend(friend.iter().cloned());
    pool.extend(enemy.iter().cloned());
    Ok(pool)
}

pub fn redraw_pool(change_choice: bool) -> Result<Cleared, MartialError> {
    if !change_choice {
        return Err(MartialError::Unavailable);
    }
    Ok(Cleared {
        list_dropped: true,
        friend: Vec::new(),
        enemy: Vec::new(),
    })
}

pub fn free_replace(free_choose: bool, choosing: bool, name: &str) -> Result<FreeOut, MartialError> {
    if !free_choose {
        return Err(MartialError::Unavailable);
    }
    if !choosing {
        return Ok(FreeOut {
            replaced: false,
            name: None,
        });
    }
    Ok(FreeOut {
        replaced: true,
        name: Some(name.to_string()),
    })
}

pub fn mighty_script() -> MightyScript {
    MightyScript {
        url: "extension/武皇/mightyFoe.js",
        ran: false,
    }
}

pub fn online_choose_runs() -> bool {
    false
}

pub fn playback_runs() -> bool {
    false
}

pub fn connect_enabled() -> bool {
    false
}

pub fn apply_martial_line(config_text: &str, request_text: &str) -> Result<String, MartialError> {
    let config = parse_martial_config(config_text)?;
    let request: Value = if request_text.trim().is_empty() {
        Value::Object(Default::default())
    } else {
        serde_json::from_str(request_text)
            .map_err(|err| MartialError::BadConfig(format!("request {err}")))?
    };
    if !request.is_object() {
        return Err(MartialError::BadConfig("request".into()));
    }
    let script = mighty_script();
    if !config.enable {
        return Ok(format!(
            "registered:0 | mode:- | connect:0 | sampled:0 | dom:0 | sleep:0 | eval:0 | fetch:0 | write:0 | reload:0 | scriptRan:0 | online:0 | playback:0 | dialog:0 | script:{}",
            script.url
        ));
    }
    let first = match request.get("first") {
        None => 0,
        Some(Value::Number(number)) => number
            .as_u64()
            .and_then(|value| u8::try_from(value).ok())
            .ok_or_else(|| MartialError::BadConfig("first".into()))?,
        _ => return Err(MartialError::BadConfig("first".into())),
    };
    let join = match request.get("join") {
        None => false,
        Some(Value::Bool(value)) => *value,
        _ => return Err(MartialError::BadConfig("join".into())),
    };
    let layout = layout_numbers(config.show, config.battle, config.hide);
    if layout.width != 5 || layout.hidden != 4 {
        return Err(MartialError::BadConfig("layout".into()));
    }
    let names: Vec<String> = ('a'..='j').map(|ch| ch.to_string()).collect();
    let ranks: Vec<f64> = (1..=10).map(|rank| rank as f64).collect();
    let hidden: Vec<usize> = (0..layout.hidden as usize).collect();
    let drafted = draft(&DraftInput {
        names,
        ranks,
        hidden,
        width: layout.width as usize,
        first,
        skip: false,
        player_choices: Vec::new(),
        ai_indices: Vec::new(),
    })?;
    let seats = assign_seats(first)?;
    let (zhu, fan) = battle_lists(first, &drafted.friend, &enemy_battle(
        &drafted.enemy,
        layout.width as usize,
        layout.num as usize,
        &[],
        &[],
    )?)?;
    let open_zhu = open_label(&zhu, config.double_character, join);
    let open_fan = open_label(&fan, config.double_character, join);
    let join_mark = if !config.double_character {
        "-"
    } else if join {
        "1"
    } else {
        "0"
    };
    Ok(format!(
        "registered:1 | mode:normal | connect:0 | double:{} | hpRule:{} | hpUi:{} | change:{} | free:{} | show:{} | battle:{} | hide:{} | width:{} | num:{} | hiddenN:{} | size:{} | ratio:1.2 | first:{} | side:{} | zhuSeat:{} | fanSeat:{} | zhuSkill:0 | hidden:{} | friend:{} | enemy:{} | zhu:{} | fan:{} | openZhu:{} | openFan:{} | drawZhu:{} | drawFan:{} | join:{} | hp:0 | foe:0 | script:{} | scriptRan:0 | online:0 | playback:0 | coeff:- | sampled:0 | dom:0 | sleep:0 | eval:0 | fetch:0 | write:0 | reload:0 | dialog:0",
        flag(config.double_character),
        config.double_hp,
        flag(double_hp_visible(config.double_character)),
        flag(config.change_choice),
        flag(config.free_choose),
        layout.show,
        layout.battle,
        layout.hide,
        layout.width,
        layout.num,
        layout.hidden,
        layout.size,
        first,
        seats.side,
        seats.zhu,
        seats.fan,
        drafted.hidden_ids.join(","),
        drafted.friend.join(","),
        drafted.enemy.join(","),
        zhu.join(","),
        fan.join(","),
        open_zhu,
        open_fan,
        opening_draw("zhu"),
        opening_draw("fan"),
        join_mark,
        script.url,
    ))
}

fn open_label(list: &[String], double_character: bool, join: bool) -> String {
    let Some(name1) = list.first() else {
        return "-".to_string();
    };
    if double_character && join && list.len() > 1 {
        format!("{name1}/{}", list[1])
    } else {
        name1.clone()
    }
}

fn flag(value: bool) -> &'static str {
    if value {
        "1"
    } else {
        "0"
    }
}

fn take_ai_at(
    cards: &[SeatCard],
    ai: &[SeatCard],
    moved: &mut [bool],
    slot: usize,
) -> Result<String, MartialError> {
    let available: Vec<usize> = ai
        .iter()
        .map(|card| card.origin)
        .filter(|origin| !moved[*origin])
        .collect();
    if slot >= available.len() {
        return Err(MartialError::BadConfig("ai".into()));
    }
    let origin = available[slot];
    moved[origin] = true;
    Ok(cards[origin].id.clone())
}

fn take_player_at(
    input: &DraftInput,
    cards: &[SeatCard],
    moved: &mut [bool],
    player_at: &mut usize,
) -> Result<String, MartialError> {
    let origin = if input.player_choices.is_empty() {
        (0..cards.len())
            .find(|origin| !moved[*origin])
            .ok_or_else(|| MartialError::BadConfig("player".into()))?
    } else {
        let id = input
            .player_choices
            .get(*player_at)
            .ok_or_else(|| MartialError::BadConfig("player".into()))?;
        *player_at += 1;
        cards
            .iter()
            .find(|card| !moved[card.origin] && card.id == *id)
            .map(|card| card.origin)
            .ok_or_else(|| MartialError::BadConfig("player".into()))?
    };
    moved[origin] = true;
    Ok(cards[origin].id.clone())
}

fn next_ai_slot(input: &DraftInput, ai_at: &mut usize) -> Result<usize, MartialError> {
    if input.ai_indices.is_empty() {
        Ok(0)
    } else {
        let slot = *input
            .ai_indices
            .get(*ai_at)
            .ok_or_else(|| MartialError::BadConfig("ai".into()))?;
        *ai_at += 1;
        Ok(slot)
    }
}

fn seat_cards(names: &[String], ranks: &[f64], hidden: &[usize]) -> Result<Vec<SeatCard>, MartialError> {
    if names.len() != ranks.len() {
        return Err(MartialError::BadConfig("rank".into()));
    }
    let mut seen = BTreeSet::new();
    for index in hidden {
        if *index >= names.len() || !seen.insert(*index) {
            return Err(MartialError::BadConfig("hidden".into()));
        }
    }
    Ok(names
        .iter()
        .enumerate()
        .map(|(origin, id)| SeatCard {
            id: id.clone(),
            origin,
            given_rank: ranks[origin],
        })
        .collect())
}

fn order_with_hidden(cards: &[SeatCard], hidden: &BTreeSet<usize>) -> Vec<SeatCard> {
    let visible: Vec<f64> = cards
        .iter()
        .filter(|card| !hidden.contains(&card.origin))
        .map(|card| card.given_rank)
        .collect();
    let ordered: Vec<SeatCard> = if visible.is_empty() {
        cards
            .iter()
            .map(|card| SeatCard {
                id: card.id.clone(),
                origin: card.origin,
                given_rank: card.given_rank,
            })
            .collect()
    } else {
        let average = visible.iter().sum::<f64>() / visible.len() as f64;
        let mut ordered: Vec<SeatCard> = cards
            .iter()
            .map(|card| SeatCard {
                id: card.id.clone(),
                origin: card.origin,
                given_rank: if hidden.contains(&card.origin) {
                    average
                } else {
                    card.given_rank
                },
            })
            .collect();
        ordered.sort_by(|left, right| {
            right
                .given_rank
                .partial_cmp(&left.given_rank)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        ordered
    };
    if visible.is_empty() {
        return ordered;
    }
    ordered
}

fn mode_label(mode: &str) -> &str {
    match mode {
        "normal" => "新1v1",
        "changban" => "血战长坂坡",
        "dianjiang" => "点将单挑",
        other => other,
    }
}

fn identity_label(identity: &str) -> &str {
    match identity {
        "zhu" => "先手",
        "fan" => "后手",
        other => other,
    }
}

fn hp_rule(text: &str) -> bool {
    matches!(
        text,
        "hejiansan" | "pingjun" | "zuidazhi" | "zuixiaozhi" | "zonghe"
    )
}

fn required_string(value: &Value, key: &str) -> Result<String, MartialError> {
    match value.get(key) {
        Some(Value::String(text)) if !text.is_empty() => Ok(text.clone()),
        _ => Err(MartialError::BadConfig(key.into())),
    }
}

fn bool_field(value: &Value, key: &str, default: bool) -> Result<bool, MartialError> {
    match value.get(key) {
        None => Ok(default),
        Some(Value::Bool(flag)) => Ok(*flag),
        _ => Err(MartialError::BadConfig(key.into())),
    }
}

fn count_field(value: &Value, key: &str, init: i64, falsy_to_init: bool) -> Result<i64, MartialError> {
    let Some(field) = value.get(key) else {
        return Ok(init);
    };
    let parsed = match field {
        Value::Null => return Ok(init),
        Value::Bool(false) if falsy_to_init => return Ok(init),
        Value::Number(number) => number
            .as_i64()
            .ok_or_else(|| MartialError::BadConfig(key.into()))?,
        Value::String(text) if text.is_empty() && falsy_to_init => return Ok(init),
        Value::String(text) => text
            .parse::<i64>()
            .map_err(|_| MartialError::BadConfig(key.into()))?,
        _ => return Err(MartialError::BadConfig(key.into())),
    };
    if parsed < 0 {
        return Err(MartialError::BadConfig(key.into()));
    }
    if falsy_to_init && parsed == 0 && !matches!(field, Value::String(_)) {
        return Ok(init);
    }
    Ok(parsed)
}
