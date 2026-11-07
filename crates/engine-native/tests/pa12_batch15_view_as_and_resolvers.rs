//! PA12 迁移批⑮（2026-11-07，零版本号变更）综合验证：
//! 1. `codec_ext3.rs` / `accept_batch15.rs` 25 个高频效果原语（tags 154..=178）编解码往返、
//!    未知单字节 tag 179 拒绝与非法参数拒绝。
//! 2. `vm_view_as.rs` + 内核/引擎视为技桥接（`view_as_index`）：出牌阶段转化、每回合限次、
//!    装备区牌转化为【杀】时的卸装/失去装备技/移入弃牌堆，以及四大响应窗口
//!    （求【闪】、求【杀】、濒死求【桃】/【酒】、【无懈可击】）双站点一致性与 `NGSA 107` 存读档往返。
//! 3. 25 个批⑮ `Resolve*` 效果原语（tags 154..=178）编译桥接，以及在原生 `Engine` 上的
//!    交互选择、`NGSA 107` 存读档往返与状态落地验证。

use content_compiler::compile_source_named;
use engine_native::choice::ChoiceKind;
use engine_native::content::extension::{ExtensionDef, ExtensionPack, InitHook};
use engine_native::event::Phase;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand};
use engine_native_kernel::catalog::{CardKind, Suit};
use engine_native_kernel::rvm_ops::{accept_effect, dec_effects, enc_effects};
use rule_api::{CardId, EffectRequest, PlayerId, SkillDefId, SkillId};

const POT_WEIYAN: &str = include_str!("../../../content/original/ported/bingshi/pot_weiyan.skill");
const POT_CAOSHUANG: &str =
    include_str!("../../../content/original/ported/bingshi/pot_caoshuang.skill");
const POT_LVYI: &str = include_str!("../../../content/original/ported/bingshi/pot_lvyi.skill");
const POT_YUJI: &str = include_str!("../../../content/original/ported/bingshi/pot_yuji.skill");
const POT_ZHOUYU: &str = include_str!("../../../content/original/ported/bingshi/pot_zhouyu.skill");
const SP_ZHONGHUI: &str =
    include_str!("../../../content/original/ported/bingshi/sp_zhonghui.skill");
const DDD_LIUHONG: &str = include_str!("../../../content/original/ported/ddd/liuhong.skill");
const TW_GUOHUAI: &str = include_str!("../../../content/original/ported/tw/twguohuai.skill");
const DDD_XUELINGYUN: &str = include_str!("../../../content/original/ported/ddd/xuelingyun.skill");
const POT_TAISHICI: &str =
    include_str!("../../../content/original/ported/bingshi/pot_taishici.skill");
const DDD_XUJING: &str = include_str!("../../../content/original/ported/ddd/xujing.skill");
const DDD_LIUBA: &str = include_str!("../../../content/original/ported/ddd/liuba.skill");
const DDD_LIUYE: &str = include_str!("../../../content/original/ported/ddd/liuye.skill");
const DDD_ZHANGKAI: &str = include_str!("../../../content/original/ported/ddd/zhangkai.skill");
const EXTRA_JILIN: &str = include_str!("../../../content/original/ported/extra/jilin.skill");
const TW_FUMAN: &str = include_str!("../../../content/original/ported/tw/twfuman.skill");
const TW_KUJIAN: &str = include_str!("../../../content/original/ported/tw/twkujian.skill");
const TW_XIAWEI: &str = include_str!("../../../content/original/ported/tw/twxiawei.skill");
const XIANDING_CAOYI: &str = include_str!("../../../content/original/ported/xianding/caoyi.skill");
const DC_SB_LUSU: &str = include_str!("../../../content/original/ported/xianding/dc_sb_lusu.skill");
const DC_SB_ZHOUYU: &str =
    include_str!("../../../content/original/ported/xianding/dc_sb_zhouyu.skill");
const MB_SUNJUN: &str = include_str!("../../../content/original/ported/bingshi/mb_sunjun.skill");
const DDD_CAOSHUANG: &str = include_str!("../../../content/original/ported/ddd/caoshuang.skill");
const KANPO: &str = include_str!("../../../content/original/ported/shenhua/kanpo.skill");
const JIUCHI: &str = include_str!("../../../content/original/ported/shenhua/jiuchi.skill");
const JXWUSHENG: &str = include_str!("../../../content/original/ported/offline/jxwusheng.skill");

static DEDUP: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1_500_000);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

fn submit_outcome(e: &mut Engine, actor: u8, action: GameAction) -> CommandOutcome {
    e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action,
    })
}

fn submit(e: &mut Engine, actor: u8, action: GameAction) {
    let out = submit_outcome(e, actor, action.clone());
    assert!(
        !matches!(out, CommandOutcome::Rejected(_)),
        "submit({actor}, {action:?}) 失败: {out:?}"
    );
}

fn answer(e: &mut Engine, selection: Vec<ChoiceOption>) {
    let ask = e.open_choice().cloned().expect("expected open_choice");
    submit(
        e,
        ask.observer,
        GameAction::Answer {
            choice_id: ask.id,
            selection,
        },
    );
}

fn drain_choices(e: &mut Engine) {
    for _ in 0..64 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
        let sel = if c.spec.options.contains(&ChoiceOption::Confirm) {
            vec![ChoiceOption::Confirm]
        } else if c.spec.options.contains(&ChoiceOption::Pass) {
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

fn advance_to_phase(e: &mut Engine, seat: u8, want: Phase) {
    for _ in 0..80 {
        drain_choices(e);
        if e.state_core().turn.current == seat && e.state_core().turn.phase == want {
            return;
        }
        let cur = e.state_core().turn.current;
        submit(e, cur, GameAction::NextPhase);
    }
    panic!(
        "未能推进到 ({seat}, {want:?})，当前=({}, {:?})",
        e.state_core().turn.current,
        e.state_core().turn.phase
    );
}

fn assert_archive_roundtrip(e: &Engine, packs: &[&ExtensionPack]) -> Engine {
    let snap = e.save();
    let bytes = engine_native::archive::encode_archive(&snap);
    let decoded = engine_native::archive::decode_archive(&bytes).expect("decode_archive");
    let restored =
        Engine::restore_with_extensions(&decoded, packs).expect("restore_with_extensions");
    assert_eq!(
        restored.state_hash(),
        e.state_hash(),
        "NGSA 107 恢复前后哈希一致"
    );
    assert_eq!(restored.open_choice(), e.open_choice(), "挂起询问一致");
    restored
}

fn pack_strict(sources: &[(&str, &str)], pack_name: &'static str) -> ExtensionPack {
    let mut vm_skills = Vec::new();
    for (id, source) in sources {
        let pkg = compile_source_named(*id, source).expect("编译 .skill");
        let out = content_native_loader::bridge_vm_skill_decls(&pkg).expect("正式桥接");
        vm_skills.extend(out.decls);
    }
    ExtensionPack {
        def: ExtensionDef {
            name: pack_name,
            version: 1,
            description: "PA12 批⑮ 严格桥接测试包",
        },
        generals: Vec::new(),
        cards: Vec::new(),
        host_skills: Vec::new(),
        vm_skills,
        init_hook: InitHook::None,
        enabled: true,
    }
}

#[test]
fn pa12_batch15_codec_roundtrip_154_to_178_and_validation() {
    let effects = vec![
        (
            154u8,
            EffectRequest::ResolvePotweiyan {
                player: PlayerId::new(1),
                target: PlayerId::new(2),
                card: CardId::new(3),
                action: 2,
                skill: SkillId::new(4200000559),
            },
        ),
        (
            155,
            EffectRequest::ResolvePotcaoshuang {
                player: PlayerId::new(1),
                target: PlayerId::new(2),
                card: CardId::new(4),
                action: 3,
                skill: SkillId::new(4200000573),
            },
        ),
        (
            156,
            EffectRequest::ResolvePotlvyi {
                player: PlayerId::new(1),
                target: PlayerId::new(2),
                card: CardId::new(5),
                action: 1,
                event_cards: vec![CardId::new(5)],
                targets: vec![PlayerId::new(2)],
                skill: SkillId::new(4200000570),
            },
        ),
        (
            157,
            EffectRequest::ResolvePotyuji {
                player: PlayerId::new(1),
                target: PlayerId::new(2),
                card: CardId::new(6),
                action: 0,
                cards: vec![CardId::new(6)],
                targets: vec![PlayerId::new(2)],
                skill: SkillId::new(4200000547),
            },
        ),
        (
            158,
            EffectRequest::ResolvePotzhouyu {
                player: PlayerId::new(1),
                target: PlayerId::new(2),
                card: CardId::new(7),
                action: 2,
                card_flags: 1,
                targets: vec![PlayerId::new(2)],
                skill: SkillId::new(4200000552),
            },
        ),
        (
            159,
            EffectRequest::ResolveMbzhonghui {
                player: PlayerId::new(1),
                target: PlayerId::new(2),
                action: 1,
                amount: 2,
                mbsizi_def: SkillDefId::new(2212391997),
                mbkechang_def: SkillDefId::new(2942773382),
                skill: SkillId::new(4200000540),
            },
        ),
        (
            160,
            EffectRequest::ResolveDddLiuhong {
                player: PlayerId::new(1),
                actor: PlayerId::new(2),
                action: 0,
                skill: SkillId::new(3001),
            },
        ),
        (
            161,
            EffectRequest::ResolveTwguohuai {
                player: PlayerId::new(1),
                target: PlayerId::new(2),
                source: PlayerId::new(1),
                card: CardId::new(10),
                mark_def: 2473230123,
                helper_skill_def: 3964351777,
                action: 4,
                stage: 1,
                skill: SkillId::new(3002),
            },
        ),
        (
            162,
            EffectRequest::ResolveDddXuelingyun {
                player: PlayerId::new(1),
                actor: PlayerId::new(2),
                card: CardId::new(11),
                cards: vec![CardId::new(11)],
                action: 0,
                skill: SkillId::new(3003),
            },
        ),
        (
            163,
            EffectRequest::ResolvePottaishici {
                player: PlayerId::new(1),
                target: PlayerId::new(2),
                action: 1,
                cards: vec![CardId::new(12)],
                hanzhan_def: SkillDefId::new(2573360121),
                zhanlie_def: SkillDefId::new(3829642123),
                skill: SkillId::new(3004),
            },
        ),
        (
            164,
            EffectRequest::ResolveDddXujing {
                player: PlayerId::new(1),
                source: PlayerId::new(2),
                card: CardId::new(13),
                action: 2,
                targets: vec![PlayerId::new(2)],
                skill: SkillId::new(3005),
            },
        ),
        (
            165,
            EffectRequest::ResolveDddLiuba {
                player: PlayerId::new(1),
                actor: PlayerId::new(2),
                action: 0,
                cards: vec![CardId::new(14)],
                targets: vec![PlayerId::new(2)],
                skill: SkillId::new(3006),
            },
        ),
        (
            166,
            EffectRequest::ResolveDddLiuye {
                player: PlayerId::new(1),
                actor: PlayerId::new(2),
                card: CardId::new(15),
                action: 1,
                skill: SkillId::new(3007),
            },
        ),
        (
            167,
            EffectRequest::ResolveDddZhangkai {
                player: PlayerId::new(1),
                actor: PlayerId::new(2),
                card: CardId::new(16),
                cards: vec![CardId::new(16)],
                amount: 1,
                action: 0,
                skill: SkillId::new(3008),
            },
        ),
        (
            168,
            EffectRequest::ResolveHiddenSkillPile {
                player: PlayerId::new(1),
                pile_skill_def: SkillDefId::new(3993974385),
                action: 2,
                card: CardId::new(17),
                stage: 1,
                hand_cards: vec![CardId::new(17)],
                pile_cards: vec![CardId::new(18)],
                event_cards: vec![CardId::new(19)],
                skill: SkillId::new(3009),
            },
        ),
        (
            169,
            EffectRequest::ResolveTwfuman {
                player: PlayerId::new(1),
                target: PlayerId::new(2),
                cards: vec![CardId::new(18)],
                action: 0,
                stage: 1,
                skill: SkillId::new(3010),
            },
        ),
        (
            170,
            EffectRequest::ResolveTwkujian {
                player: PlayerId::new(1),
                target: PlayerId::new(2),
                cards: vec![CardId::new(19), CardId::new(20)],
                action: 0,
                skill: SkillId::new(3011),
            },
        ),
        (
            171,
            EffectRequest::ResolveTwxiawei {
                player: PlayerId::new(1),
                wangxing_skill_def: 3476660685,
                action: 1,
                stage: 0,
                skill: SkillId::new(3012),
            },
        ),
        (
            172,
            EffectRequest::ResolveCaoyi {
                player: PlayerId::new(1),
                card: CardId::new(22),
                helper_skill_def: SkillDefId::new(3041789307),
                action: 0,
                event_targets: vec![PlayerId::new(2)],
                skill: SkillId::new(3013),
            },
        ),
        (
            173,
            EffectRequest::ResolveDcSbLusu {
                player: PlayerId::new(1),
                source: PlayerId::new(2),
                card: CardId::new(23),
                action: 2,
                cards: vec![CardId::new(23)],
                amount: 1,
                skill: SkillId::new(3014),
            },
        ),
        (
            174,
            EffectRequest::ResolveDcSbZhouyu {
                player: PlayerId::new(1),
                source: PlayerId::new(2),
                card: CardId::new(24),
                action: 1,
                event_targets: vec![PlayerId::new(2)],
                damaged_targets: vec![PlayerId::new(2)],
                skill: SkillId::new(3015),
            },
        ),
        (
            175,
            EffectRequest::ResolveMbxiongtu {
                player: PlayerId::new(1),
                target: PlayerId::new(2),
                stage: 0,
                card: CardId::new(25),
                cards: vec![CardId::new(25)],
                skill: SkillId::new(3016),
            },
        ),
        (
            176,
            EffectRequest::ResolvePothongyi {
                player: PlayerId::new(1),
                stage: 1,
                skill: SkillId::new(3017),
            },
        ),
        (
            177,
            EffectRequest::ResolveSpyingjia {
                player: PlayerId::new(1),
                action: 0,
                helper_def: SkillDefId::new(3612952432),
                card: CardId::new(27),
                card_kind: 1,
                target: PlayerId::new(2),
                skill: SkillId::new(3018),
            },
        ),
        (
            178,
            EffectRequest::ResolveDddCaoshuang {
                player: PlayerId::new(1),
                actor: PlayerId::new(2),
                card: CardId::new(28),
                action: 2,
                skill: SkillId::new(3019),
            },
        ),
    ];

    assert_eq!(effects.len(), 25);
    for (expected_tag, ef) in &effects {
        assert!(
            accept_effect(ef, 8).is_ok(),
            "tag {expected_tag} 应通过 accept_effect: {ef:?}"
        );
        let mut buf = Vec::new();
        enc_effects(&mut buf, std::slice::from_ref(ef));
        assert_eq!(&buf[0..4], &1u32.to_le_bytes(), "单条效果数量前缀为 1");
        assert_eq!(buf[4], *expected_tag, "tag 字节必须为 {expected_tag}");
        let decoded = dec_effects(&buf).expect("dec_effects 成功");
        assert_eq!(&decoded[0], ef);
    }

    // 批量往返
    let all_only: Vec<EffectRequest> = effects.into_iter().map(|(_, e)| e).collect();
    let mut batch_buf = Vec::new();
    enc_effects(&mut batch_buf, &all_only);
    let decoded_all = dec_effects(&batch_buf).expect("批量解码成功");
    assert_eq!(decoded_all, all_only);

    // 未知单字节 tag 179 必须被拒绝
    assert!(dec_effects(&[1, 0, 0, 0, 179, 0, 0, 0, 0]).is_none());

    // 非法参数必须被 accept_effect 拒绝
    assert!(accept_effect(
        &EffectRequest::ResolvePotweiyan {
            player: PlayerId::NONE,
            target: PlayerId::NONE,
            card: CardId::NONE,
            action: 0,
            skill: SkillId::new(1),
        },
        8
    )
    .is_err());
    assert!(accept_effect(
        &EffectRequest::ResolveHiddenSkillPile {
            player: PlayerId::new(1),
            pile_skill_def: SkillDefId::NONE,
            action: 0,
            card: CardId::NONE,
            stage: 0,
            hand_cards: Vec::new(),
            pile_cards: Vec::new(),
            event_cards: Vec::new(),
            skill: SkillId::new(1),
        },
        8
    )
    .is_err());
    assert!(accept_effect(
        &EffectRequest::ResolveTwkujian {
            player: PlayerId::new(1),
            target: PlayerId::NONE,
            cards: Vec::new(),
            action: 99,
            skill: SkillId::new(1),
        },
        8
    )
    .is_err());
}

#[test]
fn pa12_batch15_view_as_bridge_play_phase_equip_detach_and_response_windows() {
    let custom_view_as = r#"
skill va_equip_sha(91500101) {
    view_as sha play_phase any from hand_equip;
}
skill va_resp_shan(91500102) {
    view_as shan response red from hand;
}
skill va_resp_sha(91500103) {
    view_as sha response red from hand;
}
skill va_resp_wuxie(91500104) {
    view_as wuxie response black from hand;
}
skill va_jiedao(91500105) {
    view_as jiedao play_phase any from hand;
}
"#;
    let pack = pack_strict(
        &[
            ("kanpo", KANPO),
            ("jiuchi", JIUCHI),
            ("jxwusheng", JXWUSHENG),
            ("custom_va", custom_view_as),
        ],
        "pa12-b15-view-as",
    );
    let packs = [&pack];

    // 1. jxwusheng: 出牌阶段将手牌视为【杀】（每回合限 1 次）
    {
        let mut e = Engine::new_with_extensions(915_101, 2, &packs).expect("建局");
        advance_to_phase(&mut e, 0, Phase::Play);
        e.debug_grant(0, "jxwusheng");
        let def_jx = e.catalog().skill_by_name("jxwusheng").unwrap();
        assert!(
            e.catalog().is_view_as_skill(def_jx),
            "jxwusheng 必须被识别为 view_as 技能"
        );
        e.debug_clear_hand(0);
        let shan1 = e.debug_place(0, "shan");
        let shan2 = e.debug_place(0, "shan");
        assert!(e.is_legal_action(
            0,
            &GameAction::PlayCard {
                card: shan1,
                targets: vec![1],
            }
        ));
        submit(
            &mut e,
            0,
            GameAction::PlayCard {
                card: shan1,
                targets: vec![1],
            },
        );
        drain_choices(&mut e);
        // 重置 sha_used_play_phase 以单独检验 jxwusheng uses_per_turn 1 计数门禁
        e.state_core_mut().turn.sha_used_play_phase = false;
        assert!(
            !e.is_legal_action(
                0,
                &GameAction::PlayCard {
                    card: shan2,
                    targets: vec![1],
                }
            ),
            "jxwusheng uses_per_turn=1 用尽后同回合不可再将第二张闪视为杀"
        );
    }

    // 2. va_equip_sha: 装备区武器（丈八蛇矛）转化为【杀】打出时，必须从装备区卸除并撤销装备技
    {
        let mut e = Engine::new_with_extensions(915_102, 2, &packs).expect("建局");
        advance_to_phase(&mut e, 0, Phase::Play);
        e.debug_clear_hand(0);
        let zhangba = e.debug_place(0, "zhangba");
        submit(
            &mut e,
            0,
            GameAction::PlayCard {
                card: zhangba,
                targets: Vec::new(),
            },
        );
        assert!(e.state_core().player(0).equips.contains(&zhangba));
        let zb_skill_name = e.catalog().cards[zhangba as usize].equip_skills[0];
        let zb_skill = e.catalog().skill_by_name(zb_skill_name).unwrap();
        assert!(e.state_core().skills.contains_key(&(0, zb_skill)));
        e.debug_grant(0, "va_equip_sha");

        // 将已装备的丈八蛇矛经 va_equip_sha 视为【杀】对 1 号使用
        assert!(e.is_legal_action(
            0,
            &GameAction::PlayCard {
                card: zhangba,
                targets: vec![1],
            }
        ));
        submit(
            &mut e,
            0,
            GameAction::PlayCard {
                card: zhangba,
                targets: vec![1],
            },
        );
        drain_choices(&mut e);
        assert!(
            !e.state_core().player(0).equips.contains(&zhangba),
            "装备牌经视为技打出后必须从装备区移除"
        );
        assert!(
            !e.state_core().skills.contains_key(&(0, zb_skill)),
            "装备牌经视为技打出后必须卸载其装备技能"
        );
    }

    // 2b. va_jiedao: 将非借刀手牌视为【借刀杀人】打出时，必须传递 use_as=Some(Jiedao) 并启动借刀流程
    {
        let mut e = Engine::new_with_extensions(915_104, 3, &packs).expect("建局");
        advance_to_phase(&mut e, 1, Phase::Play);
        let w_pos = e
            .state_core()
            .deck
            .iter()
            .position(|&c| e.catalog().cards[c as usize].kind == CardKind::Weapon)
            .unwrap();
        let w = e.state_core_mut().deck.remove(w_pos);
        e.state_core_mut().player_mut(1).hand.push(w);
        submit(
            &mut e,
            1,
            GameAction::PlayCard {
                card: w,
                targets: Vec::new(),
            },
        );
        advance_to_phase(&mut e, 0, Phase::Play);
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        e.debug_grant(0, "va_jiedao");
        let shan = e.debug_place(0, "shan");
        submit(
            &mut e,
            0,
            GameAction::PlayCard {
                card: shan,
                targets: vec![1, 2],
            },
        );
        drain_choices(&mut e);
        assert_eq!(
            e.state_core().player(2).hp,
            3,
            "视为【借刀杀人】必须传递 use_as=Some(Jiedao) 并启动 JiedaoMachine 对 2 号造成 1 点伤害"
        );
    }

    // 3. 四大响应窗口（Shan / Sha / Dying Jiu / Wuxie）双站点与 NGSA 107 存读档验证
    {
        let mut e = Engine::new_with_extensions(915_103, 3, &packs).expect("建局");
        advance_to_phase(&mut e, 0, Phase::Play);
        e.debug_grant(1, "va_resp_shan");
        e.debug_grant(1, "va_resp_sha");
        e.debug_grant(1, "va_resp_wuxie");
        e.debug_grant(1, "jiuchi");

        // (a) 求【闪】窗口：1 号仅持红桃【桃】，可响应 0 号的【杀】
        e.debug_clear_hand(1);
        let red_tao = e.debug_place(1, "tao");
        assert!(matches!(
            e.catalog().cards[red_tao as usize].suit,
            Suit::Heart | Suit::Diamond
        ));
        let sha = e.debug_place(0, "sha");
        submit(
            &mut e,
            0,
            GameAction::PlayCard {
                card: sha,
                targets: vec![1],
            },
        );
        let ask_shan = e.open_choice().cloned().expect("求闪窗口");
        assert!(ask_shan.spec.options.contains(&ChoiceOption::Card(red_tao)));
        let mut restored = assert_archive_roundtrip(&e, &packs);
        for g in [&mut e, &mut restored] {
            answer(g, vec![ChoiceOption::Card(red_tao)]);
        }
        assert_eq!(e.state_hash(), restored.state_hash());
        assert_eq!(e.state_core().player(1).hp, 4, "视为【闪】成功抵消【杀】");

        // (b) 求【杀】窗口（南蛮入侵）：1 号仅持红桃【桃】，可经 va_resp_sha 响应
        e.debug_clear_hand(2);
        let red_tao2 = e.debug_place(1, "tao");
        let nanman = e.debug_place(0, "savage");
        submit(
            &mut e,
            0,
            GameAction::PlayCard {
                card: nanman,
                targets: Vec::new(),
            },
        );
        let ask_sha = e.open_choice().cloned().expect("南蛮求杀窗口");
        assert!(ask_sha.spec.options.contains(&ChoiceOption::Card(red_tao2)));
        let mut restored = assert_archive_roundtrip(&e, &packs);
        for g in [&mut e, &mut restored] {
            answer(g, vec![ChoiceOption::Card(red_tao2)]);
        }
        assert_eq!(e.state_hash(), restored.state_hash());
        assert_eq!(e.state_core().player(1).hp, 4, "视为【杀】成功响应南蛮入侵");
        drain_choices(&mut e);

        // (c) 【无懈可击】窗口：1 号持黑色【杀】，可经 va_resp_wuxie 抵消 0 号的【铁索连环】
        let spade_pos = e
            .state_core()
            .deck
            .iter()
            .position(|&c| {
                e.catalog().cards[c as usize].suit == Suit::Spade
                    && e.catalog().cards[c as usize].kind == CardKind::Sha
            })
            .unwrap();
        let black_sha = e.state_core_mut().deck.remove(spade_pos);
        e.state_core_mut().player_mut(1).hand.push(black_sha);
        let chain = e.debug_place(0, "chain");
        submit(
            &mut e,
            0,
            GameAction::PlayCard {
                card: chain,
                targets: vec![1],
            },
        );
        while let Some(ask) = e.open_choice().cloned() {
            if ask.observer == 1 && ask.spec.options.contains(&ChoiceOption::Card(black_sha)) {
                let mut restored = assert_archive_roundtrip(&e, &packs);
                for g in [&mut e, &mut restored] {
                    answer(g, vec![ChoiceOption::Card(black_sha)]);
                }
                assert_eq!(e.state_hash(), restored.state_hash());
            } else {
                answer(&mut e, vec![ChoiceOption::Pass]);
            }
        }
        assert!(
            !e.state_core().player(1).linked,
            "视为【无懈可击】成功抵消【铁索连环】"
        );

        // (d) 濒死求【桃】/【酒】窗口（含 BOSS_FUTAI 禁桃双站点一致性）：
        // 2 号持有 BOSS_FUTAI（非穷奇回合禁真桃），1 号濒死（hp = 0）且同时持有真【桃】与黑桃【杀】，
        // 候选选项必须排除被禁的真【桃】并保留 jiuchi 黑桃【杀】，且 NGSA 107 存读档不触发 OpenChoiceMismatch
        e.debug_grant(2, engine_native::content::standard::BOSS_FUTAI);
        let spade_pos2 = e
            .state_core()
            .deck
            .iter()
            .position(|&c| {
                e.catalog().cards[c as usize].suit == Suit::Spade
                    && e.catalog().cards[c as usize].kind == CardKind::Sha
            })
            .unwrap();
        let spade_sha = e.state_core_mut().deck.remove(spade_pos2);
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        e.debug_clear_hand(2);
        let banned_tao = e.debug_place(1, "tao");
        e.state_core_mut().player_mut(1).hand.push(spade_sha);
        e.state_core_mut().player_mut(1).hp = 1;
        e.debug_deal_damage(
            Some(0),
            1,
            1,
            engine_native::event::DamageNature::Normal,
        );
        while let Some(ask) = e.open_choice().cloned() {
            if ask.observer == 1 && ask.spec.options.contains(&ChoiceOption::Card(spade_sha)) {
                assert!(
                    !ask.spec.options.contains(&ChoiceOption::Card(banned_tao)),
                    "boss_futai 禁桃生效时濒死询问不可包含真【桃】"
                );
                let mut restored = assert_archive_roundtrip(&e, &packs);
                for g in [&mut e, &mut restored] {
                    answer(g, vec![ChoiceOption::Card(spade_sha)]);
                }
                assert_eq!(e.state_hash(), restored.state_hash());
            } else {
                answer(&mut e, vec![ChoiceOption::Pass]);
            }
        }
        assert_eq!(
            e.state_core().player(1).hp,
            1,
            "jiuchi 将黑桃牌视为【酒】在濒死窗口成功自救"
        );
    }
}

#[test]
fn pa12_batch15_resolvers_154_to_178_compile_and_execute_with_ngsa107() {
    // 1. 验证真实 `.skill` 文件全部通过编译器，且不含未支持修饰器的真实文件通过 `bridge_vm_skill_decls`
    let all_real_sources = [
        ("pot_weiyan", POT_WEIYAN),
        ("pot_caoshuang", POT_CAOSHUANG),
        ("pot_lvyi", POT_LVYI),
        ("pot_yuji", POT_YUJI),
        ("pot_zhouyu", POT_ZHOUYU),
        ("sp_zhonghui", SP_ZHONGHUI),
        ("ddd_liuhong", DDD_LIUHONG),
        ("twguohuai", TW_GUOHUAI),
        ("ddd_xuelingyun", DDD_XUELINGYUN),
        ("pot_taishici", POT_TAISHICI),
        ("ddd_xujing", DDD_XUJING),
        ("ddd_liuba", DDD_LIUBA),
        ("ddd_liuye", DDD_LIUYE),
        ("ddd_zhangkai", DDD_ZHANGKAI),
        ("extra_jilin", EXTRA_JILIN),
        ("twfuman", TW_FUMAN),
        ("twkujian", TW_KUJIAN),
        ("twxiawei", TW_XIAWEI),
        ("caoyi", XIANDING_CAOYI),
        ("dc_sb_lusu", DC_SB_LUSU),
        ("dc_sb_zhouyu", DC_SB_ZHOUYU),
        ("mb_sunjun", MB_SUNJUN),
        ("ddd_caoshuang", DDD_CAOSHUANG),
    ];
    let mut real_vm_skills = Vec::new();
    for (id, src) in &all_real_sources {
        let pkg = compile_source_named(*id, src).expect("编译真实 .skill");
        assert!(!pkg.skill_index.is_empty(), "{id} 应有 skill_index");
        if let Ok(out) = content_native_loader::bridge_vm_skill_decls(&pkg) {
            real_vm_skills.extend(out.decls);
        }
    }
    assert!(
        real_vm_skills.len() >= 10,
        "真实批⑮技能文件应成功产出 VM 技能声明，实际={}",
        real_vm_skills.len()
    );

    // 2. 构造可直触发的 25 个批⑮原语主动技，逐一验证在原生 Engine 上的交互选择、NGSA 107 存读档与状态落地
    let harness_dsl = r#"
skill potkuanggu(4200000562) {
    modifier hand_limit add 1;
}
skill h_154_potweiyan(91500201) {
    active play_phase {
        effect resolve_potweiyan(self, event_player, event_card, 0);
        effect resolve_potweiyan(self, event_player, event_card, 1);
    }
}
skill h_155_potcaoshuang(91500202) {
    active play_phase {
        effect resolve_potcaoshuang(self, event_player, event_card, 0);
    }
}
skill h_156_potlvyi(91500203) {
    active play_phase {
        effect resolve_potlvyi(self, event_player, event_card, 3);
    }
}
skill h_157_potyuji(91500204) {
    active play_phase {
        effect resolve_potyuji(self, event_player, event_card, 0);
    }
}
skill h_158_potzhouyu(91500205) {
    active play_phase {
        effect resolve_potzhouyu(self, event_player, event_card, 1);
    }
}
skill h_159_mbzhonghui(91500206) {
    active play_phase {
        effect resolve_mbzhonghui(self, self, 0, 0, 2212391997, 2942773382);
    }
}
skill h_160_ddd_liuhong(91500207) {
    active play_phase {
        effect resolve_ddd_liuhong(self, event_player, 0);
    }
}
skill h_161_twguohuai(91500208) {
    active play_phase {
        effect resolve_twguohuai(self, event_player, event_source, event_card, 2473230123, 3964351777, 0);
    }
}
skill h_162_ddd_xuelingyun(91500209) {
    active play_phase {
        effect resolve_ddd_xuelingyun(self, event_player, event_card, 0);
    }
}
skill h_163_pottaishici(91500210) {
    active play_phase {
        effect resolve_pottaishici(self, self, 0, 2573360121, 3829642123);
    }
}
skill h_164_ddd_xujing(91500211) {
    active play_phase {
        effect resolve_ddd_xujing(self, event_source, event_card, 0);
    }
}
skill h_165_ddd_liuba(91500212) {
    active play_phase {
        effect resolve_ddd_liuba(self, self, 0);
    }
}
skill h_166_ddd_liuye(91500213) {
    active play_phase {
        effect resolve_ddd_liuye(self, event_player, event_card, 0);
    }
}
skill h_167_ddd_zhangkai(91500214) {
    active play_phase {
        effect resolve_ddd_zhangkai(self, event_player, event_card, 0);
    }
}
skill h_168_hidden_pile(91500215) {
    active play_phase {
        effect resolve_hidden_skill_pile(self, 999999999, 0, event_card);
        effect resolve_hidden_skill_pile(self, 999999999, 3, event_card);
    }
}
skill h_169_twfuman(91500216) {
    active play_phase {
        effect resolve_twfuman(self, event_source, 0);
    }
}
skill h_170_twkujian(91500217) {
    active play_phase {
        effect resolve_twkujian(self, event_source, 0);
    }
}
skill h_171_twxiawei(91500218) {
    active play_phase {
        effect resolve_twxiawei(self, 3476660685, 0);
    }
}
skill h_172_caoyi(91500219) {
    active play_phase {
        effect resolve_caoyi(self, event_card, 3041789307, 0);
    }
}
skill h_173_dc_sb_lusu(91500220) {
    active play_phase {
        effect resolve_dc_sb_lusu(self, self, event_card, 0);
    }
}
skill h_174_dc_sb_zhouyu(91500221) {
    active play_phase {
        effect resolve_dc_sb_zhouyu(self, event_source, event_card, 0);
    }
}
skill h_175_mbxiongtu(91500222) {
    active play_phase {
        effect resolve_mbxiongtu(self, event_source, 0);
    }
}
skill h_176_pothongyi(91500223) {
    active play_phase {
        effect resolve_pothongyi(self, 0);
    }
}
skill h_177_spyingjia(91500224) {
    active play_phase {
        effect resolve_spyingjia(self, 0, 3612952432);
    }
}
skill h_178_ddd_caoshuang(91500225) {
    active play_phase {
        effect resolve_ddd_caoshuang(self, event_player, event_card, 0);
    }
}
"#;
    let pack = pack_strict(&[("harness", harness_dsl)], "pa12-b15-harness");
    let packs = [&pack];

    let skill_names = [
        "h_154_potweiyan",
        "h_155_potcaoshuang",
        "h_156_potlvyi",
        "h_157_potyuji",
        "h_158_potzhouyu",
        "h_159_mbzhonghui",
        "h_160_ddd_liuhong",
        "h_161_twguohuai",
        "h_162_ddd_xuelingyun",
        "h_163_pottaishici",
        "h_164_ddd_xujing",
        "h_165_ddd_liuba",
        "h_166_ddd_liuye",
        "h_167_ddd_zhangkai",
        "h_168_hidden_pile",
        "h_169_twfuman",
        "h_170_twkujian",
        "h_171_twxiawei",
        "h_172_caoyi",
        "h_173_dc_sb_lusu",
        "h_174_dc_sb_zhouyu",
        "h_175_mbxiongtu",
        "h_176_pothongyi",
        "h_177_spyingjia",
        "h_178_ddd_caoshuang",
    ];

    let mut interactive_verified = 0usize;
    for (idx, name) in skill_names.iter().enumerate() {
        let mut e = Engine::new_with_extensions(915_200 + idx as u64, 3, &packs).expect("建局");
        advance_to_phase(&mut e, 0, Phase::Play);
        for seat in 0..3u8 {
            let _ = e.debug_place(seat, "sha");
            let _ = e.debug_place(seat, "shan");
        }
        e.debug_grant(0, name);
        let def = e.catalog().skill_by_name(name).unwrap();
        submit(&mut e, 0, GameAction::ActivateSkill { skill: def });
        if let Some(ask) = e.open_choice().cloned() {
            interactive_verified += 1;
            let mut restored = assert_archive_roundtrip(&e, &packs);
            let sel: Vec<ChoiceOption> = ask
                .spec
                .options
                .iter()
                .take(ask.spec.min.max(1))
                .copied()
                .collect();
            for g in [&mut e, &mut restored] {
                answer(g, sel.clone());
                drain_choices(g);
            }
            assert_eq!(
                e.state_hash(),
                restored.state_hash(),
                "{name} 交互恢复后状态哈希一致"
            );
        } else {
            let _ = assert_archive_roundtrip(&e, &packs);
        }
    }
    assert!(
        interactive_verified >= 8,
        "多候选局面下应有批⑮原语触发真实 WaitChoice 交互与 NGSA 107 存读档往返（实际 {interactive_verified}）"
    );

    // 专项断言：h_154_potweiyan (action 0 + action 1) 赋予 potkuanggu 且置位 potweiyan_status
    {
        let mut e = Engine::new_with_extensions(915_301, 2, &packs).expect("建局");
        advance_to_phase(&mut e, 0, Phase::Play);
        e.debug_grant(0, "h_154_potweiyan");
        let def = e.catalog().skill_by_name("h_154_potweiyan").unwrap();
        submit(&mut e, 0, GameAction::ActivateSkill { skill: def });
        let kg = e.catalog().skill_by_name("potkuanggu").unwrap();
        assert!(e.state_core().skills.contains_key(&(0, kg)));
        assert_eq!(
            e.state_core()
                .vm_aux
                .flags
                .get(&(0, "potweiyan_status".into()))
                .copied(),
            Some(1)
        );
    }

    // 专项断言：h_170_twkujian 交互选手牌交给 1 号角色
    {
        let mut e = Engine::new_with_extensions(915_302, 2, &packs).expect("建局");
        advance_to_phase(&mut e, 0, Phase::Play);
        e.debug_clear_hand(0);
        let c1 = e.debug_place(0, "sha");
        let c2 = e.debug_place(0, "shan");
        let hand1_before = e.state_core().player(1).hand.len();
        e.debug_grant(0, "h_170_twkujian");
        let def = e.catalog().skill_by_name("h_170_twkujian").unwrap();
        submit(&mut e, 0, GameAction::ActivateSkill { skill: def });
        let ask = e.open_choice().cloned().expect("twkujian 选牌询问");
        assert_eq!(ask.spec.kind, ChoiceKind::SelectCards);
        answer(&mut e, vec![ChoiceOption::Card(c1), ChoiceOption::Card(c2)]);
        assert!(e.state_core().player(0).hand.is_empty());
        assert_eq!(e.state_core().player(1).hand.len(), hand1_before + 2);
        assert_eq!(
            e.state_core()
                .vm_aux
                .flags
                .get(&(0, format!("twkujian_card:{c1}")))
                .copied(),
            Some(1)
        );
    }
}

#[test]
fn pa12_batch15_ported_skill_bridge_inventory() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../content/original/ported");
    let mut files = Vec::new();
    let mut stack = vec![root];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).expect("read_dir") {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().and_then(|s| s.to_str()) == Some("skill") {
                files.push(path);
            }
        }
    }
    files.sort();

    let mut bridged_ok = 0usize;
    let mut noop_rejected = 0usize;
    let mut view_as_rejected = 0usize;
    for file in &files {
        let src = std::fs::read_to_string(file).expect("read .skill");
        let is_pure_noop = (src.contains("INTENTIONAL_EMPTY_NOOP")
            || src.contains("modifier hand_limit add 0;"))
            && !src.contains("trigger ")
            && !src.contains("active ")
            && !src.contains("view_as ");
        let Ok(pkg) = content_compiler::compile_source_named("probe", &src) else {
            continue;
        };
        match content_native_loader::bridge_vm_skill_decls(&pkg) {
            Ok(_) => {
                assert!(
                    !is_pure_noop,
                    "INTENTIONAL_EMPTY_NOOP must never pass bridge_vm_skill_decls: {}",
                    file.display()
                );
                bridged_ok += 1;
            }
            Err(err) => {
                let msg = err.to_string();
                if is_pure_noop {
                    noop_rejected += 1;
                } else if msg.contains("view_as_index") {
                    view_as_rejected += 1;
                }
            }
        }
    }

    eprintln!(
        "PA12 Batch 15 ported .skill bridge inventory: bridged_ok={}/{}, noop_rejected={}, view_as_rejected={}",
        bridged_ok,
        files.len(),
        noop_rejected,
        view_as_rejected
    );
    assert!(
        bridged_ok >= 1200,
        "PA12 Batch 15 should unlock >= 1200 ported .skill files at loader bridge (got {bridged_ok})"
    );
    assert!(
        noop_rejected >= 800,
        "All INTENTIONAL_EMPTY_NOOP placeholder files must remain rejected (got {noop_rejected})"
    );
}
