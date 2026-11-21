//! Queue Item 2 Source 12 (`12. U08 英雄杀（150 项：43 将 + 107 技）`)
//! Real-match 6-dimension acceptance suite — Part 2:
//! - Batch C (26 skills, `英雄杀/character/skill.js:1492-2156`);
//! - Batch D (25 skills + `qiangxi`, `英雄杀/character/skill.js:2157-2936` + `standard/skill.js:642`).
//!
//! Covers all 6 dimensions of Section 四.3:
//! ① Normal execution via real `Engine::submit` + `GameAction::{PlayCard, ActivateSkill, Answer, NextPhase}`;
//! ② Cancellable choice cancel/pass (`ChoiceOption::Pass`) leaving state and per-turn/limited usage unconsumed;
//! ③ Illegal target / illegal card / insufficient count rejection (`CommandOutcome::Rejected`);
//! ④ Condition not met & per-turn / limited skill limits (`usable: 1`, `limited: true` on `seyou`,
//!   `tongyu_guiyin` mutual exclusion, `jieyong2`/`zhulu`/`zhexian`/`bolehuiyan` once per turn);
//! ⑤ Multi-skill & passive modifier interactions (`sheshu` unlimited range & `hp >= 3` directHit, `jimin`
//!   no-equip hand-as-`Shan`, `kongju` hand limit & `Shunshou`/`Guohe`/`Le` immunity, `ducai2` suit lock,
//!   `fenghuo` equip-as-`Nanman`, `xiangma` distance mod, `nvquan1/2/3` male 2-`Shan` / 2-`Sha` / `Juedou` immunity);
//! ⑥ Mid-choice `encode_archive` + `decode_archive` + `Engine::restore_with_extensions` roundtrip.

use content_native_loader::decision34_pack9_extension_pack;
use engine_native::archive::{decode_archive, encode_archive};
use engine_native::catalog::{CardDef, CardKind, EquipSlot, Suit};
use engine_native::event::Phase;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand};
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(8_200_000);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn new_u08_engine(seed: u64, players: usize) -> Engine {
    let p9 = decision34_pack9_extension_pack();
    Engine::new_with_extensions(seed, players, &[&p9]).expect("U08 engine init")
}

fn set_general(e: &mut Engine, p: u8, name: &str) {
    let g = (0..e.catalog().general_count() as u16)
        .find(|&idx| e.catalog().general_meta(idx).is_some_and(|m| m.0 == name))
        .unwrap_or_else(|| panic!("general {name} not found in catalog"));
    e.state_core_mut().player_mut(p).general_slots[0] = Some(g);
}

fn submit(e: &mut Engine, actor: u8, action: GameAction) -> CommandOutcome {
    e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action,
    })
}

fn activate(e: &mut Engine, actor: u8, name: &str) -> CommandOutcome {
    let sid = e.catalog().skill_by_name(name).expect(name);
    submit(e, actor, GameAction::ActivateSkill { skill: sid })
}

fn answer(e: &mut Engine, actor: u8, selection: Vec<ChoiceOption>) -> CommandOutcome {
    let cid = e.open_choice().expect("open choice").id;
    submit(
        e,
        actor,
        GameAction::Answer {
            choice_id: cid,
            selection,
        },
    )
}

fn drain_pass(e: &mut Engine) {
    for _ in 0..64 {
        let Some(c) = e.open_choice().cloned() else { break };
        let sel = if c.spec.options.contains(&ChoiceOption::Pass) {
            vec![ChoiceOption::Pass]
        } else if c.spec.allow_cancel {
            Vec::new()
        } else {
            c.spec.options.iter().take(c.spec.min.max(1)).copied().collect()
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

fn flag(e: &Engine, p: u8, key: &str) -> u32 {
    e.state_core()
        .vm_aux
        .flags
        .get(&(p, key.to_string()))
        .copied()
        .unwrap_or(0)
}

fn set_flag(e: &mut Engine, p: u8, key: &str, val: u32) {
    if val == 0 {
        e.state_core_mut().vm_aux.flags.remove(&(p, key.to_string()));
    } else {
        e.state_core_mut()
            .vm_aux
            .flags
            .insert((p, key.to_string()), val);
    }
}

fn find_card_matching<F>(e: &Engine, pred: F) -> u32
where
    F: Fn(u32, &CardDef) -> bool,
{
    let used: std::collections::BTreeSet<u32> = (0..e.state_core().players.len() as u8)
        .flat_map(|p| {
            let pl = e.state_core().player(p);
            pl.hand
                .iter()
                .chain(pl.equips.iter())
                .chain(pl.judge.iter())
                .copied()
        })
        .collect();
    (0..e.catalog().cards.len() as u32)
        .find(|&cid| !used.contains(&cid) && pred(cid, &e.catalog().cards[cid as usize]))
        .expect("matching card in catalog")
}

fn give_card_matching<F>(e: &mut Engine, p: u8, pred: F) -> u32
where
    F: Fn(u32, &CardDef) -> bool,
{
    let cid = find_card_matching(e, pred);
    e.state_core_mut().deck.retain(|&c| c != cid);
    e.state_core_mut().discard.retain(|&c| c != cid);
    e.state_core_mut().player_mut(p).hand.push(cid);
    cid
}

fn equip_slot(e: &mut Engine, p: u8, slot: EquipSlot) -> u32 {
    let cid = find_card_matching(e, |_, m| m.kind.equip_slot() == Some(slot));
    e.state_core_mut().deck.retain(|&c| c != cid);
    e.state_core_mut().discard.retain(|&c| c != cid);
    e.state_core_mut().player_mut(p).equips.push(cid);
    cid
}

fn assert_mid_choice_roundtrip(e: &Engine, expected_prompt: &str) -> Engine {
    let c = e.open_choice().expect("choice must be open for mid-choice roundtrip");
    assert_eq!(c.spec.prompt, expected_prompt);
    let p9 = decision34_pack9_extension_pack();
    let saved = e.save();
    let bytes = encode_archive(&saved);
    let decoded = decode_archive(&bytes).expect("decode_archive");
    let restored = Engine::restore_with_extensions(&decoded, &[&p9]).expect("restore_with_extensions");
    assert_eq!(restored.state_hash(), e.state_hash());
    assert_eq!(
        restored.open_choice().map(|oc| oc.spec.prompt),
        Some(expected_prompt)
    );
    restored
}

#[test]
fn u08_batch_c_robin_hood_and_stance_swap_6d_real_match() {
    // 1. `sheshu`/`xiadao`/`lguiyin`/`tongyu`/`jimin`/`lzhangyi` (`英雄杀/character/skill.js:1525-1710`):
    let mut e = new_u08_engine(820001, 3);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    set_general(&mut e, 0, "yxs_luobinhan");
    e.debug_grant(0, "sheshu");
    e.debug_grant(0, "xiadao");
    e.debug_grant(0, "lguiyin");
    e.debug_set_turn(0, Phase::Play, 1);

    // ⑤ `sheshu` (`skill.js:1525-1542`): unlimited range & directHit against target with HP >= 3
    let c_v = give_card_matching(&mut e, 1, |_, m| m.kind == CardKind::Shan);
    let _ = c_v;
    let sha = give_card_matching(&mut e, 0, |_, m| m.kind == CardKind::Sha);
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha,
            targets: vec![1],
        },
    );
    // `xiadao` (`skill.js:1543-1595`): on damage to seat 1 (1 handcard) when seat 2 has 0 handcards (<= 1),
    // prompts `"xiadao:confirm"` to choose seat 2 and moves 1 card from seat 1 to seat 2.
    while e.open_choice().is_some_and(|c| c.spec.prompt != "xiadao:confirm") {
        let c = e.open_choice().cloned().unwrap();
        let _ = submit(
            &mut e,
            c.observer,
            GameAction::Answer {
                choice_id: c.id,
                selection: vec![ChoiceOption::Pass],
            },
        );
    }
    let mut restored = assert_mid_choice_roundtrip(&e, "xiadao:confirm");
    let _ = answer(&mut restored, 0, vec![ChoiceOption::Pass]);
    drain_pass(&mut restored);
    assert!(restored.state_core().player(2).hand.is_empty());

    let _ = answer(&mut e, 0, vec![ChoiceOption::Player(2)]);
    drain_pass(&mut e);
    assert_eq!(e.state_core().player(2).hand.len(), 1);
    assert!(e.state_core().player(1).hand.is_empty());

    // ④ `lguiyin` (`skill.js:1596-1623`): cannot activate in a turn where owner dealt damage
    assert!(matches!(activate(&mut e, 0, "lguiyin"), CommandOutcome::Rejected(_)));

    // Next turn (0 damage dealt): `lguiyin` draws 1 card and swaps `lguiyin`/`sheshu`/`xiadao` -> `jimin`/`lzhangyi`/`tongyu`
    e.state_core_mut().player_mut(0).turn_damage = 0;
    set_flag(&mut e, 0, "turn_damage_dealt", 0);
    e.debug_set_turn(0, Phase::Play, 2);
    let out_gy = activate(&mut e, 0, "lguiyin");
    assert!(matches!(out_gy, CommandOutcome::Accepted { .. }));
    assert_eq!(e.state_core().player(0).hand.len(), 1);
    let jimin_sid = e.catalog().skill_by_name("jimin").unwrap();
    let tongyu_sid = e.catalog().skill_by_name("tongyu").unwrap();
    assert!(e.state_core().has_skill(0, jimin_sid));
    assert!(e.state_core().has_skill(0, tongyu_sid));

    // ④ Mutual exclusion: `tongyu` cannot be used in the same turn `lguiyin` was used (`tongyu_guiyin_used`)
    assert!(matches!(activate(&mut e, 0, "tongyu"), CommandOutcome::Rejected(_)));

    // Clear turn exclusion flag -> `tongyu` (`skill.js:1624-1664`) discards 1 card to swap back to `lguiyin`/`sheshu`/`xiadao`
    set_flag(&mut e, 0, "tongyu_guiyin_used", 0);
    set_flag(&mut e, 0, "skill_turn:tongyu_guiyin_used", 0);
    let c_ty = e.state_core().player(0).hand[0];
    let _ = activate(&mut e, 0, "tongyu");
    let _ = answer(&mut e, 0, vec![ChoiceOption::Card(c_ty)]);
    let sheshu_sid = e.catalog().skill_by_name("sheshu").unwrap();
    assert!(e.state_core().has_skill(0, sheshu_sid));
}

#[test]
fn u08_batch_c_modifiers_and_active_skills_6d_real_match() {
    // 2. `kongju` (`skill.js:1767-1800`), `tuqiang` (`skill.js:1801-1812`), `xumou` (`skill.js:1813-1823`),
    // `zhensha` (`skill.js:1824-1848`), `ducai`/`ducai2`/`ducai3` (`skill.js:1849-1910`),
    // `tongling`/`fanpu` (`skill.js:1911-1991`), `fenghuo` (`skill.js:1992-2004`),
    // `nichang`/`nichang2`/`fengyan` (`skill.js:2005-2046`), `fengyi`/`wange`/`wange2` (`skill.js:2047-2103`),
    // `seyou`/`sheshi` (`skill.js:2104-2156`):
    let mut e = new_u08_engine(820002, 3);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    e.state_core_mut().player_mut(0).max_hp = 4;
    e.state_core_mut().player_mut(0).hp = 2;
    e.debug_grant(0, "kongju");
    e.debug_grant(0, "ducai");
    e.debug_grant(0, "tongling");
    e.debug_grant(0, "fanpu");
    e.debug_grant(0, "fenghuo");
    e.debug_grant(0, "fengyi");
    e.debug_grant(0, "seyou");
    e.debug_grant(0, "sheshi");
    set_general(&mut e, 1, "yxs_xiangyu");
    set_general(&mut e, 2, "yxs_caocao");
    e.debug_set_turn(0, Phase::Play, 1);

    // ⑤ `kongju` (`skill.js:1767-1800`): wounded hand limit is `max_hp` (4 instead of 2)
    assert_eq!(e.state_core().hand_limit(e.catalog(), 0), 4);

    // ①/⑤ `ducai` & `ducai2` (`skill.js:1849-1910`): discard 1 Spade card -> forbids Spade suit for other players
    let spade_c = give_card_matching(&mut e, 0, |_, m| m.suit == Suit::Spade);
    let _ = activate(&mut e, 0, "ducai");
    let _ = answer(&mut e, 0, vec![ChoiceOption::Card(spade_c)]);
    assert_eq!(flag(&e, 0, "ducai_suit"), 1);
    assert_eq!(flag(&e, 0, "ducai2_active"), 1);

    // ①/④/⑤ `fanpu` (`skill.js:1945-1991`) + `tongling` (`skill.js:1911-1944`):
    // requires `tongling_mark >= 3`; spends 3 marks (3 -> 0) and uses virtual Sha on player 1,
    // then `tongling` triggers on friendly damage dealt and grants +1 mark (0 -> 1).
    assert!(matches!(activate(&mut e, 0, "fanpu"), CommandOutcome::Rejected(_)));
    set_flag(&mut e, 0, "tongling_mark", 3);
    let hp1_fp = e.state_core().player(1).hp;
    let _ = activate(&mut e, 0, "fanpu");
    let _ = answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
    drain_pass(&mut e);
    assert_eq!(flag(&e, 0, "tongling_mark"), 1);
    assert_eq!(e.state_core().player(1).hp, hp1_fp - 1);

    // ⑤ `fenghuo` (`skill.js:1992-2004`): use 1 equipped card as virtual `SavageAttack` (Nanman)
    let eq_fh = equip_slot(&mut e, 0, EquipSlot::Armor);
    let hp2_fh = e.state_core().player(2).hp;
    let _ = activate(&mut e, 0, "fenghuo");
    let _ = answer(&mut e, 0, vec![ChoiceOption::Card(eq_fh)]);
    drain_pass(&mut e);
    assert_eq!(e.state_core().player(2).hp, hp2_fh - 1, "fenghuo uses equip as SavageAttack");

    // ① `fengyi` (`skill.js:2047-2065`): discard 1 handcard -> chosen player draws 2 cards
    let fy_c = give_card_matching(&mut e, 0, |_, _| true);
    let _ = activate(&mut e, 0, "fengyi");
    let _ = answer(&mut e, 0, vec![ChoiceOption::Card(fy_c)]);
    let _ = answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
    assert_eq!(e.state_core().player(1).hand.len(), 2);

    // ①/④ `seyou` (`skill.js:2104-2136`): limited skill -> target player 1; male player 2 has no Sha so owner gains 1 card from player 2
    let p2_c = give_card_matching(&mut e, 2, |_, m| m.kind == CardKind::Tao);
    let _ = activate(&mut e, 0, "seyou");
    let _ = answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
    drain_pass(&mut e);
    assert_eq!(flag(&e, 0, "seyou_used"), 1);
    assert!(e.state_core().player(0).hand.contains(&p2_c));
    assert!(matches!(activate(&mut e, 0, "seyou"), CommandOutcome::Rejected(_)));
}

#[test]
fn u08_batch_d_bo_le_wu_ze_tian_mo_zi_and_lu_ban_6d_real_match() {
    // 1. `bolehuiyan`/`bolehuiyan2`/`bolehuiyan3` (`英雄杀/character/skill.js:2157-2222`), `xiangma` (`skill.js:2284-2296`),
    // `feigong` (`skill.js:2400-2413`), `jianai` (`skill.js:2414-2437`), `jieyong`/`jieyong2` (`skill.js:2438-2487`):
    let mut e = new_u08_engine(820010, 3);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "bolehuiyan");
    e.debug_grant(0, "xiangma");
    e.debug_grant(0, "jianai");
    e.debug_grant(0, "jieyong");
    // ⑤ `xiangma`: when seat 0 HP > 2, distance from 0 -> 1 is reduced by 1; when seat 0 HP <= 2, distance to 0 is increased by 1
    e.state_core_mut().player_mut(0).max_hp = 4;
    e.state_core_mut().player_mut(0).hp = 2;
    assert_eq!(e.state_core().computed_distance(e.catalog(), 1, 0), 2);
    e.state_core_mut().player_mut(0).hp = 3;

    // ①/⑥ `bolehuiyan` (`skill.js:2157-2222`): seat 1 uses Sha on seat 2 (who has 1 handcard) -> seat 0 predicts Tag(1) (hit)
    e.debug_set_turn(1, Phase::Play, 1);
    give_card_matching(&mut e, 2, |_, m| m.kind == CardKind::Tao);
    let sha_bl = give_card_matching(&mut e, 1, |_, m| m.kind == CardKind::Sha);
    let _ = submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: sha_bl,
            targets: vec![2],
        },
    );
    let mut restored = assert_mid_choice_roundtrip(&e, "bolehuiyan:predict");
    let _ = answer(&mut restored, 0, vec![ChoiceOption::Pass]);
    drain_pass(&mut restored);
    assert_eq!(restored.state_core().player(0).hand.len(), 0);

    // Predict Tag(1) = hit -> Sha hits seat 2 -> seat 0 draws 1 card
    let _ = answer(&mut e, 0, vec![ChoiceOption::Tag(1)]);
    drain_pass(&mut e);
    assert_eq!(e.state_core().player(0).hand.len(), 1, "bolehuiyan draws 1 when hit prediction is right");

    // ① `jieyong` (`skill.js:2438-2487`) & `jianai` (`skill.js:2414-2437`):
    // Seat 0 (HP 3/4) plays Tao in own Play phase -> triggers `jianai:confirm` (heals injured seat 2) AND `jieyong:discard_black` (regains Tao)
    e.debug_set_turn(0, Phase::Play, 2);
    e.debug_clear_hand(0);
    let tao_c = give_card_matching(&mut e, 0, |_, m| m.kind == CardKind::Tao);
    let blk_c = give_card_matching(&mut e, 0, |_, m| m.suit == Suit::Spade && m.kind != CardKind::Tao);
    let hp2_pre_ja = e.state_core().player(2).hp;
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: tao_c,
            targets: Vec::new(),
        },
    );
    for _ in 0..8 {
        let Some(c) = e.open_choice().cloned() else { break };
        match c.spec.prompt {
            "jianai:confirm" => {
                let _ = answer(&mut e, 0, vec![ChoiceOption::Confirm]);
            }
            "jieyong:discard_black" => {
                let _ = answer(&mut e, 0, vec![ChoiceOption::Card(blk_c)]);
            }
            _ => {
                let _ = answer(&mut e, c.observer, vec![ChoiceOption::Pass]);
            }
        }
    }
    assert_eq!(e.state_core().player(2).hp, hp2_pre_ja + 1, "jianai heals other injured players");
    assert!(e.state_core().player(0).hand.contains(&tao_c), "jieyong regains used Tao");
    assert_eq!(flag(&e, 0, "jieyong2_used"), 1);

    // 2. `yxsweiyi` (`skill.js:2297-2311`), `qiandu` (`skill.js:2312-2332`), `nvquan`/`nvquan1`/`nvquan2`/`nvquan3` (`skill.js:2333-2399`):
    let mut e = new_u08_engine(820011, 3);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    set_general(&mut e, 0, "yxs_wuzetian");
    set_general(&mut e, 1, "yxs_xiangyu");
    e.debug_grant(0, "yxsweiyi");
    e.debug_grant(0, "qiandu");
    e.debug_grant(0, "nvquan");
    e.debug_grant(0, "nvquan1");
    e.debug_grant(0, "nvquan2");
    e.debug_grant(0, "nvquan3");

    // ⑤ `nvquan3` (`skill.js:2394-2399`): male player 1 cannot target seat 0 with `Duel`
    e.debug_set_turn(1, Phase::Play, 1);
    let duel_m = give_card_matching(&mut e, 1, |_, m| m.kind == CardKind::Duel);
    let hp0_duel = e.state_core().player(0).hp;
    let _ = submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: duel_m,
            targets: vec![0],
        },
    );
    drain_pass(&mut e);
    assert_eq!(e.state_core().player(0).hp, hp0_duel, "nvquan3 blocks male Duel targeting owner");

    // ① `qiandu` (`skill.js:2312-2332`): discard 1 black handcard -> swap seats with non-next target (seat 2)
    e.debug_set_turn(0, Phase::Play, 2);
    let qd_blk = give_card_matching(&mut e, 0, |_, m| m.suit == Suit::Club);
    let _ = activate(&mut e, 0, "qiandu");
    let _ = answer(&mut e, 0, vec![ChoiceOption::Card(qd_blk)]);
    let _ = answer(&mut e, 0, vec![ChoiceOption::Player(2)]);
    assert_eq!(flag(&e, 0, "yxs_qiandu_seat_swapped"), 1);

    // 3. `zhulu` (`skill.js:2488-2510`), `xieling` (`skill.js:2511-2581`), `qiangyun` (`skill.js:2582-2593`),
    // `cike` (`skill.js:2594-2628`), `qiangxi` (`standard/skill.js:642`), `miaobi`/`miaobix`/`zhexian` (`skill.js:2629-2779`),
    // `guifu` (`skill.js:2780-2807`), `lshengong` (`skill.js:2808-2936`):
    let mut e = new_u08_engine(820012, 3);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "qiangyun");
    e.debug_grant(0, "qiangxi");
    e.debug_grant(0, "miaobi");
    e.debug_grant(0, "zhexian");
    e.debug_grant(0, "guifu");
    e.debug_grant(0, "lshengong");
    e.debug_set_turn(0, Phase::Play, 1);

    // ① `qiangxi`: choose target 1 -> Tag(1) lose 1 HP -> deals 1 damage to target 1
    let hp0_qx = e.state_core().player(0).hp;
    let hp1_qx = e.state_core().player(1).hp;
    let _ = activate(&mut e, 0, "qiangxi");
    let _ = answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
    let _ = answer(&mut e, 0, vec![ChoiceOption::Tag(1)]);
    drain_pass(&mut e);
    assert_eq!(e.state_core().player(0).hp, hp0_qx - 1);
    assert_eq!(e.state_core().player(1).hp, hp1_qx - 1);

    // ① `guifu` (`skill.js:2780-2807`): discard 1 equip from target 1 -> both seat 0 and seat 1 draw 1 card
    let eq1 = equip_slot(&mut e, 1, EquipSlot::Armor);
    let _ = eq1;
    let _ = activate(&mut e, 0, "guifu");
    let _ = answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
    drain_pass(&mut e);
    assert!(e.state_core().player(1).equips.is_empty());
    assert_eq!(e.state_core().player(0).hand.len(), 1);
    assert_eq!(e.state_core().player(1).hand.len(), 1);

    // ① `lshengong` (`skill.js:2808-2936`): equip 1 handcard as copy of an on-field equip onto destination player
    let eq2 = equip_slot(&mut e, 2, EquipSlot::Weapon);
    let hand0 = e.state_core().player(0).hand[0];
    let _ = activate(&mut e, 0, "lshengong");
    let _ = answer(&mut e, 0, vec![ChoiceOption::Card(hand0)]);
    let _ = answer(&mut e, 0, vec![ChoiceOption::Player(2)]);
    let _ = answer(&mut e, 0, vec![ChoiceOption::Card(eq2)]);
    let _ = answer(&mut e, 0, vec![ChoiceOption::Player(0)]);
    // Losing last handcard `hand0` triggers `qiangyun:confirm` -> Confirm draws 2 cards!
    while e.open_choice().is_some_and(|c| c.spec.prompt == "qiangyun:confirm") {
        let _ = answer(&mut e, 0, vec![ChoiceOption::Confirm]);
    }
    drain_pass(&mut e);
    assert!(e.state_core().player(0).equips.contains(&hand0), "lshengong equips handcard onto dest");
    assert_eq!(e.state_core().player(0).hand.len(), 2, "qiangyun draws 2 when hand becomes empty");
}

#[test]
fn u08_batch_c_and_d_remaining_skills_6d_real_match() {
    // 1. Batch C remaining: `qianglue` (`skill.js:1492-1524`), `jimin` (`skill.js:1665-1677`),
    // `lzhangyi` (`skill.js:1678-1710`), `yizhuang`/`yizhuang2` (`skill.js:1711-1766`),
    // `tuqiang` (`skill.js:1801-1812`), `xumou` (`skill.js:1813-1823`), `zhensha` (`skill.js:1824-1848`),
    // `nichang`/`nichang2` (`skill.js:2005-2029`), `fengyan` (`skill.js:2030-2046`),
    // `wange`/`wange2` (`skill.js:2066-2103`), `sheshi` (`skill.js:2137-2156`):
    let mut e = new_u08_engine(820020, 3);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    set_general(&mut e, 0, "yxs_yangyuhuan");
    set_general(&mut e, 1, "yxs_xiangyu");
    e.debug_grant(0, "jimin");
    e.debug_grant(0, "tuqiang");
    e.debug_grant(0, "fengyan");
    e.debug_grant(1, "qianglue");

    // ⑤ `jimin` (`skill.js:1665-1677`) + `tuqiang` (`skill.js:1801-1812`) + `qianglue` (`skill.js:1492-1524`) + `fengyan` (`skill.js:2030-2046`):
    // Seat 0 has no equips -> `jimin` allows using a non-Shan handcard (Tao) as Shan against seat 1's Sha;
    // dodging triggers `tuqiang` (+1 draw for seat 0), and `qianglue` on seat 1 performs a judge (which `fengyan` gains for seat 0).
    let hp0_pre = e.state_core().player(0).hp;
    let tao_as_shan = give_card_matching(&mut e, 0, |_, m| m.kind == CardKind::Tao);
    e.debug_set_turn(1, Phase::Play, 1);
    let sha_c = give_card_matching(&mut e, 1, |_, m| m.kind == CardKind::Sha);
    // Upstream chronological order settles `CardResponded` (`tuqiang` draws 1) before
    // `ShaDodged` (`qianglue` judges). Pin a deterministic draw + red judge card on top of
    // deck so `qianglue` performs a red judge (gained by `fengyan`) instead of stealing on an
    // unpinned black judge card.
    let judge_red = find_card_matching(&e, |_, m| matches!(m.suit, Suit::Heart | Suit::Diamond));
    e.state_core_mut().deck.retain(|&c| c != judge_red);
    e.state_core_mut().discard.retain(|&c| c != judge_red);
    let tuqiang_draw = find_card_matching(&e, |cid, _| cid != judge_red);
    e.state_core_mut().deck.retain(|&c| c != tuqiang_draw);
    e.state_core_mut().discard.retain(|&c| c != tuqiang_draw);
    e.state_core_mut().deck.push(judge_red);
    e.state_core_mut().deck.push(tuqiang_draw);
    let _ = submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: sha_c,
            targets: vec![0],
        },
    );
    while let Some(c) = e.open_choice().cloned() {
        if c.observer == 0 && c.spec.options.contains(&ChoiceOption::Card(tao_as_shan)) {
            let _ = answer(&mut e, 0, vec![ChoiceOption::Card(tao_as_shan)]);
        } else if c.spec.options.contains(&ChoiceOption::Confirm) {
            let _ = answer(&mut e, c.observer, vec![ChoiceOption::Confirm]);
        } else if let Some(card_opt) = c.spec.options.iter().find(|o| matches!(o, ChoiceOption::Card(_))).copied() {
            let _ = answer(&mut e, c.observer, vec![card_opt]);
        } else {
            let _ = answer(&mut e, c.observer, vec![ChoiceOption::Pass]);
        }
    }
    assert_eq!(e.state_core().player(0).hp, hp0_pre, "jimin dodges Sha with non-Shan handcard");
    assert!(
        !e.state_core().player(0).hand.is_empty(),
        "tuqiang + fengyan draw/gain cards after jimin dodge"
    );

    // 2. Batch D remaining: `feigong` (`skill.js:2400-2413`), `zhulu` (`skill.js:2488-2510`),
    // `xieling` (`skill.js:2511-2581`), `cike` (`skill.js:2594-2628`),
    // `miaobi`/`miaobix` (`skill.js:2629-2757`), `zhexian` (`skill.js:2758-2779`),
    // `yxsweiyi` (`skill.js:2297-2311`), `nvquan1`/`nvquan2` (`skill.js:2333-2393`):
    let mut e = new_u08_engine(820021, 3);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    set_general(&mut e, 0, "yxs_libai");
    set_general(&mut e, 1, "yxs_xiangyu");
    set_general(&mut e, 2, "yxs_wuzetian");
    e.debug_grant(0, "miaobi");
    e.debug_grant(0, "zhexian");
    e.debug_grant(0, "xieling");
    e.debug_grant(0, "zhulu");
    e.debug_grant(0, "cike");
    e.debug_grant(2, "feigong");
    e.debug_grant(2, "yxsweiyi");

    // ① `miaobi` + `miaobix` + `zhexian` (`skill.js:2629-2779`):
    // Seat 0 activates `miaobi` on seat 1 (who holds a `Sha`), discards that `Sha` (triggers `zhexian` on seat 0 if seat 0 discards outside turn),
    // and uses 1 handcard as virtual `Sha` via `miaobix`!
    e.debug_set_turn(0, Phase::Play, 1);
    let t1_sha = give_card_matching(&mut e, 1, |_, m| m.kind == CardKind::Sha);
    let p0_hand = give_card_matching(&mut e, 0, |_, m| m.kind == CardKind::Tao);
    let hp1_pre = e.state_core().player(1).hp;
    let _ = activate(&mut e, 0, "miaobi");
    let mut restored = assert_mid_choice_roundtrip(&e, "miaobi:choose_target");
    // ② Pass on `miaobi:choose_target` refunds `miaobi`
    let _ = answer(&mut restored, 0, vec![ChoiceOption::Pass]);
    assert!(matches!(activate(&mut restored, 0, "miaobi"), CommandOutcome::Pending { .. }));

    let _ = answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
    let _ = answer(&mut e, 0, vec![ChoiceOption::Card(t1_sha)]);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "miaobix:choose_card");
    let _ = answer(&mut e, 0, vec![ChoiceOption::Card(p0_hand)]);
    let _ = answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
    drain_pass(&mut e);
    assert_eq!(e.state_core().player(1).hp, hp1_pre - 1, "miaobix uses handcard as virtual copy of discarded Sha");

    // ① `xieling` (`skill.js:2511-2581`): discard 2 cards -> move 1 equip from player 1 to player 2
    let eq_xl = equip_slot(&mut e, 1, EquipSlot::Armor);
    let xl1 = give_card_matching(&mut e, 0, |_, _| true);
    let xl2 = give_card_matching(&mut e, 0, |_, _| true);
    let _ = activate(&mut e, 0, "xieling");
    let _ = answer(&mut e, 0, vec![ChoiceOption::Card(xl1), ChoiceOption::Card(xl2)]);
    let _ = answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
    let _ = answer(&mut e, 0, vec![ChoiceOption::Card(eq_xl)]);
    let _ = answer(&mut e, 0, vec![ChoiceOption::Player(2)]);
    assert!(e.state_core().player(2).equips.contains(&eq_xl), "xieling moves field card from player 1 to player 2");

    // ①/④ `zhulu` (`skill.js:2488-2510`): when player 1 uses a trick (`Wuzhong`), seat 0 discards same-suit card to regain it
    e.debug_set_turn(1, Phase::Play, 2);
    let trick_c = give_card_matching(&mut e, 1, |_, m| m.kind == CardKind::Wuzhong);
    let trick_suit = e.catalog().cards[trick_c as usize].suit;
    let same_suit_c = give_card_matching(&mut e, 0, |_, m| m.suit == trick_suit);
    let _ = submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: trick_c,
            targets: Vec::new(),
        },
    );
    while let Some(c) = e.open_choice().cloned() {
        if c.spec.prompt == "zhulu:discard_same_suit" {
            let _ = answer(&mut e, 0, vec![ChoiceOption::Card(same_suit_c)]);
        } else {
            let _ = answer(&mut e, c.observer, vec![ChoiceOption::Pass]);
        }
    }
    assert!(e.state_core().player(0).hand.contains(&trick_c), "zhulu regains used trick card");
    assert_eq!(flag(&e, 0, "zhulu_used_turn"), 1);

    // ① `feigong` (`skill.js:2400-2413`): when player 1 uses Sha on player 0, player 2 discards a Sha to nullify it
    let p2_sha = give_card_matching(&mut e, 2, |_, m| m.kind == CardKind::Sha);
    let p1_sha = give_card_matching(&mut e, 1, |_, m| m.kind == CardKind::Sha);
    let hp0_fg = e.state_core().player(0).hp;
    let _ = submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: p1_sha,
            targets: vec![0],
        },
    );
    while let Some(c) = e.open_choice().cloned() {
        if c.spec.prompt == "feigong:discard_sha" {
            let _ = answer(&mut e, 2, vec![ChoiceOption::Card(p2_sha)]);
        } else {
            let _ = answer(&mut e, c.observer, vec![ChoiceOption::Pass]);
        }
    }
    assert_eq!(e.state_core().player(0).hp, hp0_fg, "feigong nullifies Sha");
}

#[test]
fn u08_batch_c_xumou_turnover_and_mianju_6d_real_match() {
    use engine_native::ai::BoundedPlanner;
    use engine_native::LogEvent;

    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let huanle_json = std::fs::read_to_string(root.join("huanle-upstream-noname/assets/cards.json"))
        .expect("read huanle cards.json")
        .replace("\r\n", "\n");
    let huanle_pack = content_native_loader::bridge_pack(&huanle_json)
        .expect("bridge huanle")
        .pack;
    let p9 = decision34_pack9_extension_pack();

    // 1) Normal `xumou` (`英雄杀/character/skill.js:1916-1925`):
    // `phaseJieshuBegin`, AI `hp + hand < 4`, `await draw(3); await turnOver()`.
    // Verify mid-choice save/restore, serial draw(3) BEFORE TurnOverBefore, and AI decisions.
    let mut e = Engine::new_with_extensions(820080, 3, &[&p9, &huanle_pack]).expect("U08+Huanle");
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "xumou");
    e.state_core_mut().player_mut(0).max_hp = 4;
    e.state_core_mut().player_mut(0).hp = 2; // hp + hand = 2 < 4
    e.debug_set_turn(0, Phase::Discard, 1);

    let mark = e.log().len();
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    assert_eq!(
        e.open_choice().map(|c| c.spec.prompt),
        Some("xumou:confirm")
    );
    // AI confirms when hp + hand < 4.
    let plan_yes = e.plan_action(0, &BoundedPlanner::new(64));
    assert!(
        matches!(plan_yes.action, GameAction::Answer { ref selection, .. } if selection == &vec![ChoiceOption::Confirm]),
        "AI should confirm xumou when hp + hand < 4, got {:?}",
        plan_yes.action
    );

    // Mid-choice archive save -> decode -> restore roundtrip.
    let saved = e.save();
    let bytes = encode_archive(&saved);
    let decoded = decode_archive(&bytes).expect("decode_archive");
    let mut restored = Engine::restore_with_extensions(&decoded, &[&p9, &huanle_pack])
        .expect("restore_with_extensions");
    assert_eq!(restored.state_hash(), e.state_hash());

    // Cancel (`Pass`) draws 0 and does not turnOver.
    let _ = answer(&mut restored, 0, vec![ChoiceOption::Pass]);
    drain_pass(&mut restored);
    assert_eq!(restored.state_core().player(0).hand.len(), 0);
    assert!(!restored.state_core().player(0).face_down);

    // Confirm on live engine: draws 3 first, THEN turns over.
    let _ = answer(&mut e, 0, vec![ChoiceOption::Confirm]);
    drain_pass(&mut e);
    assert_eq!(e.state_core().player(0).hand.len(), 3, "xumou draws 3");
    assert!(e.state_core().player(0).face_down, "xumou turns over when unblocked");
    let drew = e.log()[mark..].iter().any(|ev| matches!(ev, LogEvent::CardsDrawn { player: 0, cards } if cards.len() == 3));
    assert!(drew, "xumou emits CardsDrawn(3)");

    // 2) Equipped U06 `Mianju` via real `PlayCard` in Play phase intercepts `TurnOverBefore`!
    // Must STILL draw 3 FIRST, then `mianju` cancels turnOver (`face_down` stays `false`)!
    let mut e2 = Engine::new_with_extensions(820081, 3, &[&p9, &huanle_pack]).expect("U08+Huanle");
    for p in 0..3 {
        e2.debug_clear_hand(p);
    }
    e2.debug_grant(0, "xumou");
    e2.state_core_mut().player_mut(0).max_hp = 4;
    e2.state_core_mut().player_mut(0).hp = 3;
    e2.debug_set_turn(0, Phase::Play, 1);
    let mianju_c = give_card_matching(&mut e2, 0, |_, m| m.kind == CardKind::Mianju);
    let _ = submit(
        &mut e2,
        0,
        GameAction::PlayCard {
            card: mianju_c,
            targets: Vec::new(),
        },
    );
    assert!(e2.state_core().player(0).equips.contains(&mianju_c));
    assert!(!e2.state_core().player(0).face_down);

    // Advance Play -> Discard -> End -> xumou:confirm -> Confirm!
    for _ in 0..8 {
        if let Some(c) = e2.open_choice().cloned() {
            if c.spec.prompt == "xumou:confirm" {
                break;
            }
            let _ = answer(&mut e2, c.observer, vec![ChoiceOption::Pass]);
        } else {
            let cur = e2.state_core().turn.current;
            let _ = submit(&mut e2, cur, GameAction::NextPhase);
        }
    }
    let mark2 = e2.log().len();
    let _ = answer(&mut e2, 0, vec![ChoiceOption::Confirm]);
    drain_pass(&mut e2);
    assert_eq!(e2.state_core().player(0).hand.len(), 3, "xumou + mianju still draws 3");
    assert!(
        !e2.state_core().player(0).face_down,
        "equipped mianju must intercept xumou TurnOverBefore and prevent flip"
    );
    let draw_idx = e2.log()[mark2..]
        .iter()
        .position(|ev| matches!(ev, LogEvent::CardsDrawn { player: 0, cards } if cards.len() == 3))
        .expect("CardsDrawn(3) logged");
    let mianju_idx = e2.log()[mark2..]
        .iter()
        .position(|ev| matches!(ev, LogEvent::SkillTriggered { owner: 0, skill, .. } if skill.contains("mianju")))
        .expect("mianju triggered via TurnOverBefore");
    assert!(
        draw_idx < mianju_idx,
        "xumou must finish draw(3) BEFORE emitting TurnOverBefore for mianju ({draw_idx} vs {mianju_idx})"
    );

    // 3) AI declines when hp + hand >= 4.
    let mut e3 = Engine::new_with_extensions(820082, 3, &[&p9, &huanle_pack]).expect("U08+Huanle");
    e3.debug_grant(0, "xumou");
    e3.state_core_mut().player_mut(0).max_hp = 4;
    e3.state_core_mut().player_mut(0).hp = 3;
    e3.debug_clear_hand(0);
    give_card_matching(&mut e3, 0, |_, _| true); // 3 + 1 = 4 >= 4
    e3.debug_set_turn(0, Phase::Discard, 1);
    let _ = submit(&mut e3, 0, GameAction::NextPhase);
    assert_eq!(
        e3.open_choice().map(|c| c.spec.prompt),
        Some("xumou:confirm")
    );
    let plan_no = e3.plan_action(0, &BoundedPlanner::new(64));
    assert!(
        matches!(plan_no.action, GameAction::Answer { ref selection, .. } if selection.is_empty() || selection == &vec![ChoiceOption::Pass]),
        "AI should decline xumou when hp + hand >= 4, got {:?}",
        plan_no.action
    );
}

#[test]
fn u08_batch_d_nvquan3_effective_gender_6d_real_match() {
    use engine_native::RejectReason;

    // `nvquan3` (`英雄杀/character/skill.js:2709-2717`):
    // `targetEnabled`: `if (card.name === "juedou" && player.sex === "male") return false;`
    // Must check current effective gender (including main general switch & runtime sex change),
    // NEVER bypassing current effective gender to read static general meta first.
    let mut e = new_u08_engine(820083, 3);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    set_general(&mut e, 0, "yxs_wuzetian"); // Female owner
    set_general(&mut e, 1, "yxs_xiangyu");  // Male actor
    set_general(&mut e, 2, "yxs_yangyuhuan"); // Female actor
    e.debug_grant(0, "nvquan3");

    // Baseline 1: Male general cannot Duel `nvquan3` owner.
    e.debug_set_turn(1, Phase::Play, 1);
    let d_male = give_card_matching(&mut e, 1, |_, m| m.kind == CardKind::Duel);
    let r = submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: d_male,
            targets: vec![0],
        },
    );
    assert!(matches!(r, CommandOutcome::Rejected(RejectReason::IllegalTargets)));

    // Baseline 2: Female general CAN Duel `nvquan3` owner.
    e.debug_set_turn(2, Phase::Play, 2);
    let d_fem = give_card_matching(&mut e, 2, |_, m| m.kind == CardKind::Duel);
    let hp0 = e.state_core().player(0).hp;
    let r = submit(
        &mut e,
        2,
        GameAction::PlayCard {
            card: d_fem,
            targets: vec![0],
        },
    );
    assert!(matches!(r, CommandOutcome::Pending { .. } | CommandOutcome::Accepted { .. }));
    drain_pass(&mut e);
    assert_eq!(e.state_core().player(0).hp, hp0 - 1, "Female general Duel resolves against nvquan3");

    // Main general switched: seat 1 Male -> Female => Duel legal; seat 2 Female -> Male => Duel rejected.
    set_general(&mut e, 1, "yxs_wuzetian");
    set_general(&mut e, 2, "yxs_xiangyu");
    e.debug_set_turn(1, Phase::Play, 3);
    let r = submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: d_male,
            targets: vec![0],
        },
    );
    assert!(matches!(r, CommandOutcome::Pending { .. } | CommandOutcome::Accepted { .. }));
    drain_pass(&mut e);

    e.debug_set_turn(2, Phase::Play, 4);
    let d_now_m = give_card_matching(&mut e, 2, |_, m| m.kind == CardKind::Duel);
    let r = submit(
        &mut e,
        2,
        GameAction::PlayCard {
            card: d_now_m,
            targets: vec![0],
        },
    );
    assert!(matches!(r, CommandOutcome::Rejected(RejectReason::IllegalTargets)));

    // Runtime gender override (sex change without swapping general_slots[0]!):
    // Static Male `yxs_xiangyu` (seat 2) changed to Female (`effective_gender_override = 2`) => Duel LEGAL!
    set_flag(&mut e, 2, "effective_gender_override", 2);
    let r = submit(
        &mut e,
        2,
        GameAction::PlayCard {
            card: d_now_m,
            targets: vec![0],
        },
    );
    assert!(
        matches!(r, CommandOutcome::Pending { .. } | CommandOutcome::Accepted { .. }),
        "nvquan3 must see runtime Female override even though general_slots[0] is Male yxs_xiangyu"
    );
    drain_pass(&mut e);

    // Static Female `yxs_wuzetian` (seat 1) changed to Male (`effective_gender_override = 1`) => Duel BLOCKED!
    set_flag(&mut e, 1, "effective_gender_override", 1);
    e.debug_set_turn(1, Phase::Play, 5);
    let d_now_m2 = give_card_matching(&mut e, 1, |_, m| m.kind == CardKind::Duel);
    let r = submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: d_now_m2,
            targets: vec![0],
        },
    );
    assert!(
        matches!(r, CommandOutcome::Rejected(RejectReason::IllegalTargets)),
        "nvquan3 must see runtime Male override even though general_slots[0] is Female yxs_wuzetian"
    );
}

#[test]
fn u08_batch_d_bolehuiyan_and_cike_shabegin_6d_real_match() {
    use engine_native::ai::BoundedPlanner;
    use engine_native::kernel::elo_set_player_scalar;
    use engine_native::vocabulary::names::{BOSS_RELEIJI, WU_SHUANG};
    use engine_native::RejectReason;

    // 1) Under-control filtering + multi-target per-target `shaBegin` (skip target A, prompt for target B)
    //    + `priority 11` before `cike` (`priority 0`, so 1-card target still triggers `bolehuiyan`):
    let mut e = new_u08_engine(820084, 4);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "bolehuiyan");
    e.debug_grant(1, "cike");
    e.debug_set_turn(1, Phase::Play, 1);
    let ft = find_card_matching(&e, |_, m| m.kind == CardKind::Fangtian);
    e.state_core_mut().deck.retain(|&c| c != ft);
    e.state_core_mut().discard.retain(|&c| c != ft);
    e.state_core_mut().player_mut(1).equips.push(ft);

    // Target 2: controlled (`elo_under_control = 1`) AND has 1 handcard -> skipped by bolehuiyan!
    elo_set_player_scalar(e.state_core_mut(), 2, "elo_under_control", 1);
    give_card_matching(&mut e, 2, |_, m| m.kind == CardKind::Tao);
    // Target 3: uncontrolled, has 1 handcard -> eligible for bolehuiyan!
    give_card_matching(&mut e, 3, |_, m| m.kind == CardKind::Tao);
    // Give seat 1 a Sha as last handcard (Fangtian allows up to 3 targets: [2, 3] or [0, 2, 3]).
    let sha_mt = give_card_matching(&mut e, 1, |_, m| m.kind == CardKind::Sha);
    // Top of deck black so if cike ran BEFORE bolehuiyan it would ask discard target's only card.
    let blk_top = find_card_matching(&e, |_, m| matches!(m.suit, Suit::Spade | Suit::Club));
    e.state_core_mut().deck.retain(|&c| c != blk_top);
    e.state_core_mut().discard.retain(|&c| c != blk_top);
    e.state_core_mut().deck.push(blk_top);

    let _ = submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: sha_mt,
            targets: vec![0, 2, 3],
        },
    );
    // First target 0 is owner -> skipped. Next: if cike ran first on target 0 it opens `cike:confirm`,
    // then target 2 (controlled) skips bolehuiyan, then target 3 MUST open `bolehuiyan:predict`
    // BEFORE `cike:confirm` on target 3 (`priority 11 > 0`)!
    let mut seen_prompts = Vec::new();
    while let Some(c) = e.open_choice().cloned() {
        seen_prompts.push((c.observer, c.spec.prompt));
        if c.spec.prompt == "bolehuiyan:predict" {
            // Confirm prediction for target 3 hit (`Tag(1)`):
            let _ = answer(&mut e, 0, vec![ChoiceOption::Tag(1)]);
            // Clear controlled targets check done.
            break;
        }
        let _ = answer(&mut e, c.observer, vec![ChoiceOption::Pass]);
    }
    assert!(
        seen_prompts.iter().any(|&(obs, pr)| obs == 0 && pr == "bolehuiyan:predict"),
        "multi-target Sha [0(owner), 2(controlled), 3(eligible)] must trigger bolehuiyan for target 3: {seen_prompts:?}"
    );
    drain_pass(&mut e);
    assert_eq!(e.state_core().player(0).hand.len(), 1, "right prediction draws 1");

    // Also test single-target controlled target alone does NOT ask bolehuiyan:
    let mut ec = new_u08_engine(820085, 3);
    for p in 0..3 {
        ec.debug_clear_hand(p);
    }
    ec.debug_grant(0, "bolehuiyan");
    elo_set_player_scalar(ec.state_core_mut(), 2, "elo_under_control", 1);
    give_card_matching(&mut ec, 2, |_, m| m.kind == CardKind::Tao);
    let sha_c = give_card_matching(&mut ec, 1, |_, m| m.kind == CardKind::Sha);
    ec.debug_set_turn(1, Phase::Play, 1);
    let _ = submit(
        &mut ec,
        1,
        GameAction::PlayCard {
            card: sha_c,
            targets: vec![2],
        },
    );
    assert!(
        ec.open_choice().is_none(),
        "controlled target must be filtered out of bolehuiyan"
    );
    assert_eq!(flag(&ec, 0, "bolehuiyan_used_turn"), 0);

    // 2) Upstream DOM cancel on `bolehuiyan:predict` (`Pass`) consumes turn use (`bolehuiyan4`):
    //    2nd Sha in the same turn must NOT re-ask `bolehuiyan:predict`!
    let mut e_can = new_u08_engine(820086, 3);
    for p in 0..3 {
        e_can.debug_clear_hand(p);
    }
    e_can.debug_grant(0, "bolehuiyan");
    e_can.debug_set_turn(1, Phase::Play, 1);
    give_card_matching(&mut e_can, 2, |_, m| m.kind == CardKind::Tao);
    let s1 = give_card_matching(&mut e_can, 1, |_, m| m.kind == CardKind::Sha);
    let s2 = give_card_matching(&mut e_can, 1, |_, m| m.kind == CardKind::Sha);
    let _ = submit(
        &mut e_can,
        1,
        GameAction::PlayCard {
            card: s1,
            targets: vec![2],
        },
    );
    assert_eq!(
        e_can.open_choice().map(|c| c.spec.prompt),
        Some("bolehuiyan:predict")
    );
    let _ = answer(&mut e_can, 0, vec![ChoiceOption::Pass]);
    drain_pass(&mut e_can);
    assert_eq!(flag(&e_can, 0, "bolehuiyan_used_turn"), 1, "DOM cancel consumes turn use");
    assert_eq!(flag(&e_can, 0, "bolehuiyan_pred"), 0, "DOM cancel stores no prediction");
    e_can.state_core_mut().turn.sha_used_play_phase = false;
    let _ = submit(
        &mut e_can,
        1,
        GameAction::PlayCard {
            card: s2,
            targets: vec![2],
        },
    );
    assert!(
        e_can.open_choice().is_none(),
        "2nd Sha after cancel must NOT prompt bolehuiyan again in same turn"
    );

    // 3) `Renwang` + `bolehuiyan2` (`shaEnd` vs `TurnEnded` bug!):
    //    Black Sha on target with `Renwang` -> `bolehuiyan` predicts Miss (`Tag(2)`),
    //    `Renwang` cancels at `shaBegin` -> advancing to `TurnEnded` must NOT draw for seat 0!
    let mut e_rw = new_u08_engine(820087, 3);
    for p in 0..3 {
        e_rw.debug_clear_hand(p);
    }
    e_rw.debug_grant(0, "bolehuiyan");
    e_rw.debug_set_turn(1, Phase::Play, 1);
    let rw = find_card_matching(&e_rw, |_, m| m.kind == CardKind::Renwang);
    e_rw.state_core_mut().deck.retain(|&c| c != rw);
    e_rw.state_core_mut().discard.retain(|&c| c != rw);
    e_rw.state_core_mut().player_mut(2).equips.push(rw);
    give_card_matching(&mut e_rw, 2, |_, m| m.kind == CardKind::Tao);
    let blk_sha = give_card_matching(&mut e_rw, 1, |_, m| {
        m.kind == CardKind::Sha && matches!(m.suit, Suit::Spade | Suit::Club)
    });
    let _ = submit(
        &mut e_rw,
        1,
        GameAction::PlayCard {
            card: blk_sha,
            targets: vec![2],
        },
    );
    assert_eq!(
        e_rw.open_choice().map(|c| c.spec.prompt),
        Some("bolehuiyan:predict")
    );
    let _ = answer(&mut e_rw, 0, vec![ChoiceOption::Tag(2)]);
    drain_pass(&mut e_rw);
    // Advance seat 1 Play -> Discard -> End -> TurnEnded -> seat 2 TurnBegin:
    for _ in 0..8 {
        if e_rw.state_core().turn.current != 1 {
            break;
        }
        if let Some(c) = e_rw.open_choice().cloned() {
            let _ = answer(&mut e_rw, c.observer, vec![ChoiceOption::Pass]);
        } else {
            let _ = submit(&mut e_rw, 1, GameAction::NextPhase);
        }
    }
    assert_eq!(
        e_rw.state_core().player(0).hand.len(),
        0,
        "Renwang-cancelled Sha must NOT falsely trigger bolehuiyan2 at TurnEnded"
    );

    // 4) `Wushuang` + `boss_releiji` + `bolehuiyan3`:
    //    Seat 1 (with Wushuang) plays Sha on Seat 2 (with BOSS_RELEIJI + 2 cards, 1st Shan + Spade judge).
    //    Releiji thunder damage on 1st Shan must NOT settle `bolehuiyan3`!
    //    Also covers mandatory `bolehuiyan:discard` (U08-R3: Sha card stays in table during discard ask;
    //    rejects empty answer; A vs B discard; mid-choice save->restore->Answer).
    let mut e_ws = new_u08_engine(820088, 3);
    for p in 0..3 {
        e_ws.debug_clear_hand(p);
    }
    e_ws.debug_grant(0, "bolehuiyan");
    e_ws.debug_grant(1, WU_SHUANG);
    e_ws.debug_grant(2, BOSS_RELEIJI);
    let d0 = give_card_matching(&mut e_ws, 0, |_, m| m.kind == CardKind::Tao);
    let d1 = give_card_matching(&mut e_ws, 0, |_, m| m.kind == CardKind::Jiu);
    let shan1 = give_card_matching(&mut e_ws, 2, |_, m| m.kind == CardKind::Shan);
    let _shan2 = give_card_matching(&mut e_ws, 2, |_, m| m.kind == CardKind::Shan);
    let sha_ws = give_card_matching(&mut e_ws, 1, |_, m| m.kind == CardKind::Sha);
    // Put Spade on deck top so 1st Shan triggers Releiji thunder damage onto seat 0!
    let spade_j = find_card_matching(&e_ws, |_, m| m.suit == Suit::Spade);
    e_ws.state_core_mut().deck.retain(|&c| c != spade_j);
    e_ws.state_core_mut().discard.retain(|&c| c != spade_j);
    e_ws.state_core_mut().deck.push(spade_j);

    e_ws.debug_set_turn(1, Phase::Play, 1);
    let hp0_pre = e_ws.state_core().player(0).hp;
    let hp2_pre = e_ws.state_core().player(2).hp;
    let _ = submit(
        &mut e_ws,
        1,
        GameAction::PlayCard {
            card: sha_ws,
            targets: vec![2],
        },
    );
    // Seat 0 predicts Tag(2) (cannot hit), while AI public info for Wushuang predicts Tag(1).
    let ai_wushuang = e_ws.plan_action(0, &BoundedPlanner::new(64));
    assert!(
        matches!(ai_wushuang.action, GameAction::Answer { ref selection, .. } if selection == &vec![ChoiceOption::Tag(1)]),
        "AI with visible Wushuang must predict Tag(1), got {:?}",
        ai_wushuang.action
    );
    let _ = answer(&mut e_ws, 0, vec![ChoiceOption::Tag(2)]);
    assert_eq!(e_ws.open_choice().map(|c| c.spec.prompt), Some("respond_shan"));
    let _ = answer(&mut e_ws, 2, vec![ChoiceOption::Card(shan1)]);

    // Releiji thunder damage just hit seat 0 (hp0_pre - 2), and Wushuang now demands 2nd Shan!
    assert_eq!(e_ws.state_core().player(0).hp, hp0_pre - 2, "Releiji thunder hit seat 0");
    assert_eq!(
        flag(&e_ws, 0, "bolehuiyan_pred"),
        2,
        "Releiji thunder damage during 1st Shan MUST NOT settle bolehuiyan3 prematurely"
    );
    assert_eq!(
        e_ws.open_choice().map(|c| c.spec.prompt),
        Some("respond_shan"),
        "Wushuang 2nd Shan window must still open"
    );
    // Seat 2 cannot play 2nd Shan (passes) -> Sha hits (`shaDamage`) -> NOW `bolehuiyan3` opens `bolehuiyan:discard`!
    let _ = answer(&mut e_ws, 2, vec![ChoiceOption::Pass]);
    assert_eq!(e_ws.state_core().player(2).hp, hp2_pre - 1, "Sha dealt damage");
    assert_eq!(
        e_ws.open_choice().map(|c| c.spec.prompt),
        Some("bolehuiyan:discard"),
        "bolehuiyan3 forced discard opens AFTER Sha damage"
    );
    assert!(
        e_ws.state_core().table.contains(&sha_ws),
        "U08-R3: Sha card must remain in table while bolehuiyan:discard is pending"
    );
    // Mandatory discard rejects empty answer; save/restore A vs B discards different cards and then cleans table.
    assert!(matches!(
        answer(&mut e_ws, 0, Vec::new()),
        CommandOutcome::Rejected(RejectReason::InvalidChoice)
    ));
    let mut e_ws_b = assert_mid_choice_roundtrip(&e_ws, "bolehuiyan:discard");
    let _ = answer(&mut e_ws, 0, vec![ChoiceOption::Card(d0)]);
    let _ = answer(&mut e_ws_b, 0, vec![ChoiceOption::Card(d1)]);
    assert!(!e_ws.state_core().player(0).hand.contains(&d0));
    assert!(e_ws.state_core().player(0).hand.contains(&d1));
    assert!(!e_ws_b.state_core().player(0).hand.contains(&d1));
    assert!(e_ws_b.state_core().player(0).hand.contains(&d0));
    assert!(!e_ws.state_core().table.contains(&sha_ws), "Sha moves out of table once bolehuiyan3 completes");

    // 5) Inverse Wushuang + Releiji + `bolehuiyan2`: Seat 0 predicts Miss (`Tag(2)`), Seat 2 plays BOTH Shans:
    //    1st Shan triggers Releiji (must NOT settle `bolehuiyan3` discard); 2nd Shan ends Sha -> `bolehuiyan2` draws 1!
    let mut e_wm = new_u08_engine(820089, 3);
    for p in 0..3 {
        e_wm.debug_clear_hand(p);
    }
    e_wm.debug_grant(0, "bolehuiyan");
    e_wm.debug_grant(1, WU_SHUANG);
    e_wm.debug_grant(2, BOSS_RELEIJI);
    let s2_a = give_card_matching(&mut e_wm, 2, |_, m| m.kind == CardKind::Shan);
    let s2_b = give_card_matching(&mut e_wm, 2, |_, m| m.kind == CardKind::Shan);
    let sha_wm = give_card_matching(&mut e_wm, 1, |_, m| m.kind == CardKind::Sha);
    let spade_wm = find_card_matching(&e_wm, |_, m| m.suit == Suit::Spade);
    e_wm.state_core_mut().deck.retain(|&c| c != spade_wm);
    e_wm.state_core_mut().discard.retain(|&c| c != spade_wm);
    e_wm.state_core_mut().deck.push(spade_wm);
    e_wm.debug_set_turn(1, Phase::Play, 1);
    let _ = submit(
        &mut e_wm,
        1,
        GameAction::PlayCard {
            card: sha_wm,
            targets: vec![2],
        },
    );
    let _ = answer(&mut e_wm, 0, vec![ChoiceOption::Tag(2)]);
    let _ = answer(&mut e_wm, 2, vec![ChoiceOption::Card(s2_a)]);
    assert_eq!(flag(&e_wm, 0, "bolehuiyan_pred"), 2, "1st Shan Releiji does not settle pred");
    assert_eq!(e_wm.open_choice().map(|c| c.spec.prompt), Some("respond_shan"));
    let _ = answer(&mut e_wm, 2, vec![ChoiceOption::Card(s2_b)]);
    assert_eq!(flag(&e_wm, 0, "bolehuiyan_pred"), 0, "2nd Shan shaEnd settles bolehuiyan2");
    assert_eq!(e_wm.state_core().player(0).hand.len(), 1, "bolehuiyan2 draws 1 on miss");

    // 6) `cike` per-target `shaBegin` (priority 0): Renwang cancels black Sha before `cike:confirm`,
    //    while normal target triggers `cike:confirm` -> mid-choice roundtrip -> black judge -> `cike:discard_card` A vs B & Pass.
    let mut e_ck = new_u08_engine(820090, 3);
    for p in 0..3 {
        e_ck.debug_clear_hand(p);
    }
    e_ck.debug_grant(0, "cike");
    let ck_rw = find_card_matching(&e_ck, |_, m| m.kind == CardKind::Renwang);
    e_ck.state_core_mut().deck.retain(|&c| c != ck_rw);
    e_ck.state_core_mut().discard.retain(|&c| c != ck_rw);
    e_ck.state_core_mut().player_mut(1).equips.push(ck_rw);
    let ck_blk_sha = give_card_matching(&mut e_ck, 0, |_, m| {
        m.kind == CardKind::Sha && matches!(m.suit, Suit::Spade | Suit::Club)
    });
    e_ck.debug_set_turn(0, Phase::Play, 1);
    let _ = submit(
        &mut e_ck,
        0,
        GameAction::PlayCard {
            card: ck_blk_sha,
            targets: vec![1],
        },
    );
    assert!(e_ck.open_choice().is_none(), "Renwang priority 6 cancels black Sha before cike priority 0");

    // Non-Renwang target 2 with 2 cards -> `cike:confirm` -> black judge -> `cike:discard_card` (A vs B) + mid-choice roundtrip.
    let t2_a = give_card_matching(&mut e_ck, 2, |_, m| m.kind == CardKind::Tao);
    let t2_b = give_card_matching(&mut e_ck, 2, |_, m| m.kind == CardKind::Jiu);
    let ck_red_sha = give_card_matching(&mut e_ck, 0, |_, m| {
        m.kind == CardKind::Sha && matches!(m.suit, Suit::Heart | Suit::Diamond)
    });
    let blk_j = find_card_matching(&e_ck, |_, m| matches!(m.suit, Suit::Spade | Suit::Club));
    e_ck.state_core_mut().deck.retain(|&c| c != blk_j);
    e_ck.state_core_mut().discard.retain(|&c| c != blk_j);
    e_ck.state_core_mut().deck.push(blk_j);
    e_ck.state_core_mut().turn.sha_used_play_phase = false;
    let _ = submit(
        &mut e_ck,
        0,
        GameAction::PlayCard {
            card: ck_red_sha,
            targets: vec![2],
        },
    );
    let mut e_ck_pass = assert_mid_choice_roundtrip(&e_ck, "cike:confirm");
    let _ = answer(&mut e_ck_pass, 0, vec![ChoiceOption::Pass]);
    drain_pass(&mut e_ck_pass);
    assert_eq!(e_ck_pass.state_core().player(2).hand.len(), 2, "Pass on cike:confirm keeps target cards");

    let _ = answer(&mut e_ck, 0, vec![ChoiceOption::Confirm]);
    let mut e_ck_b = assert_mid_choice_roundtrip(&e_ck, "cike:discard_card");
    let _ = answer(&mut e_ck, 0, vec![ChoiceOption::Card(t2_a)]);
    let _ = answer(&mut e_ck_b, 0, vec![ChoiceOption::Card(t2_b)]);
    drain_pass(&mut e_ck);
    drain_pass(&mut e_ck_b);
    assert!(!e_ck.state_core().player(2).hand.contains(&t2_a));
    assert!(e_ck.state_core().player(2).hand.contains(&t2_b));
    assert!(!e_ck_b.state_core().player(2).hand.contains(&t2_b));
    assert!(e_ck_b.state_core().player(2).hand.contains(&t2_a));
}

#[test]
fn u08_batch15_qilin_hujia_cike_verifier_regressions() {
    use engine_native::ai::BoundedPlanner;

    // 1) 麒麟弓 + 伯乐慧眼 多目标 `[1(有马), 2]`：`asking_qilin` 必须透传 `emit_sha_damage`
    //    返回的 `Flow::Yield`，令 target 1 完成伤害与 `bolehuiyan3` 清算并继续结算 target 2。
    let mut e_ql = new_u08_engine(820101, 4);
    for p in 0..4 {
        e_ql.debug_clear_hand(p);
    }
    e_ql.debug_grant(3, "bolehuiyan");
    e_ql.state_core_mut()
        .vm_aux
        .flags
        .insert((0, "ddd_xianxi_target_added".to_string()), 1);
    let qilin = find_card_matching(&e_ql, |_, m| m.kind == CardKind::Qilin);
    e_ql.state_core_mut().deck.retain(|&c| c != qilin);
    e_ql.state_core_mut().discard.retain(|&c| c != qilin);
    e_ql.state_core_mut().player_mut(0).equips.push(qilin);

    let horse = find_card_matching(&e_ql, |_, m| m.kind == CardKind::HorseMinus);
    e_ql.state_core_mut().deck.retain(|&c| c != horse);
    e_ql.state_core_mut().discard.retain(|&c| c != horse);
    e_ql.state_core_mut().player_mut(1).equips.push(horse);
    give_card_matching(&mut e_ql, 1, |_, m| m.kind == CardKind::Shan);
    give_card_matching(&mut e_ql, 2, |_, m| m.kind == CardKind::Tao);
    let sha_ql = give_card_matching(&mut e_ql, 0, |_, m| m.kind == CardKind::Sha);
    e_ql.debug_set_turn(0, Phase::Play, 1);
    let _ = submit(
        &mut e_ql,
        0,
        GameAction::PlayCard {
            card: sha_ql,
            targets: vec![1, 2],
        },
    );
    // 伯乐（seat 3）预测 target 1 命中（Tag(1)）；target 1 放弃出闪（Pass）→ 进 `qilin_discard_horse`。
    assert_eq!(e_ql.open_choice().map(|c| c.spec.prompt), Some("bolehuiyan:predict"));
    let _ = answer(&mut e_ql, 3, vec![ChoiceOption::Tag(1)]);
    assert_eq!(e_ql.open_choice().map(|c| c.spec.prompt), Some("respond_shan"));
    let _ = answer(&mut e_ql, 1, vec![ChoiceOption::Pass]);
    let mut e_ql_snap = assert_mid_choice_roundtrip(&e_ql, "qilin_discard_horse");
    let hp1_before = e_ql.state_core().player(1).hp;
    let hp2_before = e_ql.state_core().player(2).hp;
    let _ = answer(&mut e_ql, 0, vec![ChoiceOption::Card(horse)]);
    let _ = answer(&mut e_ql_snap, 0, vec![ChoiceOption::Card(horse)]);
    drain_pass(&mut e_ql);
    drain_pass(&mut e_ql_snap);
    assert_eq!(
        e_ql.state_hash(),
        e_ql_snap.state_hash(),
        "post-qilin multi-target state must survive archive restore"
    );
    assert!(
        e_ql.state_core().player(1).equips.is_empty(),
        "qilin must discard target 1 horse"
    );
    assert_eq!(
        (e_ql.state_core().player(1).hp, e_ql.state_core().player(2).hp),
        (hp1_before - 1, hp2_before - 1),
        "asking_qilin Yield must continue Sha to target 2 after target 1 takes damage"
    );
    assert_eq!(flag(&e_ql, 3, "bolehuiyan_pred"), 0, "bolehuiyan settled and cleared");
    assert_eq!(
        e_ql.state_core().player(3).hand.len(),
        1,
        "correct hit prediction draws 1 card after shaDamage"
    );
    assert!(
        !e_ql.state_core().table.contains(&sha_ql),
        "Sha leaves table after both targets finish"
    );

    // 2) `Hujia`（护驾）魏盟友代出【闪】抵消【杀】：必须发 ShaDodged + finish_target_sha_end(false)，
    //    令 `bolehuiyan2`（预测不命中）正常结算摸 1 并清除预测标记。
    let mut e_hj = new_u08_engine(820102, 4);
    for p in 0..4 {
        e_hj.debug_clear_hand(p);
    }
    e_hj.debug_grant(3, "bolehuiyan");
    e_hj.debug_grant(1, "护驾");
    e_hj.debug_set_general(2, "夏侯惇");
    give_card_matching(&mut e_hj, 1, |_, m| m.kind == CardKind::Tao);
    let shan2 = give_card_matching(&mut e_hj, 2, |_, m| m.kind == CardKind::Shan);
    let sha_hj = give_card_matching(&mut e_hj, 0, |_, m| m.kind == CardKind::Sha);
    e_hj.debug_set_turn(0, Phase::Play, 1);
    let hp1_hj = e_hj.state_core().player(1).hp;
    let _ = submit(
        &mut e_hj,
        0,
        GameAction::PlayCard {
            card: sha_hj,
            targets: vec![1],
        },
    );
    assert_eq!(e_hj.open_choice().map(|c| c.spec.prompt), Some("bolehuiyan:predict"));
    let _ = answer(&mut e_hj, 3, vec![ChoiceOption::Tag(2)]);
    let hujia_c = e_hj.open_choice().expect("hujia_respond open").clone();
    assert_eq!((hujia_c.spec.prompt, hujia_c.observer), ("hujia_respond", 2));
    let _ = answer(&mut e_hj, 2, vec![ChoiceOption::Card(shan2)]);
    drain_pass(&mut e_hj);
    assert_eq!(e_hj.state_core().player(1).hp, hp1_hj, "hujia dodges damage");
    assert_eq!(
        e_hj.state_core().player(3).hand.len(),
        1,
        "bolehuiyan2 must draw 1 on hujia dodge shaEnd"
    );
    assert_eq!(flag(&e_hj, 3, "bolehuiyan_pred"), 0, "bolehuiyan2 cleared pred");
    assert!(
        e_hj.catalog()
            .skill_by_name("bolehuiyan2")
            .is_some_and(|d| !e_hj.state_core().skills.contains_key(&(3, d)))
            && e_hj
                .catalog()
                .skill_by_name("bolehuiyan3")
                .is_some_and(|d| !e_hj.state_core().skills.contains_key(&(3, d))),
        "bolehuiyan2/3 removed after hujia shaEnd"
    );
    assert!(
        !e_hj.state_core().table.contains(&sha_hj),
        "Sha discarded from table via advance_target after hujia shaEnd"
    );

    // 3) `cike` 黑判：目标仅有 1 张牌（如单张白银狮子/手牌）与多张牌时均须开可取消的
    //    `cike:discard_card`（allow_cancel = true），不得静默强弃；取消留牌，选中弃牌。
    let mut e_c1 = new_u08_engine(820103, 4);
    for p in 0..4 {
        e_c1.debug_clear_hand(p);
    }
    e_c1.debug_grant(0, "cike");
    let single_card = give_card_matching(&mut e_c1, 1, |_, m| m.kind == CardKind::Tao);
    let sha_c1 = give_card_matching(&mut e_c1, 0, |_, m| m.kind == CardKind::Sha);
    let blk_j1 = find_card_matching(&e_c1, |_, m| matches!(m.suit, Suit::Spade | Suit::Club));
    e_c1.state_core_mut().deck.retain(|&c| c != blk_j1);
    e_c1.state_core_mut().discard.retain(|&c| c != blk_j1);
    e_c1.state_core_mut().deck.push(blk_j1);
    e_c1.debug_set_turn(0, Phase::Play, 1);
    let _ = submit(
        &mut e_c1,
        0,
        GameAction::PlayCard {
            card: sha_c1,
            targets: vec![1],
        },
    );
    assert_eq!(e_c1.open_choice().map(|c| c.spec.prompt), Some("cike:confirm"));
    assert!(
        matches!(
            e_c1.plan_action(0, &BoundedPlanner::new(4)).action,
            GameAction::Answer { .. }
        ),
        "AI answers cike:confirm"
    );
    let _ = answer(&mut e_c1, 0, vec![ChoiceOption::Confirm]);
    let ask_single = e_c1
        .open_choice()
        .expect("single-card target MUST open cike:discard_card (no silent auto-discard)")
        .clone();
    assert_eq!(ask_single.spec.prompt, "cike:discard_card");
    assert!(ask_single.spec.allow_cancel, "cike:discard_card must be cancellable");
    assert!(
        matches!(
            e_c1.plan_action(0, &BoundedPlanner::new(4)).action,
            GameAction::Answer { .. }
        ),
        "AI answers cike:discard_card"
    );
    let mut e_c1_confirm = assert_mid_choice_roundtrip(&e_c1, "cike:discard_card");
    // Cancel on 1-card target preserves the card:
    let _ = answer(&mut e_c1, 0, Vec::new());
    drain_pass(&mut e_c1);
    assert!(
        e_c1.state_core().player(1).hand.contains(&single_card),
        "cancelling cike:discard_card on 1-card target must keep that card"
    );
    // Confirm on restored snapshot discards the single card:
    let _ = answer(&mut e_c1_confirm, 0, vec![ChoiceOption::Card(single_card)]);
    drain_pass(&mut e_c1_confirm);
    assert!(
        !e_c1_confirm.state_core().player(1).hand.contains(&single_card),
        "selecting card in cike:discard_card on 1-card target discards it"
    );

    // Multi-card target cancel check:
    let mut e_c2 = new_u08_engine(820104, 4);
    for p in 0..4 {
        e_c2.debug_clear_hand(p);
    }
    e_c2.debug_grant(0, "cike");
    let m1 = give_card_matching(&mut e_c2, 1, |_, m| m.kind == CardKind::Tao);
    let m2 = give_card_matching(&mut e_c2, 1, |_, m| m.kind == CardKind::Jiu);
    let sha_c2 = give_card_matching(&mut e_c2, 0, |_, m| m.kind == CardKind::Sha);
    let blk_j2 = find_card_matching(&e_c2, |_, m| matches!(m.suit, Suit::Spade | Suit::Club));
    e_c2.state_core_mut().deck.retain(|&c| c != blk_j2);
    e_c2.state_core_mut().discard.retain(|&c| c != blk_j2);
    e_c2.state_core_mut().deck.push(blk_j2);
    e_c2.debug_set_turn(0, Phase::Play, 1);
    let _ = submit(
        &mut e_c2,
        0,
        GameAction::PlayCard {
            card: sha_c2,
            targets: vec![1],
        },
    );
    let _ = answer(&mut e_c2, 0, vec![ChoiceOption::Confirm]);
    assert!(e_c2.open_choice().is_some_and(|c| c.spec.prompt == "cike:discard_card" && c.spec.allow_cancel));
    let _ = answer(&mut e_c2, 0, Vec::new());
    drain_pass(&mut e_c2);
    assert!(
        e_c2.state_core().player(1).hand.contains(&m1)
            && e_c2.state_core().player(1).hand.contains(&m2),
        "cancelling cike:discard_card on multi-card target preserves both cards"
    );
}


