//! U05-3 (`shaai-ow`) Batch C 真实事件闭环测试：
//! 黑百合 (`ow_heibaihe`: `juji`/`juji2`/`juji3`, `duwen`/`duwen2`, `dulei`/`dulei2`)、
//! D.Va (`ow_dva`: `jijia`, `chongzhuang`, `tuijin`/`tuijin2`, `zihui`)、
//! 死神 (`ow_sishen`: `xiandan`, `shouge`, `yihun`/`yihun2`)、
//! 士兵76 (`ow_shibing`: `tuji`, `mujing`/`mujing2`, `lichang`/`lichang2`)、
//! 源氏 (`ow_yuanshi`: `feiren`, `feiren3`, `lianpo`, `zhanlong`/`zhanlong2`)。

use engine_native::catalog::{CardKind, Suit};
use engine_native::event::DamageNature;
use engine_native::{ChoiceOption, CommandOutcome, GameAction, Phase, RejectReason};

use super::common::{
    activate, advance_to_play, advance_to_turn_end_prompt, answer, answer_cancel,
    assert_card_conservation, assert_open_prompt, clear_all_hands, drain_cancel, place_any_suit,
    place_kind, place_kind_suit, play, roundtrip_restore, start_formal_ow, submit,
};

/// 黑百合 (`character/ow/skill.js:1380-1630`):
/// - `juji` 弃 1~4 张**花色互异**的牌（同花色混选重弹提示），若目标手牌含其中任一花色则获 `juji2`/`juji3`（对其距离=1 且杀不可闪避）；
/// - `dulei` 暗置 1 手牌入 `ow_dulei` 并获 `dulei2`；遭其他角色使用牌指定时若该牌花色吻合 `ow_dulei` 则触发 `LoseHp(1)` + 随机弃 1 + 移出陷阱（守恒）。
#[test]
fn ow_batch_c_widowmaker_juji_duwen_dulei() {
    let (_verified, mut e) = start_formal_ow(2026_1150, 4);
    e.assign_general_named(0, "ow_heibaihe").unwrap();
    advance_to_play(&mut e, 0);
    clear_all_hands(&mut e);

    // 1. 狙击：弃黑桃+红桃（同色重弹），目标 2 号手牌有红桃闪 → 命中 `juji2` + `juji3`
    let sp1 = place_kind_suit(&mut e, 0, CardKind::Sha, Suit::Spade);
    let sp2 = place_any_suit(&mut e, 0, Suit::Spade);
    let ht1 = place_kind_suit(&mut e, 0, CardKind::Tao, Suit::Heart);
    let _target2_ht_shan = place_kind_suit(&mut e, 2, CardKind::Shan, Suit::Heart);

    activate(&mut e, 0, "juji");
    assert_open_prompt(&e, "ow_juji_cards", 0);
    answer(&mut e, vec![ChoiceOption::Card(sp1), ChoiceOption::Card(sp2)]);
    assert_open_prompt(&e, "ow_juji_cards", 0);
    answer(&mut e, vec![ChoiceOption::Card(sp1), ChoiceOption::Card(ht1)]);
    assert_open_prompt(&e, "ow_juji_target", 0);
    answer(&mut e, vec![ChoiceOption::Player(2)]);
    assert!(e.debug_has_skill(0, "juji2"));
    assert!(e.debug_has_skill(0, "juji3"));
    assert_eq!(e.state_core().computed_distance(e.catalog(), 0, 2), 1, "狙击2 距离变 1");

    e.debug_set_hp(2, 4);
    let sha0 = place_kind_suit(&mut e, 0, CardKind::Sha, Suit::Club);
    play(&mut e, 0, sha0, vec![2]);
    drain_cancel(&mut e);
    assert!(e.debug_hp(2) <= 3, "狙击3 强命击中 2 号（无视手里红桃闪）");

    // 2. 毒雷：`dulei` 埋入一张梅花牌到 `ow_dulei`
    let trap = place_any_suit(&mut e, 0, Suit::Club);
    activate(&mut e, 0, "dulei");
    assert_open_prompt(&e, "ow_dulei_card", 0);
    answer(&mut e, vec![ChoiceOption::Card(trap)]);
    assert!(e.debug_has_skill(0, "dulei2"));
    assert_eq!(e.state_core().vm_aux.piles[&(0, "ow_dulei".to_string())], vec![trap]);
    assert_card_conservation(&e, "dulei placed");

    // 1 号回合：对 0 号使用**梅花**过河拆桥 → 匹配 `ow_dulei` 花色触发 `LoseHp` + 随机弃 1
    advance_to_play(&mut e, 1);
    assert!(!e.debug_has_skill(0, "juji2"), "狙击2 回合结束自动清除");
    clear_all_hands(&mut e);
    e.debug_set_hp(1, 4);
    let club_gh = place_kind_suit(&mut e, 1, CardKind::Dismantle, Suit::Club);
    let _fodder1 = place_kind_suit(&mut e, 1, CardKind::Shan, Suit::Heart);
    play(&mut e, 1, club_gh, vec![0]);
    drain_cancel(&mut e);
    assert_eq!(e.debug_hp(1), 3, "毒雷花色匹配触发 1 号失去 1 点体力");
    assert!(!e.debug_has_skill(0, "dulei2"), "毒雷触发后拆除");
    assert_card_conservation(&e, "widowmaker end");
}

/// D.Va (`character/ow/skill.js:1821-2075`):
/// - `jijia` 初始 `jijia_hp = 4`（手牌上限 +4 且伤害优先由机甲血 1:1 吸收）；
/// - `chongzhuang` 造成伤害累计充能 4 点回满机甲；
/// - `tuijin` 弃 0 牌仅当 `jijia_hp > 0` 且对距离 > 1 者设 `tuijin2` 令距离 = 1；
/// - `zihui` 距<=2 者交 `jijia_hp` 张牌或受 2 点火焰伤害，随后 `jijia_hp = 0`。
#[test]
fn ow_batch_c_dva_jijia_chongzhuang_tuijin_zihui() {
    let (verified, mut e) = start_formal_ow(2026_1151, 4);
    e.assign_general_named(0, "ow_dva").unwrap();
    advance_to_play(&mut e, 0);
    clear_all_hands(&mut e);

    assert_eq!(e.state_core().player(0).jijia_hp, 4, "机甲初始 4 血");
    assert_eq!(e.state_core().effective_hand_limit(e.catalog(), 0), 6, "机甲在位时手牌上限 +4");
    e.debug_deal_damage(Some(1), 0, 2, DamageNature::Normal);
    drain_cancel(&mut e);
    assert_eq!(e.state_core().player(0).jijia_hp, 2, "机甲抵扣 2 点伤害");
    assert_eq!(e.debug_hp(0), 2, "D.Va 本体未损血");

    activate(&mut e, 0, "tuijin");
    assert_open_prompt(&e, "ow_tuijin_target", 0);
    answer(&mut e, vec![ChoiceOption::Player(2)]);
    assert_eq!(e.state_core().computed_distance(e.catalog(), 0, 2), 1, "推进令到 2 号距离=1");

    e.debug_set_hp(1, 4);
    e.debug_set_hp(3, 4);
    let c1a = place_kind_suit(&mut e, 1, CardKind::Sha, Suit::Spade);
    let c1b = place_kind_suit(&mut e, 1, CardKind::Shan, Suit::Heart);
    let _c3a = place_any_suit(&mut e, 3, Suit::Club);
    let _c3b = place_kind_suit(&mut e, 3, CardKind::Shan, Suit::Diamond);
    activate(&mut e, 0, "zihui");
    let mut restored_zihui = false;
    for _ in 0..8 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
        if c.spec.prompt == "ow_zihui_discard" {
            if !restored_zihui {
                e = roundtrip_restore(&e, &verified);
                restored_zihui = true;
            }
            if c.observer == 1 {
                answer(&mut e, vec![ChoiceOption::Card(c1a), ChoiceOption::Card(c1b)]);
            } else {
                answer_cancel(&mut e);
            }
        } else {
            drain_cancel(&mut e);
        }
    }
    assert!(restored_zihui, "zihui 途中需命中一次 mid-choice save/restore");
    assert_eq!(e.debug_hp(1), 4, "1 号弃 2 牌躲过自毁");
    assert_eq!(e.debug_hp(3), 2, "3 号取消弃牌受 2 点火焰伤害");
    assert_eq!(e.state_core().player(0).jijia_hp, 0, "自毁后机甲归零");

    e.debug_set_hp(1, 6);
    e.debug_deal_damage(Some(0), 1, 2, DamageNature::Normal);
    assert_eq!(e.state_core().player(0).jijia_hp, 0);
    e.debug_deal_damage(Some(0), 1, 2, DamageNature::Normal);
    assert_eq!(e.state_core().player(0).jijia_hp, 4, "重装充能满 4 召回 4 血机甲");

    // 机甲 4 血抵扣 5 点溢出伤害：D.Va 本体体力 2 保持不变，机甲归零后再受伤方扣本体
    e.debug_set_hp(0, 2);
    e.debug_deal_damage(Some(1), 0, 5, DamageNature::Normal);
    drain_cancel(&mut e);
    assert_eq!(e.state_core().player(0).jijia_hp, 0, "溢出伤害消耗机甲归零");
    assert_eq!(e.debug_hp(0), 2, "jijia 拦截整次伤害，溢出不穿扣本体体力");
    e.debug_deal_damage(Some(1), 0, 1, DamageNature::Normal);
    drain_cancel(&mut e);
    assert_eq!(e.debug_hp(0), 1, "机甲已归零后再次受伤扣减本体体力");
    assert_card_conservation(&e, "dva end");
}

/// 死神 (`character/ow/skill.js:2076-2169` & `2650-2735`):
/// - `xiandan` 弃置仅限**手牌**（不含装备区）；红色令杀不可闪避，黑色令杀伤+1，取消照常结算；
/// - `shouge` 击杀其他角色从离场补给得 1 张 `ow_zhiliaobo_*`；
/// - `yihun` 弃 1 黑牌指定目标使死神进入 `yihun2`（不能用牌、不可成为目标），到目标回合开始/死亡时发出虚拟杀并解除。
#[test]
fn ow_batch_c_reaper_xiandan_shouge_yihun() {
    let (_verified, mut e) = start_formal_ow(2026_1152, 4);
    e.assign_general_named(0, "ow_sishen").unwrap();
    advance_to_play(&mut e, 0);
    clear_all_hands(&mut e);

    let armor = place_kind(&mut e, 0, CardKind::Armor);
    play(&mut e, 0, armor, vec![]);
    drain_cancel(&mut e);
    let sha0 = place_kind_suit(&mut e, 0, CardKind::Sha, Suit::Spade);
    let red_card = place_kind_suit(&mut e, 0, CardKind::Shan, Suit::Heart);
    let _shan1 = place_kind_suit(&mut e, 1, CardKind::Shan, Suit::Diamond);
    e.debug_set_hp(1, 4);
    play(&mut e, 0, sha0, vec![1]);
    let c = e.open_choice().unwrap().clone();
    assert_eq!(c.spec.prompt, "xiandan_discard");
    assert!(
        !c.spec.options.contains(&ChoiceOption::Card(armor)),
        "OW 霰弹严格只许弃手牌（不得含装备）"
    );
    answer(&mut e, vec![ChoiceOption::Card(red_card)]);
    drain_cancel(&mut e);
    assert_eq!(e.debug_hp(1), 3, "霰弹弃红牌直命中 1 号");

    let blk = place_any_suit(&mut e, 0, Suit::Club);
    e.debug_set_hp(0, 3);
    advance_to_turn_end_prompt(&mut e, 0, "ow_yihun_card");
    answer(&mut e, vec![ChoiceOption::Card(blk)]);
    assert_open_prompt(&e, "ow_yihun_target", 0);
    answer(&mut e, vec![ChoiceOption::Player(2)]);
    assert!(e.debug_has_skill(0, "yihun2"));
    drain_cancel(&mut e);
    // 下一轮 0 号回合时死神仍未解 yihun2（除非 2 号回合先触发），先在 1 号回合测不可成为目标，再在 3 号或 0 号验证！
    // 等一下：若现在刚过 0 号 TurnEndWindow，下一位就是 1 号，之后是 2 号；先在 1 号回合测死神不可成为杀目标！
    advance_to_play(&mut e, 1);
    e.debug_set_turn(0, Phase::Play, 1);
    let tao_play0 = place_kind_suit(&mut e, 0, CardKind::Tao, Suit::Heart);
    assert!(
        matches!(
            play(&mut e, 0, tao_play0, vec![]),
            CommandOutcome::Rejected(RejectReason::CardBlocked)
        ),
        "移魂2 状态中死神不能主动使用手牌"
    );
    e.debug_set_turn(1, Phase::Play, 1);
    let sha1 = place_kind_suit(&mut e, 1, CardKind::Sha, Suit::Heart);
    assert!(
        matches!(
            play(&mut e, 1, sha1, vec![0]),
            CommandOutcome::Rejected(RejectReason::IllegalTargets)
        ),
        "移魂2 状态中死神不可成为其他角色杀的目标"
    );
    let wz1 = place_kind_suit(&mut e, 1, CardKind::Wuzhong, Suit::Heart);
    let wx0 = place_kind_suit(&mut e, 0, CardKind::Nullify, Suit::Spade);
    let h1_before_wz = e.debug_hand(1).len() - 1;
    play(&mut e, 1, wz1, vec![]);
    let mut offered_wuxie_to_0 = false;
    for _ in 0..8 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
        if c.spec.prompt == "nullify_window"
            && c.observer == 0
            && c.spec.options.contains(&ChoiceOption::Card(wx0))
        {
            offered_wuxie_to_0 = true;
        }
        drain_cancel(&mut e);
    }
    assert!(
        !offered_wuxie_to_0,
        "移魂2 封禁 cardEnabled：0 号持有无懈也不可对无中生有打无懈"
    );
    assert_eq!(e.debug_hand(1).len(), h1_before_wz + 2, "0 号被禁出无懈，无中生有正常摸 2");
    assert!(e.debug_hand(0).contains(&wx0), "无懈仍保留在 0 号手牌中");

    let tao_save0 = place_kind_suit(&mut e, 0, CardKind::Tao, Suit::Heart);
    e.debug_clear_hand(3);
    e.debug_set_hp(3, 1);
    let dmg_out = e.debug_deal_damage(Some(2), 3, 1, DamageNature::Normal);
    assert!(
        matches!(dmg_out, CommandOutcome::Accepted { .. }) && e.open_choice().is_none(),
        "移魂2 封禁 cardSavable：0 号虽持桃但不得进入 dying_save 救援: {dmg_out:?}"
    );
    assert!(!e.debug_alive(3), "无人可救，3 号阵亡；0 号手里仍留存 tao_save0");
    assert!(e.debug_hand(0).contains(&tao_save0));

    e.debug_set_hp(2, 1);
    e.debug_clear_hand(2);
    advance_to_play(&mut e, 0);
    assert!(!e.debug_has_skill(0, "yihun2"), "2 号回合开始移魂2 结算并移除");
    assert!(!e.debug_alive(2), "移魂2 虚拟杀击杀 2 号");
    assert!(
        e.debug_hand(0)
            .iter()
            .any(|c| e.catalog().cards[*c as usize].name.starts_with("ow_zhiliaobo_")),
        "收割击杀从离场补给区获得 ow_zhiliaobo"
    );
    assert_card_conservation(&e, "reaper end");
}

/// 士兵76 (`character/ow/skill.js:2170-2285`):
/// - `tuji` 按回合已用全牌数 `cards_used_turn` 算距离减值；
/// - `mujing` 可将手牌或**装备区**黑牌当【杀】打出，可通过 `PlayCard` 与 `ActivateSkill("mujing")` 发动，被【闪】抵消时重置本阶段杀限；
/// - `lichang` 弃红牌令自己下回合开始选令距离<=1 角色摸 2 或回 1（不可取消）。
#[test]
fn ow_batch_c_soldier76_tuji_mujing_lichang() {
    let (_verified, mut e) = start_formal_ow(2026_1153, 4);
    e.assign_general_named(0, "ow_shibing").unwrap();
    advance_to_play(&mut e, 0);
    clear_all_hands(&mut e);
    e.debug_set_hp(0, 3);
    e.debug_set_max_hp(0, 4);
    e.debug_set_hp(1, 4);

    let dist_before = e.state_core().computed_distance(e.catalog(), 0, 2);
    assert_eq!(dist_before, 2);
    let black_weapon = e
        .state_core()
        .deck
        .iter()
        .chain(e.state_core().discard.iter())
        .copied()
        .find(|c| {
            let d = &e.catalog().cards[*c as usize];
            d.kind.equip_slot().is_some()
                && matches!(d.suit, Suit::Spade | Suit::Club)
                && !d.name.starts_with("ow_")
        })
        .expect("black equip");
    e.state_core_mut().deck.retain(|x| *x != black_weapon);
    e.state_core_mut().discard.retain(|x| *x != black_weapon);
    e.state_core_mut().players[0].equips.push(black_weapon);
    let shan1 = place_kind_suit(&mut e, 1, CardKind::Shan, Suit::Heart);
    play(&mut e, 0, black_weapon, vec![1]);
    assert_eq!(e.open_choice().unwrap().observer, 1);
    answer(&mut e, vec![ChoiceOption::Card(shan1)]);
    assert_eq!(e.state_core().computed_distance(e.catalog(), 0, 2), 1, "突击计入用牌：0->2 距离降为 1");

    let black_card = place_any_suit(&mut e, 0, Suit::Club);
    activate(&mut e, 0, "mujing");
    assert_open_prompt(&e, "ow_mujing_card", 0);
    answer(&mut e, vec![ChoiceOption::Card(black_card)]);
    assert_open_prompt(&e, "ow_mujing_target", 0);
    answer(&mut e, vec![ChoiceOption::Player(2)]);
    drain_cancel(&mut e);
    assert_eq!(e.debug_hp(2), 3, "mujing 被闪退杀限后再度出杀击中 2 号");

    let red = place_kind_suit(&mut e, 0, CardKind::Tao, Suit::Heart);
    advance_to_turn_end_prompt(&mut e, 0, "ow_lichang_card");
    answer(&mut e, vec![ChoiceOption::Card(red)]);
    drain_cancel(&mut e);
    assert!(e.debug_has_skill(0, "lichang2"));

    advance_to_play(&mut e, 3);
    clear_all_hands(&mut e);
    let _ = submit(&mut e, 3, GameAction::NextPhase);
    drain_cancel(&mut e);
    loop {
        let Some(c) = e.open_choice().cloned() else {
            if e.turn().current == 0 && e.turn().phase == Phase::Play {
                break;
            }
            let cur = e.turn().current;
            let _ = submit(&mut e, cur, GameAction::NextPhase);
            continue;
        };
        if c.spec.prompt == "ow_lichang2_target" {
            answer(&mut e, vec![ChoiceOption::Player(0)]);
        } else if c.spec.prompt == "ow_lichang2_dr" {
            assert!(!c.spec.allow_cancel, "lichang2 选择不可取消");
            answer(&mut e, vec![ChoiceOption::Tag(1)]);
            break;
        } else {
            drain_cancel(&mut e);
        }
    }
    assert_eq!(e.debug_hp(0), 4, "立场2 回复 0 号 1 点体力");
    assert_card_conservation(&e, "soldier76 end");
}

/// 源氏 (`character/ow/skill.js:2286-2386` + `extra/skill.js`):
/// - `feiren` 杀无视距离、黑桃杀伤+1、梅花杀目标上限+1；`feiren3` 黑桃杀后可再打额外黑桃杀；
/// - `lianpo` 回合内击杀角色于 `TurnPhaseAfter` 获额外回合；
/// - `zhanlong` limited 回合开始 `hp==1` 弃全部牌获 `zhanlong2`（出杀不限次数）+ 3 张 `ow_sha_*`。
#[test]
fn ow_batch_c_genji_feiren_lianpo_zhanlong() {
    let (_verified, mut e) = start_formal_ow(2026_1154, 4);
    e.assign_general_named(0, "ow_yuanshi").unwrap();
    advance_to_play(&mut e, 0);
    clear_all_hands(&mut e);

    e.debug_set_hp(1, 4);
    e.debug_set_hp(2, 4);
    let club_sha = place_kind_suit(&mut e, 0, CardKind::Sha, Suit::Club);
    play(&mut e, 0, club_sha, vec![1, 2]);
    drain_cancel(&mut e);
    assert_eq!(e.debug_hp(1), 3);
    assert_eq!(e.debug_hp(2), 3, "飞刃：梅花杀可额外指定 1 名目标且无视距离");

    e.debug_set_hp(1, 2);
    e.debug_set_hp(0, 1);
    let zhuge = e.debug_place(0, "zhuge");
    play(&mut e, 0, zhuge, vec![]);
    let spade_sha = place_kind_suit(&mut e, 0, CardKind::Sha, Suit::Spade);
    play(&mut e, 0, spade_sha, vec![1]);
    drain_cancel(&mut e);
    assert!(!e.debug_alive(1), "飞刃黑桃杀 1+1=2 伤击杀 1 号");

    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let mut saw_zhanlong = false;
    for _ in 0..32 {
        if let Some(c) = e.open_choice().cloned() {
            if c.spec.prompt == "ow_lianpo_confirm" {
                answer(&mut e, vec![ChoiceOption::Confirm]);
                continue;
            }
            if c.spec.prompt == "ow_zhanlong_confirm" {
                saw_zhanlong = true;
                answer(&mut e, vec![ChoiceOption::Confirm]);
                continue;
            }
            drain_cancel(&mut e);
            continue;
        }
        if e.turn().current == 0 && e.turn().phase == Phase::Play && saw_zhanlong {
            break;
        }
        let cur = e.turn().current;
        let _ = submit(&mut e, cur, GameAction::NextPhase);
    }
    assert!(saw_zhanlong, "连破使 0 号连续进入自己的下一回合并触发斩龙");
    assert_eq!(e.turn().current, 0, "连破夺得 0 号额外回合");
    assert!(e.debug_has_skill(0, "zhanlong2"));
    assert!(
        e.debug_hand(0)
            .iter()
            .filter(|c| e.catalog().cards[**c as usize].name.starts_with("ow_sha_"))
            .count()
            >= 3,
        "斩龙补给 3 张 ow_sha"
    );

    let eqs: Vec<u32> = e.state_core_mut().players[0].equips.drain(..).collect();
    e.state_core_mut().discard.extend(eqs);
    e.debug_set_hp(2, 4);
    e.debug_set_hp(3, 4);
    let ow_shas: Vec<u32> = e
        .debug_hand(0)
        .iter()
        .copied()
        .filter(|c| e.catalog().cards[*c as usize].name.starts_with("ow_sha_"))
        .collect();
    play(&mut e, 0, ow_shas[0], vec![3]);
    drain_cancel(&mut e);
    assert!(
        matches!(
            play(&mut e, 0, ow_shas[1], vec![3]),
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "zhanlong2 无视诸葛连弩连续出杀"
    );
    drain_cancel(&mut e);
    assert_card_conservation(&e, "genji end");
    let _ = roundtrip_restore(&e, &_verified);
}
