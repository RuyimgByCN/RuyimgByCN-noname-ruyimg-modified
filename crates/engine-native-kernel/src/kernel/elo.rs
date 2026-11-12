use crate::catalog::{CardKind, Catalog, Suit};
use crate::event::EventKind;
use crate::ids::{CardId, PlayerId, SkillDefId};
use crate::state::{GameMode, GameStateCore, Role};

/// The 5 formal upstream `L22` (`段位制裁`, `upstream-liuli/elo-sanction`) global skills
/// and their primary catalog `EventKind`s.
pub const L22_ELO_SKILLS: [(&str, EventKind); 5] = [
    ("_eloMechanism", EventKind::TurnBegin),
    ("_eloInitCards", EventKind::PhaseBegin),
    ("_eloDraw", EventKind::Draw),
    ("_eloHandcard", EventKind::CardUsed),
    ("_timeKiller", EventKind::PhaseBegin),
];

#[inline]
pub fn is_l22_elo_global_skill(name: &str) -> bool {
    matches!(
        name,
        "_eloMechanism" | "_eloInitCards" | "_eloDraw" | "_eloHandcard" | "_timeKiller"
    )
}

#[inline]
pub fn find_l22_skill_def(catalog: &Catalog, name: &str) -> Option<SkillDefId> {
    catalog.skill_by_name(name)
}

#[inline]
pub fn player_has_l22_skill(
    core: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    name: &str,
) -> bool {
    find_l22_skill_def(catalog, name).is_some_and(|def| core.has_skill(player, def))
}

#[inline]
pub fn elo_get_global_scalar(core: &GameStateCore, key: &str) -> Option<i32> {
    core.vm_aux
        .flags
        .get(&(0, key.to_string()))
        .copied()
        .map(|v| v as i32)
}

#[inline]
pub fn elo_set_global_scalar(core: &mut GameStateCore, key: &str, val: i32) {
    core.vm_aux.flags.insert((0, key.to_string()), val as u32);
}

#[inline]
pub fn elo_get_player_scalar(core: &GameStateCore, player: PlayerId, key: &str) -> Option<i32> {
    core.vm_aux
        .flags
        .get(&(player, key.to_string()))
        .copied()
        .map(|v| v as i32)
}

#[inline]
pub fn elo_set_player_scalar(core: &mut GameStateCore, player: PlayerId, key: &str, val: i32) {
    core.vm_aux
        .flags
        .insert((player, key.to_string()), val as u32);
}

#[inline]
pub fn elo_me_seat(core: &GameStateCore) -> PlayerId {
    let raw = elo_get_global_scalar(core, "elo_me_seat").unwrap_or(0);
    if raw >= 0 && (raw as usize) < core.players.len() {
        raw as PlayerId
    } else {
        0
    }
}

#[inline]
pub fn elo_has_me(core: &GameStateCore) -> bool {
    elo_get_global_scalar(core, "elo_has_me").unwrap_or(1) != 0 && !core.players.is_empty()
}

/// `game.hasInitEloFunction` in `段位制裁/extension.js:13-17`.
#[inline]
pub fn elo_is_init(core: &GameStateCore) -> bool {
    elo_get_global_scalar(core, "elo_has_init_function").unwrap_or(0) != 0
}

#[inline]
fn elo_known_key(card: CardId) -> String {
    format!("elo_known:{card}")
}

#[inline]
pub fn elo_is_card_known(core: &GameStateCore, card: CardId) -> bool {
    core.vm_aux
        .flags
        .get(&(0, elo_known_key(card)))
        .copied()
        .unwrap_or(0)
        != 0
}

#[inline]
pub fn elo_set_card_known(core: &mut GameStateCore, card: CardId, known: bool) {
    let key = (0, elo_known_key(card));
    if known {
        core.vm_aux.flags.insert(key, 1);
    } else {
        core.vm_aux.flags.remove(&key);
    }
}

/// `ui.cardPile.insertBefore` / `appendChild` hook installed by `_eloMechanism` (`extension.js:19-42`).
/// When `game.hasInitEloFunction` is active and `!noMark`, marks `card.isEloKnown = true`.
pub fn elo_mark_deck_insert(core: &mut GameStateCore, card: CardId, no_mark: bool) {
    if elo_is_init(core) && !no_mark {
        elo_set_card_known(core, card, true);
    }
}

/// `game.washCard` hook installed by `_eloMechanism` (`extension.js:44-55`).
/// Clears `isEloKnown` on all cards washed back into the draw pile.
pub fn elo_on_wash_deck(core: &mut GameStateCore, washed_cards: &[CardId]) {
    if elo_is_init(core) {
        for &c in washed_cards {
            elo_set_card_known(core, c, false);
        }
        let prev = elo_get_global_scalar(core, "elo_wash_count").unwrap_or(0);
        elo_set_global_scalar(core, "elo_wash_count", prev.saturating_add(1));
    }
}

/// `_eloMechanism`: `gameStart` trigger (`extension.js:3-57`).
/// Filter: `!game.hasInitEloFunction && player == game.me` (`usable: 1`).
pub fn elo_try_init_mechanism(
    core: &mut GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
) -> bool {
    if !player_has_l22_skill(core, catalog, player, "_eloMechanism") {
        return false;
    }
    if elo_is_init(core) || !elo_has_me(core) || player != elo_me_seat(core) {
        return false;
    }
    elo_set_global_scalar(core, "elo_has_init_function", 1);
    elo_set_player_scalar(core, player, "elo_mechanism_initialized", 1);
    let count = elo_get_player_scalar(core, player, "elo_mechanism_init_count").unwrap_or(0);
    elo_set_player_scalar(
        core,
        player,
        "elo_mechanism_init_count",
        count.saturating_add(1),
    );
    true
}

/// `_eloInitCards`: `enterGame` / `phaseBefore` trigger (`extension.js:59-78`).
/// Filter: `player.countCards('he') > 0 && (event.name != 'phase' || game.phaseNumber == 0)`.
/// Content: marks all cards in `player.getCards('he')` (`hand` + `equips`) as `isEloKnown = true`.
pub fn elo_try_init_cards(
    core: &mut GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    is_phase_event: bool,
) -> usize {
    if !player_has_l22_skill(core, catalog, player, "_eloInitCards") {
        return 0;
    }
    if is_phase_event && core.turn.phase_number != 0 {
        return 0;
    }
    let idx = player as usize;
    if idx >= core.players.len() {
        return 0;
    }
    let mut he_cards: Vec<CardId> = Vec::new();
    he_cards.extend(core.players[idx].hand.iter().copied());
    he_cards.extend(core.players[idx].equips.iter().copied());
    if he_cards.is_empty() {
        return 0;
    }
    for &c in &he_cards {
        elo_set_card_known(core, c, true);
    }
    let marked = he_cards.len();
    elo_set_player_scalar(core, player, "elo_init_cards_count", marked as i32);
    marked
}

/// Called on match initialization (`gameStart` + initial `enterGame` for all seats).
pub fn elo_on_game_start(core: &mut GameStateCore, catalog: &Catalog) {
    let seats = core.players.len();
    for s in 0..seats {
        let p = s as PlayerId;
        let _ = elo_try_init_mechanism(core, catalog, p);
    }
    for s in 0..seats {
        let p = s as PlayerId;
        let _ = elo_try_init_cards(core, catalog, p, false);
    }
}

/// Called at `phaseBefore` (before `core.turn.phase_number` increments for the phase).
pub fn elo_on_phase_before(core: &mut GameStateCore, catalog: &Catalog, player: PlayerId) {
    let _ = elo_try_init_mechanism(core, catalog, player);
    let _ = elo_try_init_cards(core, catalog, player, true);
}

/// `game.eloGetDifficulty(true)` (`extension.js:139-180`):
/// Returns `eh` (`1` for `eeasy`/`veasy`/`easy` or low `paiwei` rank, `-1` for `hard`/`vhard`/`ehard` or high `paiwei` rank, `0` for `normal`/offline/`taixuhuanjing`).
#[inline]
pub fn elo_eh_mode(core: &GameStateCore) -> i32 {
    let raw = elo_get_global_scalar(core, "elo_eh")
        .or_else(|| elo_get_global_scalar(core, "elo_players_mode"))
        .unwrap_or(0);
    match raw {
        -1 => -1,
        1 => 1,
        _ => 0,
    }
}

/// `game.eloGetDifficulty(false)` (`extension.js:139-180`):
/// Returns `diff` (`1..=4`, or custom positive multiplier).
#[inline]
pub fn elo_difficulty_multiplier(core: &GameStateCore) -> i32 {
    elo_get_global_scalar(core, "elo_difficulty_val")
        .unwrap_or(1)
        .max(1)
}

/// Determines whether `player` and `other` are friends (`player.isFriendsOf(other)` in `extension.js:245,276`).
pub fn elo_is_friends_of(core: &GameStateCore, player: PlayerId, other: PlayerId) -> bool {
    if player == other {
        return true;
    }
    if core
        .vm_aux
        .flags
        .get(&(other, format!("elo_true_friend:{player}")))
        .copied()
        .unwrap_or(0)
        != 0
    {
        return true;
    }
    if let Some(explicit) = core
        .vm_aux
        .flags
        .get(&(player, format!("elo_is_friend:{other}")))
        .copied()
    {
        return explicit != 0;
    }
    let p_idx = player as usize;
    let o_idx = other as usize;
    if p_idx >= core.players.len() || o_idx >= core.players.len() {
        return false;
    }
    match &core.mode {
        GameMode::Identity { roles } => {
            if p_idx < roles.len() && o_idx < roles.len() {
                match (roles[p_idx], roles[o_idx]) {
                    (Role::Lord | Role::Loyalist, Role::Lord | Role::Loyalist) => true,
                    (Role::Rebel, Role::Rebel) => true,
                    _ => false,
                }
            } else {
                false
            }
        }
        GameMode::Versus { teams, .. } => teams
            .iter()
            .any(|team| team.contains(&player) && team.contains(&other)),
        GameMode::Doudizhu { lord } => (player != *lord) && (other != *lord),
        GameMode::Boss { boss, fellows, .. } => {
            let p_boss_side = player == *boss || fellows.contains(&player);
            let o_boss_side = other == *boss || fellows.contains(&other);
            p_boss_side == o_boss_side
        }
        _ => false,
    }
}

/// Card heuristic value matching `get.value(card, player)` in `_eloDraw` (`extension.js:250-252`)
/// and `_eloHandcard` (`extension.js:264-287`).
pub fn elo_card_value(
    core: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    card: CardId,
) -> i32 {
    if let Some(v) = elo_get_player_scalar(core, player, &format!("elo_card_val:{card}")) {
        return v;
    }
    if let Some(v) = elo_get_global_scalar(core, &format!("elo_card_val:{card}")) {
        return v;
    }
    let Some(meta) = catalog.cards.get(card as usize) else {
        return 50;
    };
    match meta.kind {
        CardKind::Tao => 90,
        CardKind::Snatch => 80,
        CardKind::Dismantle => 78,
        CardKind::Nullify => 76,
        CardKind::SavageAttack | CardKind::ArcheryAttack => 74,
        CardKind::Duel => 72,
        CardKind::Le => 70,
        CardKind::Bing => 68,
        CardKind::Jiu => 65,
        CardKind::Shan => 62,
        CardKind::Sha | CardKind::FireSha | CardKind::LeiSha | CardKind::ShouLiJian => 60,
        CardKind::Weapon
        | CardKind::Armor
        | CardKind::HorsePlus
        | CardKind::HorseMinus
        | CardKind::WuxingPan
        | CardKind::Sanjian => 58,
        CardKind::Chain => 55,
        CardKind::ShanDian => 30,
        _ => match meta.name {
            "wuzhong" => 85,
            "tao" => 90,
            "shunshou" => 80,
            "guohe" => 78,
            "wuxie" => 76,
            _ => 56,
        },
    }
}

/// Judge heuristic value matching `trigger.judge(card)` in `_eloDraw` (`extension.js:252-254`).
pub fn elo_judge_score(
    core: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    trick: CardKind,
    card: CardId,
) -> i32 {
    if let Some(v) = elo_get_player_scalar(core, player, &format!("elo_judge_val:{card}")) {
        return v;
    }
    if let Some(v) = elo_get_global_scalar(core, &format!("elo_judge_val:{card}")) {
        return v;
    }
    let Some(meta) = catalog.cards.get(card as usize) else {
        return 0;
    };
    match trick {
        // 乐不思蜀: Heart is favorable (+20), non-Heart skips Play phase (-20).
        CardKind::Le => {
            if meta.suit == Suit::Heart {
                20
            } else {
                -20
            }
        }
        // 兵粮寸断: Club is favorable (+20), non-Club skips Draw phase (-20).
        CardKind::Bing => {
            if meta.suit == Suit::Club {
                20
            } else {
                -20
            }
        }
        // 闪电: Spade 2..=9 deals 3 thunder damage (-50), anything else is safe (+20).
        CardKind::ShanDian => {
            if meta.suit == Suit::Spade && (2..=9).contains(&meta.rank) {
                -50
            } else {
                20
            }
        }
        _ => elo_card_value(core, catalog, player, card),
    }
}

/// Reorders the top `window` cards of `core.deck` (`core.deck.pop()` draws from the end of `core.deck`)
/// according to `game.orderEloCards` + `_eloDraw` (`extension.js:117-138, 249-261`):
/// - Let `top_cards[0]` be the top-most card of the deck (`core.deck[len - 1]`), ...,
///   `top_cards[n - 1]` be `core.deck[len - n]`.
/// - Cards with `elo_is_card_known(core, c)` stay locked at their exact `top_cards[i]` slot.
/// - Unknown cards are ordered by `order(a, b) * (is_enemy ? -1 : 1)` and reversed when `eh == -1`:
///   - `eh == 1`, `!is_enemy` (friend): descending by score (best card at `top_cards[0]`).
///   - `eh == 1`, `is_enemy` (enemy): ascending by score (worst card at `top_cards[0]`).
///   - `eh == -1`, `!is_enemy` (friend/`game.me`): descending then `.reverse()` (worst card at `top_cards[0]`).
///   - `eh == -1`, `is_enemy` (enemy): ascending then `.reverse()` (best card at `top_cards[0]`).
/// - Uses `noMark = true` (`eloInsertBefore(..., true)`): reordered cards do NOT get marked as `isEloKnown`.
fn elo_reorder_deck_top<F>(
    core: &mut GameStateCore,
    window: usize,
    is_enemy: bool,
    eh: i32,
    mut score_fn: F,
) -> usize
where
    F: FnMut(&GameStateCore, CardId) -> i32,
{
    let deck_len = core.deck.len();
    let n = window.min(deck_len);
    if n <= 1 {
        return 0;
    }
    let mut top_cards: Vec<CardId> = (0..n).map(|i| core.deck[deck_len - 1 - i]).collect();
    let unknown_idx: Vec<usize> = top_cards
        .iter()
        .enumerate()
        .filter(|(_, &c)| !elo_is_card_known(core, c))
        .map(|(i, _)| i)
        .collect();
    if unknown_idx.is_empty() {
        return n;
    }
    let mut unknown_scored: Vec<(CardId, i32)> = unknown_idx
        .iter()
        .map(|&i| {
            let c = top_cards[i];
            (c, score_fn(core, c))
        })
        .collect();
    unknown_scored.sort_by(|a, b| {
        if is_enemy {
            a.1.cmp(&b.1)
        } else {
            b.1.cmp(&a.1)
        }
    });
    if eh == -1 {
        unknown_scored.reverse();
    }
    for (slot, (c, _)) in unknown_idx.into_iter().zip(unknown_scored) {
        top_cards[slot] = c;
    }
    for (i, c) in top_cards.into_iter().enumerate() {
        core.deck[deck_len - 1 - i] = c;
    }
    n
}

/// `_eloDraw` (`drawBegin` branch, `extension.js:222-263`).
/// Filter: `eh != 0 && event.num >= 1 && cardPile.childElementCount > 1 && game.me`.
/// Content: `if (diff > 1)`, reorders top `min(deck.len(), requested * diff)` cards.
pub fn elo_on_draw_begin(
    core: &mut GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    requested: u32,
) -> usize {
    if requested < 1 || core.deck.len() <= 1 {
        return 0;
    }
    if !player_has_l22_skill(core, catalog, player, "_eloDraw") {
        return 0;
    }
    let eh = elo_eh_mode(core);
    if eh == 0 || !elo_has_me(core) {
        return 0;
    }
    let diff = elo_difficulty_multiplier(core);
    if diff <= 1 {
        return 0;
    }
    let me = elo_me_seat(core);
    let is_enemy = !elo_is_friends_of(core, player, me);
    let window = (requested as usize).saturating_mul(diff as usize);
    let sorted = elo_reorder_deck_top(core, window, is_enemy, eh, |st, c| {
        elo_card_value(st, catalog, player, c)
    });
    if sorted > 0 {
        elo_set_player_scalar(core, player, "elo_draw_last_window", sorted as i32);
        let triggers = elo_get_player_scalar(core, player, "elo_draw_triggers").unwrap_or(0);
        elo_set_player_scalar(
            core,
            player,
            "elo_draw_triggers",
            triggers.saturating_add(1),
        );
    }
    sorted
}

/// `_eloDraw` (`judgeBegin` branch, `extension.js:222-263`).
/// Filter: `eh != 0 && typeof event.judge == 'function' && cardPile.childElementCount > 1 && game.me`.
/// Content: `if (diff > 1)`, reorders top `min(deck.len(), diff)` cards by `trigger.judge`.
pub fn elo_on_judge_begin(
    core: &mut GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    trick: CardKind,
) -> usize {
    if core.deck.len() <= 1 {
        return 0;
    }
    if elo_get_player_scalar(core, player, "elo_judge_no_fn").unwrap_or(0) != 0 {
        return 0;
    }
    if !player_has_l22_skill(core, catalog, player, "_eloDraw") {
        return 0;
    }
    let eh = elo_eh_mode(core);
    if eh == 0 || !elo_has_me(core) {
        return 0;
    }
    let diff = elo_difficulty_multiplier(core);
    if diff <= 1 {
        return 0;
    }
    let me = elo_me_seat(core);
    let is_enemy = !elo_is_friends_of(core, player, me);
    let window = diff as usize;
    let sorted = elo_reorder_deck_top(core, window, is_enemy, eh, |st, c| {
        elo_judge_score(st, catalog, player, trick, c)
    });
    if sorted > 0 {
        elo_set_player_scalar(core, player, "elo_judge_last_window", sorted as i32);
        let triggers = elo_get_player_scalar(core, player, "elo_judge_triggers").unwrap_or(0);
        elo_set_player_scalar(
            core,
            player,
            "elo_judge_triggers",
            triggers.saturating_add(1),
        );
    }
    sorted
}

#[inline]
fn elo_is_under_control(core: &GameStateCore, player: PlayerId) -> bool {
    elo_get_player_scalar(core, player, "elo_under_control").unwrap_or(0) != 0
}

/// `_eloHandcard` (`gainPlayerCardBegin` / `discardPlayerCardBegin` trigger, `extension.js:264-287`).
/// Filter:
/// - `game.eloGetDifficulty(true) != 0`
/// - `game.me && player != game.me && !player.isUnderControl(true)`
/// - `game.eloGetDifficulty() >= 3`
/// - `if (game.eloGetDifficulty() < 4 && [1,0,0].randomGet() != 1) return false;`
/// - `(eh == 1 && isFriend) || (eh == -1 && !isFriend)`
/// Content: `trigger.visible = true`.
pub fn elo_on_extract_player_card_begin(
    core: &mut GameStateCore,
    catalog: &Catalog,
    source: PlayerId,
    target: PlayerId,
) -> bool {
    if !player_has_l22_skill(core, catalog, source, "_eloHandcard") {
        return false;
    }
    let eh = elo_eh_mode(core);
    if eh == 0 || !elo_has_me(core) {
        elo_set_player_scalar(core, source, "elo_handcard_visible", 0);
        return false;
    }
    let me = elo_me_seat(core);
    if source == me || elo_is_under_control(core, source) {
        elo_set_player_scalar(core, source, "elo_handcard_visible", 0);
        return false;
    }
    let diff = elo_difficulty_multiplier(core);
    if diff < 3 {
        elo_set_player_scalar(core, source, "elo_handcard_visible", 0);
        return false;
    }
    if diff < 4 {
        // `[1, 0, 0].randomGet() == 1` (1/3 chance; deterministic override via `elo_handcard_roll` = 1 or 0, or `elo_handcard_roll_pct <= 33`).
        let pass = if let Some(roll) = elo_get_player_scalar(core, source, "elo_handcard_roll") {
            roll == 1
        } else if let Some(roll_pct) = elo_get_player_scalar(core, source, "elo_handcard_roll_pct")
        {
            roll_pct <= 33
        } else {
            core.rng.gen_range(3) == 0
        };
        if !pass {
            elo_set_player_scalar(core, source, "elo_handcard_visible", 0);
            return false;
        }
    }
    let is_friend = elo_is_friends_of(core, source, me);
    let visible = (eh == 1 && is_friend) || (eh == -1 && !is_friend);
    elo_set_player_scalar(core, source, "elo_handcard_visible", i32::from(visible));
    if visible {
        let count = elo_get_player_scalar(core, source, "elo_handcard_triggers").unwrap_or(0);
        elo_set_player_scalar(
            core,
            source,
            "elo_handcard_triggers",
            count.saturating_add(1),
        );
        elo_set_player_scalar(core, source, "elo_handcard_last_target", target as i32);
    }
    visible
}

/// Picks the best hand card from `target`'s hand when `_eloHandcard` makes hand cards visible (`trigger.visible = true`).
pub fn elo_best_hand_card(
    core: &GameStateCore,
    catalog: &Catalog,
    target: PlayerId,
    hand: &[CardId],
) -> Option<CardId> {
    hand.iter()
        .copied()
        .max_by_key(|&c| elo_card_value(core, catalog, target, c))
}

/// `game.eloTimeKillerType()` (`extension.js:288-306`).
/// Returns `0` (`none` or `offline != 'online'`), `1` (`light`), `2` (`middle`), or `3` (`heavy`).
pub fn elo_time_killer_type(core: &GameStateCore) -> i32 {
    let online = elo_get_global_scalar(core, "elo_online").unwrap_or(1) != 0;
    if !online {
        return 0;
    }
    let raw = elo_get_global_scalar(core, "elo_time_killer_val").unwrap_or(0);
    match raw {
        1..=3 => raw,
        _ => 0,
    }
}

/// `lib.skill._timeKiller.isMine(player)` (`extension.js:319-322`).
#[inline]
fn elo_time_killer_is_mine(core: &GameStateCore, player: PlayerId) -> bool {
    let not_me = elo_get_global_scalar(core, "elo_not_me").unwrap_or(0) != 0;
    if not_me {
        return false;
    }
    if elo_has_me(core) && player == elo_me_seat(core) {
        return true;
    }
    elo_is_under_control(core, player)
}

#[inline]
fn elo_is_choice_before_prompt(prompt: &str) -> bool {
    matches!(
        prompt,
        "phase_use_choose_to_use"
            | "choose_to_use"
            | "choose_to_respond"
            | "choose_to_discard"
            | "choose_card"
            | "choose_target"
            | "choose_card_target"
            | "choose_control"
            | "choose_bool"
            | "choose_player_card"
            | "respond_for_sha"
            | "respond_for_shan"
            | "respond_for_peach"
            | "respond_for_wuxie"
            | "discard_cards"
            | "dismantle_choose"
            | "snatch_choose"
    ) || prompt.starts_with("choose_")
        || prompt.starts_with("respond_")
        || prompt.starts_with("discard")
        || prompt.starts_with("vm_")
}

#[inline]
fn elo_is_female_general(core: &GameStateCore, player: PlayerId) -> bool {
    if let Some(sex) = elo_get_player_scalar(core, player, "elo_sex_female") {
        return sex != 0;
    }
    let idx = player as usize;
    if idx >= core.players.len() {
        return false;
    }
    matches!(
        core.players[idx].general_slots[0],
        Some(7 | 12 | 19 | 21 | 22 | 25)
    )
}

/// `_timeKiller` (`chooseToUseBefore` / `chooseToRespondBefore` / `chooseToDiscardBefore`, `extension.js:307-458`).
/// Filter: `game.eloTimeKillerType() != 0 && !lib.skill._timeKiller.isMine(player)`.
/// Content:
/// - Computes `ran` (`0..=1000` permille) and `delay_ms = (tk_type * 1500 * ran) / 1000`.
/// - When `zuan` is enabled and `ran >= 750`, selects female or male thinking line.
pub fn elo_on_choice_before(
    core: &mut GameStateCore,
    catalog: &Catalog,
    observer: PlayerId,
    prompt: &str,
) -> Option<i32> {
    if !elo_is_choice_before_prompt(prompt) {
        return None;
    }
    if !player_has_l22_skill(core, catalog, observer, "_timeKiller") {
        return None;
    }
    let tk_type = elo_time_killer_type(core);
    if tk_type == 0 || elo_time_killer_is_mine(core, observer) {
        return None;
    }
    let ran_permille = elo_get_player_scalar(core, observer, "elo_time_killer_ran_permille")
        .unwrap_or_else(|| core.rng.gen_range(1000) as i32)
        .clamp(0, 1000);
    let delay_ms = (tk_type * 1500 * ran_permille) / 1000;
    elo_set_player_scalar(core, observer, "elo_time_killer_delay_ms", delay_ms);
    let count = elo_get_player_scalar(core, observer, "elo_time_killer_triggers").unwrap_or(0);
    elo_set_player_scalar(
        core,
        observer,
        "elo_time_killer_triggers",
        count.saturating_add(1),
    );

    let zuan = elo_get_global_scalar(core, "elo_zuan_enable").unwrap_or(0) != 0;
    if zuan && ran_permille >= 750 {
        let chat_idx = elo_get_player_scalar(core, observer, "elo_time_killer_chat_idx")
            .unwrap_or_else(|| (core.rng.gen_range(3) as i32) + 1)
            .clamp(1, 3);
        if elo_is_female_general(core, observer) {
            elo_set_player_scalar(core, observer, "elo_time_killer_chat_female", chat_idx);
        } else {
            elo_set_player_scalar(core, observer, "elo_time_killer_chat_male", chat_idx);
        }
    }
    Some(delay_ms)
}
