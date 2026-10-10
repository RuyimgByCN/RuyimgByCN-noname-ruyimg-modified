//! Golden coverage for the 2026-08-21 shahai re-documentation batch upgrades:
//! 图腾 tuteng1 / 长念 changnian2 / 疾风 ljifeng（部分）/ 虚惧魔体改 xjumo.
//! Zero new ABI (shahai 645). All four falsified the old "跨界包专属机制" noop claim.

const TUTENG1: u32 = 3_411_870_870;
const CHANGNIAN2: u32 = 2_771_404_252;
const LJIFENG: u32 = 3_765_349_055;
const XJUMO: u32 = 7_974_172_16;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/shahai/shahai.npack.json");

#[test]
fn shahai_publishes_four_rewrites() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    // tuteng1 / changnian2：结束阶段触发（回复 / 摸牌）。
    assert!(package
        .trigger_index
        .iter()
        .any(|t| t.skill_def_id == TUTENG1 && t.event == "phase_end"));
    assert!(package
        .trigger_index
        .iter()
        .any(|t| t.skill_def_id == CHANGNIAN2 && t.event == "phase_end"));
    // ljifeng：距离 = 已损体力（负值源）。
    assert!(package.modifier_index.iter().any(|m| {
        m.skill_def_id == LJIFENG
            && m.target == "distance_from"
            && m.mode == "add"
            && m.value_source.as_deref() == Some("owner_lost_hp")
    }));
    // xjumo：手牌上限 +3，受伤时再 +1+1。
    assert_eq!(
        package
            .modifier_index
            .iter()
            .filter(|m| m.skill_def_id == XJUMO && m.target == "hand_limit")
            .count(),
        3
    );
    assert!(package.modifier_index.iter().any(|m| {
        m.skill_def_id == XJUMO
            && m.mode == "add"
            && m.value == 3
            && m.value_source.is_none()
    }));
}

const MINGKONG: u32 = 2_432_512_632;
const TUJI: u32 = 3_008_175_639;

#[test]
fn shahai_publishes_mingkong_tuji() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    // mingkong：damage_before 强制触发（空手 → 减伤 + 来源摸牌）。
    assert!(package
        .trigger_index
        .iter()
        .any(|t| t.skill_def_id == MINGKONG && t.event == "damage_before"));
    // tuji：距离减标记 + card_used（当前回合门控）累积 + turn_ended 清零，共两触发。
    assert_eq!(
        package
            .trigger_index
            .iter()
            .filter(|t| t.skill_def_id == TUJI)
            .count(),
        2
    );
    assert!(package.modifier_index.iter().any(|m| {
        m.skill_def_id == TUJI
            && m.target == "distance_from"
            && m.value_source.as_deref() == Some("owner_skill_mark")
    }));
}

const GTIANDAO: u32 = 3_863_243_589;
const SUIYAN: u32 = 1_400_546_24;
const GUANHU: u32 = 3_609_190_708;

#[test]
fn shahai_publishes_gtiandao_suiyan_guanhu() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    // gtiandao：判定改写 modifier。
    assert!(package.modifier_index.iter().any(|m| {
        m.skill_def_id == GTIANDAO
            && m.target == "judgment_rewrite"
            && m.mode == "override"
            && m.value == 1
    }));
    // suiyan：伤害来源触发，自弃 1 + 弃对方全部装备。
    assert!(package
        .trigger_index
        .iter()
        .any(|t| t.skill_def_id == SUIYAN && t.event == "damage" && t.optional));
    // guanhu：杀伤害来源触发，弃对方一张 hand_equip。
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == GUANHU && t.event == "damage" && t.filter == "owner_source_sha"
    }));
}

const HONGXI: u32 = 73_507_340;
const MEIHUO: u32 = 2_953_576_202;

#[test]
fn shahai_publishes_hongxi_meihuo() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    // hongxi：任意死亡触发回满（forced）。
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == HONGXI && t.event == "death" && !t.optional
    }));
    // meihuo：ADR 0118 最后装备失牌过滤器 + 获得目标手牌。
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == MEIHUO
            && t.event == "cards_moved"
            && t.filter == "owner_alive_event_player_from_equip_empty"
    }));
}

const PINGXU: u32 = 1_203_805_137;
const JUFU: u32 = 1_151_788_467;
const HUJIA_E3: u32 = 886_050_816;
const HUJIA_E4: u32 = 581_753_271;
const SHIJIAN_E3: u32 = 3_366_005_863;
const SHIJIAN_E4: u32 = 4_143_751_073;

#[test]
fn shahai_publishes_equip_presence_family() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let expect = [
        (PINGXU, "distance_from", "owner_no_weapon", -1),
        (PINGXU, "distance_to", "owner_armor_empty", 1),
        (JUFU, "sha_damage", "owner_has_weapon", 1),
        (HUJIA_E3, "distance_to", "owner_has_armor_equip", 1),
        (HUJIA_E4, "distance_from", "owner_has_armor_equip", -1),
        (SHIJIAN_E3, "distance_to", "owner_equip_count_exactly_one", 1),
        (SHIJIAN_E4, "distance_from", "owner_equip_count_exactly_one", -1),
    ];
    for (def, target, source, value) in expect {
        assert!(
            package.modifier_index.iter().any(|m| {
                m.skill_def_id == def
                    && m.target == target
                    && m.value_source.as_deref() == Some(source)
                    && m.value == value
            }),
            "missing modifier {def} {target} {source}"
        );
    }
}

const HUIMENG: u32 = 1_217_004_629;
const SHALI: u32 = 4_145_386_093;
const HLONGYI: u32 = 3_379_329_600;
const SHAMOLVZHOU: u32 = 2_281_691_180;

#[test]
fn shahai_publishes_huimeng_shali_longyi_lvzhou() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == HUIMENG && t.event == "hp_recovered" && t.optional
    }));
    // shali：forced 回复触发 + 表达式护甲增量。
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == SHALI && t.event == "hp_recovered" && !t.optional
    }));
    for (def, selector) in [(HLONGYI, "black"), (SHAMOLVZHOU, "basic")] {
        assert!(
            package.modifier_index.iter().any(|m| {
                m.skill_def_id == def
                    && m.target == "card_ignored_hand_limit"
                    && m.selector.as_deref() == Some(selector)
            }),
            "missing ignored-hand-limit {selector}"
        );
    }
}

const SHOULIE: u32 = 4_057_970_364;
const LIANWU: u32 = 3_419_558_155;
const XIANDAN: u32 = 3_140_108_402;

#[test]
fn shahai_publishes_sha_enhancement_trio() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    // shoulie / xiandan：card_used owner_source_sha 可选触发。
    for def in [SHOULIE, XIANDAN] {
        assert!(package.trigger_index.iter().any(|t| {
            t.skill_def_id == def
                && t.event == "card_used"
                && t.filter == "owner_source_sha"
                && t.optional
        }));
    }
    // lianwu：杀目标上限 +1（常量合法）。
    assert!(package.modifier_index.iter().any(|m| {
        m.skill_def_id == LIANWU
            && m.target == "card_target_limit"
            && m.selector.as_deref() == Some("sha")
            && m.value == 1
    }));
}

const XINGTI_E1: u32 = 1_593_267_513;
const JINGCU: u32 = 2_664_334_110;
const YUEDONG: u32 = 222_078_864;
const JIAOXIA: u32 = 2_916_450_912;

#[test]
fn shahai_publishes_draw_family_four() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == XINGTI_E1 && t.event == "damage" && !t.optional
    }));
    assert!(package
        .active_index
        .iter()
        .any(|a| a.skill_def_id == JINGCU));
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == YUEDONG && t.event == "phase_play_end" && t.optional
    }));
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == JIAOXIA && t.event == "card_targeted" && t.optional
    }));
}

const MISHA: u32 = 3_620_290_780;
const BAIZHAN: u32 = 2_809_408_131;
const DALIAN: u32 = 3_942_481_021;
const TUTENG3: u32 = 764_613_100;
const XINGJUNYAN: u32 = 474_768_951;

#[test]
fn shahai_publishes_damage_family_five() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    for (def, event, filter) in [
        (MISHA, "damage", "owner_is_target"),
        (BAIZHAN, "damage", "owner_is_source"),
        (DALIAN, "damage", "owner_is_source"),
        (TUTENG3, "damage_before", "owner_is_target"),
    ] {
        assert!(
            package.trigger_index.iter().any(|t| {
                t.skill_def_id == def && t.event == event && t.filter == filter && !t.optional
            }),
            "missing trigger {def} {event}"
        );
    }
    for target in ["sha_damage", "sha_damage_taken"] {
        assert!(package.modifier_index.iter().any(|m| {
            m.skill_def_id == XINGJUNYAN && m.target == target && m.value == 1
        }));
    }
}

const TUTENG7: u32 = 1_954_578_579;
const LINGYU: u32 = 1_062_385_707;
const LINGXUE: u32 = 1_748_742_044;
const RONGHUO: u32 = 2_669_063_930;
const MOYAO: u32 = 2_399_830_529;
const GJQTWUXIE: u32 = 2_052_473_555;
const SWD_WUXIE: u32 = 3_872_592_194;

#[test]
fn shahai_publishes_misc_family_seven() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    for (def, optional) in [(TUTENG7, false), (LINGYU, true)] {
        assert!(package.trigger_index.iter().any(|t| {
            t.skill_def_id == def && t.event == "phase_end" && t.optional == optional
        }));
    }
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == LINGXUE && t.event == "hp_recovered" && !t.optional
    }));
    let expect = [
        (RONGHUO, "card_nature", Some("sha")),
        (MOYAO, "card_target_cancel_from_current", Some("ordinary_trick")),
        (GJQTWUXIE, "card_target_cancel_from_current", Some("delayed_trick")),
        (SWD_WUXIE, "card_target_cancel_from_current", Some("delayed_trick")),
    ];
    for (def, target, selector) in expect {
        assert!(
            package.modifier_index.iter().any(|m| {
                m.skill_def_id == def
                    && m.target == target
                    && m.selector.as_deref() == selector
            }),
            "missing modifier {def} {target} {selector:?}"
        );
    }
}

const YULIN: u32 = 3_588_027_126;
const HUJIA_E2B: u32 = 595_417_402;

#[test]
fn shahai_publishes_equip_defense_pair() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == YULIN && t.event == "damage_before" && t.optional
    }));
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == HUJIA_E2B && t.event == "damage" && t.optional
    }));
}

const XIXUE: u32 = 537_295_572;
const NINGBIN: u32 = 3_373_215_889;
const PANGUFU: u32 = 3_774_868_748;
const SHENGZHANG: u32 = 3_984_247_144;

#[test]
fn shahai_publishes_mid_bucket_four() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    for (def, event, filter) in [
        (XIXUE, "damage", "owner_source_sha"),
        (NINGBIN, "damage", "owner_target_thunder"),
        (PANGUFU, "damage", "owner_is_source"),
        (SHENGZHANG, "phase_discard_end", "owner_is_target"),
    ] {
        assert!(
            package.trigger_index.iter().any(|t| {
                t.skill_def_id == def && t.event == event && t.filter == filter && !t.optional
            }),
            "missing trigger {def} {event} {filter}"
        );
    }
}

#[test]
fn shahai_yuchan_family_consumes_657() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    // 玉蝉族 40 块全部成为 active 消费者。
    let yuchan: Vec<_> = package
        .skill_index
        .iter()
        .filter(|s| s.name.starts_with("yuchan") && s.name.contains("_equip"))
        .collect();
    assert_eq!(yuchan.len(), 40);
    for skill in &yuchan {
        assert!(
            package.active_index.iter().any(|a| a.skill_def_id == skill.skill_def_id),
            "yuchan {} missing active",
            skill.name
        );
    }
}

#[test]
fn shahai_relic_families_twenty_blocks() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    // 四神器族（每族 5 块）全部成为 phase_end 触发者。
    for prefix in [
        "huanglinzhicong",
        "xuanwuzhihuang",
        "zhuquezhizhang",
        "baishouzhihu",
    ] {
        let defs: Vec<_> = package
            .skill_index
            .iter()
            .filter(|s| s.name.starts_with(prefix) && s.name.contains("_equip"))
            .collect();
        assert_eq!(defs.len(), 5, "{prefix} family size");
        for skill in &defs {
            assert!(
                package.trigger_index.iter().any(|t| {
                    t.skill_def_id == skill.skill_def_id
                        && t.event == "phase_end"
                        && t.filter == "owner_is_target"
                        && t.optional
                }),
                "{} missing phase_end trigger",
                skill.name
            );
        }
    }
}

const KUIXIN: u32 = 3_692_941_629;
const FENGXIAN: u32 = 2_340_239_594;

#[test]
fn shahai_publishes_kuixin_fengxian() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == KUIXIN && t.event == "phase_end" && t.optional
    }));
    assert!(package
        .active_index
        .iter()
        .any(|a| a.skill_def_id == FENGXIAN));
}

const DAOFA: u32 = 1_397_328_699;
const CHENGJIAN: u32 = 4_267_414_717;
const XUEZHOU: u32 = 3_579_460_904;

#[test]
fn shahai_bystander_filter_unlocks_trio() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    for def in [DAOFA, CHENGJIAN, XUEZHOU] {
        assert!(
            package.trigger_index.iter().any(|t| {
                t.skill_def_id == def
                    && t.event == "damage"
                    && t.filter == "owner_alive_other_source_damage"
                    && t.optional
            }),
            "bystander trigger missing for {def}"
        );
    }
}

const BAIHUPIFENG: u32 = 1_598_904_184;
const FUYAN: u32 = 2_278_207_906;
const XINCI: u32 = 3_129_827_268;
const SHENGGUANG: u32 = 3_007_270_760;
const BINXIN: u32 = 4_230_503_518;

#[test]
fn shahai_publishes_heal_armor_five() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    for (def, kind) in [
        (BAIHUPIFENG, "phase_end"),
        (FUYAN, "damage"),
        (BINXIN, "phase_end"),
    ] {
        assert!(package
            .trigger_index
            .iter()
            .any(|t| t.skill_def_id == def && t.event == kind && t.optional));
    }
    for def in [XINCI, SHENGGUANG] {
        assert!(package.active_index.iter().any(|a| a.skill_def_id == def));
    }
}

const SHENNONGDING: u32 = 1_430_535_693;
const ZHIYUAN: u32 = 1_656_864_566;
const SHIJIE: u32 = 2_405_574_206;

#[test]
fn shahai_publishes_shennong_zhiyuan_shijie() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    assert!(package
        .active_index
        .iter()
        .any(|a| a.skill_def_id == SHENNONGDING));
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == ZHIYUAN && t.event == "damage_before" && t.optional
    }));
    assert!(package
        .trigger_index
        .iter()
        .any(|t| t.skill_def_id == SHIJIE && t.event == "phase_end" && t.optional));
}

const HUDUN: u32 = 2_311_509_903;
const GUIYUAN: u32 = 742_627_015;
const MOJIAN: u32 = 2_944_384_606;

#[test]
fn shahai_publishes_hudun_guiyuan_mojian() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == HUDUN && t.event == "damage" && !t.optional
    }));
    assert!(package
        .active_index
        .iter()
        .any(|a| a.skill_def_id == GUIYUAN));
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == MOJIAN && t.event == "card_targeted" && t.optional
    }));
}

const DIESHA: u32 = 4_212_657_781;
const FENGMING: u32 = 627_509_838;
const FANSHI: u32 = 4_191_706_529;

#[test]
fn shahai_publishes_diesha_fengming_fanshi() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == DIESHA && t.event == "damage" && t.filter == "owner_source_sha"
    }));
    assert!(package
        .active_index
        .iter()
        .any(|a| a.skill_def_id == FENGMING));
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == FANSHI && t.event == "phase_discard_end" && !t.optional
    }));
}

const XUEREN: u32 = 2_696_735_475;
const TXIANQU: u32 = 218_812_647;
const YALIANG: u32 = 2_107_321_786;

#[test]
fn shahai_publishes_xueren_txianqu_yaliang() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    // xueren：杀伤害 → 目标+自己各失去 1 体力，然后摸两张。
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == XUEREN
            && t.event == "damage"
            && t.filter == "owner_source_sha"
            && t.optional
    }));
    // txianqu：伤害前可选防伤（出牌阶段限一次，storage 门控）。
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == TXIANQU && t.event == "damage_before" && t.optional
    }));
    // yaliang：造成或受到伤害双方各摸一张（双 trigger）。
    let yaliang_triggers: Vec<_> = package
        .trigger_index
        .iter()
        .filter(|t| t.skill_def_id == YALIANG)
        .collect();
    assert_eq!(yaliang_triggers.len(), 2);
    assert!(yaliang_triggers.iter().any(|t| t.filter == "owner_is_source"));
    assert!(yaliang_triggers.iter().any(|t| t.filter == "owner_is_target"));
}

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const META_SRC: &str = include_str!("../../../content/original/ported/shahai/shahai_meta.skill");

fn meta_bootstrap(skill_def: u32) -> (Transaction, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named("shahai-meta-test", META_SRC).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, u64::from(skill_def), "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    let owner = tx.state().turn.current;
    let other = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(skill_def),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, other)
}

fn meta_resolve(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let (state, pipe) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections,
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    state
}

#[test]
fn xueren_sha_damage_loses_both_then_draws_two() {
    let (tx, owner, other) = meta_bootstrap(XUEREN);
    let owner_hp = tx.state().players.get(owner).unwrap().hp;
    let other_hp = tx.state().players.get(other).unwrap().hp;
    let owner_hand = tx.state().hand_count(owner);
    let mut tx = tx;
    // optional 触发 → 先确认，再结算。
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealShaDamage {
                source: owner,
                target: other,
                card: engine_api::CardId::NONE,
                amount: 1,
                nature: rule_api::damage_nature::NORMAL,
                card_flags: 0,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = meta_resolve(tx.commit().state, owner, vec![1], 2);
    assert_eq!(state.players.get(other).unwrap().hp, other_hp - 2);
    assert_eq!(state.players.get(owner).unwrap().hp, owner_hp - 1);
    assert_eq!(state.hand_count(owner), owner_hand + 2);
}

#[test]
fn txianqu_can_prevent_damage_once_per_turn_inside_own_play_phase() {
    let (tx, owner, other) = meta_bootstrap(TXIANQU);
    let mut tx = tx;
    tx.state_mut().turn.phase = engine_core::state::Phase::Play;
    tx.state_mut().turn.current = owner;
    // 先受伤，令回复可结算。
    let owner_max = tx.state().players.get(owner).unwrap().max_hp;
    tx.state_mut().players.get_mut(owner).unwrap().hp = (owner_max - 1) as i32;
    let owner_hp = tx.state().players.get(owner).unwrap().hp;
    let other_hp = tx.state().players.get(other).unwrap().hp;
    let owner_hand = tx.state().hand_count(owner);
    // 出牌阶段内防止伤害 → 摸两张 + 回复 1。
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: owner,
                target: other,
                amount: 1,
                is_lose_hp: false,
                nature: 0,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = meta_resolve(tx.commit().state, owner, vec![1], 2);
    assert_eq!(state.players.get(owner).unwrap().hp, owner_hp + 1);
    assert_eq!(state.players.get(other).unwrap().hp, other_hp);
    assert_eq!(state.hand_count(owner), owner_hand + 2);
    // 第二次伤害：optional 仍会询问，但 storage 已置 1，确认后 body 门控不防伤 → 伤害照常。
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: owner,
                target: other,
                amount: 1,
                is_lose_hp: false,
                nature: 0,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = meta_resolve(tx.commit().state, owner, vec![1], 3);
    assert_eq!(state.players.get(other).unwrap().hp, other_hp - 1);
    assert_eq!(state.players.get(owner).unwrap().hp, owner_hp + 1);
}

#[test]
fn yaliang_source_and_target_each_draw_one() {
    let (tx, owner, other) = meta_bootstrap(YALIANG);
    let owner_hand = tx.state().hand_count(owner);
    let other_hand = tx.state().hand_count(other);
    let mut tx = tx;
    // 造成伤害方向：双方各摸一张。
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: owner,
                target: other,
                amount: 1,
                is_lose_hp: false,
                nature: 0,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = meta_resolve(tx.commit().state, owner, vec![1], 2);
    assert_eq!(state.hand_count(owner), owner_hand + 1);
    assert_eq!(state.hand_count(other), other_hand + 1);
}


const YOUGENG: u32 = 2_130_169_647;
const LIYUTANG: u32 = 1_246_457_632;
const MIESHI: u32 = 2_286_963_619;

#[test]
fn shahai_publishes_yougeng_liyutang_mieshi() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == YOUGENG
            && t.event == "phase_prepare"
            && t.filter == "owner_min_hp"
            && !t.optional
    }));
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == LIYUTANG
            && t.event == "phase_end"
            && t.filter == "owner_min_hp"
            && !t.optional
    }));
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == MIESHI && t.event == "phase_end" && !t.optional
    }));
}

fn dispatch_phase(tx: &mut Transaction, current: PlayerId, phase: engine_api::TurnPhaseTag) {
    // DSL `phase_end` 事件来自 TurnStateChanged{phase:End}（EventKind::PhaseEnd）；
    // PhaseEnded{End} 映射的是 turn_ended。
    let effects = engine_core::trigger::dispatch_for_events(
        tx,
        &[DomainEvent::TurnStateChanged { current, phase }],
        &mut engine_core::trigger::TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(tx, effects).unwrap();
}

fn hujia_mark(state: &engine_core::state::GameState, player: PlayerId) -> i32 {
    state
        .players
        .get(player)
        .unwrap()
        .marks
        .iter()
        .find(|(def, _)| *def == 3)
        .map_or(0, |(_, n)| *n)
}

#[test]
fn yougeng_prep_recover_when_min_hp_and_damaged() {
    let (tx, owner, other) = meta_bootstrap(YOUGENG);
    let mut tx = tx;
    // owner 打伤至全场最少 → 准备阶段回复 1。
    let wounded = tx.state().players.get(owner).unwrap().hp - 1;
    tx.state_mut().players.get_mut(owner).unwrap().hp = wounded;
    assert!(wounded < tx.state().players.get(other).unwrap().hp);
    dispatch_phase(&mut tx, owner, engine_api::TurnPhaseTag::Prepare);
    assert_eq!(tx.state().players.get(owner).unwrap().hp, wounded + 1);

    // 满血（体力不小于他人）→ 过滤器不通过，无回复。
    let (tx, owner, other) = meta_bootstrap(YOUGENG);
    let mut tx = tx;
    let owner_hp = tx.state().players.get(owner).unwrap().hp;
    let other_hp = tx.state().players.get(other).unwrap().hp;
    if owner_hp <= other_hp {
        // 平体力也过过滤器，但 body hp<max 守卫挡回复 → 断言体力不变。
        dispatch_phase(&mut tx, owner, engine_api::TurnPhaseTag::Prepare);
        assert_eq!(tx.state().players.get(owner).unwrap().hp, owner_hp);
    } else {
        // 非最少体力 → 过滤器直接不触发。
        dispatch_phase(&mut tx, owner, engine_api::TurnPhaseTag::Prepare);
        assert_eq!(tx.state().players.get(owner).unwrap().hp, owner_hp);
    }
}

#[test]
fn liyutang_end_phase_grants_armor_when_min_hp() {
    let (tx, owner, other) = meta_bootstrap(LIYUTANG);
    let mut tx = tx;
    // 显式令 owner 为全场最少体力（身份/体力上限随 seed 波动，不能依赖初值）。
    tx.state_mut().players.get_mut(owner).unwrap().hp = 1;
    if tx.state().players.get(other).unwrap().hp <= 1 {
        tx.state_mut().players.get_mut(other).unwrap().hp = 2;
    }
    dispatch_phase(&mut tx, owner, engine_api::TurnPhaseTag::End);
    assert_eq!(hujia_mark(tx.state(), owner), 1);
}

#[test]
fn mieshi_end_phase_lose_one_then_damage_other() {
    let (tx, owner, other) = meta_bootstrap(MIESHI);
    let mut tx = tx;
    let owner_hp = tx.state().players.get(owner).unwrap().hp;
    let other_hp = tx.state().players.get(other).unwrap().hp;
    // forced：结束阶段失去 1，随后 choose_other_damage 开目标选择。
    let mut sched_tx = tx;
    let effects = engine_core::trigger::dispatch_for_events(
        &mut sched_tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: engine_api::TurnPhaseTag::End,
        }],
        &mut engine_core::trigger::TriggerFireRecord::new(),
    )
    .unwrap();
    assert_eq!(
        run_effects(&mut sched_tx, effects),
        Err(EffectError::ChoiceRequested)
    );
    let state = meta_resolve(sched_tx.commit().state, owner, vec![other.raw()], 4);
    assert_eq!(state.players.get(owner).unwrap().hp, owner_hp - 1);
    assert_eq!(state.players.get(other).unwrap().hp, other_hp - 1);
}

const MOLICHA: u32 = 881_591_894;
const YUANBAOROU: u32 = 515_767_027;
const TANHUADONG: u32 = 2_536_578_780;
const MAPODOUFU: u32 = 2_565_465_103;

#[test]
fn shahai_publishes_food_card_four() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    assert!(package.modifier_index.iter().any(|m| {
        m.skill_def_id == MOLICHA
            && m.target == "card_target_cancel"
            && m.selector.as_deref() == Some("black")
            && m.mode == "override"
    }));
    assert!(package.modifier_index.iter().any(|m| {
        m.skill_def_id == YUANBAOROU
            && m.target == "sha_use_limit"
            && m.mode == "add"
            && m.value == 1
    }));
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == TANHUADONG
            && t.event == "phase_play_end"
            && t.filter == "owner_is_target"
            && !t.optional
    }));
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == MAPODOUFU && t.event == "phase_end" && !t.optional
    }));
}

#[test]
fn tanhuadong_draws_one_when_play_phase_ends() {
    let (tx, owner, _other) = meta_bootstrap(TANHUADONG);
    let mut tx = tx;
    let before = tx.state().hand_count(owner);
    // 出牌阶段结束（PhaseEnded{Play} → PhasePlayEnd）→ 摸一张。
    let effects = engine_core::trigger::dispatch_for_events(
        &mut tx,
        &[DomainEvent::PhaseEnded {
            current: owner,
            phase: engine_api::TurnPhaseTag::Play,
            damage_sources: 0,
            damaged_players: 0,
            card_targeted_other: false,
            current_dealt_damage_other: false,
            phase_targeted_players: 0,
            phase_hp_changed_players: 0,
        }],
        &mut engine_core::trigger::TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(tx.state().hand_count(owner), before + 1);
}

#[test]
fn mapodoufu_end_phase_discards_one_from_chosen_other() {
    use engine_core::state::{Card, CardKind, ZoneKind};
    let (tx, owner, other) = meta_bootstrap(MAPODOUFU);
    let mut tx = tx;
    // 给 other 一张手牌供弃置。
    let hand = tx.state().zones.find(ZoneKind::Hand, other).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: engine_api::CardId::NONE,
        def: engine_api::CardDefId::new(1),
        kind: CardKind::Sha,
        suit: engine_api::Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: other,
        face_up: true,
    });
    tx.state_mut()
        .zones
        .get_mut(hand)
        .unwrap()
        .cards
        .push(card);
    let other_hand_before = tx.state().hand_count(other);
    // forced：结束阶段直接开目标选择（无 optional confirm）。
    let effects = engine_core::trigger::dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: engine_api::TurnPhaseTag::End,
        }],
        &mut engine_core::trigger::TriggerFireRecord::new(),
    )
    .unwrap();
    assert_eq!(
        run_effects(&mut tx, effects),
        Err(EffectError::ChoiceRequested)
    );
    let state = meta_resolve(tx.commit().state, owner, vec![other.raw()], 5);
    let state = meta_resolve(state, owner, vec![card.raw()], 6);
    assert_eq!(state.hand_count(other), other_hand_before - 1);
}

const XIAOJIAO: u32 = 255_870_647;
const XIAOJIAO2: u32 = 50_847_945;
const MIZHILIANOU: u32 = 3_950_084_625;
const CHUNBING: u32 = 2_163_719_187;
const GUDONGGENG: u32 = 1_840_348_640;
const QINGTUAN: u32 = 502_548_858;

#[test]
fn shahai_publishes_food_card_family_tail_six() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    assert!(package.modifier_index.iter().any(|m| {
        m.skill_def_id == XIAOJIAO && m.target == "draw_count" && m.mode == "add"
    }));
    assert!(package.modifier_index.iter().any(|m| {
        m.skill_def_id == CHUNBING && m.target == "hand_limit" && m.mode == "add" && m.value == 1
    }));
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == XIAOJIAO2
            && t.event == "phase_draw_end"
            && t.filter == "owner_is_target"
            && !t.optional
    }));
    assert!(package.view_as_index.iter().any(|v| {
        v.skill_def_id == MIZHILIANOU
            && v.as_kind == "tao"
            && v.filter == "heart"
            && v.source_zone.as_deref() == Some("hand_equip")
    }));
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == GUDONGGENG && t.event == "damage_before" && !t.optional
    }));
    assert!(package
        .trigger_index
        .iter()
        .any(|t| t.skill_def_id == QINGTUAN && t.event == "card_used"));
}

#[test]
fn gudonggeng_reduces_next_damage_then_removes_itself() {
    let (tx, owner, other) = meta_bootstrap(GUDONGGENG);
    let owner_hp = tx.state().players.get(owner).unwrap().hp;
    let mut tx = tx;
    // 1 点杀伤害被减至 0 → 防止，且技能自移除（下一次不再减）；
    // 批 45 收紧为 owner_target_sha 后以 DealShaDamage 驱动。
    run_effects(
        &mut tx,
        vec![EffectRequest::DealShaDamage {
            source: other,
            target: owner,
            card: engine_api::CardId::NONE,
            amount: 1,
            nature: rule_api::damage_nature::NORMAL,
            card_flags: 0,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, owner_hp);
    assert!(!tx
        .state()
        .skills
        .sorted_live()
        .iter()
        .any(|s| s.def.raw() == GUDONGGENG));
}

#[test]
fn qingtuan_first_sha_in_own_turn_draws_one() {
    use engine_core::state::{Card, CardKind, ZoneKind};
    let (tx, owner, _other) = meta_bootstrap(QINGTUAN);
    let mut tx = tx;
    tx.state_mut().turn.current = owner;
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let sha = tx.state_mut().cards.alloc(Card {
        id: engine_api::CardId::NONE,
        def: engine_api::CardDefId::new(1),
        kind: CardKind::Sha,
        suit: engine_api::Suit::Spade,
        rank: 8,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(sha);
    let before = tx.state().hand_count(owner);
    // 回合内首张杀 → 摸一张。
    let effects = engine_core::trigger::dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: owner,
            card: sha,
            targets: vec![],
        }],
        &mut engine_core::trigger::TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(tx.state().hand_count(owner), before + 1);
    // 同回合第二张杀 → storage 已置 1，不再摸（手牌不再 +1）。
    let sha2 = tx.state_mut().cards.alloc(Card {
        id: engine_api::CardId::NONE,
        def: engine_api::CardDefId::new(2),
        kind: CardKind::Sha,
        suit: engine_api::Suit::Heart,
        rank: 9,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(sha2);
    let before2 = tx.state().hand_count(owner);
    let effects = engine_core::trigger::dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: owner,
            card: sha2,
            targets: vec![],
        }],
        &mut engine_core::trigger::TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(tx.state().hand_count(owner), before2);
}

const GUIYANFADAO: u32 = 2_191_743_671;
const GUAILI: u32 = 191_323_984;
const HUIQI: u32 = 1_073_166_029;
const LINGZHOU: u32 = 3_174_113_722;
const ZIYU: u32 = 2_266_084_957;

#[test]
fn shahai_publishes_scatter_five() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == GUIYANFADAO
            && t.event == "damage_before"
            && t.filter == "owner_source_sha"
            && t.optional
    }));
    assert!(package.modifier_index.iter().any(|m| {
        m.skill_def_id == GUAILI && m.target == "sha_damage" && m.mode == "add" && m.value == 1
    }));
    assert!(package
        .trigger_index
        .iter()
        .any(|t| t.skill_def_id == GUAILI && t.event == "damage" && !t.optional));
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == HUIQI && t.event == "damage" && t.filter == "owner_is_target"
    }));
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == LINGZHOU
            && t.event == "card_used"
            && t.filter == "owner_source_ordinary_trick"
            && t.optional
    }));
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == ZIYU && t.event == "phase_end" && t.filter == "owner_alive"
    }));
}

#[test]
fn guiyanfadao_prevents_sha_damage_and_loses_one_instead() {
    let (tx, owner, other) = meta_bootstrap(GUIYANFADAO);
    let owner_hp = tx.state().players.get(owner).unwrap().hp;
    let other_hp = tx.state().players.get(other).unwrap().hp;
    let mut tx = tx;
    // 2 点杀伤害被防止，目标改为失去 1 点体力。
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealShaDamage {
                source: owner,
                target: other,
                card: engine_api::CardId::NONE,
                amount: 2,
                nature: rule_api::damage_nature::NORMAL,
                card_flags: 0,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = meta_resolve(tx.commit().state, owner, vec![1], 7);
    assert_eq!(state.players.get(other).unwrap().hp, other_hp - 1);
    assert_eq!(state.players.get(owner).unwrap().hp, owner_hp);
}

#[test]
fn guaili_buffs_sha_damage_then_discards_two() {
    let (tx, owner, other) = meta_bootstrap(GUAILI);
    let other_hp = tx.state().players.get(other).unwrap().hp;
    let owner_hand = tx.state().hand_count(owner);
    let mut tx = tx;
    // 杀伤害后强制弃两张（count 弃置开 Custom 选择；+1 伤害走真实 PlayCard
    // 管线的 sha_damage modifier，合成 DealShaDamage 不经过——与 xingjunyan
    // 同为 publish 断言覆盖）。
    let hand_cards: Vec<engine_api::CardId> = tx.state().hand_cards(owner).to_vec();
    assert!(hand_cards.len() >= 2);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealShaDamage {
                source: owner,
                target: other,
                card: engine_api::CardId::NONE,
                amount: 1,
                nature: rule_api::damage_nature::NORMAL,
                card_flags: 0,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let selections: Vec<u32> = hand_cards[..2].iter().map(|c| c.raw()).collect();
    let state = meta_resolve(tx.commit().state, owner, selections, 8);
    assert_eq!(state.players.get(other).unwrap().hp, other_hp - 1);
    assert_eq!(state.hand_count(owner), owner_hand - 2);
}

const NVWASHI: u32 = 544_584_365;
const SHOUWANG: u32 = 1_152_842_697;
const XIELEI: u32 = 2_500_288_383;
const YULU: u32 = 3_153_959_245;

#[test]
fn shahai_publishes_dying_and_active_four() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    for def in [NVWASHI, SHOUWANG] {
        assert!(
            package
                .trigger_index
                .iter()
                .any(|t| t.skill_def_id == def && t.event == "dying" && t.optional),
            "dying trigger missing for {def}"
        );
    }
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == XIELEI && t.event == "card_used" && t.filter == "owner_source_sha"
    }));
    assert!(package
        .active_index
        .iter()
        .any(|a| a.skill_def_id == YULU));
}

#[test]
fn shouwang_dying_recovers_and_grants_armor() {
    let (tx, owner, other) = meta_bootstrap(SHOUWANG);
    let mut tx = tx;
    // 濒死者置 1 体力（恢复可结算），合成 EnteredDying 派发。
    tx.state_mut().players.get_mut(other).unwrap().hp = 1;
    tx.state_mut().players.get_mut(other).unwrap().max_hp = 4;
    // optional 触发在 dispatch 期开 confirm（返回 ChoiceRequested）。
    assert_eq!(
        engine_core::trigger::dispatch_for_events(
            &mut tx,
            &[DomainEvent::EnteredDying {
                player: other,
                source: owner,
            }],
            &mut engine_core::trigger::TriggerFireRecord::new(),
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = meta_resolve(tx.commit().state, owner, vec![1], 9);
    assert_eq!(state.players.get(other).unwrap().hp, 2);
    assert_eq!(hujia_mark(&state, other), 1);
}

#[test]
fn nvwashi_dying_saves_other_at_cost_of_own_hp() {
    let (tx, owner, other) = meta_bootstrap(NVWASHI);
    let mut tx = tx;
    tx.state_mut().players.get_mut(other).unwrap().hp = 0;
    tx.state_mut().players.get_mut(other).unwrap().max_hp = 4;
    let owner_hp = tx.state().players.get(owner).unwrap().hp;
    assert!(owner_hp > 1);
    assert_eq!(
        engine_core::trigger::dispatch_for_events(
            &mut tx,
            &[DomainEvent::EnteredDying {
                player: other,
                source: PlayerId::NONE,
            }],
            &mut engine_core::trigger::TriggerFireRecord::new(),
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = meta_resolve(tx.commit().state, owner, vec![1], 10);
    assert_eq!(state.players.get(other).unwrap().hp, 1);
    assert_eq!(state.players.get(owner).unwrap().hp, owner_hp - 1);
}

const YUELU: u32 = 331_934_449;
const KBOLAN: u32 = 1_879_960_978;
const DIEWU: u32 = 1_880_179_712;
const XIYONG: u32 = 365_568_992;

#[test]
fn shahai_publishes_batch39_four() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == YUELU && t.event == "dying" && t.filter == "owner_alive" && t.optional
    }));
    assert!(package.modifier_index.iter().any(|m| {
        m.skill_def_id == KBOLAN && m.target == "draw_count" && m.mode == "add" && m.value == 1
    }));
    assert!(package
        .active_index
        .iter()
        .any(|a| a.skill_def_id == DIEWU));
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == DIEWU && t.event == "turn_ended" && !t.optional
    }));
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == XIYONG && t.event == "phase_end" && t.filter == "owner_is_target"
    }));
}

#[test]
fn yuelu_dying_discards_recovers_and_grants_armor() {
    let (tx, owner, other) = meta_bootstrap(YUELU);
    let mut tx = tx;
    let owner_hand = tx.state().hand_count(owner);
    tx.state_mut().players.get_mut(other).unwrap().hp = 1;
    tx.state_mut().players.get_mut(other).unwrap().max_hp = 4;
    assert!(owner_hand >= 1);
    assert_eq!(
        engine_core::trigger::dispatch_for_events(
            &mut tx,
            &[DomainEvent::EnteredDying {
                player: other,
                source: PlayerId::NONE,
            }],
            &mut engine_core::trigger::TriggerFireRecord::new(),
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = meta_resolve(tx.commit().state, owner, vec![1], 11);
    // 确认后效果链：弃 1（Custom）→ 回复 + 护甲。
    let hand_cards: Vec<u32> = state
        .hand_cards(owner)
        .iter()
        .map(|c| c.raw())
        .collect();
    let _ = hand_cards;
    // 弃牌选择可能在 resolve 后仍挂起（若手牌非空）——直接补 resolve 一步。
    if state.has_pending_choice() {
        let hand_now: Vec<u32> = state.hand_cards(owner).iter().map(|c| c.raw()).collect();
        let state2 = meta_resolve(state, owner, vec![hand_now[0]], 12);
        assert_eq!(state2.players.get(other).unwrap().hp, 2);
        assert_eq!(hujia_mark(&state2, other), 1);
        assert_eq!(state2.hand_count(owner), owner_hand - 1);
    } else {
        assert_eq!(state.players.get(other).unwrap().hp, 2);
        assert_eq!(hujia_mark(&state, other), 1);
        assert_eq!(state.hand_count(owner), owner_hand - 1);
    }
}

const HUAJIAN: u32 = 1_052_762_827;
const GUOZAI: u32 = 1_907_094_152;
const GUOZAIX: u32 = 457_163_821;

#[test]
fn shahai_publishes_batch40_three() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == HUAJIAN
            && t.event == "phase_play_end"
            && t.filter == "owner_is_target"
            && t.optional
    }));
    for def in [GUOZAI, GUOZAIX] {
        assert!(package.active_index.iter().any(|a| a.skill_def_id == def));
        assert!(
            package
                .trigger_index
                .iter()
                .any(|t| t.skill_def_id == def && t.event == "phase_play_end" && !t.optional),
            "guozai debt trigger missing for {def}"
        );
    }
}

#[test]
fn guozai_draws_to_four_then_discards_equal_at_phase_end() {
    let (tx, owner, _other) = meta_bootstrap(GUOZAI);
    let mut tx = tx;
    tx.state_mut().turn.phase = engine_core::state::Phase::Play;
    tx.state_mut().turn.current = owner;
    // 弃至 1 张手牌（走真实弃置效果 + 选择屏障，保持牌区一致）。
    let hand_n = tx.state().hand_count(owner) as usize;
    assert!(hand_n >= 2);
    let drop = (hand_n - 1) as u32;
    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardFromHandEquip {
            player: owner,
            count: drop,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), 1);
    // active：补至 4（摸 3、欠账 3）。
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .iter()
        .find(|s| s.def.raw() == GUOZAI)
        .unwrap()
        .id;
    let revision = tx.state().revision;
    let (state, output) = engine_core::command::apply_command(
        tx.commit().state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(13),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![],
                card_targets: vec![],
            },
        },
    );
    assert!(
        matches!(output.result, CommandResult::Accepted { .. }),
        "{:?}",
        output.result
    );
    assert_eq!(state.hand_count(owner), 4);
    // 阶段结束：弃等量（3）——forced + count 弃置开选择。
    let mut tx = Transaction::begin(state);
    let effects = engine_core::trigger::dispatch_for_events(
        &mut tx,
        &[DomainEvent::PhaseEnded {
            current: owner,
            phase: engine_api::TurnPhaseTag::Play,
            damage_sources: 0,
            damaged_players: 0,
            card_targeted_other: false,
            current_dealt_damage_other: false,
            phase_targeted_players: 0,
            phase_hp_changed_players: 0,
        }],
        &mut engine_core::trigger::TriggerFireRecord::new(),
    )
    .unwrap();
    // discard_hand_equip_expr（表达式张数）为确定性弃前 N，无选择屏障。
    run_effects(&mut tx, effects).unwrap();
    let state = tx.commit().state;
    assert_eq!(state.hand_count(owner), 1);
}

const LIANZHAN: u32 = 1_293_119_497;
const LIECHAO: u32 = 4_005_223_353;
const LINYUN: u32 = 1_113_781_034;
const NUHOU: u32 = 1_984_511_198;

#[test]
fn shahai_publishes_batch41_four() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == LIANZHAN
            && t.event == "damage"
            && t.filter == "owner_is_source"
            && !t.optional
    }));
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == LIANZHAN && t.event == "turn_ended" && !t.optional
    }));
    assert!(package
        .active_index
        .iter()
        .any(|a| a.skill_def_id == LIECHAO));
    assert!(package.view_as_index.iter().any(|v| {
        v.skill_def_id == LINYUN
            && v.as_kind == "sha"
            && v.source_zone.as_deref() == Some("hand_equip")
    }));
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == NUHOU && t.event == "damage" && t.filter == "owner_is_target"
    }));
}

#[test]
fn lianzhan_first_damage_draws_then_subsequent_gains_max_hp() {
    let (tx, owner, other) = meta_bootstrap(LIANZHAN);
    let owner_max = tx.state().players.get(owner).unwrap().max_hp;
    let owner_hp = tx.state().players.get(owner).unwrap().hp;
    let owner_hand = tx.state().hand_count(owner);
    let mut tx = tx;
    // 首次造成伤害 → 摸两张（forced 无确认）。
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target: other,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), owner_hand + 2);
    assert_eq!(tx.state().players.get(owner).unwrap().max_hp, owner_max);
    // 第二次 → 体力上限+1 并回复 1。
    let wounded = tx.state().players.get(owner).unwrap().hp;
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target: other,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert_eq!(
        tx.state().players.get(owner).unwrap().max_hp,
        owner_max + 1
    );
    assert_eq!(tx.state().hand_count(owner), owner_hand + 2);
    let _ = (owner_hp, wounded);
}

#[test]
fn liechao_draws_four_turns_over_and_skips_discard() {
    let (tx, owner, _other) = meta_bootstrap(LIECHAO);
    let mut tx = tx;
    tx.state_mut().turn.phase = engine_core::state::Phase::Play;
    tx.state_mut().turn.current = owner;
    // 手牌缩至体力值以下（确定性弃置，无屏障）。
    let hand_n = tx.state().hand_count(owner);
    let hp_now = tx.state().players.get(owner).unwrap().hp;
    let drop = hand_n.saturating_sub(hp_now.max(1) as u32);
    if drop > 0 {
        run_effects(
            &mut tx,
            vec![EffectRequest::DiscardFromHandEquip {
                player: owner,
                count: drop,
            }],
        )
        .unwrap();
    }
    let before = tx.state().hand_count(owner);
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .iter()
        .find(|s| s.def.raw() == LIECHAO)
        .unwrap()
        .id;
    let revision = tx.state().revision;
    let (state, output) = engine_core::command::apply_command(
        tx.commit().state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(15),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![],
                card_targets: vec![],
            },
        },
    );
    assert!(
        matches!(output.result, CommandResult::Accepted { .. }),
        "{:?}",
        output.result
    );
    assert_eq!(state.hand_count(owner), before + 4);
    assert!(state.players.get(owner).unwrap().turned_over);
    assert!(state.turn.skip_discard(), "discard phase should be skipped");
}

const QINGLIU: u32 = 1_608_096_360;
const CHILONGYA: u32 = 349_089_161;
const DIYCHANYUAN: u32 = 3_691_619_122;
const SHENGDUN: u32 = 4_050_427_322;
const LINGYUE: u32 = 4_100_105_804;

#[test]
fn shahai_publishes_batch42_five() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    assert!(package.modifier_index.iter().any(|m| {
        m.skill_def_id == QINGLIU
            && m.target == "damage_taken_cancel"
            && m.selector.as_deref() == Some("fire")
            && m.mode == "override"
    }));
    assert!(package.modifier_index.iter().any(|m| {
        m.skill_def_id == CHILONGYA
            && m.target == "damage_amount"
            && m.selector.as_deref() == Some("fire")
            && m.mode == "add"
    }));
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == DIYCHANYUAN
            && t.event == "death"
            && t.filter == "owner_is_target_source_alive"
            && !t.optional
    }));
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == SHENGDUN && t.event == "phase_prepare" && !t.optional
    }));
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == LINGYUE
            && t.event == "card_targeted"
            && t.filter == "owner_source_sha"
            && t.optional
    }));
}

#[test]
fn shengdun_prep_phase_grants_armor_only_when_absent() {
    let (tx, owner, _other) = meta_bootstrap(SHENGDUN);
    let mut tx = tx;
    dispatch_phase(&mut tx, owner, engine_api::TurnPhaseTag::Prepare);
    assert_eq!(hujia_mark(tx.state(), owner), 1);
    // 已有护甲 → 不再加。
    dispatch_phase(&mut tx, owner, engine_api::TurnPhaseTag::Prepare);
    assert_eq!(hujia_mark(tx.state(), owner), 1);
}

#[test]
fn diychanyuan_killer_loses_max_hp_on_death() {
    let (tx, owner, other) = meta_bootstrap(DIYCHANYUAN);
    let other_max = tx.state().players.get(other).unwrap().max_hp;
    let mut tx = tx;
    let effects = engine_core::trigger::dispatch_for_events(
        &mut tx,
        &[DomainEvent::PlayerKilled {
            player: owner,
            killer: other,
            hand_equip_cards: vec![],
        }],
        &mut engine_core::trigger::TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(tx.state().players.get(other).unwrap().max_hp, other_max - 1);
}

#[test]
fn qingliu_prevents_fire_damage() {
    let (tx, owner, other) = meta_bootstrap(QINGLIU);
    let owner_hp = tx.state().players.get(owner).unwrap().hp;
    let mut tx = tx;
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: other,
            target: owner,
            amount: 2,
            is_lose_hp: false,
            nature: rule_api::damage_nature::FIRE,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, owner_hp);
}

const WANJUN: u32 = 3_255_588_776;
const XFANGHUA: u32 = 2_817_461_852;
const CANGCHIZHIBI_E5: u32 = 2_870_821_660;

#[test]
fn shahai_publishes_batch43_three() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    assert!(package.view_as_index.iter().any(|v| {
        v.skill_def_id == WANJUN
            && v.as_kind == "nanman"
            && v.filter == "card_equip"
            && v.source_zone.as_deref() == Some("hand_equip")
    }));
    // 批 46 收紧为红色限定过滤器。
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == XFANGHUA
            && t.event == "card_targeted"
            && t.filter == "owner_alive_targeted_by_other_red_card"
            && t.optional
    }));
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == CANGCHIZHIBI_E5
            && t.event == "phase_end"
            && t.filter == "owner_alive"
            && t.optional
    }));
}

#[test]
fn xfanghua_recovers_once_when_targeted_while_wounded() {
    use engine_core::state::{Card, CardKind, ZoneKind};
    let (tx, owner, other) = meta_bootstrap(XFANGHUA);
    let mut tx = tx;
    // 先受伤（满血 recover 是 no-op）。
    let wounded = tx.state().players.get(owner).unwrap().hp - 1;
    tx.state_mut().players.get_mut(owner).unwrap().hp = wounded;
    // 其他角色用牌指定 owner → 回复 1（optional confirm 先行）。
    let hand = tx.state().zones.find(ZoneKind::Hand, other).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: engine_api::CardId::NONE,
        def: engine_api::CardDefId::new(1),
        kind: CardKind::TrickWuzhong,
        suit: engine_api::Suit::Heart,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: other,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    // optional 触发在 dispatch 期开 confirm（返回 ChoiceRequested）。
    assert_eq!(
        engine_core::trigger::dispatch_for_events(
            &mut tx,
            &[DomainEvent::CardTargetAdded {
                player: other,
                card,
                target: owner,
                targets: vec![owner],
            }],
            &mut engine_core::trigger::TriggerFireRecord::new(),
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = meta_resolve(tx.commit().state, owner, vec![1], 16);
    assert_eq!(state.players.get(owner).unwrap().hp, wounded + 1);
}

const JJLD_E1: u32 = 1_421_340_753;
const JJLD_E2: u32 = 125_620_555;
const XUANYAN: u32 = 1_784_935_203;
const REXUE: u32 = 308_026_7505;

#[test]
fn shahai_publishes_batch44_four() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == JJLD_E1
            && t.event == "damage"
            && t.filter == "owner_source_sha"
            && t.optional
    }));
    // 新过滤器 owner_target_sha 首消费者。
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == JJLD_E2
            && t.event == "damage"
            && t.filter == "owner_target_sha"
            && t.optional
    }));
    assert!(package.modifier_index.iter().any(|m| {
        m.skill_def_id == XUANYAN
            && m.target == "damage_amount"
            && m.selector.as_deref() == Some("fire")
            && m.mode == "add"
    }));
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == XUANYAN && t.event == "damage" && t.filter == "owner_source_fire"
    }));
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == REXUE && t.event == "phase_prepare" && t.optional
    }));
}

#[test]
fn owner_target_sha_equip2_source_draws_two_and_turns_over() {
    let (tx, owner, other) = meta_bootstrap(JJLD_E2);
    let other_hand = tx.state().hand_count(other);
    let mut tx = tx;
    // owner 受到杀伤害 → 伤害来源（other）摸 2 并翻面。
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealShaDamage {
                source: other,
                target: owner,
                card: engine_api::CardId::NONE,
                amount: 1,
                nature: rule_api::damage_nature::NORMAL,
                card_flags: 0,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = meta_resolve(tx.commit().state, owner, vec![1], 17);
    assert_eq!(state.hand_count(other), other_hand + 2);
    assert!(state.players.get(other).unwrap().turned_over);
}

#[test]
fn xuanyan_fire_damage_buffs_then_owner_loses_one() {
    let (tx, owner, other) = meta_bootstrap(XUANYAN);
    let owner_hp = tx.state().players.get(owner).unwrap().hp;
    let other_hp = tx.state().players.get(other).unwrap().hp;
    let mut tx = tx;
    // 1 点火焰伤害：modifier +1 → 2 点，随后自己失去 1。
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target: other,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::FIRE,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(other).unwrap().hp, other_hp - 2);
    assert_eq!(tx.state().players.get(owner).unwrap().hp, owner_hp - 1);
}

const XTGZ_E2: u32 = 503_950_057;

#[test]
fn shahai_publishes_batch45_target_sha_backfill() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    // xingtigaizao_equip2：受击杀伤害摸 1（forced，新过滤器第二消费者）。
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == XTGZ_E2
            && t.event == "damage"
            && t.filter == "owner_target_sha"
            && !t.optional
    }));
    // gudonggeng：damage_before 收紧为 owner_target_sha（杀限定）。
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == GUDONGGENG
            && t.event == "damage_before"
            && t.filter == "owner_target_sha"
            && !t.optional
    }));
}

#[test]
fn xingtigaizao_equip2_draws_on_sha_damage_taken() {
    let (tx, owner, other) = meta_bootstrap(XTGZ_E2);
    let owner_hand = tx.state().hand_count(owner);
    let mut tx = tx;
    // 杀伤害 → 摸 1（forced）。
    run_effects(
        &mut tx,
        vec![EffectRequest::DealShaDamage {
            source: other,
            target: owner,
            card: engine_api::CardId::NONE,
            amount: 1,
            nature: rule_api::damage_nature::NORMAL,
            card_flags: 0,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), owner_hand + 1);
    // 非杀伤害 → 不摸。
    let before = tx.state().hand_count(owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: other,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), before);
}

#[test]
fn gudonggeng_now_only_reduces_sha_damage() {
    let (tx, owner, other) = meta_bootstrap(GUDONGGENG);
    let owner_hp = tx.state().players.get(owner).unwrap().hp;
    let mut tx = tx;
    // 非杀伤害：不减、技能保留。
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: other,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, owner_hp - 1);
    assert!(tx
        .state()
        .skills
        .sorted_live()
        .iter()
        .any(|s| s.def.raw() == GUDONGGENG));
    // 杀伤害：减至 0 + 自移除。
    let hp_now = tx.state().players.get(owner).unwrap().hp;
    run_effects(
        &mut tx,
        vec![EffectRequest::DealShaDamage {
            source: other,
            target: owner,
            card: engine_api::CardId::NONE,
            amount: 1,
            nature: rule_api::damage_nature::NORMAL,
            card_flags: 0,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp_now);
    assert!(!tx
        .state()
        .skills
        .sorted_live()
        .iter()
        .any(|s| s.def.raw() == GUDONGGENG));
}

const NINGSHUANG: u32 = 1_602_703_733;
const DIY_JIAOXIA: u32 = 2_916_450_912;

#[test]
fn shahai_publishes_batch46_red_targeted() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    // 新过滤器 owner_alive_targeted_by_other_red_card 两消费者收紧。
    for def in [DIY_JIAOXIA, XFANGHUA] {
        assert!(
            package.trigger_index.iter().any(|t| {
                t.skill_def_id == def
                    && t.event == "card_targeted"
                    && t.filter == "owner_alive_targeted_by_other_red_card"
            }),
            "red-targeted tighten missing for {def}"
        );
    }
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == NINGSHUANG
            && t.event == "card_targeted"
            && t.filter == "owner_alive_targeted_by_other_black_card"
            && t.optional
    }));
}

#[test]
fn xfanghua_red_targeted_only_recovers_on_red_card() {
    use engine_core::state::{Card, CardKind, ZoneKind};
    let (tx, owner, other) = meta_bootstrap(XFANGHUA);
    let mut tx = tx;
    let wounded = tx.state().players.get(owner).unwrap().hp - 1;
    tx.state_mut().players.get_mut(owner).unwrap().hp = wounded;
    let hand = tx.state().zones.find(ZoneKind::Hand, other).unwrap();
    // 黑桃（黑色）锦囊指定 → 收紧后不触发（dispatch 直接 Ok 空）。
    let black = tx.state_mut().cards.alloc(Card {
        id: engine_api::CardId::NONE,
        def: engine_api::CardDefId::new(1),
        kind: CardKind::TrickWuzhong,
        suit: engine_api::Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: other,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(black);
    let effects = engine_core::trigger::dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardTargetAdded {
            player: other,
            card: black,
            target: owner,
            targets: vec![owner],
        }],
        &mut engine_core::trigger::TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(effects.is_empty());
    assert_eq!(tx.state().players.get(owner).unwrap().hp, wounded);
    // 红桃（红色）指定 → 触发回复。
    let red = tx.state_mut().cards.alloc(Card {
        id: engine_api::CardId::NONE,
        def: engine_api::CardDefId::new(2),
        kind: CardKind::TrickWuzhong,
        suit: engine_api::Suit::Heart,
        rank: 8,
        weapon_range: 0,
        zone: hand,
        owner: other,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(red);
    assert_eq!(
        engine_core::trigger::dispatch_for_events(
            &mut tx,
            &[DomainEvent::CardTargetAdded {
                player: other,
                card: red,
                target: owner,
                targets: vec![owner],
            }],
            &mut engine_core::trigger::TriggerFireRecord::new(),
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = meta_resolve(tx.commit().state, owner, vec![1], 18);
    assert_eq!(state.players.get(owner).unwrap().hp, wounded + 1);
}

const SONG: u32 = 1_830_964_935;
const YA: u32 = 1_475_890_447;
const FENG: u32 = 953_027_553;

#[test]
fn shahai_publishes_counter_family_three() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == SONG
            && t.event == "damage_before"
            && t.filter == "owner_is_source"
            && !t.optional
    }));
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == YA && t.event == "damage_before" && t.filter == "owner_is_target"
    }));
    assert!(package.modifier_index.iter().any(|m| {
        m.skill_def_id == FENG && m.target == "draw_count" && m.mode == "add"
    }));
    assert!(package
        .trigger_index
        .iter()
        .any(|t| t.skill_def_id == FENG && t.event == "cards_drawn"));
}

#[test]
fn song_third_damage_gets_plus_one_then_counter_resets() {
    let (tx, owner, other) = meta_bootstrap(SONG);
    let mut tx = tx;
    // 拉高体力避免濒死窗口：前两次 1 伤（计数），第三次 +1（2 伤），
    // 第四次重新计数（1 伤）→ 合计 5。
    tx.state_mut().players.get_mut(other).unwrap().max_hp = 10;
    tx.state_mut().players.get_mut(other).unwrap().hp = 10;
    for _ in 0..4 {
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: owner,
                target: other,
                amount: 1,
                is_lose_hp: false,
                nature: 0,
            }],
        )
        .unwrap();
    }
    assert_eq!(tx.state().players.get(other).unwrap().hp, 5);
}

#[test]
fn ya_third_damage_taken_reduced_by_one() {
    let (tx, owner, other) = meta_bootstrap(YA);
    let mut tx = tx;
    // 拉高体力避免濒死求桃窗口：前两次 1 伤（计数），第三次 -1（防住），
    // 第四次 1 伤 → 合计 3。
    tx.state_mut().players.get_mut(owner).unwrap().max_hp = 10;
    tx.state_mut().players.get_mut(owner).unwrap().hp = 10;
    for _ in 0..4 {
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: other,
                target: owner,
                amount: 1,
                is_lose_hp: false,
                nature: 0,
            }],
        )
        .unwrap();
    }
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 7);
}

const SHENGTENG: u32 = 3_509_977_353;
const HUAJING: u32 = 3_376_859_850;
const BINGSHUANG: u32 = 2_991_924_429;

#[test]
fn shahai_publishes_trick_damage_three() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == SHENGTENG
            && t.event == "damage"
            && t.filter == "owner_source_trick"
            && !t.optional
    }));
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == HUAJING
            && t.event == "damage"
            && t.filter == "owner_source_trick"
            && t.optional
    }));
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == BINGSHUANG
            && t.event == "damage"
            && t.filter == "owner_source_trick"
            && t.optional
    }));
}

#[test]
fn shengteng_trick_damage_gains_max_hp_and_recovers() {
    use engine_core::state::{Card, CardKind, ZoneKind};
    let (tx, owner, other) = meta_bootstrap(SHENGTENG);
    let mut tx = tx;
    tx.state_mut().players.get_mut(owner).unwrap().max_hp = 10;
    tx.state_mut().players.get_mut(owner).unwrap().hp = 8;
    tx.state_mut().players.get_mut(other).unwrap().max_hp = 10;
    tx.state_mut().players.get_mut(other).unwrap().hp = 10;
    // 锦囊伤害（DealCardDamage + TrickWuzhong）→ 上限+1 回复1（forced）。
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let trick = tx.state_mut().cards.alloc(Card {
        id: engine_api::CardId::NONE,
        def: engine_api::CardDefId::new(1),
        kind: CardKind::TrickWuzhong,
        suit: engine_api::Suit::Club,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(trick);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealCardDamage {
            source: owner,
            target: other,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
            card: trick,
            card_kind: CardKind::TrickWuzhong.tag(),
            card_flags: 0,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().max_hp, 11);
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 9);
}

#[test]
fn bingshuang_trick_damage_target_draws_two_and_turns_over() {
    use engine_core::state::{Card, CardKind, ZoneKind};
    let (tx, owner, other) = meta_bootstrap(BINGSHUANG);
    let mut tx = tx;
    tx.state_mut().players.get_mut(other).unwrap().max_hp = 10;
    tx.state_mut().players.get_mut(other).unwrap().hp = 10;
    let other_hand = tx.state().hand_count(other);
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let trick = tx.state_mut().cards.alloc(Card {
        id: engine_api::CardId::NONE,
        def: engine_api::CardDefId::new(2),
        kind: CardKind::TrickJuedou,
        suit: engine_api::Suit::Spade,
        rank: 9,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(trick);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealCardDamage {
                source: owner,
                target: other,
                amount: 1,
                is_lose_hp: false,
                nature: rule_api::damage_nature::NORMAL,
                card: trick,
                card_kind: CardKind::TrickJuedou.tag(),
                card_flags: 0,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = meta_resolve(tx.commit().state, owner, vec![1], 19);
    assert_eq!(state.hand_count(other), other_hand + 2);
    assert!(state.players.get(other).unwrap().turned_over);
}

const ZUIZHAN: u32 = 1_706_576_549;
const SHENQIANG: u32 = 1_521_132_567;
const ZHONGJI: u32 = 2_057_472_469;
const HUAJIN: u32 = 3_382_516_484;

#[test]
fn shahai_publishes_batch49_four() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == ZUIZHAN
            && t.event == "card_used"
            && t.filter == "owner_source_sha"
            && t.optional
    }));
    assert!(package.modifier_index.iter().any(|m| {
        m.skill_def_id == SHENQIANG
            && m.target == "sha_use_limit"
            && m.mode == "add"
    }));
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == ZHONGJI
            && t.event == "damage_before"
            && t.filter == "owner_is_source"
            && t.optional
    }));
    assert!(package.modifier_index.iter().any(|m| {
        m.skill_def_id == HUAJIN && m.target == "damage_amount" && m.mode == "add"
    }));
    assert!(package
        .active_index
        .iter()
        .any(|a| a.skill_def_id == HUAJIN));
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == HUAJIN && t.event == "phase_prepare" && !t.optional
    }));
}

#[test]
fn zhongji_discards_one_to_boost_damage() {
    let (tx, owner, other) = meta_bootstrap(ZHONGJI);
    let mut tx = tx;
    tx.state_mut().players.get_mut(other).unwrap().max_hp = 10;
    tx.state_mut().players.get_mut(other).unwrap().hp = 10;
    let owner_hand = tx.state().hand_count(owner);
    assert!(owner_hand >= 1);
    // optional → 弃 1（Custom 选择）→ 伤害 1→2。
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: owner,
                target: other,
                amount: 1,
                is_lose_hp: false,
                nature: 0,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state0 = tx.commit().state;
    // confirm [1] → 弃牌选择（若无屏障则直接结算）。
    let state1 = meta_resolve(state0, owner, vec![1], 20);
    if state1.has_pending_choice() {
        let first_card = state1.hand_cards(owner)[0];
        let state2 = meta_resolve(state1, owner, vec![first_card.raw()], 21);
        assert_eq!(state2.hand_count(owner), owner_hand - 1);
        assert_eq!(state2.players.get(other).unwrap().hp, 8);
    } else {
        assert_eq!(state1.hand_count(owner), owner_hand - 1);
        assert_eq!(state1.players.get(other).unwrap().hp, 8);
    }
}

const TUTENG4: u32 = 4_224_257_102;
const TUTENG6: u32 = 339_193_492;
const HUANXIA: u32 = 3_593_561_905;

#[test]
fn shahai_publishes_batch50_three() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == TUTENG4
            && t.event == "damage_before"
            && t.filter == "owner_source_trick"
            && !t.optional
    }));
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == TUTENG6
            && t.event == "damage_before"
            && t.filter == "owner_source_sha"
            && !t.optional
    }));
    assert!(package.view_as_index.iter().any(|v| {
        v.skill_def_id == HUANXIA
            && v.as_kind == "sha"
            && v.filter == "red"
            && v.source_zone.as_deref() == Some("hand_equip")
    }));
}

#[test]
fn tuteng6_first_sha_damage_in_own_turn_gets_plus_one() {
    let (tx, owner, other) = meta_bootstrap(TUTENG6);
    let mut tx = tx;
    tx.state_mut().turn.current = owner;
    tx.state_mut().players.get_mut(other).unwrap().max_hp = 10;
    tx.state_mut().players.get_mut(other).unwrap().hp = 10;
    // 回合内首杀 +1（1→2），第二杀不再加（1）。
    run_effects(
        &mut tx,
        vec![EffectRequest::DealShaDamage {
            source: owner,
            target: other,
            card: engine_api::CardId::NONE,
            amount: 1,
            nature: rule_api::damage_nature::NORMAL,
            card_flags: 0,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(other).unwrap().hp, 8);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealShaDamage {
            source: owner,
            target: other,
            card: engine_api::CardId::NONE,
            amount: 1,
            nature: rule_api::damage_nature::NORMAL,
            card_flags: 0,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(other).unwrap().hp, 7);
}

const SHENGYAN: u32 = 2_414_447_102;
const XIONGZI: u32 = 2_036_505_599;
const SHUIMOMUXUE: u32 = 1_015_005_133;

#[test]
fn shahai_publishes_batch51_three() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == SHENGYAN && t.event == "hp_recovered" && t.optional
    }));
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == SHENGYAN && t.event == "turn_ended" && !t.optional
    }));
    let xiongzi_mods: Vec<_> = package
        .modifier_index
        .iter()
        .filter(|m| m.skill_def_id == XIONGZI && m.target == "draw_count" && m.mode == "add")
        .collect();
    assert_eq!(xiongzi_mods.len(), 2);
    assert!(xiongzi_mods.iter().any(|m| m.value == 1));
    assert!(xiongzi_mods
        .iter()
        .any(|m| m.value_source.as_deref() == Some("owner_equip_count_half_floor")));
    assert!(package.modifier_index.iter().any(|m| {
        m.skill_def_id == SHUIMOMUXUE && m.target == "hand_limit" && m.mode == "add" && m.value == -1
    }));
}

#[test]
fn shengyan_extra_recovery_once_per_turn() {
    let (tx, owner, other) = meta_bootstrap(SHENGYAN);
    let mut tx = tx;
    // other 受伤未满血 → 回复事件触发追加 +1。
    tx.state_mut().players.get_mut(other).unwrap().max_hp = 10;
    tx.state_mut().players.get_mut(other).unwrap().hp = 8;
    // optional 在 dispatch 期开 confirm。
    assert_eq!(
        engine_core::trigger::dispatch_for_events(
            &mut tx,
            &[DomainEvent::HpRecovered {
                player: other,
                source: PlayerId::NONE,
                amount: 1,
            }],
            &mut engine_core::trigger::TriggerFireRecord::new(),
        ),
        Err(EffectError::ChoiceRequested)
    );
    // 合成 HpRecovered 事件本身不改 hp（只跑触发链）→ other 8 + 追加 1 = 9。
    let state = meta_resolve(tx.commit().state, owner, vec![1], 22);
    assert_eq!(state.players.get(other).unwrap().hp, 9);
    // 排空首次 resolve 后可能残留的选择（cancelled=true）。
    let mut state = state;
    let mut key = 30;
    while state.has_pending_choice() {
        let (s2, pipe) = engine_core::command::apply_command(
            state.clone(),
            Command {
                actor: owner,
                expected_revision: ExpectedRevision(state.revision),
                idempotency_key: IdempotencyKey(key),
                kind: CommandKind::ResolveChoice {
                    choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                    selections: vec![],
                    cancelled: true,
                },
            },
        );
        assert!(matches!(pipe.result, CommandResult::Accepted { .. }), "{:?}", pipe.result);
        state = s2;
        key += 1;
    }
    // 同回合第二次：optional 仍开 confirm（body 门控不挡询问），
    // 确认后 storage 已置 1 → 不追加。
    let mut tx = Transaction::begin(state);
    tx.state_mut().players.get_mut(other).unwrap().hp = 8;
    assert_eq!(
        engine_core::trigger::dispatch_for_events(
            &mut tx,
            &[DomainEvent::HpRecovered {
                player: other,
                source: PlayerId::NONE,
                amount: 1,
            }],
            &mut engine_core::trigger::TriggerFireRecord::new(),
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state2 = meta_resolve(tx.commit().state, owner, vec![1], 23);
    assert_eq!(state2.players.get(other).unwrap().hp, 8);
}

const QIANGGU: u32 = 696_157_082;
const MIEDAO: u32 = 1_243_932_178;

#[test]
fn shahai_publishes_batch52_two_partials() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    assert!(package
        .active_index
        .iter()
        .any(|a| a.skill_def_id == QIANGGU));
    assert!(package.modifier_index.iter().any(|m| {
        m.skill_def_id == MIEDAO
            && m.target == "draw_count"
            && m.mode == "add"
            && m.value_source.as_deref() == Some("owner_lost_hp")
    }));
}

#[test]
fn qianggu_discards_two_and_gains_two_armor() {
    let (tx, owner, _other) = meta_bootstrap(QIANGGU);
    let mut tx = tx;
    tx.state_mut().turn.phase = engine_core::state::Phase::Play;
    tx.state_mut().turn.current = owner;
    let hand_n = tx.state().hand_count(owner) as usize;
    assert!(hand_n >= 2);
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .iter()
        .find(|s| s.def.raw() == QIANGGU)
        .unwrap()
        .id;
    let revision = tx.state().revision;
    let (state, output) = engine_core::command::apply_command(
        tx.commit().state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(31),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![],
                card_targets: vec![],
            },
        },
    );
    assert!(
        matches!(output.result, CommandResult::Accepted { .. }),
        "{:?}",
        output.result
    );
    // count 弃置开 Custom 选择（selections 精确 2 张）。
    let mut state = state;
    if state.has_pending_choice() {
        let cards: Vec<u32> = state.hand_cards(owner).iter().map(|c| c.raw()).collect();
        state = meta_resolve(state, owner, cards[..2].to_vec(), 32);
    }
    assert_eq!(state.hand_count(owner) as usize, hand_n - 2);
    assert_eq!(hujia_mark(&state, owner), 2);
}

const RW_LANYINJIA: u32 = 2_437_994_609;
const RW_MINGUANGKAI: u32 = 3_033_064_688;
const MOBILE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

#[test]
fn mobile_publishes_rw_equip_pair() {
    let package = content_schema::ContentPackage::from_json_bytes(MOBILE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 734);
    // "hand" 为默认源区，序列化时省略（None）。
    assert!(package.view_as_index.iter().any(|v| {
        v.skill_def_id == RW_LANYINJIA
            && v.as_kind == "shan"
            && v.source_zone.as_deref().is_none_or(|z| z == "hand")
    }));
    for selector in ["sha_fire", "huogong"] {
        assert!(
            package.modifier_index.iter().any(|m| {
                m.skill_def_id == RW_MINGUANGKAI
                    && m.target == "card_target_cancel"
                    && m.selector.as_deref() == Some(selector)
                    && m.mode == "override"
            }),
            "missing target-cancel for {selector}"
        );
    }
    assert!(package.modifier_index.iter().any(|m| {
        m.skill_def_id == RW_MINGUANGKAI
            && m.target == "link_loss_cancel"
            && m.mode == "override"
    }));
}


const LXIANG_LONG: u32 = 4_007_504_303;

#[test]
fn shahai_publishes_lxianglong_dodged_counter() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == LXIANG_LONG
            && t.event == "sha_dodged"
            && t.filter == "owner_is_target"
            && t.optional
    }));
}
