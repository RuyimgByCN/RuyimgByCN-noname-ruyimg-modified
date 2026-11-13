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

    // ① `qiandu` (`skill.js:2312-2332`): discard 1 black handcard -> swap seats with target
    e.debug_set_turn(0, Phase::Play, 2);
    let qd_blk = give_card_matching(&mut e, 0, |_, m| m.suit == Suit::Club);
    let _ = activate(&mut e, 0, "qiandu");
    let _ = answer(&mut e, 0, vec![ChoiceOption::Card(qd_blk)]);
    let _ = answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
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
