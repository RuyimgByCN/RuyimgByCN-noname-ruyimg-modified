use super::*;

/// 星宿阶段2系造异片（规则 v246）：造异（颛顼）——①共工存活时基本牌
/// 不可指定敌方（targetEnabled 共工半面）②共工死亡且无存活同名将 → 颛
/// 顼摸 4+获「造异异变」③门控解除④异变回合开始：敌方最低体力全体失去
/// 全部体力（濒死→死亡）。颛顼条目（群目录近似/hp6/[神裔,神恩,造异]；
/// 造异异变=运行时授予；玄冥半面随自润批登记）。
#[test]
fn boss_zhuanxu_zaoyi_grant_and_exec() {
    // 建局：1=颛顼；fellow 4=共工
    let mut e = Engine::new_boss_stage(66001, 4, 1, 3, 6, 4, 0);
    let zx_def = (e.catalog().general_count()
        - engine_native::content::standard::BOSS_GENERALS.len()
        + 23) as u16; // BOSS_GENERALS[23] 颛顼
    let gg_def = (e.catalog().general_count()
        - engine_native::content::standard::BOSS_GENERALS.len()
        + 21) as u16; // BOSS_GENERALS[21] 共工
    e.debug_set_general_def(1, zx_def);
    e.debug_set_hp(1, 6);
    let fellow = e.debug_add_fellow(5).expect("fellow 入队");
    e.debug_set_general_def(fellow, gg_def);
    assert_eq!(
        e.catalog().general_meta(zx_def).map(|g| g.0),
        Some("颛顼"),
        "颛顼入目录"
    );
    let zdef = e.catalog().skill_by_name("造异").expect("造异入目录");
    let zhp = e
        .catalog()
        .skill_by_name("造异异变")
        .expect("造异异变入目录");
    assert!(e.state_core().skills.contains_key(&(1, zdef)));

    // ① 共工存活：颛顼基本牌不可指定敌方（sha→2 拒；tao 自目标不受限）
    advance_to_play(&mut e, 1);
    e.debug_clear_hand(1);
    let sha = e.debug_place(1, "sha");
    let out = play(&mut e, sha, vec![2]);
    assert!(
        !matches!(out, CommandOutcome::Accepted { .. }),
        "造异：共工在场基本牌不可指定敌方"
    );
    let tao = e.debug_place(1, "tao");
    e.debug_set_hp(1, 4);
    play(&mut e, tao, vec![]);
    drain_choices(&mut e);
    assert_eq!(e.debug_hp(1), 5, "自目标不受门控");

    // ② 击杀共工 → 颛顼摸 4+获造异异变
    e.debug_clear_hand(1);
    e.debug_clear_hand(0);
    e.debug_clear_hand(2);
    e.debug_clear_hand(3);
    e.debug_set_hp(fellow, 1);
    e.debug_deal_damage(Some(0), fellow, 1, DamageNature::Normal);
    for _ in 0..4 {
        if e.open_choice().is_some() {
            answer(&mut e, ChoiceOption::Pass);
            drain_choices(&mut e);
        } else {
            break;
        }
    }
    assert!(!e.debug_alive(fellow), "共工死亡");
    assert_eq!(e.debug_hand(1).len(), 4, "造异：颛顼摸 4");
    assert!(
        e.state_core().skills.contains_key(&(1, zhp)),
        "造异异变授予"
    );

    // ③ 共工死后：基本牌门控解除（sha 可指定敌方）
    let sha2 = e.debug_place(1, "sha");
    let out = play(&mut e, sha2, vec![2]);
    assert!(
        matches!(out, CommandOutcome::Accepted { .. }),
        "共工死亡后门控解除 {out:?}"
    );
    drain_choices(&mut e);

    // ④ 异变回合开始：敌方最低体力全体失去全部体力（压 0/2 至 2、
    //    3 保持 5——仅最低者中招；挑战者不全灭不终局）。三家挑战者钉
    //    成共工（绝红对空装备无干扰），消除随机武将不确定性
    for t in [0u8, 2, 3] {
        e.debug_set_general_def(t, gg_def);
    }
    e.debug_set_hp(0, 2);
    e.debug_set_hp(2, 2);
    e.debug_set_hp(3, 5);
    e.debug_clear_hand(0);
    e.debug_clear_hand(2);
    e.debug_clear_hand(3);
    next_phase(&mut e); // Play→Discard
    drain_choices(&mut e);
    next_phase(&mut e); // Discard→End
    drain_choices(&mut e);
    // 推进到颛顼下一判定锚（异变触发）；濒死救援全 Pass（0/2 手牌已清，
    // 轮中摸牌可能含桃——显式 Pass 放弃救援）
    for _ in 0..48 {
        match e.open_choice() {
            Some(_c) => {
                answer(&mut e, ChoiceOption::Pass);
                drain_choices(&mut e);
            }
            None => {
                if e.turn().current == 1 && e.turn().phase == Phase::Judge {
                    break;
                }
                match next_phase(&mut e) {
                    CommandOutcome::Accepted { .. } => {}
                    CommandOutcome::Pending { choice } => {
                        // 救援询问一律 Pass（放弃）；其余（弃牌等）按牌应答
                        if choice.spec.prompt == "dying_save" {
                            answer(&mut e, ChoiceOption::Pass);
                            drain_choices(&mut e);
                        } else {
                            answer_cards(&mut e, &choice);
                        }
                    }
                    other => panic!("{other:?}"),
                }
            }
        }
    }
    assert_eq!(e.debug_hp(0), 0, "异变：0 号失去全部体力");
    assert_eq!(e.debug_hp(2), 0, "异变：2 号失去全部体力");
    assert_eq!(e.debug_hp(3), 5, "非最低者不触发");
    assert!(e.outcome().is_none(), "挑战者未全灭不终局");
    assert_zone_conservation(&e);

    // ⑤ 存档往返
    let snap = e.save();
    let r = Engine::restore(&snap).expect("往返");
    assert!(
        r.state_core().skills.contains_key(&(1, zhp)),
        "造异异变实例往返"
    );
    assert_zone_conservation(&r);
}
