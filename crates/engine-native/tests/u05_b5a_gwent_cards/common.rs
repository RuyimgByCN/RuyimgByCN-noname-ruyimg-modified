//! U05-5a (`杀海拾遗 · gwent`) 真实事件流共享测试夹具（零 `debug_execute_effect`）。

use std::collections::{BTreeMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};

use content_native_loader::{
    prepare_locked_content_with_bridge, restore_locked_engine, start_locked_engine,
    validate_and_freeze_raw_package, LockedStartMode, VerifiedLockedContent,
};
use engine_native::archive::{decode_archive, encode_archive};
use engine_native::catalog::{CardKind, Faction, Suit};
use engine_native::ids::{CardId, PlayerId};
use engine_native::state::{Role, SkillInstance};
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, GameMode, Phase};

static DEDUP: AtomicU64 = AtomicU64::new(5_500_000);

pub fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

pub fn submit(e: &mut Engine, actor: u8, action: GameAction) -> CommandOutcome {
    e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action,
    })
}

pub fn pass_window(e: &mut Engine) {
    for _ in 0..64 {
        if let Some(c) = e.open_choice().cloned() {
            if c.spec.prompt == "nullify_window"
                || c.spec.prompt == "respond_wuxie"
                || c.spec.prompt == "respond_shan"
            {
                let obs = c.observer;
                let cid = c.id;
                let sel = if c.spec.allow_cancel {
                    Vec::new()
                } else if c.spec.options.contains(&ChoiceOption::Pass) {
                    vec![ChoiceOption::Pass]
                } else {
                    break;
                };
                submit(e, obs, GameAction::Answer { choice_id: cid, selection: sel });
                continue;
            }
            break;
        }
        break;
    }
}

pub fn drain_default(e: &mut Engine) {
    for _ in 0..96 {
        let Some(c) = e.open_choice().cloned() else { break };
        let sel = if c.spec.options.contains(&ChoiceOption::Pass) {
            vec![ChoiceOption::Pass]
        } else if c.spec.allow_cancel {
            Vec::new()
        } else {
            c.spec.options.iter().take(c.spec.min.max(1)).copied().collect()
        };
        submit(
            e,
            c.observer,
            GameAction::Answer {
                choice_id: c.id,
                selection: sel,
            },
        );
    }
}

pub fn load_gwent_locked_bundle() -> VerifiedLockedContent {
    let ext_root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let pkg_dir = ext_root.join("shahai-gwent-upstream-noname");
    let raw_json = std::fs::read_to_string(pkg_dir.join("package.npack.json"))
        .expect("read shahai-gwent package.npack.json")
        .replace("\r\n", "\n");
    let parsed: serde_json::Value = serde_json::from_str(&raw_json).unwrap();
    let mut assets = BTreeMap::new();
    if let Some(paths) = parsed.get("asset_paths").and_then(|v| v.as_object()) {
        for rel_path in paths.values().filter_map(|v| v.as_str()) {
            let bytes = std::fs::read(pkg_dir.join(rel_path))
                .unwrap_or_else(|e| panic!("read gwent asset {rel_path}: {e}"))
                .into_iter()
                .filter(|&b| b != b'\r')
                .collect::<Vec<u8>>();
            assets.insert(rel_path.to_string(), bytes);
        }
    }
    let frozen = validate_and_freeze_raw_package(raw_json, assets).expect("freeze gwent pkg");
    prepare_locked_content_with_bridge(vec![frozen]).expect("prepare gwent locked content")
}

/// 建 4 人真锁定对局：0=主(友好2)，2=忠(友好0)，1=反(敌对0)，3=反(敌对0)；清手牌、推进到 0 号出牌阶段。
pub fn setup_gwent_locked(seed: u64) -> (VerifiedLockedContent, Engine) {
    let verified = load_gwent_locked_bundle();
    let mut e = start_locked_engine(seed, 4, LockedStartMode::Prototype, &verified)
        .expect("start locked gwent engine");
    for _ in 0..64 {
        if e.turn().current == 0 && e.turn().phase == Phase::Play && e.open_choice().is_none() {
            break;
        }
        if let Some(c) = e.open_choice().cloned() {
            let sel = if c.spec.min > 0 {
                c.spec.options.iter().take(c.spec.min).copied().collect()
            } else {
                vec![c.spec.options[0]]
            };
            submit(
                &mut e,
                c.observer,
                GameAction::Answer {
                    choice_id: c.id,
                    selection: sel,
                },
            );
            continue;
        }
        let actor = e.turn().current;
        submit(&mut e, actor, GameAction::NextPhase);
    }
    let mut snap = e.save();
    for p in snap.core.players.iter_mut() {
        let h = std::mem::take(&mut p.hand);
        snap.core.discard.extend(h);
        let eq = std::mem::take(&mut p.equips);
        snap.core.discard.extend(eq);
        let jg = std::mem::take(&mut p.judge);
        snap.core.discard.extend(jg);
        p.hp = 4;
        p.max_hp = 4;
        p.face_down = false;
        p.linked = false;
        p.hujia = 0;
    }
    snap.core.mode = GameMode::Identity {
        roles: vec![Role::Lord, Role::Rebel, Role::Loyalist, Role::Traitor],
    };
    let wei_gens: Vec<u16> = (0..e.catalog().general_count() as u16)
        .filter(|&g| e.catalog().general_meta(g).is_some_and(|m| m.1 == Faction::Wei))
        .take(2)
        .collect();
    let shu_gens: Vec<u16> = (0..e.catalog().general_count() as u16)
        .filter(|&g| e.catalog().general_meta(g).is_some_and(|m| m.1 == Faction::Shu))
        .take(2)
        .collect();
    snap.core.players[0].general_slots = [Some(wei_gens[0]), None];
    snap.core.players[2].general_slots = [Some(wei_gens[1]), None];
    snap.core.players[1].general_slots = [Some(shu_gens[0]), None];
    snap.core.players[3].general_slots = [Some(shu_gens[1]), None];
    let e = restore_locked_engine(&snap, &verified).expect("restore clean 4p gwent setup");
    (verified, e)
}

pub fn find_card(e: &Engine, exact_or_prefix: &str) -> CardId {
    let cards = &e.catalog().cards;
    cards
        .iter()
        .position(|c| c.name == exact_or_prefix)
        .or_else(|| {
            let prefix = format!("{exact_or_prefix}_");
            cards.iter().position(|c| {
                c.name
                    .strip_prefix(&prefix)
                    .is_some_and(|t| !t.is_empty() && t.chars().all(|ch| ch.is_ascii_digit()))
            })
        })
        .unwrap_or_else(|| panic!("card '{exact_or_prefix}' not found in Catalog")) as CardId
}

pub fn find_cards_matching<F>(e: &Engine, mut pred: F, limit: usize) -> Vec<CardId>
where
    F: FnMut(&engine_native::catalog::CardDef) -> bool,
{
    e.catalog()
        .cards
        .iter()
        .enumerate()
        .filter(|(_, d)| pred(d))
        .take(limit)
        .map(|(i, _)| i as CardId)
        .collect()
}

pub fn card_of_kind(e: &Engine, kind: CardKind, suit: Option<Suit>) -> CardId {
    e.catalog()
        .cards
        .iter()
        .position(|c| c.kind == kind && suit.is_none_or(|s| c.suit == s))
        .unwrap_or_else(|| panic!("card kind {kind:?} suit {suit:?} not found")) as CardId
}

fn pull_id_from_zones(core: &mut engine_native::state::GameStateCore, c: CardId) {
    for p in core.players.iter_mut() {
        p.hand.retain(|x| *x != c);
        p.equips.retain(|x| *x != c);
        p.judge.retain(|x| *x != c);
    }
    core.deck.retain(|x| *x != c);
    core.discard.retain(|x| *x != c);
    core.table.retain(|x| *x != c);
    for pile in core.vm_aux.piles.values_mut() {
        pile.retain(|x| *x != c);
    }
}

pub fn inject_hand(
    e: &Engine,
    verified: &VerifiedLockedContent,
    ids: &[CardId],
    to: PlayerId,
) -> Engine {
    let mut snap = e.save();
    for &c in ids {
        pull_id_from_zones(&mut snap.core, c);
        snap.core.players[to as usize].hand.push(c);
    }
    restore_locked_engine(&snap, verified).expect("inject_hand restore")
}

pub fn inject_equip(
    e: &Engine,
    verified: &VerifiedLockedContent,
    card: CardId,
    to: PlayerId,
) -> Engine {
    let mut snap = e.save();
    pull_id_from_zones(&mut snap.core, card);
    snap.core.players[to as usize].equips.push(card);
    for sk in &e.catalog().cards[card as usize].equip_skills {
        if let Some(def) = e.catalog().skill_by_name(sk) {
            snap.core.skills.insert(
                (to, def),
                SkillInstance {
                    owner: to,
                    enabled: true,
                    once_used: false,
                    expires_end_of_turn: false,
                    rounds: 0,
                },
            );
        }
    }
    restore_locked_engine(&snap, verified).expect("inject_equip restore")
}

pub fn inject_judge(
    e: &Engine,
    verified: &VerifiedLockedContent,
    card: CardId,
    to: PlayerId,
) -> Engine {
    let mut snap = e.save();
    pull_id_from_zones(&mut snap.core, card);
    snap.core.players[to as usize].judge.push(card);
    restore_locked_engine(&snap, verified).expect("inject_judge restore")
}

pub fn grant_skill(
    e: &Engine,
    verified: &VerifiedLockedContent,
    skill_name: &str,
    to: PlayerId,
) -> Engine {
    let def = e
        .catalog()
        .skill_by_name(skill_name)
        .unwrap_or_else(|| panic!("skill {skill_name} missing"));
    let mut snap = e.save();
    snap.core.skills.insert(
        (to, def),
        SkillInstance {
            owner: to,
            enabled: true,
            once_used: false,
            expires_end_of_turn: false,
            rounds: 0,
        },
    );
    restore_locked_engine(&snap, verified).expect("grant_skill restore")
}

pub fn save_restore_answer(
    e: Engine,
    verified: &VerifiedLockedContent,
    actor: PlayerId,
    selection: Vec<ChoiceOption>,
) -> (Engine, CommandOutcome) {
    let bytes = encode_archive(&e.save());
    let decoded = decode_archive(&bytes).expect("decode archive mid-choice");
    let mut restored = restore_locked_engine(&decoded, verified).expect("restore archive mid-choice");
    let choice = restored.open_choice().cloned().expect("restored open_choice");
    assert_eq!(choice.observer, actor);
    let out = submit(
        &mut restored,
        actor,
        GameAction::Answer {
            choice_id: choice.id,
            selection,
        },
    );
    (restored, out)
}

pub fn advance_to_seat_play(e: &mut Engine, seat: PlayerId) {
    for _ in 0..96 {
        if e.turn().current == seat && e.turn().phase == Phase::Play && e.open_choice().is_none() {
            return;
        }
        if let Some(c) = e.open_choice().cloned() {
            let sel = if c.spec.options.contains(&ChoiceOption::Pass) {
                vec![ChoiceOption::Pass]
            } else if c.spec.allow_cancel {
                Vec::new()
            } else {
                c.spec.options.iter().take(c.spec.min.max(1)).copied().collect()
            };
            submit(
                e,
                c.observer,
                GameAction::Answer {
                    choice_id: c.id,
                    selection: sel,
                },
            );
            continue;
        }
        let cur = e.turn().current;
        submit(e, cur, GameAction::NextPhase);
    }
    panic!("failed to advance to seat {seat} Play phase");
}

pub fn advance_past_turn(e: &mut Engine, seat: PlayerId) {
    for _ in 0..64 {
        if e.turn().current != seat {
            return;
        }
        if let Some(c) = e.open_choice().cloned() {
            let sel = if c.spec.options.contains(&ChoiceOption::Pass) {
                vec![ChoiceOption::Pass]
            } else if c.spec.allow_cancel {
                Vec::new()
            } else {
                c.spec.options.iter().take(c.spec.min.max(1)).copied().collect()
            };
            submit(
                e,
                c.observer,
                GameAction::Answer {
                    choice_id: c.id,
                    selection: sel,
                },
            );
            continue;
        }
        let cur = e.turn().current;
        submit(e, cur, GameAction::NextPhase);
    }
    panic!("failed to advance past seat {seat} turn");
}

pub fn is_vanished(e: &Engine, c: CardId) -> bool {
    let core = e.state_core();
    let in_vanished = core
        .vm_aux
        .piles
        .get(&(0, "u05_vanished".to_string()))
        .is_some_and(|p| p.contains(&c));
    let in_discard = core.discard.contains(&c);
    in_vanished && !in_discard
}

pub fn assert_zone_conservation(e: &Engine) {
    let s = e.save();
    let total = e.catalog().cards.len();
    let mut seen = HashSet::new();
    for c in &s.core.deck {
        assert!(seen.insert(*c), "dup deck {c}");
    }
    for c in &s.core.discard {
        assert!(seen.insert(*c), "dup discard {c}");
    }
    for c in &s.core.table {
        assert!(seen.insert(*c), "dup table {c}");
    }
    for p in &s.core.players {
        for c in &p.hand {
            assert!(seen.insert(*c), "dup hand {c}");
        }
        for c in &p.equips {
            assert!(seen.insert(*c), "dup equip {c}");
        }
        for c in &p.judge {
            assert!(seen.insert(*c), "dup judge {c}");
        }
    }
    for pile in s.core.vm_aux.piles.values() {
        for c in pile {
            assert!(seen.insert(*c), "dup pile {c}");
        }
    }
    if e.open_choice().is_none() {
        assert_eq!(seen.len(), total, "card conservation mismatch");
    } else {
        assert!(seen.len() <= total, "card count exceeded catalog");
    }
}
