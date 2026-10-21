use super::*;

/// 应昭批末片（规则 v232）：司圃使用面——激活后应昭出牌阶段已用牌数
/// ≤2 期间其他玩家出牌被拒；第 3 张牌后解除；应昭自己不受限；响应四
/// 面登记差异未建模。
#[test]
fn boss_yingzhao_sipu_use_ban_first_two() {
    let mut e = Engine::new_boss_stage(52001, 4, 1, 3, 6, 4, 0);
    let yingzhao_def = (e.catalog().general_count()
        - engine_native::content::standard::BOSS_GENERALS.len()
        + 8) as u16;
    e.debug_set_general_def(1, yingzhao_def);
    e.debug_set_hp(1, 25);
    let sdef = e.catalog().skill_by_name("司圃").expect("司圃入目录");
    assert!(e.state_core().skills.contains_key(&(1, sdef)));
    assert!(!e.state_core().has_skill(1, sdef), "司圃装配即未激活");

    // 激活门控：hp 置 12（≤12 半线）→ 一次伤害（12→11）激活
    e.debug_set_hp(1, 12);
    advance_to_play(&mut e, 2);
    e.debug_clear_hand(2);
    e.debug_clear_hand(1);
    let sha = e.debug_place(2, "sha");
    // 巡游可能取到装备牌自动装上（马使距离+1）——清除装备后激活
    {
        let core = e.state_core_mut();
        let eqs: Vec<u32> = core.player_mut(1).equips.drain(..).collect();
        core.discard.extend(eqs);
    }
    let _out = play(&mut e, sha, vec![1]);
    drain_choices(&mut e);
    assert!(e.state_core().has_skill(1, sdef), "体力≤一半 → 司圃激活");

    // ① 应昭回合：跨到应昭出牌阶段（已用 0 牌<3）→ 杀的目标（2 号）
    //    持闪但司圃禁响应（仅 Pass）→ 2 号受 1 伤
    advance_to_play(&mut e, 1);
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);
    {
        let core = e.state_core_mut();
        let eqs: Vec<u32> = core.player_mut(1).equips.drain(..).collect();
        core.discard.extend(eqs);
    }
    let sha2 = e.debug_place(1, "sha");
    let shan_t2 = e.debug_place(2, "shan");
    e.debug_set_hp(2, 4);
    play(&mut e, sha2, vec![2]);
    let c = e.open_choice().expect("出闪询问应挂起");
    assert_eq!(c.observer, 2);
    assert!(
        !c.spec
            .options
            .iter()
            .any(|o| matches!(o, ChoiceOption::Card(x) if *x == shan_t2)),
        "司圃：已用<3 期间 2 号闪选项被禁（仅 Pass）"
    );
    answer(&mut e, ChoiceOption::Pass);
    drain_choices(&mut e);
    assert_eq!(e.debug_hp(2), 3, "无法闪 → 受 1 伤（4-1）");

    // ② 已用 1 牌（<3）→ 仍禁：应昭用桃（非杀避免杀限，自气回 1）
    let tao1 = e.debug_place(1, "tao");
    e.debug_set_hp(1, 10);
    play(&mut e, tao1, vec![]);
    drain_choices(&mut e);
    assert_eq!(e.state_core().turn.cards_used_play_phase, 2, "已用 2 牌");

    // ③ 已用 2 牌（<3）→ 仍禁：再用一桃（used=3 → 司圃解除）
    let tao2 = e.debug_place(1, "tao");
    play(&mut e, tao2, vec![]);
    drain_choices(&mut e);
    assert_eq!(e.state_core().turn.cards_used_play_phase, 3, "已用 3 牌");

    // ④ 解除验证（cardSavable 恢复）：濒死者自救用桃恢复可用——0 号
    //    hp=1，应昭（回合内）杀 0 号 → 濒死询问中 0 号自救桃入选项（已
    //    用 3 张解除）
    e.debug_clear_hand(0);
    e.debug_set_hp(0, 1);
    let tao0 = e.debug_place(0, "tao");
    // 决斗（非杀避免杀限）：0 号无杀 → 受 1 伤 → hp 1→0 濒死
    let duel = e.debug_place(1, "duel");
    play(&mut e, duel, vec![0]);
    let c = e.open_choice().expect("濒死询问应挂起");
    assert_eq!(c.observer, 0);
    assert!(
        c.spec
            .options
            .iter()
            .any(|o| matches!(o, ChoiceOption::Card(x) if *x == tao0)),
        "已用 3 张 → 司圃解除，自救桃恢复可用"
    );
    answer_sel(&mut e, vec![ChoiceOption::Card(tao0)]);
    drain_choices(&mut e);
    assert!(e.debug_alive(0), "自救成功");

    // ⑤ 应昭自己回合内濒死（used=0<3）：自救桃入选项、他人救援桃被禁
    //    （cardSavable 建模面）——先收掉当前回合再推进到应昭新回合
    next_phase(&mut e); // Play→Discard
    drain_choices(&mut e);
    next_phase(&mut e); // Discard→End
    drain_choices(&mut e);
    advance_to_play(&mut e, 1);
    assert_eq!(
        e.state_core().turn.cards_used_play_phase,
        0,
        "新回合已用清零"
    );
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);
    e.debug_set_hp(1, 1);
    let tao_yz = e.debug_place(1, "tao");
    let _tao_2 = e.debug_place(2, "tao");
    e.debug_deal_damage(Some(2), 1, 2, DamageNature::Normal);
    drain_choices(&mut e);
    let c = e.open_choice().expect("濒死询问应挂起");
    assert_eq!(c.observer, 1, "自救优先询问应昭");
    assert!(
        c.spec
            .options
            .iter()
            .any(|o| matches!(o, ChoiceOption::Card(x) if *x == tao_yz)),
        "司圃不禁自救"
    );
    // 应昭放弃自救 → 轮转他人：2 号持桃但禁救（仅 Pass）
    answer(&mut e, ChoiceOption::Pass);
    drain_choices(&mut e);
    let c = e.open_choice().expect("他人救援询问应挂起");
    assert_eq!(c.observer, 2);
    assert!(
        !c.spec
            .options
            .iter()
            .any(|o| matches!(o, ChoiceOption::Card(_))),
        "已用<3 期间他人禁救（cardSavable）"
    );
    answer(&mut e, ChoiceOption::Pass);
    drain_choices(&mut e);
    assert!(!e.debug_alive(1), "无人可救 → 应昭死亡");
    assert_zone_conservation(&e);
}

/// 应昭批续片（规则 v231）：巡游——他人回合开始获得座位序首名有手牌
/// 者的一张手牌；装备牌自动使用（同栏替换）。
#[test]
fn boss_yingzhao_xunyou_steal_and_equip() {
    let mut e = Engine::new_boss_stage(50001, 4, 1, 3, 6, 4, 0);
    let yingzhao_def = (e.catalog().general_count()
        - engine_native::content::standard::BOSS_GENERALS.len()
        + 8) as u16;
    e.debug_set_general_def(1, yingzhao_def);
    e.debug_set_hp(1, 25);
    let xdef = e.catalog().skill_by_name("巡游").expect("巡游入目录");
    assert!(e.state_core().skills.contains_key(&(1, xdef)));

    // ① 2 号回合开始（巡游触发：座位序从应昭起——首个有手牌者=2 号）
    advance_to_play(&mut e, 2);
    e.debug_clear_hand(2);
    let tao = e.debug_place(2, "tao");
    let hand1_before = e.debug_hand(1).len();
    drain_choices(&mut e);
    assert!(
        !e.debug_hand(1).contains(&tao) && e.debug_hand(2).contains(&tao),
        "巡游：2 号回合开始不触发（自身除外语义）"
    );
    let _ = hand1_before;

    // ② 0 号回合开始：座位序 [1,2,3] 首个有手牌者——1 号（应昭自身除外）
    //    0 号 clear 后 2 号有 tao？0 号回合开始时 seat_order_from(0)=[1,2,3]，
    //    1 号除外（=巡游持有者），2 号有牌 → 取 2 号手牌首张
    e.debug_clear_hand(1);
    e.debug_set_hp(2, 4);
    let victim_card = e.debug_place(2, "shan");
    advance_to_play(&mut e, 0);
    drain_choices(&mut e);
    assert!(
        !e.debug_hand(2).contains(&victim_card),
        "巡游：0 号回合开始取走 2 号手牌首张"
    );
    let stolen_in_hands = e.debug_hand(1).contains(&victim_card)
        || e.state_core().player(1).equips.contains(&victim_card);
    assert!(stolen_in_hands, "巡游：牌入应昭手牌（闪非装备）");
}

/// 应昭批首片（规则 v228）：封冻——应昭回合开始所有其他角色获封印
/// （本回合无法使用手牌）；回合结束解除；装备区牌不受限。
#[test]
fn boss_yingzhao_fengdong_hand_ban() {
    let mut e = Engine::new_boss_stage(49001, 4, 1, 3, 6, 4, 0);
    let yingzhao_def = (e.catalog().general_count()
        - engine_native::content::standard::BOSS_GENERALS.len()
        + 8) as u16; // BOSS_GENERALS[8] 应昭（表序：…梼杌/毕方/书精/相柳/朱厌/应昭）
    e.debug_set_general_def(1, yingzhao_def);
    e.debug_set_hp(1, 25);
    let fdef = e.catalog().skill_by_name("封冻").expect("封冻入目录");
    assert!(e.state_core().skills.contains_key(&(1, fdef)));

    // 应昭回合开始：其他角色全获封印
    advance_to_play(&mut e, 1);
    assert!(e.state_core().player(0).fengyin_active, "0 号获封印");
    assert!(e.state_core().player(2).fengyin_active, "2 号获封印");
    assert!(e.state_core().player(3).fengyin_active, "3 号获封印");
    assert!(!e.state_core().player(1).fengyin_active, "应昭自身不封印");

    // ① 2 号（封印中）在非自己回合无法出牌——快进到 2 号自己的回合：
    //    封印在应昭回合结束已解除 → 2 号可正常出牌
    advance_to_play(&mut e, 2);
    drain_choices(&mut e);
    assert!(!e.state_core().player(2).fengyin_active, "回合结束解除");
    e.debug_clear_hand(2);
    let tao = e.debug_place(2, "tao");
    e.debug_set_hp(2, 3);
    let out = play(&mut e, tao, vec![]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "封印解除后 2 号可用手牌：{out:?}"
    );

    // ② 存档往返（v53）：fengyin_active 编码
    let snap = e.save();
    let r = Engine::restore(&snap).expect("v53 往返");
    assert_eq!(
        r.state_core().player(2).fengyin_active,
        e.state_core().player(2).fengyin_active,
        "封印状态往返一致"
    );
    assert_zone_conservation(&r);
}
