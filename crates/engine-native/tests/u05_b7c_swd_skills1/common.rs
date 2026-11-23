use engine_native::ai::BoundedPlanner;
use engine_native::catalog::{CardKind, EquipSlot, Suit};
use engine_native::content::extension::ExtensionPack;
use engine_native::event::Phase;
use engine_native::state::{GameMode, Role};
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand};
use std::sync::atomic::{AtomicU64, Ordering};

pub(super) static DEDUP: AtomicU64 = AtomicU64::new(9_070_000);

pub(super) fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

pub(super) fn swd_pack() -> ExtensionPack {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions/shahai-swd-upstream-noname/assets/cards.json");
    let json = std::fs::read_to_string(root).expect("读 swd 包数据");
    content_native_loader::bridge_pack(&json).expect("桥接 swd 包").pack
}

pub(super) fn swd_engine(seed: u64) -> (ExtensionPack, Engine) {
    let pack = swd_pack();
    let e = Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局");
    (pack, e)
}

pub(super) fn submit(e: &mut Engine, actor: u8, action: GameAction) -> CommandOutcome {
    e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action,
    })
}

pub(super) fn drain_pass_or_first(e: &mut Engine) {
    for _ in 0..48 {
        let Some(c) = e.open_choice().cloned() else { break };
        let sel = if c.spec.options.contains(&ChoiceOption::Pass) {
            vec![ChoiceOption::Pass]
        } else if c.spec.allow_cancel {
            Vec::new()
        } else {
            c.spec
                .options
                .iter()
                .take(c.spec.min.max(1))
                .copied()
                .collect()
        };
        let _ = submit(
            e,
            c.observer,
            GameAction::Answer {
                choice_id: c.id,
                selection: sel,
            },
        );
    }
}

pub(super) fn advance_to_play(e: &mut Engine, seat: u8) {
    for _ in 0..24 {
        drain_pass_or_first(e);
        if e.turn().current == seat && e.turn().phase == Phase::Play {
            return;
        }
        let cur = e.turn().current;
        let _ = submit(e, cur, GameAction::NextPhase);
    }
    panic!("未推进至 {seat} 号出牌阶段");
}

pub(super) fn move_card_to_hand(e: &mut Engine, seat: u8, cid: u32) -> u32 {
    let core = e.state_core_mut();
    core.deck.retain(|x| *x != cid);
    core.discard.retain(|x| *x != cid);
    core.table.retain(|x| *x != cid);
    core.declared_kinds.remove(&cid);
    core.declared_slots.remove(&cid);
    for pl in &mut core.players {
        pl.hand.retain(|x| *x != cid);
        pl.equips.retain(|x| *x != cid);
        pl.judge.retain(|x| *x != cid);
        pl.off_court.retain(|x| *x != cid);
    }
    core.players[seat as usize].hand.push(cid);
    cid
}

pub(super) fn find_card<F: Fn(CardKind, Suit) -> bool>(e: &Engine, pred: F) -> u32 {
    (0..e.catalog().cards.len() as u32)
        .find(|c| {
            let def = &e.catalog().cards[*c as usize];
            pred(def.kind, def.suit)
        })
        .expect("目录中存在满足条件之牌")
}

pub(super) fn equip_slot_card(e: &mut Engine, seat: u8, cid: u32) {
    let core = e.state_core_mut();
    core.deck.retain(|x| *x != cid);
    core.discard.retain(|x| *x != cid);
    core.table.retain(|x| *x != cid);
    core.declared_kinds.remove(&cid);
    core.declared_slots.remove(&cid);
    for pl in &mut core.players {
        pl.hand.retain(|x| *x != cid);
        pl.equips.retain(|x| *x != cid);
    }
    core.players[seat as usize].equips.push(cid);
}

pub(super) fn ai_answer_once(e: &mut Engine) -> Option<String> {
    let c = e.open_choice().cloned()?;
    let observer = c.observer;
    let prompt = c.spec.prompt.to_string();
    let plan = e.plan_action(observer, &BoundedPlanner::new(4));
    let out = submit(e, observer, plan.action);
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "AI 应答 {prompt} 失败: {out:?}"
    );
    Some(prompt)
}

pub(super) fn push_top_deck(e: &mut Engine, cid: u32) {
    let core = e.state_core_mut();
    core.deck.retain(|x| *x != cid);
    core.discard.retain(|x| *x != cid);
    core.table.retain(|x| *x != cid);
    core.declared_kinds.remove(&cid);
    core.declared_slots.remove(&cid);
    for pl in &mut core.players {
        pl.hand.retain(|x| *x != cid);
        pl.equips.retain(|x| *x != cid);
    }
    core.deck.push(cid);
}

pub(super) fn set_roles_lord_rebel(e: &mut Engine) {
    e.state_core_mut().mode = GameMode::Identity {
        roles: vec![Role::Lord, Role::Rebel, Role::Loyalist, Role::Traitor],
    };
}

pub(super) fn weapon_with_range(e: &Engine, min_range: u8) -> u32 {
    (0..e.catalog().cards.len() as u32)
        .find(|c| {
            let d = &e.catalog().cards[*c as usize];
            d.kind.equip_slot() == Some(EquipSlot::Weapon) && d.range >= u32::from(min_range)
        })
        .expect("存在满足射程武器")
}
