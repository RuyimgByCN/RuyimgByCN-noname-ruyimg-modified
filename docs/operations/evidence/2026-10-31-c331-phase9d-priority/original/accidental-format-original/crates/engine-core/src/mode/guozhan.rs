//! Guozhan mode (国战局) rules — Phase B skeleton (ADR 0598 §6).
//!
//! Implements the state-model pieces that differ from identity mode:
//! - double-general dealing (主将 + 副将);
//! - faction (势力) determination from the general pair;
//! - faction-based victory (`GroupWin`).
//!
//! Hidden-general (暗将) mechanics, 珠联璧合, and 野心家 (ambition lord) nation-building
//! are deferred to phase C; the skeleton here treats both generals as revealed (`unseen`
//! defaults to `[false, false]`) and computes faction from the pair directly.

use crate::state::{GameState, Identity, Player};
use engine_api::{MatchOutcome, PlayerId};

/// Faction keys recognised by guozhan.
pub const FACTIONS: &[&str] = &["wei", "shu", "wu", "qun", "jin"];

/// Faction key for 野心家 (ambition lord) — used when a pair spans two different factions.
pub const AMBITION: &str = "ye";

/// Determine the faction for a general pair.
///
/// Per upstream `mode/guozhan/src/patch/player.js`:
/// - same faction → that faction;
/// - different factions → 野心家 (`ye`).
///
/// A `None` general (def id 0, i.e. unset) contributes no faction; if both are unset the
/// result is `None` (caller treats as unresolved).
pub fn compute_group_from_pair(a: Option<&str>, b: Option<&str>) -> Option<String> {
    match (a.filter(|s| !s.is_empty()), b.filter(|s| !s.is_empty())) {
        (Some(g1), Some(g2)) if g1 == g2 => Some(g1.to_string()),
        (Some(_), Some(_)) => Some(AMBITION.to_string()),
        (Some(g), None) | (None, Some(g)) => Some(g.to_string()),
        (None, None) => None,
    }
}

/// Guozhan max-HP: sum of the two generals' max-HP values (upstream: 主将体力上限 + 副将体力上限).
/// Phase C will add the 珠联璧合 +1 bonus and per-character adjustments.
pub fn guozhan_max_hp(main_max: u32, lieutenant_max: u32) -> u32 {
    main_max.saturating_add(lieutenant_max)
}

/// Mark definition id for the 珠联璧合 (perfect pair) boon. Stored on the player's `marks`
/// list as `(ZHULIAN_BIHE_MARK, 1)`. The boon may be spent as a 无中生有 effect or to recover
/// 1 HP when dying (upstream `zhulianbihe_mark`); the spend effects land with phase-C
/// content alongside guozhan card migration.
pub const ZHULIAN_BIHE_MARK: u32 = 0x5A4C_4842; // "ZLHB"

/// Whether the main/lieutenant pair is a 珠联璧合 (perfect pair), per upstream
/// `perfectPairs.js`. The lookup is bidirectional and symmetric: either general appearing
/// as the key with the other in its mate list counts.
pub fn is_perfect_pair(main_name: &str, lieutenant_name: &str) -> bool {
    PERFECT_PAIRS
        .iter()
        .find(|(k, _)| *k == main_name)
        .is_some_and(|(_, mates)| mates.iter().any(|m| *m == lieutenant_name))
        || PERFECT_PAIRS
            .iter()
            .find(|(k, _)| *k == lieutenant_name)
            .is_some_and(|(_, mates)| mates.iter().any(|m| *m == main_name))
}

/// Count alive players per faction. 野心家 (`ye`) is its own bucket.
pub fn faction_population(state: &GameState) -> std::collections::BTreeMap<String, u32> {
    let mut pop = std::collections::BTreeMap::new();
    for p in state.players.iter_live().filter(|p| p.alive) {
        if let Some(g) = state.player_group(p.id) {
            *pop.entry(g.to_string()).or_insert(0) += 1;
        }
    }
    pop
}

/// Total alive player count.
pub fn total_population(state: &GameState) -> u32 {
    state.players.iter_live().filter(|p| p.alive).count() as u32
}

/// Whether `player` should become (or stay) an 野心家 (ambition lord).
///
/// Upstream `wontYe` (inverted): a player becomes 野心家 when their faction is the
/// *minority* — specifically when the faction's alive count is more than half of the total
/// alive count would leave no room for the player's faction to be the sole survivor. The
/// simplified rule used here: a faction whose population is **strictly more than half** the
/// total alive count is the dominant faction; any other faction's member may flip to 野心家.
///
/// This is evaluated when a general is revealed (明置); the caller updates `group_overrides`
/// to `AMBITION` when it returns true. Nation-building (`拉拢人心`/`yexinjia_friend` alliance)
/// is deferred — it needs an ABI-backed choice event and an alliance graph.
pub fn should_become_ambition(state: &GameState, player: PlayerId) -> bool {
    let Some(group) = state.player_group(player) else {
        return false;
    };
    if group == AMBITION {
        return false; // already ambition
    }
    let total = total_population(state);
    if total == 0 {
        return false;
    }
    let pop = faction_population(state);
    let my_count = pop.get(group).copied().unwrap_or(0);
    // Dominant faction: alive count > half the total.
    let dominant_threshold = total / 2;
    pop.values()
        .any(|&c| c > dominant_threshold && c > my_count)
}

/// Reveal (明置) one of `player`'s face-down generals, then re-resolve faction.
///
/// After flipping `unseen[slot]` to false, the faction is recomputed from both generals'
/// base factions (via `compute_group_from_pair`) and written into `group_overrides`. If
/// the player's faction is now a minority under a dominant faction, they flip to 野心家
/// (`AMBITION`). Returns `Err` if the slot is out of range or already revealed.
///
/// This is the entry point behind `CommandKind::RevealGeneral`; identity matches never
/// call it.
pub fn reveal_general(state: &mut GameState, player: PlayerId, slot: u8) -> Result<(), String> {
    // Read character factions first to avoid borrowing state.players mutably twice.
    let (main_def, lt_def) = state
        .players
        .get(player)
        .map(|p| (p.character_def, p.character_def2))
        .ok_or_else(|| "reveal: unknown player".to_string())?;
    let main_group = state.content.character(main_def).map(|c| c.group.clone());
    let lt_group = if lt_def != 0 {
        state.content.character(lt_def).map(|c| c.group.clone())
    } else {
        None
    };

    // Flip the slot face-up.
    let p = state
        .players
        .get_mut(player)
        .ok_or_else(|| "reveal: unknown player".to_string())?;
    let idx = usize::try_from(slot).map_err(|_| "reveal: bad slot")?;
    if idx >= p.unseen.len() {
        return Err("reveal: slot out of range".to_string());
    }
    if !p.unseen[idx] {
        return Err("reveal: general already face-up".to_string());
    }
    p.unseen[idx] = false;
    let unseen = p.unseen;

    // Re-resolve faction from revealed generals only.
    let resolved = match (unseen[0], unseen[1]) {
        (false, false) => compute_group_from_pair(main_group.as_deref(), lt_group.as_deref()),
        (false, true) => main_group,
        (true, false) => lt_group,
        (true, true) => None,
    };

    // If minority under a dominant faction, become ambition.
    let final_group = match resolved {
        Some(g) if should_become_ambition(state, player) && g != AMBITION => {
            Some(AMBITION.to_string())
        }
        other => other,
    };
    match final_group {
        Some(g) => {
            state.group_overrides.insert(player, g);
        }
        None => {
            state.group_overrides.remove(&player);
        }
    }
    Ok(())
}

/// Whether `a` and `b` are ambition-lord allies (either direction). Identity mode has no
/// alliances, so this is always false there.
pub fn are_allies(state: &GameState, a: PlayerId, b: PlayerId) -> bool {
    let (lo, hi) = if a.raw() <= b.raw() {
        (a.raw(), b.raw())
    } else {
        (b.raw(), a.raw())
    };
    state.ambition_alliances.contains(&[lo, hi])
}

/// Whether two alive players belong to the same 国战「方」(side): same non-ambition faction,
/// or an ambition alliance. Ambition players are their own side unless allied.
pub fn same_side(state: &GameState, a: PlayerId, b: PlayerId) -> bool {
    if a == b {
        return true;
    }
    let ga = state.player_group(a).map(str::to_string);
    let gb = state.player_group(b).map(str::to_string);
    if ga.as_deref() == Some(AMBITION) || gb.as_deref() == Some(AMBITION) {
        return are_allies(state, a, b);
    }
    (ga.is_some() && ga == gb) || are_allies(state, a, b)
}

/// 大势力判定（国战 lulitongxin 等）：按「同势力或野心联盟」分组，本方人数 ≥ 2 且
/// 不少于任何其他方即为大势力（上游 isMajor：本方 ≥ 2 且无人多于本方）。
pub fn is_major_faction_player(state: &GameState, player: PlayerId) -> bool {
    let alive: Vec<_> = state
        .players
        .iter_live()
        .filter(|p| p.alive)
        .map(|p| p.id)
        .collect();
    if alive.len() < 2 {
        return false;
    }
    let mut groups: Vec<Vec<PlayerId>> = Vec::new();
    for &p in &alive {
        if let Some(group) = groups
            .iter_mut()
            .find(|group| group.iter().any(|&m| same_side(state, p, m)))
        {
            group.push(p);
        } else {
            groups.push(vec![p]);
        }
    }
    let my_size = groups
        .iter()
        .find(|group| group.contains(&player))
        .map(|group| group.len())
        .unwrap_or(0);
    if my_size < 2 {
        return false;
    }
    let max_size = groups.iter().map(|group| group.len()).max().unwrap_or(0);
    my_size >= max_size
}

/// Form a bidirectional ambition-lord alliance between `ambition` and `target`.
/// Stored canonically as the sorted `[min, max]` pair so lookups are order-independent.
pub fn form_alliance(state: &mut GameState, ambition: PlayerId, target: PlayerId) {
    if ambition == target || ambition.is_none() || target.is_none() {
        return;
    }
    let (lo, hi) = if ambition.raw() <= target.raw() {
        (ambition.raw(), target.raw())
    } else {
        (target.raw(), ambition.raw())
    };
    state.ambition_alliances.insert([lo, hi]);
}

/// Continuation payload marker for the 拉拢人心 (win hearts) choice.
/// Stored in `payload[0]`; the full payload is `[LOUREN_XIN_CHOICE, ambition.raw(), target.raw()]`.
pub const LOUREN_XIN_CHOICE: u32 = 0x5A4C_5258; // "ZLRX"

/// Resolve the 拉拢人心 choice: the ambition lord (`observer`) either selects a target
/// player to ally with (the first selection token is the target PlayerId) or cancels.
/// On a valid selection the alliance is recorded bidirectionally via [`form_alliance`].
pub fn resolve_lourenxin_choice(
    state: &mut GameState,
    observer: PlayerId,
    selections: &[u32],
    cancelled: bool,
) {
    if cancelled || observer.is_none() {
        return;
    }
    let Some(&target_raw) = selections.first() else {
        return;
    };
    let target = PlayerId::new(target_raw);
    if target.is_none() || target == observer {
        return;
    }
    if state.players.get(target).is_some_and(|p| p.alive) {
        form_alliance(state, observer, target);
    }
}

/// Evaluate guozhan victory from the alive players' resolved factions.
///
/// A faction wins when it is the only faction with alive players (all other factions
/// eliminated). 野心家 (`ye`) counts as its own faction for this check. Allied players
/// (via 拉拢人心) are treated as the same camp: if all alive players belong to one camp
/// (faction or alliance graph component), that camp wins.
pub fn check_guozhan_victory(state: &GameState) -> MatchOutcome {
    if state.outcome.is_finished() {
        return state.outcome.clone();
    }

    let alive: Vec<PlayerId> = state
        .players
        .iter_live()
        .filter(|p| p.alive)
        .map(|p| p.id)
        .collect();
    if alive.is_empty() {
        return MatchOutcome::Draw;
    }

    // Partition alive players into camps by faction, merging allied players.
    let camps = alliance_camps(state, &alive);
    if camps.len() == 1 {
        // Exactly one camp remains — report its faction label.
        let label = camps[0]
            .iter()
            .find_map(|p| state.player_group(*p).map(str::to_string))
            .unwrap_or_else(|| AMBITION.to_string());
        MatchOutcome::GroupWin { group: label }
    } else {
        MatchOutcome::InProgress
    }
}

/// Group alive players into camps: same faction is one camp; allied players (拉拢人心)
/// merge into a single camp regardless of faction. Returns one `Vec<PlayerId>` per camp.
fn alliance_camps(state: &GameState, alive: &[PlayerId]) -> Vec<Vec<PlayerId>> {
    use std::collections::{BTreeMap, BTreeSet};
    // Union-find over alive players keyed by index.
    let idx: BTreeMap<PlayerId, usize> = alive.iter().enumerate().map(|(i, &p)| (p, i)).collect();
    let mut parent: Vec<usize> = (0..alive.len()).collect();
    fn find(parent: &mut Vec<usize>, x: usize) -> usize {
        if parent[x] != x {
            parent[x] = find(parent, parent[x]);
        }
        parent[x]
    }
    // Merge by alliance edges.
    for pair in &state.ambition_alliances {
        let a = PlayerId::new(pair[0]);
        let b = PlayerId::new(pair[1]);
        if let (Some(&ia), Some(&ib)) = (idx.get(&a), idx.get(&b)) {
            let ra = find(&mut parent, ia);
            let rb = find(&mut parent, ib);
            parent[ra] = rb;
        }
    }
    // Merge by same faction.
    let mut faction_members: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
    for (i, &p) in alive.iter().enumerate() {
        if let Some(g) = state.player_group(p) {
            faction_members.entry(g).or_default().push(i);
        }
    }
    for (_, members) in faction_members {
        for w in members.windows(2) {
            let ra = find(&mut parent, w[0]);
            let rb = find(&mut parent, w[1]);
            parent[ra] = rb;
        }
    }
    // Collect camps.
    let mut groups: BTreeMap<usize, Vec<PlayerId>> = BTreeMap::new();
    let mut seen: BTreeSet<usize> = BTreeSet::new();
    for i in 0..alive.len() {
        let r = find(&mut parent, i);
        groups.entry(r).or_default().push(alive[i]);
        seen.insert(r);
    }
    groups.into_values().collect()
}

/// Apply guozhan victory into working state if finished; returns domain event if any.
///
/// Mirrors `identity::apply_victory_if_any` for the guozhan victory path.
pub fn apply_guozhan_victory_if_any(state: &mut GameState) -> Option<engine_api::DomainEvent> {
    let outcome = check_guozhan_victory(state);
    if outcome.is_finished() {
        state.outcome = outcome.clone();
        Some(engine_api::DomainEvent::MatchEnded { outcome })
    } else {
        None
    }
}

/// Build a guozhan player with the given general pair.
///
/// Both generals start face-down (`unseen = [true, true]`), per guozhan rules: players reveal
/// generals on their own initiative (via `RevealGeneral`). The faction is written into
/// `group_overrides` by the caller after allocation (it needs the player id); a player with
/// both generals hidden has no resolved faction until they reveal.
pub fn guozhan_player(seat_index: u8, main_def: u32, lieutenant_def: u32, max_hp: u32) -> Player {
    Player {
        id: PlayerId::NONE,
        seat: engine_api::SeatIndex::new(seat_index),
        hp: max_hp as i32,
        max_hp,
        alive: true,
        dying: false,
        turned_over: false,
        // Guozhan has no identity-mode roles; use Lord as a neutral placeholder.
        identity: Identity::Lord,
        character_def: main_def,
        character_def2: lieutenant_def,
        skills: Vec::new(),
        marks: Vec::new(),
        disabled_equip_slots: [0; 5],
        unseen: [true, true],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: allocate an alive player with a faction override.
    fn make_player(state: &mut GameState, group: Option<&str>) -> PlayerId {
        let p = state.players.alloc(crate::state::Player {
            id: PlayerId::NONE,
            seat: engine_api::SeatIndex::new(0),
            hp: 4,
            max_hp: 4,
            alive: true,
            dying: false,
            turned_over: false,
            identity: Identity::Lord,
            character_def: 0,
            character_def2: 0,
            skills: Vec::new(),
            marks: Vec::new(),
            disabled_equip_slots: [0; 5],
            unseen: [false, false],
        });
        if let Some(g) = group {
            state.group_overrides.insert(p, g.to_string());
        }
        p
    }

    #[test]
    fn same_faction_pair_keeps_faction() {
        assert_eq!(
            compute_group_from_pair(Some("wei"), Some("wei")),
            Some("wei".to_string())
        );
    }

    #[test]
    fn different_faction_pair_becomes_ambition() {
        assert_eq!(
            compute_group_from_pair(Some("wei"), Some("shu")),
            Some("ye".to_string())
        );
    }

    #[test]
    fn one_unset_general_keeps_other_faction() {
        assert_eq!(
            compute_group_from_pair(Some("wu"), None),
            Some("wu".to_string())
        );
        assert_eq!(
            compute_group_from_pair(None, Some("qun")),
            Some("qun".to_string())
        );
    }

    #[test]
    fn both_unset_is_unresolved() {
        assert_eq!(compute_group_from_pair(None, None), None);
    }

    #[test]
    fn empty_string_treated_as_unset() {
        assert_eq!(
            compute_group_from_pair(Some(""), Some("jin")),
            Some("jin".to_string())
        );
    }

    #[test]
    fn max_hp_sums_two_generals() {
        assert_eq!(guozhan_max_hp(3, 4), 7);
        assert_eq!(guozhan_max_hp(2, 2), 4);
    }

    #[test]
    fn perfect_pair_detected_bidirectionally() {
        // caocao pairs with xuzhu per perfectPairs.js.
        assert!(is_perfect_pair("caocao", "xuzhu"));
        // Symmetric: order does not matter.
        assert!(is_perfect_pair("xuzhu", "caocao"));
    }

    #[test]
    fn non_pair_generals_rejected() {
        assert!(!is_perfect_pair("caocao", "liubei"));
        assert!(!is_perfect_pair("unknown_general", "caocao"));
    }

    #[test]
    fn empty_names_never_pair() {
        assert!(!is_perfect_pair("", ""));
        assert!(!is_perfect_pair("caocao", ""));
    }

    #[test]
    fn dominant_faction_minority_member_becomes_ambition() {
        // 5 players: 3 wei (dominant), 2 shu. A shu member should flip to ambition.
        let mut tx = crate::transaction::Transaction::begin(GameState::empty());
        let mut make = |st: &mut GameState, group: &str| {
            let p = st.players.alloc(crate::state::Player {
                id: PlayerId::NONE,
                seat: engine_api::SeatIndex::new(0),
                hp: 4,
                max_hp: 4,
                alive: true,
                dying: false,
                turned_over: false,
                identity: Identity::Lord,
                character_def: 0,
                character_def2: 0,
                skills: Vec::new(),
                marks: Vec::new(),
                disabled_equip_slots: [0; 5],
                unseen: [false, false],
            });
            st.group_overrides.insert(p, group.to_string());
            p
        };
        for _ in 0..3 {
            make(tx.state_mut(), "wei");
        }
        let shu1 = make(tx.state_mut(), "shu");
        let _shu2 = make(tx.state_mut(), "shu");
        // shu is minority (2 vs 3 wei); dominant threshold = 5/2 = 2, wei has 3 > 2.
        assert!(should_become_ambition(tx.state(), shu1));
    }

    #[test]
    fn balanced_factions_no_ambition() {
        // 4 players: 2 wei, 2 shu. No dominant faction → nobody flips.
        let mut tx = crate::transaction::Transaction::begin(GameState::empty());
        let mut make = |st: &mut GameState, group: &str| {
            let p = st.players.alloc(crate::state::Player {
                id: PlayerId::NONE,
                seat: engine_api::SeatIndex::new(0),
                hp: 4,
                max_hp: 4,
                alive: true,
                dying: false,
                turned_over: false,
                identity: Identity::Lord,
                character_def: 0,
                character_def2: 0,
                skills: Vec::new(),
                marks: Vec::new(),
                disabled_equip_slots: [0; 5],
                unseen: [false, false],
            });
            st.group_overrides.insert(p, group.to_string());
            p
        };
        let w1 = make(tx.state_mut(), "wei");
        make(tx.state_mut(), "wei");
        make(tx.state_mut(), "shu");
        make(tx.state_mut(), "shu");
        // threshold = 4/2 = 2; neither faction has > 2.
        assert!(!should_become_ambition(tx.state(), w1));
    }

    #[test]
    fn ambition_member_stays_ambition() {
        let mut tx = crate::transaction::Transaction::begin(GameState::empty());
        let p = tx.state_mut().players.alloc(crate::state::Player {
            id: PlayerId::NONE,
            seat: engine_api::SeatIndex::new(0),
            hp: 4,
            max_hp: 4,
            alive: true,
            dying: false,
            turned_over: false,
            identity: Identity::Lord,
            character_def: 0,
            character_def2: 0,
            skills: Vec::new(),
            marks: Vec::new(),
            disabled_equip_slots: [0; 5],
            unseen: [false, false],
        });
        tx.state_mut()
            .group_overrides
            .insert(p, AMBITION.to_string());
        // Already ambition → should_become_ambition returns false (no re-flip needed).
        assert!(!should_become_ambition(tx.state(), p));
    }

    #[test]
    fn reveal_general_flips_slot_and_no_double_reveal() {
        let mut tx = crate::transaction::Transaction::begin(GameState::empty());
        let p = tx.state_mut().players.alloc(crate::state::Player {
            id: PlayerId::NONE,
            seat: engine_api::SeatIndex::new(0),
            hp: 4,
            max_hp: 4,
            alive: true,
            dying: false,
            turned_over: false,
            identity: Identity::Lord,
            character_def: 0,
            character_def2: 0,
            skills: Vec::new(),
            marks: Vec::new(),
            disabled_equip_slots: [0; 5],
            unseen: [true, true],
        });
        // Reveal main (slot 0).
        reveal_general(tx.state_mut(), p, 0).unwrap();
        assert_eq!(tx.state().players.get(p).unwrap().unseen, [false, true]);
        // Double-reveal rejected.
        assert!(reveal_general(tx.state_mut(), p, 0).is_err());
        // Out-of-range slot rejected.
        assert!(reveal_general(tx.state_mut(), p, 5).is_err());
    }

    #[test]
    fn reveal_general_rejects_in_identity_mode() {
        // Identity-mode players have no unseen state set (defaults [false,false]); a reveal
        // attempt is rejected because the general is already face-up.
        let mut tx = crate::transaction::Transaction::begin(GameState::empty());
        let p = tx.state_mut().players.alloc(crate::state::Player {
            id: PlayerId::NONE,
            seat: engine_api::SeatIndex::new(0),
            hp: 4,
            max_hp: 4,
            alive: true,
            dying: false,
            turned_over: false,
            identity: Identity::Lord,
            character_def: 0,
            character_def2: 0,
            skills: Vec::new(),
            marks: Vec::new(),
            disabled_equip_slots: [0; 5],
            unseen: [false, false],
        });
        assert!(reveal_general(tx.state_mut(), p, 0).is_err());
    }

    #[test]
    fn alliance_formed_and_queried_bidirectionally() {
        let mut tx = crate::transaction::Transaction::begin(GameState::empty());
        let a = make_player(tx.state_mut(), Some("ye"));
        let b = make_player(tx.state_mut(), Some("wei"));
        assert!(!are_allies(tx.state(), a, b));
        form_alliance(tx.state_mut(), a, b);
        // Bidirectional lookup.
        assert!(are_allies(tx.state(), a, b));
        assert!(are_allies(tx.state(), b, a));
    }

    #[test]
    fn lourenxin_choice_records_alliance() {
        let mut tx = crate::transaction::Transaction::begin(GameState::empty());
        let ambition = make_player(tx.state_mut(), Some("ye"));
        let target = make_player(tx.state_mut(), Some("wei"));
        // Observer selects target → alliance recorded.
        resolve_lourenxin_choice(tx.state_mut(), ambition, &[target.raw()], false);
        assert!(are_allies(tx.state(), ambition, target));
    }

    #[test]
    fn lourenxin_cancelled_records_nothing() {
        let mut tx = crate::transaction::Transaction::begin(GameState::empty());
        let ambition = make_player(tx.state_mut(), Some("ye"));
        let target = make_player(tx.state_mut(), Some("wei"));
        resolve_lourenxin_choice(tx.state_mut(), ambition, &[target.raw()], true);
        assert!(!are_allies(tx.state(), ambition, target));
    }

    #[test]
    fn allied_players_merge_into_one_camp_for_victory() {
        // 2 players, different factions, but allied → one camp → match ends.
        let mut tx = crate::transaction::Transaction::begin(GameState::empty());
        let a = make_player(tx.state_mut(), Some("ye"));
        let b = make_player(tx.state_mut(), Some("wei"));
        form_alliance(tx.state_mut(), a, b);
        let outcome = check_guozhan_victory(tx.state());
        assert!(outcome.is_finished(), "allied pair should end the match");
    }

    #[test]
    fn single_faction_alive_wins() {
        let mut tx = crate::transaction::Transaction::begin(GameState::empty());
        // Two wei players alive.
        let p1 = tx.state_mut().players.alloc(crate::state::Player {
            id: PlayerId::NONE,
            seat: engine_api::SeatIndex::new(0),
            hp: 4,
            max_hp: 4,
            alive: true,
            dying: false,
            turned_over: false,
            identity: Identity::Lord,
            character_def: 1,
            character_def2: 0,
            skills: Vec::new(),
            marks: Vec::new(),
            disabled_equip_slots: [0; 5],
            unseen: [false, false],
        });
        let p2 = tx.state_mut().players.alloc(crate::state::Player {
            id: PlayerId::NONE,
            seat: engine_api::SeatIndex::new(1),
            hp: 4,
            max_hp: 4,
            alive: true,
            dying: false,
            turned_over: false,
            identity: Identity::Lord,
            character_def: 2,
            character_def2: 0,
            skills: Vec::new(),
            marks: Vec::new(),
            disabled_equip_slots: [0; 5],
            unseen: [false, false],
        });
        tx.state_mut().group_overrides.insert(p1, "wei".to_string());
        tx.state_mut().group_overrides.insert(p2, "wei".to_string());
        let outcome = check_guozhan_victory(tx.state());
        assert_eq!(
            outcome,
            MatchOutcome::GroupWin {
                group: "wei".to_string()
            }
        );
    }

    #[test]
    fn two_factions_alive_still_in_progress() {
        let mut tx = crate::transaction::Transaction::begin(GameState::empty());
        let p1 = tx.state_mut().players.alloc(crate::state::Player {
            id: PlayerId::NONE,
            seat: engine_api::SeatIndex::new(0),
            hp: 4,
            max_hp: 4,
            alive: true,
            dying: false,
            turned_over: false,
            identity: Identity::Lord,
            character_def: 1,
            character_def2: 0,
            skills: Vec::new(),
            marks: Vec::new(),
            disabled_equip_slots: [0; 5],
            unseen: [false, false],
        });
        let p2 = tx.state_mut().players.alloc(crate::state::Player {
            id: PlayerId::NONE,
            seat: engine_api::SeatIndex::new(1),
            hp: 4,
            max_hp: 4,
            alive: true,
            dying: false,
            turned_over: false,
            identity: Identity::Lord,
            character_def: 2,
            character_def2: 0,
            skills: Vec::new(),
            marks: Vec::new(),
            disabled_equip_slots: [0; 5],
            unseen: [false, false],
        });
        tx.state_mut().group_overrides.insert(p1, "wei".to_string());
        tx.state_mut().group_overrides.insert(p2, "shu".to_string());
        let outcome = check_guozhan_victory(tx.state());
        assert_eq!(outcome, MatchOutcome::InProgress);
    }

    #[test]
    fn no_alive_players_is_draw() {
        let mut tx = crate::transaction::Transaction::begin(GameState::empty());
        tx.state_mut().players.alloc(crate::state::Player {
            id: PlayerId::NONE,
            seat: engine_api::SeatIndex::new(0),
            hp: 0,
            max_hp: 4,
            alive: false,
            dying: false,
            turned_over: false,
            identity: Identity::Lord,
            character_def: 1,
            character_def2: 0,
            skills: Vec::new(),
            marks: Vec::new(),
            disabled_equip_slots: [0; 5],
            unseen: [false, false],
        });
        let outcome = check_guozhan_victory(tx.state());
        assert_eq!(outcome, MatchOutcome::Draw);
    }
}
/// 珠联璧合配对表（上游 perfectPairs.js，ADR 0598 phase C）。
/// key = 主将 name_key，value = 可触发珠联璧合的副将 name_key 列表。
pub const PERFECT_PAIRS: &[(&str, &[&str])] = &[
    ("bailingyun", &["jin_simayi", "new_jin_simayi"]),
    ("bulianshi", &["sunquan"]),
    ("caifuren", &["liubiao"]),
    ("caoang", &["caocao"]),
    ("caocao", &["xuzhu", "re_xuzhu", "dianwei", "bianfuren"]),
    (
        "re_caocao",
        &[
            "caoang",
            "caoying",
            "xuzhu",
            "re_xuzhu",
            "dianwei",
            "bianfuren",
            "cuiyan",
        ],
    ),
    ("caohong", &["caoren"]),
    ("caopi", &["zhenji", "cuimao", "luzhi"]),
    ("caoying", &["caocao", "caopi"]),
    ("caozhi", &["zhenji"]),
    ("chengong", &["lvbu"]),
    ("chengpu", &["zhouyu", "huanggai", "re_huanggai"]),
    ("cuiyan", &["caocao"]),
    ("daqiao", &["xiaoqiao"]),
    ("daxiaoqiao", &["zhouyu", "sunce"]),
    ("dingfeng", &["xusheng", "re_xusheng", "zhugeke"]),
    ("dongzhuo", &["liru", "dongbai", "xurong"]),
    ("fuhuanghou", &["liuxie"]),
    ("fuwan", &["fuhuanghou"]),
    ("gaolan", &["yj_zhanghe"]),
    ("ganning", &["lingtong", "xf_sufei"]),
    ("guanping", &["guanyu"]),
    ("guanqiujian", &["wenqin"]),
    ("guansuo", &["guanyu"]),
    ("guanyinping", &["guanyu"]),
    ("guanyu", &["zhangfei", "liaohua", "guanyinping"]),
    ("huangzhong", &["weiyan", "pot_weiyan", "ol_weiyan"]),
    ("jiangqing", &["zhoutai"]),
    ("jiangwei", &["xiahouba", "liaohua"]),
    ("jiaxu", &["liqueguosi"]),
    ("jin_guohuai", &["jin_jiachong"]),
    ("jin_simashi", &["jin_xiahouhui", "jin_yanghuiyu"]),
    (
        "jin_simayi",
        &["jin_zhangchunhua", "shibao", "new_jin_zhangchunhua"],
    ),
    ("jin_simazhao", &["jin_wangyuanji"]),
    ("jin_yanghu", &["duyu"]),
    ("key_ao", &["key_inari"]),
    ("key_chihaya", &["key_sakuya"]),
    ("key_doruji", &["key_rin"]),
    ("key_hinata", &["key_yui"]),
    ("key_iwasawa", &["key_hisako"]),
    ("key_jojiro", &["key_yusa"]),
    ("key_kaori", &["key_shiori"]),
    ("key_komari", &["key_rin", "key_sasami"]),
    ("key_kud", &["key_harukakanata"]),
    ("key_lucia", &["key_shizuru"]),
    ("key_masato", &["key_kengo"]),
    (
        "key_riki",
        &["key_rin", "key_saya", "key_kyousuke", "key_kud"],
    ),
    ("key_shiroha", &["key_umi"]),
    ("key_shizuku", &["key_tsumugi"]),
    ("key_tomoya", &["key_nagisa", "key_sunohara", "key_kotomi"]),
    ("key_yuiko", &["key_harukakanata"]),
    ("key_yuri", &["key_kanade", "key_abyusa"]),
    ("key_yuu", &["key_nao"]),
    ("key_yuzuru", &["key_hinata", "key_kanade", "key_ayato"]),
    ("lijue", &["guosi", "jiaxu"]),
    ("lingcao", &["lingtong"]),
    ("lingju", &["diaochan", "lvbu", "re_lvbu"]),
    ("lingtong", &["ganning"]),
    ("liubei", &["guanyu", "zhangfei", "ganfuren", "fazheng"]),
    ("liuchen", &["liushan"]),
    ("lukang", &["luxun", "re_luxun"]),
    ("lvbu", &["diaochan", "lvlingqi"]),
    ("re_lvbu", &["diaochan", "lvlingqi"]),
    ("machao", &["madai", "mayunlu", "yangwan"]),
    ("maliang", &["masu"]),
    ("mateng", &["machao", "madai", "mayunlu"]),
    ("menghuo", &["zhurong"]),
    ("nanhualaoxian", &["zuoci", "yuji"]),
    (
        "new_jin_simayi",
        &["jin_zhangchunhua", "shibao", "new_jin_zhangchunhua"],
    ),
    ("pangtong", &["zhugejin"]),
    ("re_guohuai", &["xiahouyuan", "zhanghe"]),
    ("re_nanhualaoxian", &["zuoci", "yuji"]),
    ("re_xushu", &["zhaoyun", "sp_zhugeliang"]),
    (
        "re_yuanshao",
        &[
            "yanwen",
            "ol_yanwen",
            "tianfeng",
            "yanliang",
            "wenchou",
            "liufuren",
            "tw_liufuren",
        ],
    ),
    ("simahui", &["pangdegong"]),
    ("simazhao", &["wangyuanji", "simayi"]),
    (
        "sp_zhugeliang",
        &["pangtong", "huangyueying", "re_huangyueying"],
    ),
    ("sunben", &["zhouyu", "taishici", "re_taishici", "daqiao"]),
    ("sunce", &["zhouyu", "taishici", "re_taishici", "daqiao"]),
    ("sundeng", &["sunquan", "zhoufei"]),
    ("sunluban", &["quancong"]),
    ("sunquan", &["zhoutai"]),
    ("sunshangxiang", &["liubei"]),
    ("sunxiù", &["simalun"]),
    ("taishici", &["liuyao", "kongrong"]),
    ("tw_xiahoushang", &["caopi"]),
    ("wangping", &["jiangfei"]),
    ("wuguotai", &["sunjian", "ol_sb_sunjian", "sunshangxiang"]),
    ("wujing", &["sunce", "sunben", "wuguotai"]),
    ("xf_sufei", &["ganning"]),
    ("xiahoudun", &["xiahouyuan", "yujin"]),
    ("xiahoushi", &["zhangfei"]),
    ("xuangongzhu", &["duyu"]),
    ("xugong", &["yanbaihu"]),
    ("xunyou", &["xunyu"]),
    ("xushu", &["liubei"]),
    ("yuanshu", &["jiling"]),
    ("yl_yuanshu", &["jiling"]),
    ("yuejin", &["re_lidian"]),
    ("yuji", &["zuoci"]),
    ("zhangbao", &["zhangliang", "zhangjiao", "re_zhangjiao"]),
    ("zhangchunhua", &["simayi"]),
    ("zhangji", &["zhangxiu", "zoushi"]),
    ("zhangliang", &["zhangjiao", "re_zhangjiao"]),
    ("zhangliao", &["zangba"]),
    ("zhangqiying", &["zhanglu"]),
    ("zhangxingcai", &["liushan"]),
    ("zhangxiu", &["jiaxu"]),
    ("zhangxuan", &["zhangyao"]),
    ("zhaoyun", &["liushan"]),
    ("zhonghui", &["jiangwei"]),
    ("ol_zhonghui", &["jiangwei"]),
    ("zhoucang", &["guanyu"]),
    ("zhouyu", &["huanggai", "re_huanggai", "xiaoqiao", "zhouyi"]),
    ("zhugejin", &["zhugeke", "sunquan"]),
    (
        "zhugeliang",
        &["jiangwei", "jiangfei", "huangyueying", "re_huangyueying"],
    ),
    ("zuoci", &["yuji"]),
];
