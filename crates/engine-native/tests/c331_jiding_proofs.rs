//! C9L：既定牌的转移必须有**可验证的付款证明**且绑定真实既定牌帧。
//!
//! C9J 审查的两项 P1：
//! 1. 同疾（TongJi）装备分支付款时直接 `to_discard`，不发 `CardsLost` 收据，
//!    记录成 `loss=None`；校验端又对「收据帧已弹栈」的变更直接放行，于是
//!    伪造档可以不带任何付款证明就完成转移（unpaid redirect / proof strip）。
//! 2. 同疾/移花的变更没有先绑定来源帧（移花还用 `unwrap_or(0)` 落到伪帧），
//!    转移可以先于记录发生（origin 未绑定）。
//!
//! 回归口径：真实档必须可恢复；`loss=None`、代价牌未进弃牌堆、以及把变更
//! 记录整段删掉（payload 已转移却没有证明）必须被原子拒绝。
use super::c331_genuine_invocations::{answer, emitter, fire, fixture};
use super::c331_source_predicates::reject_restore_atomic;
use engine_native::event::{EventData, JidingCardCost};
use engine_native::{ChoiceOption as O, Engine, SnapshotError, StateSnapshot};

fn jiding_changes(s: &StateSnapshot) -> Vec<engine_native::event::JidingCardChange> {
    s.resolution
        .stack
        .iter()
        .find_map(|f| match &f.data {
            EventData::SkillInvocation(d) if !d.jiding_changes.is_empty() => {
                Some(d.jiding_changes.clone())
            }
            _ => None,
        })
        .unwrap_or_default()
}

fn restorable_with(
    e: &mut Engine,
    pack: &engine_native::content::extension::ExtensionPack,
    want: impl Fn(&StateSnapshot) -> bool,
) -> StateSnapshot {
    e.set_time_slice(Some(1));
    fire(e);
    for _ in 0..3000 {
        let s = e.save();
        if want(&s) && Engine::restore_with_extensions(&s, &[pack]).is_ok() {
            return s;
        }
        if let Some(c) = e.open_choice().cloned() {
            let selection = match c.spec.prompt {
                "trigger_confirm" => vec![O::Confirm],
                "yihua_discard" => c.spec.options.iter().take(2).copied().collect(),
                "tongji_transfer_discard" => vec![c.spec.options[0]],
                _ => {
                    if c.spec.allow_cancel {
                        vec![]
                    } else {
                        c.spec.options.iter().take(c.spec.min).copied().collect()
                    }
                }
            };
            answer(e, selection);
        } else {
            e.resume();
        }
    }
    panic!("no restorable state with the expected jiding change");
}

fn mutate(s: &StateSnapshot, f: impl Fn(&mut JidingCardCost)) -> StateSnapshot {
    let mut out = s.clone();
    for frame in &mut out.resolution.stack {
        if let EventData::SkillInvocation(d) = &mut frame.data {
            for change in &mut d.jiding_changes {
                f(&mut change.cost);
            }
        }
    }
    out
}

fn strip_changes(s: &StateSnapshot) -> StateSnapshot {
    let mut out = s.clone();
    for frame in &mut out.resolution.stack {
        if let EventData::SkillInvocation(d) = &mut frame.data {
            d.jiding_changes.clear();
        }
    }
    out
}

fn assert_rejected(
    e: &Engine,
    pack: &engine_native::content::extension::ExtensionPack,
    bad: &StateSnapshot,
    what: &str,
) -> SnapshotError {
    let err = Engine::restore_with_extensions(bad, &[pack]).err().expect(what);
    reject_restore_atomic(e, &[pack], bad);
    err
}

fn tongji_fixture(equip_payer: bool) -> (engine_native::content::extension::ExtensionPack, Engine) {
    let mut pack = emitter();
    pack.host_skills
        .push(engine_native::content::host::HostSkillDecl {
            name: "yihua",
            behavior: engine_native::content::host::HostBehavior::Yihua,
        });
    let mut e = fixture(&pack, "急叮");
    e.debug_grant(3, "同疾");
    if equip_payer {
        // 原目标只有装备牌：走「装备分支」付款（C9J 反例所在路径）。
        let (id, names) = {
            let cat = e.catalog().clone();
            let idx = cat
                .cards
                .iter()
                .position(|c| c.kind.is_equip() && c.equip_skills.is_empty())
                .expect("bare equipment card");
            (idx as u32, cat.cards[idx].equip_skills.clone())
        };
        e.state_core_mut().deck.retain(|c| *c != id);
        e.state_core_mut().player_mut(2).equips.push(id);
        for n in names {
            e.debug_grant(2, n);
        }
    } else {
        e.debug_place(2, "sha");
    }
    e.debug_grant(3, "yihua");
    e.debug_place(3, "sha");
    e.debug_place(3, "shan");
    (pack, e)
}

#[test]
fn c9l_tongji_payment_proof_is_required_on_both_paths() {
    for equip_payer in [false, true] {
        let (pack, mut e) = tongji_fixture(equip_payer);
        let s = restorable_with(&mut e, &pack, |s| {
            jiding_changes(s)
                .iter()
                .any(|c| matches!(c.cost, JidingCardCost::TongJi { .. }))
        });
        let changes = jiding_changes(&s);
        assert_eq!(changes.len(), 1, "equip={equip_payer}: {changes:?}");
        let JidingCardCost::TongJi { card, loss } = &changes[0].cost else {
            panic!("tongji cost")
        };
        // 两条付款路径都必须留下收据（装备分支此前是 None = 无证明）。
        assert!(loss.is_some(), "equip={equip_payer}: 付款必须带收据");
        // 持久证明：代价牌已进弃牌堆且不在付款者任何牌区。
        let payer = changes[0].before_target;
        assert!(s.core.discard.contains(card), "equip={equip_payer}: 代价牌须入弃牌堆");
        assert!(!s.core.player(payer).hand.contains(card));
        assert!(!s.core.player(payer).equips.contains(card));

        // ① 未付代价（loss 清空）→ 拒绝。
        let err = assert_rejected(
            &e,
            &pack,
            &mutate(&s, |c| {
                if let JidingCardCost::TongJi { loss, .. } = c {
                    *loss = None;
                }
            }),
            "unpaid tongji redirect must be rejected",
        );
        assert!(matches!(err, SnapshotError::BadTag("jiding_card_change", 0)), "{err:?}");
        // ② 代价牌不在弃牌堆（换成另一张牌）→ 拒绝。
        let err = assert_rejected(
            &e,
            &pack,
            &mutate(&s, |c| {
                if let JidingCardCost::TongJi { card, .. } = c {
                    *card = *card ^ 1;
                }
            }),
            "undiscarded tongji cost must be rejected",
        );
        assert!(matches!(err, SnapshotError::BadTag("jiding_card_change", 0)), "{err:?}");
        // ③ 把变更记录整段删掉（载荷已转移却没有证明）→ 拒绝。
        let err = assert_rejected(&e, &pack, &strip_changes(&s), "stripped record must be rejected");
        assert!(matches!(err, SnapshotError::BadTag("jiding_card_change", 0)), "{err:?}");
        // 真实档仍可恢复并继续（修复不能只靠加拒绝）。
        let mut typed = Engine::restore_with_extensions(&s, &[&pack]).expect("real archive");
        super::c331_source_predicates::finish(&mut typed);
    }
}

#[test]
fn c9l_yihua_payment_proof_and_origin_are_required() {
    let mut pack = emitter();
    pack.host_skills
        .push(engine_native::content::host::HostSkillDecl {
            name: "yihua",
            behavior: engine_native::content::host::HostBehavior::Yihua,
        });
    let mut e = fixture(&pack, "急叮");
    e.debug_grant(2, "yihua");
    e.debug_place(2, "sha");
    e.debug_place(2, "shan");
    let s = restorable_with(&mut e, &pack, |s| {
        jiding_changes(s)
            .iter()
            .any(|c| matches!(c.cost, JidingCardCost::Yihua { .. }))
    });
    let changes = jiding_changes(&s);
    assert_eq!(changes.len(), 1, "{changes:?}");
    let JidingCardCost::Yihua { cards, losses } = &changes[0].cost else {
        panic!("yihua cost")
    };
    assert_eq!((cards.len(), losses.len()), (2, 2), "两次付款都要收据");
    for card in cards {
        assert!(s.core.discard.contains(card), "移花代价牌 {card} 须入弃牌堆");
        assert!(!s.core.player(changes[0].actor).hand.contains(card));
        assert!(!s.core.player(changes[0].actor).equips.contains(card));
    }
    // ② 任一代价牌不成立 → 拒绝（两条证明都不可跳过）。
    let err = assert_rejected(
        &e,
        &pack,
        &mutate(&s, |c| {
            if let JidingCardCost::Yihua { cards, .. } = c {
                cards[0] = cards[0] ^ 1;
            }
        }),
        "undiscarded yihua cost must be rejected",
    );
    assert!(matches!(err, SnapshotError::BadTag("jiding_card_change", 0)), "{err:?}");
    // ③ 记录被剥光但载荷已反弹 → 拒绝（origin 绑定不能靠删记录绕过）。
    let err = assert_rejected(&e, &pack, &strip_changes(&s), "stripped yihua record must be rejected");
    assert!(matches!(err, SnapshotError::BadTag("jiding_card_change", 0)), "{err:?}");
    let mut typed = Engine::restore_with_extensions(&s, &[&pack]).expect("real archive");
    super::c331_source_predicates::finish(&mut typed);
}
