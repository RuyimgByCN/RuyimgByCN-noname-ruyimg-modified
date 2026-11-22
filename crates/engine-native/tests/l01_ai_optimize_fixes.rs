//! L01 `AI优化` 全量 69 技能 + 1 卡牌(`yiyi`) + 27 含规则配置 实盘闭环测试。
//!
//! 对照上游：`archive/upstream/noname-liuli/noname/extension/AI优化/extension.js`
//! - 全量清单契约与正式包加载（不改动冻结 `decision34_pack9_for_coexistence` 哈希）；
//! - `yiyi` 标准 `1..=3` 目标、`Kingdoms` / `Versus` 自动全体友方(`-1`)、无懈可击窗口、依次摸2弃2(`he`)与挂起读档；
//! - `fixFour` 4人去忠加主血改明反 + 击杀反贼奖摸3 + 存档校验放行；
//! - `_findZhong` 仅主公侧已知1忠不泄密；`_aiyh_neiKey` (`off`/`can`/`must`) 亮明内奸并回复 + 读档恢复；
//! - `_aiyh_fake_prohibited` (`Wj`/`wjs`/`ban*`) AI 禁将替换与空池保底；
//! - `xinfu_limu` 方块牌自【乐】即回血与无限距离出杀、`zhenlie_lose` 暴露分、`xyyh` 冲虚免小游戏、`_aiyh_fixEQz`/`_aiyh_fixECf` 局内微调；
//! - `_attzhi` + 13 `att*` + `_yingshilanggu` 公开事件追踪、`BoundedPlanner` 策略与严格零隐藏信息泄密不变式。

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use content_native_loader::{
    apply_cf_rating_threat, apply_l01_config_to_engine, attach_formal_package_upstream_rules,
    decision34_pack9_extension_pack, parse_ai_tuning_config, to_l01_ai_options,
    yiyi_select_target_range, yiyi_wuxie_weight, CfMode, NeiKeyMode, QzMode, L01_ALL_69_SKILLS,
};
use engine_native::ai::{
    chaofeng_threat, l01_public_attitude, BoundedPlanner, GeneralRarity, L01AiOptions,
    L01StrategyMode, NhFriendsMode,
};
use engine_native::archive::{decode_archive, encode_archive};
use engine_native::catalog::Suit;
use engine_native::content::extension::{ExtensionDef, ExtensionPack, InitHook};
use engine_native::content::host::{HostBehavior, HostSkillDecl};
use engine_native::event::Phase;
use engine_native::state::KingdomIdentity;
use engine_native::{
    ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, GameMode, RejectReason, Role,
};

static DEDUP: AtomicU64 = AtomicU64::new(10_000);

pub(super) fn submit(e: &mut Engine, actor: u8, action: GameAction) -> CommandOutcome {
    e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: DEDUP.fetch_add(1, Ordering::Relaxed),
        action,
    })
}

pub(super) fn drain(e: &mut Engine) {
    for _ in 0..64 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
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
    for _ in 0..80 {
        if e.turn().current == seat && e.turn().phase == Phase::Play && e.open_choice().is_none() {
            return;
        }
        if e.open_choice().is_some() {
            drain(e);
            continue;
        }
        let cur = e.turn().current;
        let _ = submit(e, cur, GameAction::NextPhase);
    }
    panic!("failed to reach seat={seat} Play phase");
}

pub(super) fn snapshot_roundtrip(e: &Engine, packs: &[&ExtensionPack]) -> Engine {
    let bytes = encode_archive(&e.save());
    let snap = decode_archive(&bytes).expect("decode archive");
    let restored = Engine::restore_with_extensions(&snap, packs).expect("restore archive");
    assert_eq!(e.state_hash(), restored.state_hash());
    assert_eq!(e.open_choice(), restored.open_choice());
    restored
}

pub(super) fn l01_battle_pack() -> ExtensionPack {
    let mut p = decision34_pack9_extension_pack();
    attach_formal_package_upstream_rules("upstream-liuli", "ai-tuning", &mut p, &[]);
    for (name, behavior) in [
        ("zhenlie", HostBehavior::Zhenlie),
        ("th_chongxu", HostBehavior::L28ThunderBatchA),
    ] {
        if !p.host_skills.iter().any(|h| h.name == name) {
            p.host_skills.push(HostSkillDecl {
                name,
                behavior,
                presentation_binding: None,
            });
        }
    }
    p
}

fn l01_formal_only_pack() -> ExtensionPack {
    let mut p = ExtensionPack {
        def: ExtensionDef {
            name: "upstream-liuli/ai-tuning",
            version: 1,
            description: "L01 AI优化",
        },
        generals: Vec::new(),
        cards: Vec::new(),
        host_skills: Vec::new(),
        vm_skills: Vec::new(),
        init_hook: InitHook::None,
        enabled: true,
    };
    attach_formal_package_upstream_rules("upstream-liuli", "ai-tuning", &mut p, &[]);
    p
}

pub(super) fn activate_skill(e: &mut Engine, actor: u8, name: &str) -> CommandOutcome {
    let skill = e.catalog().skill_by_name(name).expect("registered skill");
    submit(e, actor, GameAction::ActivateSkill { skill })
}

pub(super) fn flag(e: &Engine, seat: u8, key: &str) -> u32 {
    e.state_core()
        .vm_aux
        .flags
        .get(&(seat, key.to_string()))
        .copied()
        .unwrap_or(0)
}

pub(super) fn parse_l01_cfg(extra_json: &str) -> content_native_loader::AiTuningConfig {
    let mut base = serde_json::json!({
        "source": "upstream-liuli",
        "upstream_path": "archive/upstream/noname-liuli/noname/extension/AI优化/extension.js",
        "upstream_commit": "test",
        "cf": {},
        "qz": {}
    });
    let extra: serde_json::Value = serde_json::from_str(extra_json).expect("valid extra json");
    let obj = base.as_object_mut().unwrap();
    for (k, v) in extra.as_object().unwrap() {
        obj.insert(k.clone(), v.clone());
    }
    parse_ai_tuning_config(&base.to_string()).expect("valid l01 config")
}

#[test]
fn l01_all_69_skills_yiyi_and_27_rule_coupled_configs_contract() {
    let inv_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/migration/g0/inventory_status.json");
    let raw = std::fs::read_to_string(&inv_path).expect("read inventory_status.json");
    let v: serde_json::Value = serde_json::from_str(&raw).expect("parse inventory_status.json");
    let items = v["items"].as_object().expect("items object");

    let mut inv_skills = BTreeSet::new();
    let mut inv_cards = BTreeSet::new();
    let mut inv_configs = BTreeSet::new();
    for item in items.values() {
        if item.get("source_id").and_then(|x| x.as_str()) != Some("L01") {
            continue;
        }
        let cat = item.get("category").and_then(|x| x.as_str()).unwrap_or("");
        let id = item.get("id").and_then(|x| x.as_str()).unwrap_or("").to_string();
        match cat {
            "skills" => {
                inv_skills.insert(id);
            }
            "cards" => {
                inv_cards.insert(id);
            }
            "configs" => {
                inv_configs.insert(id);
            }
            _ => {}
        }
    }

    assert_eq!(inv_cards.len(), 1);
    assert!(inv_cards.contains("yiyi"));
    assert_eq!(inv_skills.len(), 69);
    let const_skills: BTreeSet<String> = L01_ALL_69_SKILLS.iter().map(|s| s.to_string()).collect();
    assert_eq!(inv_skills, const_skills, "L01_ALL_69_SKILLS must match inventory_status.json 1:1");

    // 27 项含规则配置必须全部在清单中存在，并能严格解析。
    let rule_coupled_27 = [
        "btAi", "zbAi", "wjAi", "sfjAi", "nhFriends", "ntAoe", "mjAi", "xyyh", "hsyh",
        "findZhong", "neiKey", "attceshi", "fixFour", "Wj", "wjs", "banzhu", "banzhong",
        "banfan", "bannei", "bandizhu", "bannongmin", "fixEQz", "applyQz", "ckQz", "qzCf",
        "fixECf", "applyCf",
    ];
    for k in rule_coupled_27 {
        assert!(inv_configs.contains(k), "missing L01 config key in inventory: {k}");
    }

    let full_cfg_json = serde_json::json!({
        "source": "upstream-liuli",
        "upstream_path": "archive/upstream/noname-liuli/noname/extension/AI优化/extension.js",
        "upstream_commit": "test",
        "cf": {},
        "qz": {},
        "applyCf": "pj",
        "applyQz": true,
        "ckQz": "cf",
        "qzCf": true,
        "btAi": "gjcx",
        "sfjAi": "shyg",
        "zbAi": true,
        "wjAi": false,
        "nhFriends": "hp",
        "ntAoe": true,
        "mjAi": true,
        "xyyh": true,
        "hsyh": false,
        "findZhong": true,
        "neiKey": "must",
        "attceshi": true,
        "fixFour": true,
        "Wj": true,
        "wjs": "4",
        "wj": ["caocao"],
        "banzhu": ["liubei"],
        "banzhong": ["sunquan"],
        "banfan": ["guanyu"],
        "bannei": ["lvbu"],
        "bandizhu": ["machao"],
        "bannongmin": ["zhaoyun"],
        "fixEQz": true,
        "fixECf": true
    })
    .to_string();
    let parsed = parse_ai_tuning_config(&full_cfg_json).expect("valid full 27 config");
    assert_eq!(parsed.apply_cf, CfMode::Auto);
    assert_eq!(parsed.ck_qz, QzMode::Threat);
    assert_eq!(parsed.bt_ai, L01StrategyMode::Gjcx);
    assert_eq!(parsed.sfj_ai, L01StrategyMode::Shyg);
    assert!(!parsed.wj_ai);
    assert_eq!(parsed.nh_friends_mode, NhFriendsMode::SelfHp);
    assert!(parsed.nt_aoe);
    assert!(parsed.mj_ai);
    assert!(parsed.xyyh);
    assert!(!parsed.hsyh);
    assert!(parsed.find_zhong);
    assert_eq!(parsed.nei_key, NeiKeyMode::Must);
    assert!(parsed.fix_four);
    assert!(parsed.wj);
    assert_eq!(parsed.wjs, Some(4));
    assert_eq!(parsed.ban_all, vec!["caocao"]);
    assert_eq!(parsed.ban_zhu, vec!["liubei"]);
    assert_eq!(parsed.ban_zhong, vec!["sunquan"]);
    assert_eq!(parsed.ban_fan, vec!["guanyu"]);
    assert_eq!(parsed.ban_nei, vec!["lvbu"]);
    assert_eq!(parsed.ban_dizhu, vec!["machao"]);
    assert_eq!(parsed.ban_nongmin, vec!["zhaoyun"]);
    assert!(parsed.fix_eqz);
    assert!(parsed.fix_ecf);

    // 坏值拒绝
    assert!(parse_ai_tuning_config(
        r#"{"source":"upstream-liuli","upstream_path":"x","upstream_commit":"y","cf":{},"qz":{},"neiKey":"bad"}"#
    )
    .is_err());
    assert!(parse_ai_tuning_config(
        r#"{"source":"upstream-liuli","upstream_path":"x","upstream_commit":"y","cf":{},"qz":{},"nhFriends":"9"}"#
    )
    .is_err());
    assert!(parse_ai_tuning_config(
        r#"{"source":"upstream-liuli","upstream_path":"x","upstream_commit":"y","cf":{},"qz":{},"wjs":"99"}"#
    )
    .is_err());
    assert!(parse_ai_tuning_config(
        r#"{"source":"upstream-liuli","upstream_path":"x","upstream_commit":"y","cf":{},"qz":{},"btAi":"unknown"}"#
    )
    .is_err());

    // 正式包单独加载必须携带 yiyi + 全部 69 项技能，且不改动其他既有包内容。
    let pack = l01_formal_only_pack();
    let eng = Engine::new_with_extensions(1, 4, &[&pack]).expect("standalone ai-tuning pack");
    assert!(eng.catalog().cards.iter().any(|c| c.name == "yiyi"));
    for s in L01_ALL_69_SKILLS {
        assert!(eng.catalog().skill_by_name(s).is_some(), "missing formal skill {s}");
    }
    let l01_tags: BTreeSet<u16> = pack
        .host_skills
        .iter()
        .filter(|h| L01_ALL_69_SKILLS.contains(&h.name))
        .map(|h| h.behavior.tag())
        .collect();
    assert!(l01_tags.is_subset(&BTreeSet::from([861, 862, 863, 864])));
    let _coex = decision34_pack9_extension_pack();
}

#[test]
fn l01_yiyi_standard_1_to_3_kingdoms_versus_nullify_and_restore() {
    let pack = l01_battle_pack();

    // Case A: Prototype / Identity 1..=3 目标上限 + 依次摸2弃2(he含装备) + 挂起读档恢复
    {
        let mut e = Engine::new_with_extensions(202601, 4, &[&pack]).expect("proto engine");
        advance_to_play(&mut e, 2);
        for p in 0..4 {
            e.debug_clear_hand(p);
        }
        let equip2 = e.debug_place(2, "armor");
        let _ = submit(
            &mut e,
            2,
            GameAction::PlayCard {
                card: equip2,
                targets: vec![],
            },
        );
        drain(&mut e);
        advance_to_play(&mut e, 0);
        for p in 0..4 {
            e.debug_clear_hand(p);
        }
        let yiyi_card = e.debug_move_named_card_to_hand(0, "yiyi");
        let _extra0 = e.debug_place(0, "sha");

        // 0 目标与 4 目标在非 guozhan/versus 下均应拒绝
        assert!(matches!(
            submit(
                &mut e,
                0,
                GameAction::PlayCard {
                    card: yiyi_card,
                    targets: vec![]
                }
            ),
            CommandOutcome::Rejected(RejectReason::IllegalTargets)
        ));
        assert!(matches!(
            submit(
                &mut e,
                0,
                GameAction::PlayCard {
                    card: yiyi_card,
                    targets: vec![0, 1, 2, 3]
                }
            ),
            CommandOutcome::Rejected(RejectReason::IllegalTargets)
        ));
        // 上游数值口径公式一并校验
        assert_eq!(yiyi_select_target_range("identity"), (1, 3));
        assert_eq!(yiyi_select_target_range("guozhan"), (-1, -1));
        assert_eq!(yiyi_select_target_range("versus"), (-1, -1));
        assert_eq!(yiyi_wuxie_weight(), 0);

        let hand0_before = e.debug_hand(0).len();
        let hand2_before = e.debug_hand(2).len();
        let out = submit(
            &mut e,
            0,
            GameAction::PlayCard {
                card: yiyi_card,
                targets: vec![0, 2],
            },
        );
        assert!(matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ));
        // 先让 seat 0 完成弃 2：确保遇到非 l01_yiyi_discard 的无懈询问时安全回落跳过
        for step in 0..64 {
            let Some(c) = e.open_choice().cloned() else {
                panic!("yiyi finished before seat 0 discard");
            };
            if c.spec.prompt == "l01_yiyi_discard" && c.observer == 0 {
                // 此刻 0 刚摸 2：先选 2 张弃置
                let pick: Vec<ChoiceOption> = c.spec.options.iter().take(2).copied().collect();
                assert_eq!(c.spec.min, 2);
                assert_eq!(c.spec.max, 2);
                let ans = submit(
                    &mut e,
                    0,
                    GameAction::Answer {
                        choice_id: c.id,
                        selection: pick,
                    },
                );
                assert!(
                    matches!(ans, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
                    "{ans:?}"
                );
                break;
            }
            let sel = if c.spec.options.contains(&ChoiceOption::Pass) {
                vec![ChoiceOption::Pass]
            } else if c.spec.allow_cancel {
                Vec::new()
            } else {
                c.spec.options.iter().take(c.spec.min.max(1)).copied().collect()
            };
            let ans = submit(
                &mut e,
                c.observer,
                GameAction::Answer {
                    choice_id: c.id,
                    selection: sel,
                },
            );
            assert!(
                matches!(ans, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
                "step {step} prompt={} rejected: {ans:?}",
                c.spec.prompt
            );
        }
        // 到 2 弃牌：验证 options 包含 2 的装备(he 区域)，并在挂起态做完整读档恢复！
        let ask2 = e.open_choice().expect("seat 2 l01_yiyi_discard").clone();
        assert_eq!(ask2.observer, 2);
        assert_eq!(ask2.spec.prompt, "l01_yiyi_discard");
        assert!(
            ask2.spec.options.contains(&ChoiceOption::Card(equip2)),
            "l01_yiyi_discard must include equips (he zone)"
        );

        let mut restored = snapshot_roundtrip(&e, &[&pack]);
        let pick2: Vec<ChoiceOption> = vec![
            ChoiceOption::Card(equip2),
            *ask2
                .spec
                .options
                .iter()
                .find(|&&o| o != ChoiceOption::Card(equip2))
                .expect("second he card"),
        ];
        let _ = submit(
            &mut e,
            2,
            GameAction::Answer {
                choice_id: ask2.id,
                selection: pick2.clone(),
            },
        );
        let _ = submit(
            &mut restored,
            2,
            GameAction::Answer {
                choice_id: ask2.id,
                selection: pick2,
            },
        );
        assert_eq!(e.state_hash(), restored.state_hash());
        assert!(e.open_choice().is_none());
        assert_eq!(flag(&e, 0, "l01_yiyi_resolved"), 1);
        assert_eq!(flag(&e, 0, "l01_yiyi_discarded"), 4);
        assert_eq!(e.debug_hand(0).len(), hand0_before + 1 - 2);
        assert_eq!(e.debug_hand(2).len(), hand2_before + 2 - 1);
        assert!(e.debug_equips(2).is_empty(), "equip2 discarded via he selection");
    }

    // Case B: Nullify 阻断 yiyi + L01 AI 默认不无懈 yiyi
    {
        let mut e = Engine::new_with_extensions(202602, 4, &[&pack]).expect("nullify test");
        advance_to_play(&mut e, 0);
        for p in 0..4 {
            e.debug_clear_hand(p);
        }
        let yiyi_card = e.debug_move_named_card_to_hand(0, "yiyi");
        let wuxie = e.debug_place(1, "nullify");
        let _ = submit(
            &mut e,
            0,
            GameAction::PlayCard {
                card: yiyi_card,
                targets: vec![0],
            },
        );
        let ask = e.open_choice().expect("nullify prompt").clone();
        assert_eq!(ask.observer, 1);

        let mut opts = L01AiOptions::default();
        opts.bt_ai = L01StrategyMode::Shyg;
        let planner = BoundedPlanner::new(64).with_l01_options(opts);
        let planned = e.plan_action(1, &planner).action;
        match planned {
            GameAction::Answer { selection, .. } => {
                assert!(
                    selection.is_empty() || selection == vec![ChoiceOption::Pass],
                    "L01 AI must decline wuxie on yiyi: {selection:?}"
                );
            }
            other => panic!("unexpected plan action: {other:?}"),
        }

        // 手动使用无懈阻断 -> 不摸牌、无弃牌询问
        let _ = submit(
            &mut e,
            1,
            GameAction::Answer {
                choice_id: ask.id,
                selection: vec![ChoiceOption::Card(wuxie)],
            },
        );
        drain(&mut e);
        assert_eq!(flag(&e, 0, "l01_yiyi_resolved"), 0);
        assert_eq!(e.debug_hand(0).len(), 0);
    }

    // Case C: Versus (2v2) 与 Kingdoms 自动全体友方 (-1)
    {
        let versus = GameMode::Versus {
            teams: vec![vec![0, 2], vec![1, 3]],
            substitutes: Vec::new(),
        };
        let mut e = Engine::try_new_with_extensions_mode(202603, 4, &[&pack], versus)
            .expect("versus mode");
        advance_to_play(&mut e, 0);
        for p in 0..4 {
            e.debug_clear_hand(p);
        }
        let yiyi_v = e.debug_move_named_card_to_hand(0, "yiyi");
        assert!(matches!(
            submit(
                &mut e,
                0,
                GameAction::PlayCard {
                    card: yiyi_v,
                    targets: vec![1]
                }
            ),
            CommandOutcome::Rejected(RejectReason::IllegalTargets)
        ));
        assert!(matches!(
            submit(
                &mut e,
                0,
                GameAction::PlayCard {
                    card: yiyi_v,
                    targets: vec![0]
                }
            ),
            CommandOutcome::Rejected(RejectReason::IllegalTargets)
        ));
        // 空目标自动选取全体友方 [0, 2]
        let _ = submit(
            &mut e,
            0,
            GameAction::PlayCard {
                card: yiyi_v,
                targets: vec![],
            },
        );
        drain(&mut e);
        assert_eq!(flag(&e, 0, "l01_yiyi_resolved"), 1);
        assert_eq!(flag(&e, 0, "l01_yiyi_discarded"), 4);

        let mut ke =
            Engine::try_new_with_extensions_mode(202604, 4, &[&pack], GameMode::Kingdoms)
                .expect("kingdoms mode");
        ke.state_core_mut()
            .set_kingdom_identity(0, KingdomIdentity::Declared("wei".to_string()))
            .unwrap();
        ke.state_core_mut()
            .set_kingdom_identity(3, KingdomIdentity::Declared("wei".to_string()))
            .unwrap();
        ke.state_core_mut()
            .set_kingdom_identity(1, KingdomIdentity::Declared("shu".to_string()))
            .unwrap();
        ke.state_core_mut()
            .set_kingdom_identity(2, KingdomIdentity::Declared("shu".to_string()))
            .unwrap();
        advance_to_play(&mut ke, 0);
        for p in 0..4 {
            ke.debug_clear_hand(p);
        }
        let yiyi_k = ke.debug_move_named_card_to_hand(0, "yiyi");
        assert!(matches!(
            submit(
                &mut ke,
                0,
                GameAction::PlayCard {
                    card: yiyi_k,
                    targets: vec![0, 1]
                }
            ),
            CommandOutcome::Rejected(RejectReason::IllegalTargets)
        ));
        let _ = submit(
            &mut ke,
            0,
            GameAction::PlayCard {
                card: yiyi_k,
                targets: vec![0, 3],
            },
        );
        drain(&mut ke);
        assert_eq!(flag(&ke, 0, "l01_yiyi_resolved"), 1);
        assert_eq!(flag(&ke, 0, "l01_yiyi_discarded"), 4);
    }
}

#[test]
fn l01_fix_four_find_zhong_neikey_and_fake_prohibited_rules() {
    let pack = l01_battle_pack();

    // Case A: fixFour 4人局（主+1血，忠->明反，读档放行，击杀反贼奖摸3）
    {
        let mut e = Engine::try_new_identity_with_extensions(202605, 4, &[&pack]).expect("4p id");
        e.debug_set_identity_roles(vec![Role::Lord, Role::Loyalist, Role::Rebel, Role::Traitor]);
        let lord_hp0 = e.debug_hp(0);
        let lord_max0 = e.state_core().player(0).max_hp;
        let cfg = parse_l01_cfg(r#"{"fixFour":true}"#);
        apply_l01_config_to_engine(&cfg, &mut e);

        assert_eq!(flag(&e, 0, "l01_fix_four_applied"), 1);
        assert_eq!(e.state_core().player(0).max_hp, lord_max0 + 1);
        assert_eq!(e.debug_hp(0), lord_hp0 + 1);
        assert_eq!(
            e.state_core().identity_roles().unwrap(),
            &[Role::Lord, Role::Rebel, Role::Rebel, Role::Traitor]
        );
        assert_eq!(flag(&e, 1, "l01_role_shown"), 1);
        // 第三方观察者视图可见 seat 1 已明反
        assert_eq!(e.view_for(2).players[1].role, Some(Role::Rebel));
        assert_eq!(e.view_for(2).players[3].role, None);

        // 存档恢复校验对 fixFour 0忠局面必须合法
        let mut restored = snapshot_roundtrip(&e, &[&pack]);
        // seat 0 击杀原忠臣(现反贼) seat 1 仍得反贼奖摸3
        restored.debug_clear_hand(0);
        restored.debug_clear_hand(1);
        restored.debug_set_hp(1, 1);
        advance_to_play(&mut restored, 0);
        let hand_before = restored.debug_hand(0).len();
        let sha = restored.debug_place(0, "sha");
        let _ = submit(
            &mut restored,
            0,
            GameAction::PlayCard {
                card: sha,
                targets: vec![1],
            },
        );
        drain(&mut restored);
        assert!(!restored.debug_alive(1));
        assert_eq!(restored.debug_hand(0).len(), hand_before + 3);
    }

    // Case B: _findZhong 仅主公侧已知1名忠臣（其余视角严格不可见）
    {
        let mut e = Engine::try_new_identity_with_extensions(202606, 5, &[&pack]).expect("5p id");
        e.debug_set_identity_roles(vec![
            Role::Lord,
            Role::Loyalist,
            Role::Loyalist,
            Role::Rebel,
            Role::Traitor,
        ]);
        let cfg = parse_l01_cfg(r#"{"findZhong":true}"#);
        apply_l01_config_to_engine(&cfg, &mut e);
        let known_plus1 = flag(&e, 0, "l01_find_zhong_target_plus1");
        assert!(known_plus1 == 2 || known_plus1 == 3);
        let opts_lord = to_l01_ai_options(&cfg, 0, &e);
        let opts_rebel = to_l01_ai_options(&cfg, 3, &e);
        assert_eq!(opts_lord.lord_known_loyalist, Some((known_plus1 - 1) as u8));
        assert_eq!(opts_rebel.lord_known_loyalist, None);
        // 公共态度推导仅对主公本人加分，不对其他人泄密
        let lord_view = e.view_for(0);
        let rebel_view = e.view_for(3);
        let target_seat = (known_plus1 - 1) as u8;
        assert!(l01_public_attitude(&lord_view, Some(&opts_lord), 0, target_seat) > 0);
        assert_eq!(l01_public_attitude(&rebel_view, Some(&opts_rebel), 3, target_seat), 0);
    }

    // Case C: _aiyh_neiKey off/can/must + 读档恢复与回复双方
    {
        let mut e = Engine::try_new_identity_with_extensions(202607, 4, &[&pack]).expect("4p id");
        e.debug_set_identity_roles(vec![Role::Lord, Role::Loyalist, Role::Rebel, Role::Traitor]);
        e.debug_grant(3, "_aiyh_neiKey");
        e.debug_set_hp(0, 2);
        e.debug_set_hp(3, 2);
        advance_to_play(&mut e, 3);

        // off 模式拒绝发动
        let off_cfg = parse_l01_cfg(r#"{"neiKey":"off"}"#);
        apply_l01_config_to_engine(&off_cfg, &mut e);
        assert!(matches!(
            activate_skill(&mut e, 3, "_aiyh_neiKey"),
            CommandOutcome::Rejected(_)
        ));

        // can 模式允许发动 -> 公开内奸身份 + maxHp+1 + l01_neikey_recover 挂起读档
        let can_cfg = parse_l01_cfg(r#"{"neiKey":"can"}"#);
        apply_l01_config_to_engine(&can_cfg, &mut e);
        let max3_before = e.state_core().player(3).max_hp;
        let _ = activate_skill(&mut e, 3, "_aiyh_neiKey");
        let ask = e.open_choice().expect("l01_neikey_recover").clone();
        assert_eq!(ask.spec.prompt, "l01_neikey_recover");
        assert_eq!(flag(&e, 3, "l01_role_shown"), 1);
        assert_eq!(e.view_for(2).players[3].role, Some(Role::Traitor));
        assert_eq!(e.state_core().player(3).max_hp, max3_before + 1);

        let mut restored = snapshot_roundtrip(&e, &[&pack]);
        let ans = GameAction::Answer {
            choice_id: ask.id,
            selection: vec![ChoiceOption::Confirm],
        };
        let _ = submit(&mut e, 3, ans.clone());
        let _ = submit(&mut restored, 3, ans);
        assert_eq!(e.state_hash(), restored.state_hash());
        assert_eq!(e.debug_hp(0), 3);
        assert_eq!(e.debug_hp(3), 3);
        assert_eq!(flag(&e, 3, "l01_neikey_used"), 1);
        // 限一次不可再发
        assert!(matches!(
            activate_skill(&mut e, 3, "_aiyh_neiKey"),
            CommandOutcome::Rejected(_)
        ));

        // must 模式在角色已标记显示时仍可触发（只要未使用过）
        let mut e2 = Engine::try_new_identity_with_extensions(202608, 4, &[&pack]).expect("4p");
        e2.debug_set_identity_roles(vec![Role::Lord, Role::Loyalist, Role::Rebel, Role::Traitor]);
        e2.debug_grant(3, "_aiyh_neiKey");
        e2.state_core_mut()
            .vm_aux
            .flags
            .insert((3, "l01_role_shown".to_string()), 1);
        advance_to_play(&mut e2, 3);
        apply_l01_config_to_engine(&can_cfg, &mut e2);
        assert!(matches!(
            activate_skill(&mut e2, 3, "_aiyh_neiKey"),
            CommandOutcome::Rejected(_)
        ));
        let must_cfg = parse_l01_cfg(r#"{"neiKey":"must"}"#);
        apply_l01_config_to_engine(&must_cfg, &mut e2);
        let _ = activate_skill(&mut e2, 3, "_aiyh_neiKey");
        assert_eq!(
            e2.open_choice().map(|c| c.spec.prompt),
            Some("l01_neikey_recover")
        );
    }

    // Case D: _aiyh_fake_prohibited (Wj / wjs / ban*) AI 禁将替换 + 空池保底
    {
        let mut e = Engine::try_new_identity_with_extensions(202609, 4, &[&pack]).expect("4p wj");
        e.debug_set_identity_roles(vec![Role::Lord, Role::Loyalist, Role::Rebel, Role::Traitor]);
        e.debug_set_general(0, "曹操");
        e.debug_set_general(1, "刘备");
        e.debug_set_general(2, "孙权");
        e.debug_set_general(3, "张飞");
        let cfg = parse_l01_cfg(r#"{"Wj":true,"wjs":"3","wj":["刘备"],"banfan":["孙权"]}"#);
        apply_l01_config_to_engine(&cfg, &mut e);
        assert_eq!(flag(&e, 0, "l01_wj_pass_done"), 1);
        // human seat 0 保持不变；AI seat 1(banned by wj) 与 AI seat 2(banned by banfan) 已替换
        let g1_name = e
            .catalog()
            .general_meta(e.state_core().player(1).general_slots[0].unwrap())
            .unwrap()
            .0;
        let g2_name = e
            .catalog()
            .general_meta(e.state_core().player(2).general_slots[0].unwrap())
            .unwrap()
            .0;
        assert_ne!(g1_name, "刘备");
        assert_ne!(g2_name, "孙权");

        // 全禁空池不崩溃，置 l01_wj_empty_pool == 1
        let mut all_banned = Vec::new();
        for i in 0..e.catalog().general_count() as u16 {
            if let Some((name, _, _, _, _)) = e.catalog().general_meta(i) {
                all_banned.push(name.to_string());
            }
        }
        let empty_cfg = parse_l01_cfg(
            &serde_json::json!({ "Wj": true, "wj": all_banned }).to_string(),
        );
        let mut e_empty =
            Engine::try_new_identity_with_extensions(202610, 4, &[&pack]).expect("4p empty pool");
        e_empty.debug_set_general(1, "刘备");
        apply_l01_config_to_engine(&empty_cfg, &mut e_empty);
        assert_eq!(flag(&e_empty, 0, "l01_wj_empty_pool"), 1);
    }
}

#[test]
fn l01_xinfu_limu_self_le_recover_zhenlie_xyyh_and_runtime_eqz_ecf() {
    let pack = l01_battle_pack();

    // Case A: xinfu_limu 方块牌自【乐】即回1血 + 判区有乐期间攻击范围/出杀次数无限 + 禁重乐与非方块
    {
        let mut e = Engine::new_with_extensions(202611, 4, &[&pack]).expect("limu engine");
        advance_to_play(&mut e, 0);
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        e.debug_clear_hand(2);
        e.debug_grant(0, "xinfu_limu");
        e.debug_set_hp(0, 2);

        let d_sha1 = e.debug_place_suit(0, Suit::Diamond);
        let sha_a = e.debug_place(0, "sha");
        let sha_b = e.debug_place(0, "sha");
        let s_non = e.debug_place_suit(0, Suit::Spade);
        let d_extra = e.debug_place_suit(0, Suit::Diamond);

        // 1) 非方块不可作为自乐
        assert!(matches!(
            submit(
                &mut e,
                0,
                GameAction::PlayCard {
                    card: s_non,
                    targets: vec![0]
                }
            ),
            CommandOutcome::Rejected(RejectReason::IllegalTargets)
        ));
        // 2) 方块牌对己发动【利木】-> onuse 即回 1 血、入判定区
        let limu_out = submit(
            &mut e,
            0,
            GameAction::PlayCard {
                card: d_sha1,
                targets: vec![0],
            },
        );
        assert!(
            matches!(
                limu_out,
                CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
            ),
            "limu self-le rejected: {limu_out:?}"
        );
        drain(&mut e);
        assert_eq!(e.debug_hp(0), 3, "xinfu_limu recovers 1 HP on use");
        assert_eq!(e.debug_judge(0), vec![d_sha1]);
        // 3) 判定区已有乐：拒绝再对自己放第二张方块乐
        assert!(matches!(
            submit(
                &mut e,
                0,
                GameAction::PlayCard {
                    card: d_extra,
                    targets: vec![0]
                }
            ),
            CommandOutcome::Rejected(_)
        ));
        // 4) 判定区非空：未装武器时打不到距离 2 的对角线 2 号位（Sha 与 Snatch 均应被拒）；
        // 装备距离 2 武器后，对攻击范围内的 2 号位可无限次出杀 + 远距离顺手牵羊！
        let snatch2 = e.debug_place(0, "snatch");
        assert!(matches!(
            submit(
                &mut e,
                0,
                GameAction::PlayCard {
                    card: sha_a,
                    targets: vec![2],
                }
            ),
            CommandOutcome::Rejected(RejectReason::IllegalTargets)
        ));
        assert!(matches!(
            submit(
                &mut e,
                0,
                GameAction::PlayCard {
                    card: snatch2,
                    targets: vec![2],
                }
            ),
            CommandOutcome::Rejected(RejectReason::IllegalTargets)
        ));
        let weapon2 = e.debug_place(0, "weapon2");
        assert!(matches!(
            submit(
                &mut e,
                0,
                GameAction::PlayCard {
                    card: weapon2,
                    targets: vec![],
                }
            ),
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ));
        drain(&mut e);
        let sn_ok = submit(
            &mut e,
            0,
            GameAction::PlayCard {
                card: snatch2,
                targets: vec![2],
            },
        );
        // 2 号位空手空装备时 Snatch 会 Reject，放一张牌给 2 号位测攻击范围内顺手牵羊
        let _ = sn_ok;
        let p2_card = e.debug_place(2, "shan");
        let _ = p2_card;
        let sn_in_range = submit(
            &mut e,
            0,
            GameAction::PlayCard {
                card: snatch2,
                targets: vec![2],
            },
        );
        assert!(
            matches!(sn_in_range, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
            "limu Snatch within weapon range should succeed: {sn_in_range:?}"
        );
        drain(&mut e);

        let sha1_ok = submit(
            &mut e,
            0,
            GameAction::PlayCard {
                card: sha_a,
                targets: vec![2],
            },
        );
        assert!(matches!(
            sha1_ok,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ));
        drain(&mut e);
        assert_eq!(e.debug_hp(2), e.state_core().player(2).max_hp as i32 - 1);
        let sha2_ok = submit(
            &mut e,
            0,
            GameAction::PlayCard {
                card: sha_b,
                targets: vec![2],
            },
        );
        assert!(matches!(
            sha2_ok,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ));
        drain(&mut e);
        assert_eq!(e.debug_hp(2), e.state_core().player(2).max_hp as i32 - 2);
    }

    // Case A2: xinfu_limu 谦逊阻断自乐 / 装备区方块自乐（离场拆技回1血）/ 禁止重装与打他人 / 方块以逸待劳不劫持
    {
        let mut e = Engine::new_with_extensions(2026112, 4, &[&pack]).expect("limu edge");
        advance_to_play(&mut e, 0);
        e.debug_clear_hand(0);
        e.debug_grant(0, "xinfu_limu");
        e.debug_grant(0, "谦逊");
        e.debug_set_hp(0, 2);
        let d_card = e.debug_place_suit(0, Suit::Diamond);
        assert!(matches!(
            submit(
                &mut e,
                0,
                GameAction::PlayCard {
                    card: d_card,
                    targets: vec![0]
                }
            ),
            CommandOutcome::Rejected(_)
        ));

        let mut e2 = Engine::new_with_extensions(2026113, 4, &[&pack]).expect("limu equip he");
        advance_to_play(&mut e2, 0);
        e2.debug_clear_hand(0);
        e2.debug_grant(0, "xinfu_limu");
        e2.debug_set_hp(0, 2);
        let d_eq = e2.debug_place(0, "fangtian");
        assert_eq!(e2.catalog().cards[d_eq as usize].suit, Suit::Diamond);
        let _ = submit(
            &mut e2,
            0,
            GameAction::PlayCard {
                card: d_eq,
                targets: vec![],
            },
        );
        drain(&mut e2);
        assert_eq!(e2.debug_equips(0), vec![d_eq]);
        // 不能将装备区牌空目标重装或指定其他角色为 Le
        assert!(matches!(
            submit(
                &mut e2,
                0,
                GameAction::PlayCard {
                    card: d_eq,
                    targets: vec![]
                }
            ),
            CommandOutcome::Rejected(_)
        ));
        assert!(matches!(
            submit(
                &mut e2,
                0,
                GameAction::PlayCard {
                    card: d_eq,
                    targets: vec![1]
                }
            ),
            CommandOutcome::Rejected(_)
        ));
        // 从装备区将方块装备对己作【利木】：脱离装备区 + 入判定区 + 回 1 血
        let from_eq = submit(
            &mut e2,
            0,
            GameAction::PlayCard {
                card: d_eq,
                targets: vec![0],
            },
        );
        assert!(
            matches!(
                from_eq,
                CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
            ),
            "diamond equip from equip zone as limu must succeed: {from_eq:?}"
        );
        drain(&mut e2);
        assert!(e2.debug_equips(0).is_empty(), "equip must leave equip zone");
        assert_eq!(e2.debug_judge(0), vec![d_eq]);
        assert_eq!(e2.debug_hp(0), 3);

        // 方块【以逸待劳】对 [0] 使用按以逸待劳结算，不被利木劫持（构造 Suit::Diamond 的 yiyi 包无条件断言）
        let mut d_pack = l01_battle_pack();
        let mut yiyi_found = false;
        for c in &mut d_pack.cards {
            if c.name == "yiyi" {
                c.suit = Suit::Diamond;
                yiyi_found = true;
            }
        }
        assert!(yiyi_found, "yiyi must exist in l01_battle_pack");
        let mut e3 = Engine::new_with_extensions(2026114, 4, &[&d_pack]).expect("d yiyi");
        advance_to_play(&mut e3, 0);
        e3.debug_clear_hand(0);
        e3.debug_grant(0, "xinfu_limu");
        let y_id = e3.debug_move_named_card_to_hand(0, "yiyi");
        assert_eq!(e3.catalog().cards[y_id as usize].suit, Suit::Diamond);
        let out_y = submit(
            &mut e3,
            0,
            GameAction::PlayCard {
                card: y_id,
                targets: vec![0],
            },
        );
        assert!(matches!(
            out_y,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ));
        drain(&mut e3);
        assert_eq!(flag(&e3, 0, "l01_yiyi_resolved"), 1);
        assert!(e3.debug_judge(0).is_empty());
    }

    // Case B: zhenlie_lose 不读暗身份（零泄密）、严格受 L01 开关门控，并在 L01 dying_save 下偏向救援
    {
        // B1: 未启用 L01 时，触发真烈不得写入 l01_zhenlie_lose
        let mut plain = Engine::try_new_identity_with_extensions(2026120, 4, &[&pack]).expect("plain");
        plain.debug_grant(1, "zhenlie");
        advance_to_play(&mut plain, 0);
        plain.debug_clear_hand(0);
        let sha_p = plain.debug_place(0, "sha");
        let _extra_p = plain.debug_place(0, "shan");
        let _ = submit(
            &mut plain,
            0,
            GameAction::PlayCard {
                card: sha_p,
                targets: vec![1],
            },
        );
        let c0 = plain.open_choice().expect("zhenlie trigger_confirm").clone();
        let _ = submit(
            &mut plain,
            c0.observer,
            GameAction::Answer {
                choice_id: c0.id,
                selection: vec![ChoiceOption::Confirm],
            },
        );
        assert_eq!(flag(&plain, 1, "l01_zhenlie_lose"), 0, "non-L01 game must not set l01_zhenlie_lose");
        drain(&mut plain);

        // B2: 开启 L01 后，忠诚与内奸暗置身份下 zhenlie 绝不写 l01_pub_exposure 也不泄露角色
        let mut e1 = Engine::try_new_identity_with_extensions(202612, 4, &[&pack]).expect("z1");
        let mut e2 = Engine::try_new_identity_with_extensions(202612, 4, &[&pack]).expect("z2");
        e1.debug_set_identity_roles(vec![Role::Lord, Role::Loyalist, Role::Rebel, Role::Traitor]);
        e2.debug_set_identity_roles(vec![Role::Lord, Role::Traitor, Role::Rebel, Role::Loyalist]);
        for eng in [&mut e1, &mut e2] {
            eng.debug_grant(1, "zhenlie");
            eng.debug_grant(1, "zhenlie_lose");
            advance_to_play(eng, 0);
            eng.debug_clear_hand(0);
            let _sha = eng.debug_place(0, "sha");
            let _extra = eng.debug_place(0, "shan");
            eng.debug_set_hp(1, 1);
            eng.debug_clear_hand(2);
            let _tao = eng.debug_place(2, "tao");
        }
        let s1 = e1.debug_hand(0)[0];
        let s2 = e2.debug_hand(0)[0];
        let _ = submit(&mut e1, 0, GameAction::PlayCard { card: s1, targets: vec![1] });
        let _ = submit(&mut e2, 0, GameAction::PlayCard { card: s2, targets: vec![1] });
        let a1 = e1.open_choice().unwrap().clone();
        let a2 = e2.open_choice().unwrap().clone();
        let _ = submit(&mut e1, a1.observer, GameAction::Answer { choice_id: a1.id, selection: vec![ChoiceOption::Confirm] });
        let _ = submit(&mut e2, a2.observer, GameAction::Answer { choice_id: a2.id, selection: vec![ChoiceOption::Confirm] });
        assert_eq!(flag(&e1, 1, "l01_zhenlie_lose"), 1);
        assert_eq!(flag(&e2, 1, "l01_zhenlie_lose"), 1);
        assert_eq!(flag(&e1, 0, "l01_pub_exposure:1"), 0);
        assert_eq!(flag(&e2, 0, "l01_pub_exposure:1"), 0);

        let cfg = parse_l01_cfg(r#"{"btAi":"shyg"}"#);
        let p1 = BoundedPlanner::new(64).with_l01_options(to_l01_ai_options(&cfg, 0, &e1));
        let p2 = BoundedPlanner::new(64).with_l01_options(to_l01_ai_options(&cfg, 0, &e2));
        // 受伤忠/主在有 l01_zhenlie_lose 时救主/已亮友方；且暗置忠诚 vs 内奸决策完全一致
        e1.debug_set_hp(0, 2);
        e2.debug_set_hp(0, 2);
        assert_eq!(e1.plan_action(0, &p1).action, e2.plan_action(0, &p2).action);

        // B3: 受伤忠诚(seat 2, hp=2) 面对濒死主公(seat 0)：有 l01_zhenlie_lose 时救，无则留桃自保
        let mut e_save = Engine::try_new_identity_with_extensions(2026129, 4, &[&pack]).expect("save");
        e_save.debug_set_identity_roles(vec![Role::Lord, Role::Rebel, Role::Loyalist, Role::Traitor]);
        e_save.debug_grant(0, "zhenlie");
        e_save.debug_grant(0, "zhenlie_lose");
        advance_to_play(&mut e_save, 1);
        e_save.debug_clear_hand(0);
        e_save.debug_clear_hand(1);
        e_save.debug_clear_hand(2);
        e_save.debug_clear_hand(3);
        let sha_to_lord = e_save.debug_place(1, "sha");
        let _extra_r = e_save.debug_place(1, "shan");
        let tao_ally = e_save.debug_place(2, "tao");
        e_save.debug_set_hp(0, 1);
        e_save.debug_set_hp(2, 2);
        let _ = submit(
            &mut e_save,
            1,
            GameAction::PlayCard {
                card: sha_to_lord,
                targets: vec![0],
            },
        );
        for _ in 0..16 {
            let Some(c) = e_save.open_choice().cloned() else {
                break;
            };
            if c.observer == 2 && c.spec.prompt == "dying_save" {
                break;
            }
            let sel = if c.observer == 0 && c.spec.prompt == "trigger_confirm" {
                vec![ChoiceOption::Confirm]
            } else if c.spec.options.contains(&ChoiceOption::Pass) {
                vec![ChoiceOption::Pass]
            } else if c.spec.allow_cancel {
                Vec::new()
            } else {
                c.spec.options.iter().take(c.spec.min.max(1)).copied().collect()
            };
            let _ = submit(
                &mut e_save,
                c.observer,
                GameAction::Answer {
                    choice_id: c.id,
                    selection: sel,
                },
            );
        }
        let ask_ally = e_save.open_choice().expect("seat 2 dying_save").clone();
        assert_eq!(ask_ally.observer, 2);
        assert_eq!(ask_ally.spec.prompt, "dying_save");

        let mut opts_yes = to_l01_ai_options(&cfg, 2, &e_save);
        opts_yes.zhenlie_lose_seats = vec![0];
        let plan_with_zl = e_save
            .plan_action(2, &BoundedPlanner::new(64).with_l01_options(opts_yes))
            .action;
        assert_eq!(
            plan_with_zl,
            GameAction::Answer {
                choice_id: ask_ally.id,
                selection: vec![ChoiceOption::Card(tao_ally)],
            },
            "damaged ally should save dying zhenlie_lose lord"
        );
        e_save
            .state_core_mut()
            .vm_aux
            .flags
            .remove(&(0, "l01_zhenlie_lose".to_string()));
        let opts_no = to_l01_ai_options(&cfg, 2, &e_save);
        let plan_no_zl = e_save
            .plan_action(2, &BoundedPlanner::new(64).with_l01_options(opts_no))
            .action;
        assert_eq!(
            plan_no_zl,
            GameAction::Answer {
                choice_id: ask_ally.id,
                selection: vec![ChoiceOption::Pass],
            },
            "without zhenlie_lose damaged ally preserves Tao at hp<=2"
        );
        drain(&mut e_save);
    }

    // Case C: xyyh (`th_chongxu` 跳过小游戏直接结算档位) + _aiyh_fixEQz / _aiyh_fixECf 读档续答
    {
        let mut e = Engine::new_with_extensions(202613, 4, &[&pack]).expect("xyyh+rt");
        e.debug_set_general(1, "刘备");
        e.debug_grant(0, "th_chongxu");
        e.debug_grant(0, "_aiyh_fixEQz");
        e.debug_grant(0, "_aiyh_fixECf");
        advance_to_play(&mut e, 0);

        // xyyh 开启前：l01_xyyh_chongxu_score 保持 0
        let _ = activate_skill(&mut e, 0, "th_chongxu");
        drain(&mut e);
        assert_eq!(flag(&e, 0, "l01_xyyh_chongxu_score"), 0);

        // 开启 xyyh 后再次在后续回合发动：l01_xyyh_chongxu_score 写入 1..=5
        e.state_core_mut()
            .vm_aux
            .flags
            .insert((0, "l01_cfg:xyyh".to_string()), 1);
        advance_to_play(&mut e, 1);
        advance_to_play(&mut e, 0);
        let _ = activate_skill(&mut e, 0, "th_chongxu");
        let score = flag(&e, 0, "l01_xyyh_chongxu_score");
        assert!((1..=5).contains(&score), "xyyh must assign tier 1..=5: {score}");
        drain(&mut e);

        // 局内权重与嘲讽微调 (_aiyh_fixEQz / _aiyh_fixECf) + 挂起态读档恢复
        let cfg = parse_l01_cfg(r#"{"fixEQz":true,"fixECf":true}"#);
        apply_l01_config_to_engine(&cfg, &mut e);
        let _ = activate_skill(&mut e, 0, "_aiyh_fixEQz");
        let qz_target_ask = e.open_choice().expect("fix qz target").clone();
        assert_eq!(qz_target_ask.spec.prompt, "l01_fix_qz_target");
        let _ = submit(
            &mut e,
            0,
            GameAction::Answer {
                choice_id: qz_target_ask.id,
                selection: vec![ChoiceOption::Player(1)],
            },
        );
        let qz_delta_ask = e.open_choice().expect("fix qz delta").clone();
        assert_eq!(qz_delta_ask.spec.prompt, "l01_fix_qz_delta");
        let mut r_qz = snapshot_roundtrip(&e, &[&pack]);
        let ans_qz = GameAction::Answer {
            choice_id: qz_delta_ask.id,
            selection: vec![ChoiceOption::Tag(1)], // +50
        };
        let _ = submit(&mut e, 0, ans_qz.clone());
        let _ = submit(&mut r_qz, 0, ans_qz);
        assert_eq!(e.state_hash(), r_qz.state_hash());
        let liubei_key = format!(
            "l01_rt_qz_pct:{}",
            e.catalog()
                .general_meta(e.state_core().player(1).general_slots[0].unwrap())
                .unwrap()
                .0
        );
        assert_eq!(e.state_core().vm_aux.storage.get(&liubei_key).copied(), Some(150));

        let _ = activate_skill(&mut e, 0, "_aiyh_fixECf");
        let cf_sk_ask = e.open_choice().expect("fix cf skill").clone();
        assert_eq!(cf_sk_ask.spec.prompt, "l01_fix_cf_skill");
        let first_sk = cf_sk_ask.spec.options[0];
        let ChoiceOption::Skill(sk_id) = first_sk else {
            panic!("expected ChoiceOption::Skill");
        };
        let sk_name = e.catalog().skills[sk_id as usize].name;
        let _ = submit(
            &mut e,
            0,
            GameAction::Answer {
                choice_id: cf_sk_ask.id,
                selection: vec![first_sk],
            },
        );
        let cf_delta_ask = e.open_choice().expect("fix cf delta").clone();
        assert_eq!(cf_delta_ask.spec.prompt, "l01_fix_cf_delta");
        let mut r_cf = snapshot_roundtrip(&e, &[&pack]);
        let ans_cf = GameAction::Answer {
            choice_id: cf_delta_ask.id,
            selection: vec![ChoiceOption::Tag(0)], // +10
        };
        let _ = submit(&mut e, 0, ans_cf.clone());
        let _ = submit(&mut r_cf, 0, ans_cf);
        assert_eq!(e.state_hash(), r_cf.state_hash());
        assert_eq!(
            e.state_core()
                .vm_aux
                .storage
                .get(&format!("l01_rt_cf_pct:{sk_name}"))
                .copied(),
            Some(110)
        );
    }
}

#[test]
fn l01_attitude_tracker_planner_strategies_and_zero_hidden_leak() {
    let pack = l01_battle_pack();

    // 1. _attzhi + att* 公开事件追踪（AOE 排除、打主/救主修正与全反阵亡残局标）
    {
        let mut e = Engine::try_new_identity_with_extensions(202614, 5, &[&pack]).expect("5p att");
        e.debug_set_identity_roles(vec![
            Role::Lord,
            Role::Loyalist,
            Role::Rebel,
            Role::Rebel,
            Role::Traitor,
        ]);
        e.debug_grant(0, "_attzhi");
        e.debug_grant(0, "attcard");
        e.debug_grant(0, "attdamage");
        e.debug_grant(0, "atttiaofan");
        e.debug_grant(0, "_yingshilanggu");
        e.debug_grant(0, "_aiyh_firstKs");
        advance_to_play(&mut e, 2);
        e.debug_clear_hand(0);
        e.debug_clear_hand(2);

        // 南蛮入侵(AOE) 不应触发打主/跳反分
        let nm = e.debug_place(2, "savage");
        let _ = submit(
            &mut e,
            2,
            GameAction::PlayCard {
                card: nm,
                targets: vec![],
            },
        );
        drain(&mut e);
        assert_eq!(flag(&e, 0, "l01_pub_att:2"), 0);
        assert_eq!(flag(&e, 0, "l01_pub_tiaofan:2"), 0);

        // 对主公(seat 0)出杀并造成伤害 -> 公开态度转负，跳反分>0
        e.debug_clear_hand(0);
        let sha = e.debug_place(2, "sha");
        e.debug_grant(2, "paoxiao");
        let eq = e.debug_place(2, "weapon3");
        let _ = submit(
            &mut e,
            2,
            GameAction::PlayCard {
                card: eq,
                targets: vec![],
            },
        );
        drain(&mut e);
        let _ = submit(
            &mut e,
            2,
            GameAction::PlayCard {
                card: sha,
                targets: vec![0],
            },
        );
        drain(&mut e);
        let att2 = i64::from(flag(&e, 0, "l01_pub_att:2")) - 1_000_000;
        let tf2 = i64::from(flag(&e, 0, "l01_pub_tiaofan:2")) - 1_000_000;
        assert!(att2 < 0, "attacking Lord drops public attitude: {att2}");
        assert!(tf2 > 0, "attacking Lord increases tiaofan score: {tf2}");

        // seat 1(忠) 在主公濒死时用【桃】相救 -> 公开态度转正 (+200)
        e.debug_clear_hand(0);
        e.debug_set_hp(0, 1);
        e.debug_clear_hand(1);
        let tao = e.debug_place(1, "tao");
        let sha_dying = e.debug_place(2, "sha");
        let out_dying = submit(
            &mut e,
            2,
            GameAction::PlayCard {
                card: sha_dying,
                targets: vec![0],
            },
        );
        assert!(
            matches!(out_dying, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
            "sha_dying rejected: {out_dying:?}"
        );
        for step in 0..64 {
            let Some(c) = e.open_choice().cloned() else {
                break;
            };
            let sel = if c.observer == 1 && c.spec.prompt == "dying_save" {
                vec![ChoiceOption::Card(tao)]
            } else if c.spec.options.contains(&ChoiceOption::Pass) {
                vec![ChoiceOption::Pass]
            } else if c.spec.allow_cancel {
                Vec::new()
            } else {
                c.spec.options.iter().take(c.spec.min.max(1)).copied().collect()
            };
            let ans = submit(
                &mut e,
                c.observer,
                GameAction::Answer {
                    choice_id: c.id,
                    selection: sel,
                },
            );
            assert!(
                matches!(ans, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
                "dying_save step {step} rejected: {ans:?}"
            );
            if step == 63 {
                panic!("dying_save loop exceeded 64 steps");
            }
        }
        let att1 = i64::from(flag(&e, 0, "l01_pub_att:1")) - 1_000_000;
        assert!(att1 > 0, "saving Lord raises public attitude: {att1}");

        // 两名反贼均死亡 -> l01_endgame_no_fan == 1（纯公开阵亡人数统计，绝不偷看存活座位）
        assert_eq!(flag(&e, 0, "l01_endgame_no_fan"), 0);
        advance_to_play(&mut e, 0);
        e.debug_grant(0, "paoxiao");
        let w0 = e.debug_place(0, "weapon3");
        let out_w0 = submit(
            &mut e,
            0,
            GameAction::PlayCard {
                card: w0,
                targets: vec![],
            },
        );
        assert!(
            matches!(out_w0, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
            "equip weapon3 rejected: {out_w0:?}"
        );
        drain(&mut e);
        for fan in [2u8, 3u8] {
            e.debug_clear_hand(fan);
            e.debug_set_hp(fan, 1);
            let s = e.debug_place(0, "sha");
            let out_kill = submit(
                &mut e,
                0,
                GameAction::PlayCard {
                    card: s,
                    targets: vec![fan],
                },
            );
            assert!(
                matches!(out_kill, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
                "kill fan {fan} rejected: {out_kill:?}"
            );
            drain(&mut e);
            assert!(!e.debug_alive(fan), "fan {fan} should die");
        }
        assert_eq!(flag(&e, 0, "l01_endgame_no_fan"), 1);
    }

    // 2. applyCf / chaofeng 公式与 BoundedPlanner nhFriends / ntAoe / hsyh 拼点决策
    {
        assert!((apply_cf_rating_threat(GeneralRarity::Rare, 2, false) - 1.2f64.sqrt()).abs() < 1e-6);
        assert!(
            (apply_cf_rating_threat(GeneralRarity::Legend, 2, true) - 0.8 * 2.4f64.sqrt()).abs()
                < 1e-6
        );
        assert_eq!(chaofeng_threat(GeneralRarity::Epic), 3.0);
        assert_eq!(chaofeng_threat(GeneralRarity::Legend), 4.0);

        let mut e = Engine::try_new_identity_with_extensions(202615, 4, &[&pack]).expect("plan");
        e.debug_set_identity_roles(vec![Role::Lord, Role::Loyalist, Role::Rebel, Role::Traitor]);
        // 明置忠臣与反贼后测 nhFriends 与 ntAoe
        e.state_core_mut()
            .vm_aux
            .flags
            .insert((1, "l01_role_shown".to_string()), 1);
        e.state_core_mut()
            .vm_aux
            .flags
            .insert((2, "l01_role_shown".to_string()), 1);
        e.debug_set_hp(1, 1);
        advance_to_play(&mut e, 0);
        e.debug_clear_hand(0);
        let nm = e.debug_place(0, "savage");
        let _ = nm;
        let opts_nt = L01AiOptions {
            bt_ai: L01StrategyMode::Shyg,
            sfj_ai: L01StrategyMode::Shyg,
            nt_aoe: true,
            nh_friends: NhFriendsMode::MaxHp(2),
            ..Default::default()
        };
        let p_nt = BoundedPlanner::new(64).with_l01_options(opts_nt);
        assert!(!matches!(
            e.plan_action(0, &p_nt).action,
            GameAction::PlayCard { .. }
        ));
    }

    // 3. 严格零隐藏信息泄密不变式：未明置对手角色 + 暗手牌替换 -> plan_action 比特一致！
    {
        let mut e1 = Engine::try_new_identity_with_extensions(202616, 5, &[&pack]).expect("e1");
        let mut e2 = Engine::try_new_identity_with_extensions(202616, 5, &[&pack]).expect("e2");
        // 完全不同的未明置对手身份排列
        e1.debug_set_identity_roles(vec![
            Role::Lord,
            Role::Loyalist,
            Role::Rebel,
            Role::Rebel,
            Role::Traitor,
        ]);
        e2.debug_set_identity_roles(vec![
            Role::Lord,
            Role::Rebel,
            Role::Traitor,
            Role::Loyalist,
            Role::Rebel,
        ]);
        for eng in [&mut e1, &mut e2] {
            advance_to_play(eng, 0);
            for p in 0..5 {
                eng.debug_clear_hand(p);
            }
        }
        // 给 observer(seat 0) 完全相同的手牌
        let sha_1 = e1.debug_place(0, "sha");
        let nm_1 = e1.debug_place(0, "savage");
        let sha_2 = e2.debug_place(0, "sha");
        let nm_2 = e2.debug_place(0, "savage");
        assert_eq!(sha_1, sha_2);
        assert_eq!(nm_1, nm_2);

        // 给未明置座位 1/2/3/4 不同的暗手牌（但保持手牌总数相同）
        let c1 = e1.debug_place(1, "shan");
        let _ = e1.debug_place(2, "tao");
        let _ = e2.debug_place(1, "tao");
        let _ = e2.debug_place(2, "jiu");
        let _ = c1;

        let cfg = parse_l01_cfg(
            r#"{"btAi":"shyg","sfjAi":"shyg","mjAi":true,"ntAoe":true,"nhFriends":"hp","findZhong":false}"#,
        );
        let planner1 = BoundedPlanner::new(64)
            .with_policy_variant(2)
            .with_l01_options(to_l01_ai_options(&cfg, 0, &e1));
        let planner2 = BoundedPlanner::new(64)
            .with_policy_variant(2)
            .with_l01_options(to_l01_ai_options(&cfg, 0, &e2));

        let d1 = e1.plan_action(0, &planner1);
        let d2 = e2.plan_action(0, &planner2);
        assert_eq!(e1.view_for(0), e2.view_for(0), "redacted view must ignore hidden roles and hands");
        assert_eq!(d1.action, d2.action, "zero hidden info leak on action choice");
        assert_eq!(d1.nodes_used, d2.nodes_used, "zero hidden info leak on search nodes");
    }
}
