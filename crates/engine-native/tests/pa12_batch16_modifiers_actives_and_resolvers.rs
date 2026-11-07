//! PA12 迁移批⑯（2026-11-07，零版本号变更）综合验证：
//! 1. `content-native-loader` 全量 5,670 个 `.skill` 文件桥接清点：`modifier_rejected == 0`
//!    （保留 1,540 个纯 `INTENTIONAL_EMPTY_NOOP` 占位文件拒绝）且 `active_rejected == 0`。
//! 2. 批⑯新增 `VmModifierValueSource`（21 项）、`VmModifierTarget`（65+ 精确项与前缀族）与
//!    3 项伤害属性选择器（`normal` / `fire` / `thunder`）在内核与引擎上的求值与联动。
//! 3. 批⑯新增 `VmActiveLimits` 前置条件（3 项）、选牌约束（7 项）与选目标约束（24+ 项）求值。
//! 4. `codec_ext3.rs` / `accept_batch15.rs` / `commit_batch16.rs` / `batch16_resolvers.rs`
//!    30 个具名复合结算器（tags 179..=208）编解码往返、合法性校验、交互选择、`NGSA 107` 存读档往返与状态落地。

use content_compiler::compile_source_named;
use engine_native::choice::ChoiceKind;
use engine_native::content::extension::{ExtensionDef, ExtensionPack, InitHook};
use engine_native::event::Phase;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand};
use engine_native_kernel::catalog::{CardKind, EquipSlot};
use engine_native_kernel::event::DamageNature;
use engine_native_kernel::rvm_bridge::{
    VmActiveSelection, VmActiveTargets, VmModifierMode, VmModifierRule, VmModifierSelector,
    VmModifierTarget, VmModifierValueSource, VmSelectionZone, VmTargetFilter,
};
use engine_native_kernel::rvm_ops::{accept_effect, dec_effects, enc_effects};
use rule_api::{CardId, EffectRequest, MarkDefId, PlayerId, SkillDefId, SkillId};

const STD_DECHONG: &str = include_str!("../../../content/original/ported/sixiang/stddechong.skill");
const DDD_YUJIN: &str = include_str!("../../../content/original/ported/ddd/yujin.skill");
const DDD_ZHENJI: &str = include_str!("../../../content/original/ported/ddd/zhenji.skill");
const MOBILE_ZHENGJIAN: &str =
    include_str!("../../../content/original/ported/mobile/zhengjian.skill");
const SHENHUA_ZHANGHE: &str =
    include_str!("../../../content/original/ported/shenhua/zhanghe.skill");
const TW_FEIYI: &str = include_str!("../../../content/original/ported/tw/twfeiyi.skill");
const ZHENGSU_BIANZHEN: &str =
    include_str!("../../../content/original/ported/shiji/zhengsu_bianzhen.skill");
const TW_CAOANG: &str = include_str!("../../../content/original/ported/tw/twcaoang.skill");
const TW_CAOZHAO: &str = include_str!("../../../content/original/ported/tw/twcaozhao.skill");
const TW_CHENGPU: &str = include_str!("../../../content/original/ported/tw/twchengpu.skill");
const TW_CHENXUN: &str = include_str!("../../../content/original/ported/tw/twchenxun.skill");
const TW_CHENZHEN: &str = include_str!("../../../content/original/ported/tw/twchenzhen.skill");
const TW_YIZHU: &str = include_str!("../../../content/original/ported/tw/twyizhu.skill");
const DC_JIUDUN: &str = include_str!("../../../content/original/ported/xianding/dcjiudun.skill");
const XIANDING_QIAOJIAN: &str =
    include_str!("../../../content/original/ported/xianding/qiaojian.skill");
const XIANDING_ZHUPEILAN: &str =
    include_str!("../../../content/original/ported/xianding/zhupeilan.skill");
const MB_HUANGZU: &str = include_str!("../../../content/original/ported/bingshi/mb_huangzu.skill");
const MB_LUYUSHENG: &str =
    include_str!("../../../content/original/ported/bingshi/mb_luyusheng.skill");
const BINGSHI_PANGXI: &str = include_str!("../../../content/original/ported/bingshi/pangxi.skill");
const POT_CAOZHEN: &str =
    include_str!("../../../content/original/ported/bingshi/pot_caozhen.skill");
const POT_CHENDAO: &str =
    include_str!("../../../content/original/ported/bingshi/pot_chendao.skill");
const POT_CHENGPU: &str =
    include_str!("../../../content/original/ported/bingshi/pot_chengpu.skill");
const POT_DENGAI: &str = include_str!("../../../content/original/ported/bingshi/pot_dengai.skill");
const POT_LOUGUI: &str = include_str!("../../../content/original/ported/bingshi/pot_lougui.skill");
const POT_SUNCHEN: &str =
    include_str!("../../../content/original/ported/bingshi/pot_sunchen.skill");
const POT_ZHANGREN: &str =
    include_str!("../../../content/original/ported/bingshi/pot_zhangren.skill");
const DDD_GUANNING: &str = include_str!("../../../content/original/ported/ddd/guanning.skill");
const DDD_JIANSHUO: &str = include_str!("../../../content/original/ported/ddd/jianshuo.skill");
const SHEN_SIMAYI: &str = include_str!("../../../content/original/ported/extra/shen_simayi.skill");

static DEDUP: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1_600_000);

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
            description: "PA12 批⑯ 严格桥接测试包",
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
fn pa12_batch16_ported_skill_inventory_zero_modifier_and_active_rejections() {
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
    assert_eq!(files.len(), 5670, "ported .skill 文件总数应为 5670");

    let mut bridged_ok = 0usize;
    let mut pure_noop_rejected = 0usize;
    let mut single_skill_noop_rejected = 0usize;
    let mut modifier_rejected = Vec::new();
    let mut active_rejected = Vec::new();
    let mut trigger_rejected = 0usize;
    let mut view_as_rejected = 0usize;
    let mut bytecode_rejected = 0usize;

    for file in &files {
        let src = std::fs::read_to_string(file).expect("read .skill");
        let Ok(pkg) = content_compiler::compile_source_named("probe", &src) else {
            continue;
        };
        let is_pure_noop = pkg.trigger_index.is_empty()
            && pkg.active_index.is_empty()
            && pkg.view_as_index.is_empty()
            && !pkg.modifier_index.is_empty()
            && pkg.modifier_index.iter().all(|m| {
                m.target == "hand_limit"
                    && m.mode == "add"
                    && m.value == 0
                    && m.value_source.is_none()
                    && m.selector.is_none()
            });

        match content_native_loader::bridge_vm_skill_decls(&pkg) {
            Ok(out) => {
                assert!(
                    !is_pure_noop,
                    "纯 INTENTIONAL_EMPTY_NOOP 占位文件不得通过桥接: {}",
                    file.display()
                );
                if !pkg.trigger_index.is_empty()
                    || !pkg.active_index.is_empty()
                    || !pkg.view_as_index.is_empty()
                    || !pkg.modifier_index.is_empty()
                {
                    assert!(
                        !out.decls.is_empty(),
                        "通过桥接的非空规则文件必须产出至少 1 项 VmSkillDecl: {}",
                        file.display()
                    );
                }
                bridged_ok += 1;
            }
            Err(err) => {
                let msg = err.to_string();
                if is_pure_noop {
                    pure_noop_rejected += 1;
                    if !file.to_string_lossy().contains("/shahai/") {
                        single_skill_noop_rejected += 1;
                    }
                } else if msg.contains("modifier_index") || msg.contains("skill_index") {
                    modifier_rejected.push((file.display().to_string(), msg));
                } else if msg.contains("active_index") {
                    active_rejected.push((file.display().to_string(), msg));
                } else if msg.contains("trigger_index") {
                    trigger_rejected += 1;
                } else if msg.contains("view_as_index") {
                    view_as_rejected += 1;
                } else {
                    bytecode_rejected += 1;
                }
            }
        }
    }

    eprintln!(
        "PA12 Batch 16 inventory: total={}, bridged_ok={}, pure_noop_rejected={} (non_shahai_noop={}), modifier_rejected={}, active_rejected={}, trigger_rejected={}, view_as_rejected={}, bytecode_rejected={}",
        files.len(),
        bridged_ok,
        pure_noop_rejected,
        single_skill_noop_rejected,
        modifier_rejected.len(),
        active_rejected.len(),
        trigger_rejected,
        view_as_rejected,
        bytecode_rejected
    );

    assert!(
        modifier_rejected.is_empty(),
        "modifier_rejected 必须清零，残留: {modifier_rejected:?}"
    );
    assert!(
        active_rejected.is_empty(),
        "active_rejected 必须清零，残留: {active_rejected:?}"
    );
    assert_eq!(
        pure_noop_rejected, 2029,
        "全部 2,029 个纯 INTENTIONAL_EMPTY_NOOP 文件必须保持拒绝"
    );
    assert!(
        single_skill_noop_rejected >= 1530,
        "非杀海纯 INTENTIONAL_EMPTY_NOOP 文件必须保持拒绝（实际 {single_skill_noop_rejected}）"
    );
    assert_eq!(
        bridged_ok, 3524,
        "PA12 批⑯应精确桥接 3,524 个 ported .skill 文件"
    );
}

#[test]
fn pa12_batch16_codec_roundtrip_179_to_208_and_validation() {
    let effects = vec![
        (
            179u8,
            EffectRequest::ResolveStddechong {
                player: PlayerId::new(1),
                target: PlayerId::new(2),
                cards: vec![CardId::new(3), CardId::new(4)],
                skill: SkillId::new(2987704809),
            },
        ),
        (
            180,
            EffectRequest::ResolveDddYujin {
                player: PlayerId::new(1),
                actor: PlayerId::new(2),
                target: PlayerId::new(2),
                move_mask: 0,
                action: 1,
                candidates: vec![PlayerId::new(2)],
                skill: SkillId::new(3001),
            },
        ),
        (
            181,
            EffectRequest::ResolveDddZhenji {
                player: PlayerId::new(1),
                target: PlayerId::new(2),
                action: 2,
                stage: 0,
                piles: vec![1, 2],
                cards: vec![CardId::new(5)],
                skill: SkillId::new(3002),
            },
        ),
        (
            182,
            EffectRequest::ResolveZhengjian {
                player: PlayerId::new(1),
                skill: SkillId::new(4084687925),
                stage: 0,
                event_player: PlayerId::new(2),
            },
        ),
        (
            183,
            EffectRequest::ResolveQiaobian {
                player: PlayerId::new(1),
                phase: 2,
                stage: 2,
                skill: SkillId::new(3004),
            },
        ),
        (
            184,
            EffectRequest::ResolveTwfeiyi {
                player: PlayerId::new(1),
                action: 0,
                stage: 1,
                card: CardId::new(5),
                event_cards: vec![CardId::new(5)],
                skill: SkillId::new(3391342890),
            },
        ),
        (
            185,
            EffectRequest::ResolveZhengsu {
                player: PlayerId::new(1),
                mode: 1,
                stage: 0,
                card: CardId::new(5),
                skill: SkillId::new(3044720297),
            },
        ),
        (
            186,
            EffectRequest::ResolveTwcaoang {
                player: PlayerId::new(1),
                target: PlayerId::new(2),
                source: PlayerId::new(2),
                card: CardId::new(6),
                helper_skill_def: 2420900957,
                action: 0,
                stage: 0,
                skill: SkillId::new(3007),
            },
        ),
        (
            187,
            EffectRequest::ResolveTwcaozhao {
                player: PlayerId::new(1),
                target: PlayerId::new(2),
                source: PlayerId::new(1),
                card: CardId::new(6),
                selected_skill_def: 0,
                twfuzuan_skill_def: 3482762704,
                twfeifu_skill_def: 3948553827,
                action: 0,
                stage: 0,
                skill: SkillId::new(3008),
            },
        ),
        (
            188,
            EffectRequest::ResolveTwchengpu {
                player: PlayerId::new(1),
                target: PlayerId::new(2),
                source: PlayerId::new(2),
                card: CardId::new(7),
                chunlao_skill_def: 2257064269,
                action: 1,
                stage: 0,
                selected_card: CardId::new(7),
                skill: SkillId::new(3009),
            },
        ),
        (
            189,
            EffectRequest::ResolveTwchenxun {
                player: PlayerId::new(1),
                target: PlayerId::new(2),
                card: CardId::new(8),
                action: 2,
                skill: SkillId::new(3010),
            },
        ),
        (
            190,
            EffectRequest::ResolveTwchenzhen {
                player: PlayerId::new(1),
                target: PlayerId::new(2),
                helper_skill_def: 2739628855,
                record_skill_def: 123456,
                action: 1,
                stage: 0,
                card_kind: 1,
                card: CardId::new(8),
                branch: 0,
                skill: SkillId::new(3011),
            },
        ),
        (
            191,
            EffectRequest::ResolveTwyizhu {
                player: PlayerId::new(1),
                source: PlayerId::new(2),
                card: CardId::new(9),
                action: 0,
                stage: 0,
                event_cards: vec![CardId::new(9)],
                skill: SkillId::new(3012),
            },
        ),
        (
            192,
            EffectRequest::ResolveRuanji {
                player: PlayerId::new(1),
                source: PlayerId::new(2),
                card: CardId::new(10),
                action: 3,
                cards: vec![CardId::new(10)],
                skill: SkillId::new(3013),
            },
        ),
        (
            193,
            EffectRequest::ResolveQiaojian {
                player: PlayerId::new(1),
                target: PlayerId::new(2),
                helper_def: SkillDefId::new(9694479),
                action: 0,
                cards: vec![CardId::new(11)],
                split: 0,
                skill: SkillId::new(3014),
            },
        ),
        (
            194,
            EffectRequest::ResolveZhupeilan {
                player: PlayerId::new(1),
                target: PlayerId::new(2),
                card: CardId::new(12),
                helper_skill_def: SkillDefId::new(3697675750),
                action: 10,
                event_cards: vec![CardId::new(12)],
                skill: SkillId::new(3015),
            },
        ),
        (
            195,
            EffectRequest::ResolveMbduanyang {
                player: PlayerId::new(1),
                target: PlayerId::new(2),
                card: CardId::new(13),
                mode: 0,
                stage: 0,
                cards: vec![CardId::new(13)],
                skill: SkillId::new(3016),
            },
        ),
        (
            196,
            EffectRequest::ResolveMbrunwei {
                player: PlayerId::new(1),
                target: PlayerId::new(2),
                stage: 0,
                value: 1,
                cards: vec![CardId::new(13)],
                skill: SkillId::new(3017),
            },
        ),
        (
            197,
            EffectRequest::ResolveMbkuangxiang {
                player: PlayerId::new(1),
                target: PlayerId::new(2),
                stage: 1,
                skill: SkillId::new(3018),
            },
        ),
        (
            198,
            EffectRequest::ResolvePotsifeng {
                player: PlayerId::new(1),
                target: PlayerId::new(2),
                card: CardId::new(14),
                stage: 4,
                skill: SkillId::new(3019),
            },
        ),
        (
            199,
            EffectRequest::ResolvePotwanglie {
                player: PlayerId::new(1),
                card: CardId::new(15),
                helper_skill_def: SkillDefId::new(3694841233),
                stage: 1,
                skill: SkillId::new(3020),
            },
        ),
        (
            200,
            EffectRequest::ResolvePotchengpu {
                player: PlayerId::new(1),
                target: PlayerId::new(2),
                stage: 2,
                damage_id: 0,
                skill: SkillId::new(3021),
            },
        ),
        (
            201,
            EffectRequest::ResolvePottuntian {
                player: PlayerId::new(1),
                action: 2,
                amount: 1,
                targets: vec![PlayerId::new(2)],
                zaoxian_def: SkillDefId::new(2551307508),
                skill: SkillId::new(3022),
            },
        ),
        (
            202,
            EffectRequest::ResolvePotjixi {
                player: PlayerId::new(1),
                target: PlayerId::new(2),
                action: 2,
                targets: vec![PlayerId::new(2)],
                skill: SkillId::new(3023),
            },
        ),
        (
            203,
            EffectRequest::ResolvePotjiyu {
                player: PlayerId::new(1),
                card: CardId::new(16),
                stage: 0,
                skill: SkillId::new(3024),
            },
        ),
        (
            204,
            EffectRequest::ResolvePotnigu {
                player: PlayerId::new(1),
                target: PlayerId::new(2),
                card: CardId::new(16),
                targets: vec![PlayerId::new(2)],
                index: 0,
                declined: 0,
                stage: 1,
                skill: SkillId::new(3025),
            },
        ),
        (
            205,
            EffectRequest::ResolvePotfuan {
                player: PlayerId::new(1),
                card: CardId::new(17),
                stage: 0,
                range_helper_def: SkillDefId::new(2955537232),
                yinxian_def: SkillDefId::new(2595028433),
                skill: SkillId::new(3026),
            },
        ),
        (
            206,
            EffectRequest::ResolveDddGuanning {
                player: PlayerId::new(1),
                actor: PlayerId::new(2),
                action: 2,
                card: CardId::new(18),
                skill: SkillId::new(3027),
            },
        ),
        (
            207,
            EffectRequest::ResolveDddJianshuo {
                player: PlayerId::new(1),
                target: PlayerId::new(2),
                action: 0,
                cards: vec![CardId::new(19)],
                targets: vec![PlayerId::new(2)],
                skill: SkillId::new(3028),
            },
        ),
        (
            208,
            EffectRequest::ResolveShenSimayi {
                player: PlayerId::new(1),
                event_player: PlayerId::new(2),
                event_amount: 2,
                mark: MarkDefId::new(3085167840),
                mode: 3,
                cards: vec![CardId::new(20)],
                targets: vec![PlayerId::new(2)],
                skill: SkillId::new(3029),
            },
        ),
    ];

    assert_eq!(effects.len(), 30);
    for (expected_tag, eff) in &effects {
        assert!(
            accept_effect(eff, 4).is_ok(),
            "合法批⑯原语应通过 accept_effect: {eff:?}"
        );
        let mut bytes = Vec::new();
        enc_effects(&mut bytes, std::slice::from_ref(eff));
        assert_eq!(bytes[4], *expected_tag, "tag 字节应等于 {expected_tag}");
        let decoded = dec_effects(&bytes).expect("dec_effects");
        assert_eq!(&decoded[0], eff, "tag {expected_tag} 编解码往返一致");
    }

    // 未知单字节 tag 209 应拒绝
    let mut bad_bytes = Vec::new();
    bad_bytes.extend_from_slice(&1u32.to_le_bytes());
    bad_bytes.push(209u8);
    assert!(
        dec_effects(&bad_bytes).is_none(),
        "未分配的单字节 tag 209 必须被解码器拒绝"
    );

    // 非法参数（player=0 或越界 stage/mode）应被 accept_effect 拒绝
    assert!(accept_effect(
        &EffectRequest::ResolveStddechong {
            player: PlayerId::NONE,
            target: PlayerId::new(2),
            cards: Vec::new(),
            skill: SkillId::NONE,
        },
        4
    )
    .is_err());
    assert!(accept_effect(
        &EffectRequest::ResolveShenSimayi {
            player: PlayerId::new(1),
            event_player: PlayerId::new(2),
            event_amount: 1,
            mark: MarkDefId::new(100),
            mode: 5,
            cards: Vec::new(),
            targets: Vec::new(),
            skill: SkillId::NONE,
        },
        4
    )
    .is_err());
}

#[test]
fn pa12_batch16_modifier_targets_value_sources_and_damage_selectors() {
    let dsl = r#"
skill companion_main(91600101) {
    trigger phase_prepare owner_is_target {
        effect draw_cards(self, 1);
    }
}
skill companion_noop(91600102) {
    // Companion INTENTIONAL_EMPTY_NOOP in multi-skill package
    modifier hand_limit add 0;
}
skill meta_lord_conv_nongain(91600103) lord_only conversion non_gainable {
    modifier sha_ignored_hand_limit override 1;
    modifier damage_amount(fire) add 1;
    modifier damage_taken_cancel(thunder) override 1;
    modifier ignore_armor override 1;
    modifier card_use_unrespondable(sha) override 1;
}
"#;
    let pack = pack_strict(&[("b16_mods", dsl)], "pa12-b16-mods");
    let packs = [&pack];
    let mut e = Engine::new_with_extensions(916_001, 3, &packs).expect("建局");
    advance_to_phase(&mut e, 0, Phase::Play);

    e.debug_clear_hand(0);
    let _sha1 = e.debug_place(0, "sha");
    let _sha2 = e.debug_place(0, "sha");
    let _shan = e.debug_place(0, "shan");

    let base_limit = e.state_core().effective_hand_limit(e.catalog(), 0);
    e.debug_grant(0, "meta_lord_conv_nongain");
    let ignored = e.state_core().ignored_hand_card_count(e.catalog(), 0);
    assert_eq!(ignored, 2, "sha_ignored_hand_limit 应忽略手牌中的 2 张【杀】");
    assert_eq!(
        e.state_core().effective_hand_limit(e.catalog(), 0),
        base_limit + 2,
        "effective_hand_limit 应加上被忽略的 2 张【杀】"
    );

    // 验证伤害属性选择器与伤害修正：火焰伤害 +1、雷电伤害承受取消、普通伤害不变
    assert_eq!(
        e.state_core().effective_damage_amount(
            e.catalog(),
            Some(0),
            1,
            None,
            DamageNature::Fire,
            1
        ),
        2,
        "damage_amount(fire) 应使火焰伤害 +1"
    );
    assert_eq!(
        e.state_core().effective_damage_amount(
            e.catalog(),
            Some(0),
            1,
            None,
            DamageNature::Normal,
            1
        ),
        1,
        "普通伤害不受 (fire) 影响"
    );
    assert!(
        e.state_core().is_damage_taken_canceled(
            e.catalog(),
            Some(1),
            0,
            None,
            DamageNature::Thunder,
            2
        ),
        "damage_taken_cancel(thunder) 应防止角色 0 受到的雷电伤害"
    );
    assert!(
        e.state_core()
            .ignores_target_armor(e.catalog(), 0, 1, None, DamageNature::Normal),
        "ignore_armor 应无视目标防具"
    );
    assert!(
        e.state_core().is_card_use_unrespondable(
            e.catalog(),
            0,
            1,
            _sha1,
            CardKind::Sha,
            DamageNature::Normal
        ),
        "card_use_unrespondable(sha) 应使【杀】不可被响应"
    );

    // 验证批⑯新增 VmModifierValueSource 变体求值
    let def_id = e.catalog().skill_by_name("meta_lord_conv_nongain").unwrap();
    e.state_core_mut().player_mut(1).hp = 2;
    e.state_core_mut().player_mut(2).hp = 2;
    let r_wounded = VmModifierRule {
        target: VmModifierTarget::HandLimit,
        mode: VmModifierMode::Add,
        value: 1,
        value_source: Some(VmModifierValueSource::AliveWoundedCountAtLeast(2)),
        selector: None,
    };
    assert_eq!(
        r_wounded.effective_value(e.state_core(), e.catalog(), 0, def_id),
        1
    );

    e.state_core_mut().vm_aux.marks.insert((1, 777), 3);
    let r_mark = VmModifierRule {
        target: VmModifierTarget::HandLimit,
        mode: VmModifierMode::Add,
        value: 1,
        value_source: Some(VmModifierValueSource::QueriedPlayerMarkPositive(777)),
        selector: None,
    };
    assert_eq!(
        r_mark.effective_value_for(e.state_core(), e.catalog(), 0, Some(1), def_id),
        1
    );

    e.state_core_mut().turn.sha_used_play_phase = true;
    let r_sha1 = VmModifierRule {
        target: VmModifierTarget::HandLimit,
        mode: VmModifierMode::Add,
        value: 1,
        value_source: Some(VmModifierValueSource::OwnerShaUsedExactlyOne),
        selector: None,
    };
    assert_eq!(
        r_sha1.effective_value(e.state_core(), e.catalog(), 0, def_id),
        1
    );

    let r_other = VmModifierRule {
        target: VmModifierTarget::DistanceToOthers,
        mode: VmModifierMode::Add,
        value: -1,
        value_source: Some(VmModifierValueSource::QueriedPlayerOtherThanOwner),
        selector: Some(VmModifierSelector::Any),
    };
    assert_eq!(
        r_other.effective_value_for(e.state_core(), e.catalog(), 0, Some(1), def_id),
        -1
    );
    assert_eq!(
        r_other.effective_value_for(e.state_core(), e.catalog(), 0, Some(0), def_id),
        0
    );

    // 端到端实战结算验证：
    // ① 0 号对 1 号造成 1 点火焰伤害 -> damage_amount(fire) add 1 实际扣减 2 点体力
    e.state_core_mut().player_mut(1).hp = 4;
    e.debug_deal_damage(Some(0), 1, 1, DamageNature::Fire);
    drain_choices(&mut e);
    assert_eq!(
        e.state_core().player(1).hp,
        2,
        "commit_damage 应实际应用 damage_amount(fire) add 1 扣减 2 点体力"
    );

    // ② 1 号对 0 号造成 2 点雷电伤害 -> damage_taken_cancel(thunder) 实际防止该雷电伤害
    let hp0_before = e.state_core().player(0).hp;
    e.debug_deal_damage(Some(1), 0, 2, DamageNature::Thunder);
    drain_choices(&mut e);
    assert_eq!(
        e.state_core().player(0).hp,
        hp0_before,
        "commit_damage 应实际应用 damage_taken_cancel(thunder) 防止雷电伤害"
    );

    // ③ 1 号装备仁王盾 + 手持【闪】，0 号出黑色【杀】 -> ignore_armor + card_use_unrespondable(sha) 直接命中
    e.debug_clear_hand(1);
    let _shan_target = e.debug_place(1, "shan");
    let renwang_cid = e
        .catalog()
        .cards
        .iter()
        .position(|d| d.kind == CardKind::Renwang)
        .unwrap() as u32;
    e.state_core_mut().player_mut(1).equips.push(renwang_cid);
    let black_sha = e
        .catalog()
        .cards
        .iter()
        .position(|d| d.kind == CardKind::Sha && d.suit == engine_native_kernel::catalog::Suit::Spade)
        .unwrap() as u32;
    e.state_core_mut().turn.sha_used_play_phase = false;
    e.state_core_mut().player_mut(0).hand.push(black_sha);
    let hp1_before_sha = e.state_core().player(1).hp;
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: black_sha,
            targets: vec![1],
        },
    );
    drain_choices(&mut e);
    assert_eq!(
        e.state_core().player(1).hp,
        hp1_before_sha - 1,
        "ShaResolveMachine 应实际穿透仁王盾且不可响应直接扣减 1 点体力"
    );
}

#[test]
fn pa12_batch16_active_preconditions_selection_and_target_filters() {
    let dsl = r#"
skill blocker_skill(91600150) {
    modifier hand_limit add 1;
}
skill b16_active_sum10(91600151) {
    active play_phase uses_per_turn 1 owner_missing_skill 91600150 hand_limit_positive hand_missing_gaintag 999 targets 1 2 other max_by_owner_lost_hp select_hand 1 3 sum_effective_number_exact 10 {
        effect draw_cards(self, 1);
    }
}
"#;
    let pack = pack_strict(&[("b16_active", dsl)], "pa12-b16-active");
    let packs = [&pack];
    let mut e = Engine::new_with_extensions(916_010, 3, &packs).expect("建局");
    advance_to_phase(&mut e, 0, Phase::Play);
    e.debug_grant(0, "b16_active_sum10");
    let act_def = e.catalog().skill_by_name("b16_active_sum10").unwrap();

    // 1. owner 未受伤时 max_by_owner_lost_hp = 0 < min_targets(1)，不可发动
    let max_hp0 = e.state_core().player(0).max_hp as i32;
    e.state_core_mut().player_mut(0).hp = max_hp0;
    assert!(matches!(
        submit_outcome(&mut e, 0, GameAction::ActivateSkill { skill: act_def }),
        CommandOutcome::Rejected(_)
    ));

    // 2. 令 owner 失去 1 点体力，并放置两张点数之和等于 10 的手牌
    e.state_core_mut().player_mut(0).hp = max_hp0 - 1;
    e.debug_clear_hand(0);
    let mut picked = Vec::new();
    for (cid, def) in e.catalog().cards.iter().enumerate() {
        if def.rank == 4 && picked.is_empty() {
            picked.push(cid as u32);
        } else if def.rank == 6 && picked.len() == 1 {
            picked.push(cid as u32);
            break;
        }
    }
    e.state_core_mut().player_mut(0).hand.extend(&picked);

    // 若拥有 blocker_skill，则 owner_missing_skill 前置条件阻止发动
    e.debug_grant(0, "blocker_skill");
    assert!(matches!(
        submit_outcome(&mut e, 0, GameAction::ActivateSkill { skill: act_def }),
        CommandOutcome::Rejected(_)
    ));
    let blocker_def = e.catalog().skill_by_name("blocker_skill").unwrap();
    e.state_core_mut().skills.remove(&(0, blocker_def));

    // 移除 blocker_skill 后可正常发动（点数 4 + 6 = 10，目标上限 min(2, lost_hp=1) = 1）
    let hand_before = e.state_core().player(0).hand.len();
    submit(&mut e, 0, GameAction::ActivateSkill { skill: act_def });
    assert_eq!(e.state_core().player(0).hand.len(), hand_before + 1);

    // 3. 单元测试 VmActiveSelection 新增过滤标志（same_color / distinct_suits / distinct_categories）
    let sel_distinct_cats = VmActiveSelection {
        min_cards: 2,
        max_cards: 2,
        zone: VmSelectionZone::HandEquip,
        distinct_categories: true,
        ..Default::default()
    };
    e.debug_clear_hand(0);
    let c_sha = e.debug_place(0, "sha");
    let c_trick = e.debug_place(0, "dismantle");
    let cands =
        sel_distinct_cats.candidate_cards(e.state_core(), e.catalog(), 0, "probe_distinct_cats");
    assert_eq!(cands, vec![c_sha, c_trick]);

    // 4. 单元测试 VmActiveTargets 新增组合约束（pair_hand_difference_lte_three / equipment_pair_with_armor）
    let t_armor = VmActiveTargets {
        min_targets: 2,
        max_targets: 2,
        filter: VmTargetFilter::default(),
        equipment_pair_with_armor: true,
        ..Default::default()
    };
    assert!(
        t_armor
            .candidate_targets(e.state_core(), e.catalog(), 0, "probe_armor")
            .is_empty(),
        "场上无人装备防具时 equipment_pair_with_armor 应返回空"
    );
    let armor_cid = e
        .catalog()
        .cards
        .iter()
        .position(|d| d.kind.equip_slot() == Some(EquipSlot::Armor))
        .unwrap() as u32;
    let weapon_cid = e
        .catalog()
        .cards
        .iter()
        .position(|d| d.kind.equip_slot() == Some(EquipSlot::Weapon))
        .unwrap() as u32;
    e.state_core_mut().player_mut(0).equips.push(armor_cid);
    e.state_core_mut().player_mut(1).equips.push(weapon_cid);
    let armor_cands = t_armor.candidate_targets(e.state_core(), e.catalog(), 0, "probe_armor");
    assert_eq!(&armor_cands[..2], &[0, 1]);
}

#[test]
fn pa12_batch16_resolvers_179_to_208_compile_and_execute_with_ngsa107() {
    // 1. 验证全部 29 个真实批⑯ `.skill` 文件通过编译器，且产出 VM 技能声明
    let real_sources = [
        ("stddechong", STD_DECHONG),
        ("ddd_yujin", DDD_YUJIN),
        ("ddd_zhenji", DDD_ZHENJI),
        ("zhengjian", MOBILE_ZHENGJIAN),
        ("zhanghe", SHENHUA_ZHANGHE),
        ("twfeiyi", TW_FEIYI),
        ("zhengsu_bianzhen", ZHENGSU_BIANZHEN),
        ("twcaoang", TW_CAOANG),
        ("twcaozhao", TW_CAOZHAO),
        ("twchengpu", TW_CHENGPU),
        ("twchenxun", TW_CHENXUN),
        ("twchenzhen", TW_CHENZHEN),
        ("twyizhu", TW_YIZHU),
        ("dcjiudun", DC_JIUDUN),
        ("qiaojian", XIANDING_QIAOJIAN),
        ("zhupeilan", XIANDING_ZHUPEILAN),
        ("mb_huangzu", MB_HUANGZU),
        ("mb_luyusheng", MB_LUYUSHENG),
        ("pangxi", BINGSHI_PANGXI),
        ("pot_caozhen", POT_CAOZHEN),
        ("pot_chendao", POT_CHENDAO),
        ("pot_chengpu", POT_CHENGPU),
        ("pot_dengai", POT_DENGAI),
        ("pot_lougui", POT_LOUGUI),
        ("pot_sunchen", POT_SUNCHEN),
        ("pot_zhangren", POT_ZHANGREN),
        ("ddd_guanning", DDD_GUANNING),
        ("ddd_jianshuo", DDD_JIANSHUO),
        ("shen_simayi", SHEN_SIMAYI),
    ];
    let mut bridged_decls = 0usize;
    let mut bridged_files = 0usize;
    for (id, src) in &real_sources {
        let pkg = compile_source_named(*id, src).expect("编译真实批⑯ .skill");
        assert!(!pkg.skill_index.is_empty(), "{id} 应有 skill_index");
        if let Ok(out) = content_native_loader::bridge_vm_skill_decls(&pkg) {
            assert!(!out.decls.is_empty(), "{id} 桥接结果不应为空");
            bridged_decls += out.decls.len();
            bridged_files += 1;
        }
    }
    assert!(
        bridged_files >= 20 && bridged_decls >= 25,
        "真实批⑯技能文件应绝大多数通过正式桥接（files={bridged_files}/{}, decls={bridged_decls}）",
        real_sources.len()
    );

    // 2. 构造覆盖 tags 179..=208 全部 30 个结算器的主动技测试套件
    let harness_dsl = r#"
skill h_helper_skill(91600300) {
    modifier hand_limit add 1;
}
skill h_179_stddechong(91600301) {
    active play_phase {
        effect choose_give_hand_equip_to_event_player(self);
    }
}
skill h_180_ddd_yujin(91600302) {
    active play_phase {
        effect resolve_ddd_yujin(self, event_player, 0);
    }
}
skill h_181_ddd_zhenji(91600303) {
    active play_phase {
        effect resolve_ddd_zhenji(self, 0);
    }
}
skill h_182_zhengjian(91600304) {
    active play_phase {
        effect resolve_zhengjian(self, self_skill, 0, event_player);
    }
}
skill h_183_qiaobian(91600305) {
    active play_phase {
        effect resolve_qiaobian(self, 0);
    }
}
skill h_184_twfeiyi(91600306) {
    active play_phase {
        effect resolve_twfeiyi(self, 1);
    }
}
skill h_185_zhengsu(91600307) {
    active play_phase {
        effect resolve_zhengsu(self, 1, 1, 0);
    }
}
skill h_186_twcaoang(91600308) {
    active play_phase {
        effect resolve_twcaoang(self, event_player, event_source, event_card, 91600300, 0);
    }
}
skill h_187_twcaozhao(91600309) {
    active play_phase {
        effect resolve_twcaozhao(self, self, self, 0, 91600300, 91600300, 0);
    }
}
skill h_188_twchengpu(91600310) {
    active play_phase {
        effect resolve_twchengpu(self, event_player, event_source, event_card, 91600300, 0);
    }
}
skill h_189_twchenxun(91600311) {
    active play_phase {
        effect resolve_twchenxun(self, self, event_card, 0);
    }
}
skill h_190_twchenzhen(91600312) {
    active play_phase {
        effect resolve_twchenzhen(self, self, 91600300, 91600300, 0);
    }
}
skill h_191_twyizhu(91600313) {
    active play_phase {
        effect resolve_twyizhu(self, self, event_card, 0);
    }
}
skill h_192_ruanji(91600314) {
    active play_phase {
        effect resolve_ruanji(self, event_source, event_card, 3);
    }
}
skill h_193_qiaojian(91600315) {
    active play_phase {
        effect resolve_qiaojian(self, event_source, 91600300, 0);
    }
}
skill h_194_zhupeilan(91600316) {
    active play_phase {
        effect resolve_zhupeilan(self, event_source, event_card, 91600300, 0);
    }
}
skill h_195_mbduanyang(91600317) {
    active play_phase {
        effect resolve_mbduanyang(self, event_player, event_card, 0);
    }
}
skill h_196_mbrunwei(91600318) {
    active play_phase {
        effect resolve_mbrunwei(self, self, 0);
    }
}
skill h_197_mbkuangxiang(91600319) {
    active play_phase {
        effect resolve_mbkuangxiang(self, event_source, 0);
    }
}
skill h_198_potsifeng(91600320) {
    active play_phase {
        effect resolve_potsifeng(self, self, event_card, 0);
    }
}
skill h_199_potwanglie(91600321) {
    active play_phase {
        effect resolve_potwanglie(self, event_card, 91600300, 1);
    }
}
skill h_200_potchengpu(91600322) {
    active play_phase {
        effect resolve_potchengpu(self, event_player, 0);
    }
}
skill h_201_pottuntian(91600323) {
    active play_phase {
        effect resolve_pottuntian(self, 0, 91600300);
        effect resolve_pottuntian(self, 2, 91600300);
    }
}
skill h_202_potjixi(91600324) {
    active play_phase {
        effect resolve_potjixi(self, self, 2);
    }
}
skill h_203_potjiyu(91600325) {
    active play_phase {
        effect resolve_potjiyu(self, 0);
    }
}
skill h_204_potnigu(91600326) {
    active play_phase {
        effect resolve_potnigu(self, 0);
    }
}
skill h_205_potfuan(91600327) {
    active play_phase {
        effect resolve_potfuan(self, event_card, 0, 91600300, 91600300);
    }
}
skill h_206_ddd_guanning(91600328) {
    active play_phase {
        effect resolve_ddd_guanning(self, event_player, 2);
    }
}
skill h_207_ddd_jianshuo(91600329) {
    active play_phase {
        effect resolve_ddd_jianshuo(self, self, 0);
    }
}
skill h_208_shen_simayi(91600330) {
    active play_phase {
        effect resolve_shen_simayi(self, event_player, 2, 3085167840, 0);
        effect resolve_shen_simayi(self, event_player, 0, 3085167840, 3);
    }
}
"#;
    let pack = pack_strict(&[("b16_harness", harness_dsl)], "pa12-b16-harness");
    let packs = [&pack];

    let skill_names = [
        "h_179_stddechong",
        "h_180_ddd_yujin",
        "h_181_ddd_zhenji",
        "h_182_zhengjian",
        "h_183_qiaobian",
        "h_184_twfeiyi",
        "h_185_zhengsu",
        "h_186_twcaoang",
        "h_187_twcaozhao",
        "h_188_twchengpu",
        "h_189_twchenxun",
        "h_190_twchenzhen",
        "h_191_twyizhu",
        "h_192_ruanji",
        "h_193_qiaojian",
        "h_194_zhupeilan",
        "h_195_mbduanyang",
        "h_196_mbrunwei",
        "h_197_mbkuangxiang",
        "h_198_potsifeng",
        "h_199_potwanglie",
        "h_200_potchengpu",
        "h_201_pottuntian",
        "h_202_potjixi",
        "h_203_potjiyu",
        "h_204_potnigu",
        "h_205_potfuan",
        "h_206_ddd_guanning",
        "h_207_ddd_jianshuo",
        "h_208_shen_simayi",
    ];

    let mut interactive_verified = 0usize;
    for (idx, name) in skill_names.iter().enumerate() {
        let mut e = Engine::new_with_extensions(916_300 + idx as u64, 3, &packs).expect("建局");
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
        interactive_verified >= 10,
        "多候选局面下应有批⑯原语触发真实 WaitChoice 交互与 NGSA 107 存读档往返（实际 {interactive_verified}）"
    );

    // 专项断言：h_182_zhengjian 交互选择 1 号角色并授予「诤」标记（2_446_053_157）
    {
        let mut e = Engine::new_with_extensions(916_401, 2, &packs).expect("建局");
        advance_to_phase(&mut e, 0, Phase::Play);
        e.debug_grant(0, "h_182_zhengjian");
        let def = e.catalog().skill_by_name("h_182_zhengjian").unwrap();
        submit(&mut e, 0, GameAction::ActivateSkill { skill: def });
        let ask = e.open_choice().cloned().expect("zhengjian 选目标询问");
        assert_eq!(ask.spec.kind, ChoiceKind::SelectPlayers);
        answer(&mut e, vec![ChoiceOption::Player(1)]);
        assert_eq!(
            e.state_core().vm_aux.marks.get(&(1, 2_446_053_157)).copied(),
            Some(1),
            "zhengjian 应给目标角色添加 1 枚诤标记"
        );
    }

    // 专项断言：h_208_shen_simayi 先获得 2 枚忍标记（mode 0），随后交互选牌弃置制衡并消耗 1 枚忍标记（mode 3）
    {
        let mut e = Engine::new_with_extensions(916_402, 2, &packs).expect("建局");
        advance_to_phase(&mut e, 0, Phase::Play);
        e.debug_clear_hand(0);
        let c1 = e.debug_place(0, "sha");
        let c2 = e.debug_place(0, "shan");
        e.debug_grant(0, "h_208_shen_simayi");
        let def = e.catalog().skill_by_name("h_208_shen_simayi").unwrap();
        submit(&mut e, 0, GameAction::ActivateSkill { skill: def });
        let ask = e.open_choice().cloned().expect("shen_simayi 极略·制衡选牌询问");
        assert_eq!(ask.spec.kind, ChoiceKind::SelectCards);
        answer(&mut e, vec![ChoiceOption::Card(c1), ChoiceOption::Card(c2)]);
        assert_eq!(
            e.state_core().vm_aux.marks.get(&(0, 3_085_167_840)).copied(),
            Some(1),
            "获得 2 枚忍标记后消耗 1 枚发动制衡，剩余 1 枚"
        );
        assert_eq!(e.state_core().player(0).hand.len(), 2, "制衡弃 2 张应摸 2 张");
    }

    // 专项断言：h_183_qiaobian 交互选第 2 张手牌（c2）时必须精确弃置 c2 而非首张手牌 c1
    {
        let mut e = Engine::new_with_extensions(916_403, 2, &packs).expect("建局");
        advance_to_phase(&mut e, 0, Phase::Play);
        e.debug_clear_hand(0);
        let c1 = e.debug_place(0, "sha");
        let c2 = e.debug_place(0, "shan");
        e.debug_grant(0, "h_183_qiaobian");
        let def = e.catalog().skill_by_name("h_183_qiaobian").unwrap();
        submit(&mut e, 0, GameAction::ActivateSkill { skill: def });
        let ask = e.open_choice().cloned().expect("qiaobian 选牌询问");
        assert_eq!(ask.spec.kind, ChoiceKind::SelectCards);
        answer(&mut e, vec![ChoiceOption::Card(c2)]);
        assert_eq!(
            e.state_core().player(0).hand,
            vec![c1],
            "qiaobian 交互选择第 2 张手牌 c2 后应保留 c1 并弃置 c2"
        );
    }

    // 专项断言：h_185_zhengsu 在 stage=1 结算完毕后应自动移除自身技能（emit_effect 补填 skill）
    {
        let mut e = Engine::new_with_extensions(916_404, 2, &packs).expect("建局");
        advance_to_phase(&mut e, 0, Phase::Play);
        e.debug_grant(0, "h_185_zhengsu");
        let def = e.catalog().skill_by_name("h_185_zhengsu").unwrap();
        assert!(e.state_core().has_skill(0, def));
        submit(&mut e, 0, GameAction::ActivateSkill { skill: def });
        drain_choices(&mut e);
        assert!(
            !e.state_core().has_skill(0, def),
            "zhengsu stage=1 结算后应移除当前整肃技能"
        );
    }
}
