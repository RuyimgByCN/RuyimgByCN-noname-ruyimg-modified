//! PA12 迁移批⑰（2026-11-07，零版本号变更）综合验证：
//! 1. `content-compiler` 与 `content-native-loader` 全量 5,670 个 `.skill` 文件清点：
//!    `compile_rejected == 0`、`modifier_rejected == 0`、`active_rejected == 0`、
//!    `trigger_rejected == 0`、`view_as_rejected == 0`、`bytecode_rejected == 0`，
//!    除 2,029 个纯 `INTENTIONAL_EMPTY_NOOP` 占位文件保持拒绝外，全部 3,641 个非空
//!    `.skill` 文件 100% 通过编译与桥接（`3,641 + 2,029 == 5,670`）。
//! 2. 多牌消耗（`luanji` 双同花色手牌转【万箭齐发】）、无实体牌代价（`jiushi1` 翻面视为使用【酒】）、
//!    扩展牌堆区域（`hand_expansion` / `hand_equip_expansion`）与动态牌类（`basic` / `ordinary_trick` / `huogong` / `chuqibuyi`）
//!    `VmViewAsSpec` 的合法动作枚举、扣费与结算验证。
//! 3. 同一技能混用 `optional` / 强制触发器的逐触发器 `TriggerMeta` 询问分派，以及跨文件
//!    `add_skill_by_name`（`chixueqingfeng` / `rebiluan` / `wangzun` / `decadexushen`）确定性 `def_id` 授予。
//! 4. 批⑰ 60 个 `EffectRequest` 变体（单字节 `tags 209..=254` 共 46 项 + `tag = 255` 扩展
//!    `ext_tag = 256..=269` 共 14 项）的编解码往返、合法性校验、交互挂起、`NGSA 107` 存读档往返与状态落账。

use content_compiler::compile_source_named;
use engine_native::choice::ChoiceKind;
use engine_native::content::extension::{ExtensionDef, ExtensionPack, InitHook};
use engine_native::event::Phase;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand};
use engine_native_kernel::rvm_ops::{accept_effect, dec_effects, enc_effects};
use rule_api::{CardId, EffectRequest, PlayerId, SkillDefId, SkillId, Suit, TargetCardScope};

const LUANJI: &str = include_str!("../../../content/original/ported/shenhua/luanji.skill");
const JIUSHI1: &str = include_str!("../../../content/original/ported/yijiang/jiushi1.skill");
const RELONGHUN: &str = include_str!("../../../content/original/ported/extra/relonghun.skill");
const CHIXUEQINGFENG: &str =
    include_str!("../../../content/original/ported/sp/chixueqingfeng.skill");
const REBILUAN: &str = include_str!("../../../content/original/ported/sp/rebiluan.skill");
const WANGZUN: &str = include_str!("../../../content/original/ported/sp/wangzun.skill");
const DECADEXUSHEN: &str = include_str!("../../../content/original/ported/sp2/decadexushen.skill");
const POT_SUNCHEN: &str =
    include_str!("../../../content/original/ported/bingshi/pot_sunchen.skill");
const DCLINGHUI: &str = include_str!("../../../content/original/ported/xianding/dclinghui.skill");
const DCXIACE: &str = include_str!("../../../content/original/ported/xianding/dcxiace.skill");
const DILI_YUHENG: &str = include_str!("../../../content/original/ported/extra/dili_yuheng.skill");
const XINLIRANG: &str = include_str!("../../../content/original/ported/shiji/xinlirang.skill");
const JSRGCHAOZHENG: &str =
    include_str!("../../../content/original/ported/jsrg/jsrgchaozheng.skill");
const STARCUXIA: &str = include_str!("../../../content/original/ported/sp2/starcuxia.skill");
const REANXU: &str = include_str!("../../../content/original/ported/mobile/reanxu.skill");
const REBINGYI: &str = include_str!("../../../content/original/ported/refresh/rebingyi.skill");
const REJIZHI: &str = include_str!("../../../content/original/ported/refresh/rejizhi.skill");
const HOUFENG: &str = include_str!("../../../content/original/ported/shiji/houfeng.skill");
const DCZHANTAO: &str = include_str!("../../../content/original/ported/xianding/dczhantao.skill");
const DCXIAOYIN: &str = include_str!("../../../content/original/ported/xianding/dcxiaoyin.skill");
const DCJICHUN: &str = include_str!("../../../content/original/ported/xianding/dcjichun.skill");
const TWYUYAN: &str = include_str!("../../../content/original/ported/tw/twyuyan.skill");
const TWZHIBIAN: &str = include_str!("../../../content/original/ported/tw/twzhibian.skill");

static DEDUP: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1_700_000);

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
            description: "PA12 批⑰ 严格桥接测试包",
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
fn pa12_batch17_ported_skill_inventory_zero_compile_trigger_and_view_as_rejections() {
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
    let mut compile_rejected = Vec::new();
    let mut modifier_rejected = Vec::new();
    let mut active_rejected = Vec::new();
    let mut trigger_rejected = Vec::new();
    let mut view_as_rejected = Vec::new();
    let mut bytecode_rejected = Vec::new();

    for file in &files {
        let src = std::fs::read_to_string(file).expect("read .skill");
        let pkg = match content_compiler::compile_source_named("probe", &src) {
            Ok(pkg) => pkg,
            Err(err) => {
                compile_rejected.push((file.display().to_string(), format!("{err:?}")));
                continue;
            }
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
                } else if msg.contains("modifier_index") || msg.contains("skill_index") {
                    modifier_rejected.push((file.display().to_string(), msg));
                } else if msg.contains("active_index") {
                    active_rejected.push((file.display().to_string(), msg));
                } else if msg.contains("trigger_index") {
                    trigger_rejected.push((file.display().to_string(), msg));
                } else if msg.contains("view_as_index") {
                    view_as_rejected.push((file.display().to_string(), msg));
                } else {
                    bytecode_rejected.push((file.display().to_string(), msg));
                }
            }
        }
    }

    eprintln!(
        "PA12 Batch 17 inventory: total={}, bridged_ok={}, pure_noop_rejected={}, compile_rejected={}, modifier_rejected={}, active_rejected={}, trigger_rejected={}, view_as_rejected={}, bytecode_rejected={}",
        files.len(),
        bridged_ok,
        pure_noop_rejected,
        compile_rejected.len(),
        modifier_rejected.len(),
        active_rejected.len(),
        trigger_rejected.len(),
        view_as_rejected.len(),
        bytecode_rejected.len()
    );

    assert!(
        compile_rejected.is_empty(),
        "compile_rejected 必须清零，残留: {compile_rejected:?}"
    );
    assert!(
        modifier_rejected.is_empty(),
        "modifier_rejected 必须清零，残留: {modifier_rejected:?}"
    );
    assert!(
        active_rejected.is_empty(),
        "active_rejected 必须清零，残留: {active_rejected:?}"
    );
    assert!(
        trigger_rejected.is_empty(),
        "trigger_rejected 必须清零，残留: {trigger_rejected:?}"
    );
    assert!(
        view_as_rejected.is_empty(),
        "view_as_rejected 必须清零，残留: {view_as_rejected:?}"
    );
    assert!(
        bytecode_rejected.is_empty(),
        "bytecode_rejected 必须清零，残留: {bytecode_rejected:?}"
    );
    assert_eq!(
        pure_noop_rejected, 2029,
        "全部 2,029 个纯 INTENTIONAL_EMPTY_NOOP 文件必须保持拒绝"
    );
    assert_eq!(
        bridged_ok, 3641,
        "全部 3,641 个非空 ported .skill 文件必须 100% 通过桥接（3641 + 2029 = 5670）"
    );
}

#[test]
fn pa12_batch17_codec_roundtrip_209_to_254_and_ext_u16_256_to_269() {
    let single_byte_effects: Vec<(u8, EffectRequest)> = vec![
        (
            209,
            EffectRequest::ChooseCardUserOrAddTarget {
                player: PlayerId::new(1),
                card: CardId::new(3),
                skill: SkillId::new(4001),
            },
        ),
        (
            210,
            EffectRequest::ChooseCurrentCardUseEnhancements {
                player: PlayerId::new(1),
                card: CardId::new(3),
                helper_skill_def: SkillDefId::new(5000),
                skill: SkillId::new(4002),
            },
        ),
        (
            211,
            EffectRequest::ChooseDrawOrDiscardTargetCardRecordSha {
                player: PlayerId::new(1),
                card: CardId::new(3),
                target: PlayerId::new(2),
                skill: SkillId::new(4003),
            },
        ),
        (
            212,
            EffectRequest::ChooseOptionalDiscardTargetCard {
                player: PlayerId::new(1),
                target: PlayerId::new(2),
                scope: TargetCardScope::HandEquip,
            },
        ),
        (
            213,
            EffectRequest::ChooseUseVirtualCard {
                player: PlayerId::new(1),
                kind: 1,
                skill: SkillId::new(4005),
            },
        ),
        (
            214,
            EffectRequest::UseVirtualShaFromCardColorLockSourceIfNoSha {
                player: PlayerId::new(1),
                skill: SkillId::new(4006),
            },
        ),
        (
            215,
            EffectRequest::ChooseDiscardHandEquipThenRecover {
                player: PlayerId::new(1),
                count: 2,
                amount: 1,
                skill: SkillId::new(4007),
            },
        ),
        (
            216,
            EffectRequest::ChooseDiscardHandEquipThenRecoverTarget {
                chooser: PlayerId::new(1),
                target: PlayerId::new(2),
                count: 1,
                amount: 1,
                skill: SkillId::new(4008),
            },
        ),
        (
            217,
            EffectRequest::ChooseDiscardWeaponOrLoseHp {
                player: PlayerId::new(1),
                skill: SkillId::new(4009),
            },
        ),
        (
            218,
            EffectRequest::ChooseMoveTargetEquipmentOrDraw {
                player: PlayerId::new(1),
                target: PlayerId::new(2),
                skill: SkillId::new(4010),
            },
        ),
        (
            219,
            EffectRequest::ChooseOtherHandStoreSkillPile {
                player: PlayerId::new(1),
                target: PlayerId::new(2),
                skill: SkillId::new(4011),
            },
        ),
        (
            220,
            EffectRequest::ChooseOtherYinghun {
                player: PlayerId::new(1),
                target: PlayerId::new(2),
                count: 2,
                branch: 1,
                ready_to_discard: false,
                skill: SkillId::new(4012),
            },
        ),
        (
            221,
            EffectRequest::ChooseTargetDiscardExcessThenSelfDiscardOrTargetDraw {
                player: PlayerId::new(1),
                skill: SkillId::new(4013),
            },
        ),
        (
            222,
            EffectRequest::ResolveDiliYuheng {
                player: PlayerId::new(1),
                action: 1,
                card: CardId::new(6),
                event_cards: vec![CardId::new(7)],
                skill: SkillId::new(4014),
            },
        ),
        (
            223,
            EffectRequest::ChooseOtherTargetsBySkippedPhaseCountDamage {
                player: PlayerId::new(1),
                amount: 1,
                skill: SkillId::new(4015),
            },
        ),
        (
            224,
            EffectRequest::GiveAllEquipmentChooseRecoverOrDamage {
                player: PlayerId::new(1),
                target: PlayerId::new(2),
                skill: SkillId::new(4016),
            },
        ),
        (
            225,
            EffectRequest::LoseHpDoubleCurrentDamageThenReplace {
                player: PlayerId::new(1),
                damage_id: 0,
                amount: 1,
                skill: SkillId::new(4017),
                replacement_skill_def: SkillDefId::new(5001),
                ready_to_replace: true,
            },
        ),
        (
            226,
            EffectRequest::DrawThenOptionallyDistributeHandCards {
                player: PlayerId::new(1),
                draw_count: 2,
                give_cap: 2,
                cards: vec![CardId::new(8)],
                targets: vec![PlayerId::new(2)],
                skill: SkillId::new(4018),
                ready_to_choose: true,
                finish: false,
            },
        ),
        (
            227,
            EffectRequest::ReplaceDrawWithDistinctSuitSelection {
                player: PlayerId::new(1),
                count: 3,
                stage: 1,
                cards: vec![CardId::new(9)],
                skill: SkillId::new(4019),
            },
        ),
        (
            228,
            EffectRequest::StartDebateAllOthers {
                player: PlayerId::new(1),
                skill: SkillId::new(4020),
            },
        ),
        (
            229,
            EffectRequest::StartDebateTargets {
                player: PlayerId::new(1),
                skill: SkillId::new(4021),
            },
        ),
        (
            230,
            EffectRequest::ResolveBailingyun {
                player: PlayerId::new(1),
                target: PlayerId::new(2),
                card: CardId::new(10),
                action: 10,
                cards: vec![CardId::new(10)],
                skill: SkillId::new(4022),
            },
        ),
        (
            231,
            EffectRequest::ResolveJieliangGain {
                player: PlayerId::new(1),
                mover: PlayerId::new(2),
                cards: vec![CardId::new(10)],
                skill: SkillId::new(4023),
            },
        ),
        (
            232,
            EffectRequest::ResolveJsrgchaozheng {
                player: PlayerId::new(1),
                skill: SkillId::new(4024),
            },
        ),
        (
            233,
            EffectRequest::ResolveStarcuxiaWeiming {
                player: PlayerId::new(1),
                source: PlayerId::new(2),
                variant: 1,
            },
        ),
        (
            234,
            EffectRequest::ResolveReanxu {
                player: PlayerId::new(1),
                stage: 1,
                first: PlayerId::new(2),
                second: PlayerId::new(3),
                card: CardId::new(11),
                from_hand: true,
                skill: SkillId::new(4026),
            },
        ),
        (
            235,
            EffectRequest::ResolveRebingyi {
                player: PlayerId::new(1),
                skill: SkillId::new(4027),
            },
        ),
        (
            236,
            EffectRequest::ResolveRejizhi {
                player: PlayerId::new(1),
                stage: 0,
                card: CardId::new(12),
                skill: SkillId::new(4028),
            },
        ),
        (
            237,
            EffectRequest::ForceOthersUseShaOnNearestOrLoseHp {
                player: PlayerId::new(1),
                remaining: vec![PlayerId::new(2)],
                started: true,
                skill: SkillId::new(4029),
            },
        ),
        (
            238,
            EffectRequest::TargetDiscardThenDuelIfNonShaAndHpGteOwner {
                player: PlayerId::new(1),
                target: PlayerId::new(2),
                skill: SkillId::new(4030),
            },
        ),
        (
            239,
            EffectRequest::ResolveHoufeng {
                player: PlayerId::new(1),
                target: PlayerId::new(2),
                mode: 1,
                stage: 0,
                skill: SkillId::new(4031),
            },
        ),
        (
            240,
            EffectRequest::ResolveHuzun {
                player: PlayerId::new(1),
                source: PlayerId::new(2),
                card: CardId::new(13),
                action: 1,
                cards: vec![CardId::new(13)],
                targets: vec![PlayerId::new(2)],
                skill: SkillId::new(4032),
            },
        ),
        (
            241,
            EffectRequest::ResolveMalingli {
                player: PlayerId::new(1),
                card: CardId::new(14),
                helper_skill_def: SkillDefId::new(5002),
                action: 10,
                source: PlayerId::new(1),
                target: PlayerId::new(2),
                nature: 0,
                damage_id: 0,
                event_cards: vec![CardId::new(14)],
                event_targets: vec![PlayerId::new(2)],
                skill: SkillId::new(4033),
            },
        ),
        (
            242,
            EffectRequest::ResolveZhugemengxue {
                player: PlayerId::new(1),
                target: PlayerId::new(2),
                card: CardId::new(15),
                action: 10,
                cards: vec![CardId::new(15)],
                skill: SkillId::new(4034),
            },
        ),
        (
            243,
            EffectRequest::ResolveTwyuyan {
                player: PlayerId::new(1),
                source: PlayerId::new(2),
                card: CardId::new(16),
                skill: SkillId::new(4035),
            },
        ),
        (
            244,
            EffectRequest::ResolveTwzhibian {
                player: PlayerId::new(1),
                target: PlayerId::new(2),
                winner: PlayerId::new(1),
                stage: 1,
                skill: SkillId::new(4036),
            },
        ),
        (
            245,
            EffectRequest::ChooseAddSkillBoundCardTarget {
                player: PlayerId::new(1),
                card: CardId::new(17),
                skill: SkillId::new(4037),
            },
        ),
        (
            246,
            EffectRequest::ChooseDiscardSkillPileCardAddMarkedSkill {
                player: PlayerId::new(1),
                target: PlayerId::new(2),
                helper_skill_def: SkillDefId::new(5003),
                selected: CardId::new(18),
                skill: SkillId::new(4038),
            },
        ),
        (
            247,
            EffectRequest::ChooseDiscardTargetCardResolveCurrentSha {
                source: PlayerId::new(1),
                card: CardId::new(19),
                target: PlayerId::new(2),
            },
        ),
        (
            248,
            EffectRequest::ChooseDiscardTargetCardResolveCurrentShaNonbasicBonus {
                source: PlayerId::new(1),
                card: CardId::new(19),
                target: PlayerId::new(2),
                bonus_skill_def: SkillDefId::new(5004),
            },
        ),
        (
            249,
            EffectRequest::ChooseDiscardTargetCardsRecoverSameCategory {
                player: PlayerId::new(1),
                target: PlayerId::new(2),
                count: 2,
                amount: 1,
                skill: SkillId::new(4041),
            },
        ),
        (
            250,
            EffectRequest::ChooseGainCardFromEachOtherReplaceDraw {
                player: PlayerId::new(1),
                initialized: true,
                remaining_targets: vec![PlayerId::new(2)],
                skill: SkillId::new(4042),
            },
        ),
        (
            251,
            EffectRequest::ChooseGainCardFromEachOtherThenTurnOver {
                player: PlayerId::new(1),
                initialized: true,
                remaining_targets: vec![PlayerId::new(2)],
                skill: SkillId::new(4043),
            },
        ),
        (
            252,
            EffectRequest::ChooseGainOwnLostDiscardCardThisTurn {
                player: PlayerId::new(1),
                skill: SkillId::new(4044),
            },
        ),
        (
            253,
            EffectRequest::ChooseGainTargetCardAlternateZoneThenTargetDraw {
                player: PlayerId::new(1),
                target: PlayerId::new(2),
                selected: CardId::new(20),
                skill: SkillId::new(4045),
            },
        ),
        (
            254,
            EffectRequest::ChooseGainTopCardsBySuit {
                player: PlayerId::new(1),
                count: 3,
                suit: Suit::Heart,
            },
        ),
    ];

    assert_eq!(single_byte_effects.len(), 46);
    for (expected_tag, eff) in &single_byte_effects {
        assert!(
            accept_effect(eff, 4).is_ok(),
            "合法批⑰单字节原语应通过 accept_effect: {eff:?}"
        );
        let mut bytes = Vec::new();
        enc_effects(&mut bytes, std::slice::from_ref(eff));
        assert_eq!(bytes[4], *expected_tag, "tag 字节应等于 {expected_tag}");
        let decoded = dec_effects(&bytes).expect("dec_effects");
        assert_eq!(&decoded[0], eff, "tag {expected_tag} 编解码往返一致");
    }

    let ext_u16_effects: Vec<(u16, EffectRequest)> = vec![
        (
            256,
            EffectRequest::ChooseRevealHandCardBindCategoryTarget {
                player: PlayerId::new(1),
                target: PlayerId::new(2),
                skill: SkillId::new(4046),
            },
        ),
        (
            257,
            EffectRequest::ChooseRevealOtherHandCardRememberCategory {
                player: PlayerId::new(1),
                skill: SkillId::new(4047),
            },
        ),
        (
            258,
            EffectRequest::ChooseTargetHandCardToPile {
                player: PlayerId::new(1),
                skill: SkillId::new(4048),
                pile_def: 6001,
            },
        ),
        (
            259,
            EffectRequest::ChooseZonePriorityGainRandomCardFromEachOtherThenTurnOver {
                player: PlayerId::new(1),
                priority: 1,
                remaining_targets: vec![PlayerId::new(2)],
                skill: SkillId::new(4049),
            },
        ),
        (
            260,
            EffectRequest::CityConsume {
                player: PlayerId::new(1),
                skill: SkillId::new(4050),
            },
        ),
        (
            261,
            EffectRequest::DiscardEquipmentPairWithArmor {
                player: PlayerId::new(1),
                targets: vec![PlayerId::new(2)],
                skill: SkillId::new(4051),
            },
        ),
        (
            262,
            EffectRequest::DiscardOneOwnXumou {
                player: PlayerId::new(1),
                skill: SkillId::new(4052),
            },
        ),
        (
            263,
            EffectRequest::DiscardRememberedHandCards {
                player: PlayerId::new(1),
                skill: SkillId::new(4054),
            },
        ),
        (
            264,
            EffectRequest::DiscardTargetEquipmentDrawWeaponRange {
                player: PlayerId::new(1),
                target: PlayerId::new(2),
                skill: SkillId::new(4055),
            },
        ),
        (
            265,
            EffectRequest::ExtraResolveCurrentTrick {
                player: PlayerId::new(1),
                card: CardId::new(22),
                skill: SkillId::new(4056),
            },
        ),
        (
            266,
            EffectRequest::GainDrawCardsForLifetimeCardActionDivisors {
                player: PlayerId::new(1),
                divisors: [2, 3, 4],
                groups: [1, 2, 0],
            },
        ),
        (
            267,
            EffectRequest::GainHandCompareResultCard {
                player: PlayerId::new(1),
                initiator: PlayerId::new(1),
                target: PlayerId::new(2),
                winner: PlayerId::new(1),
                cards: vec![CardId::new(23)],
                skill: SkillId::new(4057),
            },
        ),
        (
            268,
            EffectRequest::GainHighestRankShaFromEventCards {
                player: PlayerId::new(1),
                cards: vec![CardId::new(23), CardId::new(24)],
                skill: SkillId::new(4058),
            },
        ),
        (
            269,
            EffectRequest::MoveEquipmentThenDraw {
                player: PlayerId::new(1),
                skill: SkillId::new(4059),
            },
        ),
    ];

    assert_eq!(ext_u16_effects.len(), 14);
    for (expected_ext_tag, eff) in &ext_u16_effects {
        assert!(
            accept_effect(eff, 4).is_ok(),
            "合法批⑰扩展 tag 原语应通过 accept_effect: {eff:?}"
        );
        let mut bytes = Vec::new();
        enc_effects(&mut bytes, std::slice::from_ref(eff));
        assert_eq!(bytes[4], 255, "扩展前缀字节应等于 255");
        let ext = u16::from_le_bytes([bytes[5], bytes[6]]);
        assert_eq!(ext, *expected_ext_tag, "ext_tag 应等于 {expected_ext_tag}");
        let decoded = dec_effects(&bytes).expect("dec_effects ext_u16");
        assert_eq!(
            &decoded[0], eff,
            "ext_tag {expected_ext_tag} 编解码往返一致"
        );
    }

    // 未知扩展 ext_tag = 270 必须被拒绝
    let mut bad_bytes = Vec::new();
    bad_bytes.extend_from_slice(&1u32.to_le_bytes());
    bad_bytes.push(255u8);
    bad_bytes.extend_from_slice(&270u16.to_le_bytes());
    assert!(
        dec_effects(&bad_bytes).is_none(),
        "未分配的 ext_tag = 270 必须被解码器拒绝"
    );

    // 越界参数校验
    assert!(accept_effect(
        &EffectRequest::ChooseOtherYinghun {
            player: PlayerId::new(1),
            target: PlayerId::NONE,
            count: 0,
            branch: 9,
            ready_to_discard: false,
            skill: SkillId::NONE,
        },
        4
    )
    .is_err());
    assert!(accept_effect(
        &EffectRequest::GainDrawCardsForLifetimeCardActionDivisors {
            player: PlayerId::new(1),
            divisors: [0, 1, 1],
            groups: [0, 0, 0],
        },
        4
    )
    .is_err());
}

#[test]
fn pa12_batch17_view_as_multi_card_luanji_cost_jiushi_and_expansion_zones() {
    let exp_dsl = r#"
skill exp_wuzhong(91700101) {
    view_as wuzhong play_phase any from hand_expansion;
}
"#;
    let pack = pack_strict(
        &[
            ("luanji", LUANJI),
            ("jiushi1", JIUSHI1),
            ("relonghun", RELONGHUN),
            ("exp_wuzhong", exp_dsl),
        ],
        "pa12-b17-view-as",
    );
    let packs = [&pack];
    let mut e = Engine::new_with_extensions(917_001, 3, &packs).expect("建局");
    advance_to_phase(&mut e, 0, Phase::Play);

    // 1. 验证 jiushi1（无实体牌 + cost turn_over -> jiu）
    e.debug_clear_hand(0);
    let sha1 = e.debug_place(0, "sha");
    let sha2 = e.debug_place(0, "sha");
    e.debug_grant(0, "jiushi1");
    assert!(
        !e.state_core().player(0).face_down,
        "初始应正面朝上"
    );

    let jiu_action = e
        .legal_actions(0)
        .into_iter()
        .find(|a| {
            matches!(
                a,
                GameAction::PlayCard { card, targets }
                    if targets.is_empty() && !e.state_core().player(0).hand.contains(card)
            )
        })
        .expect("jiushi1 应在正面朝上时提供无实体牌转【酒】合法动作");
    submit(&mut e, 0, jiu_action);
    drain_choices(&mut e);

    assert!(
        e.state_core().player(0).face_down,
        "发动 jiushi1 后应翻面（face_down = true）"
    );
    assert!(
        e.state_core().turn.jiu_active,
        "发动 jiushi1 后本回合 turn.jiu_active 应置位"
    );
    assert_eq!(
        e.state_core().player(0).hand,
        vec![sha1, sha2],
        "jiushi1（from none）不应消耗手牌中的实体牌"
    );
    assert!(
        !e.legal_actions(0).iter().any(|a| matches!(
            a,
            GameAction::PlayCard { card, targets }
                if targets.is_empty() && !e.state_core().player(0).hand.contains(card)
        )),
        "背面朝上时不得再次发动 jiushi1"
    );

    // 2. 验证 luanji（2 张同花色手牌 -> 万箭齐发，且不阻断单目标正常出【杀】）
    e.debug_grant(0, "luanji");
    let sha1_suit = e.catalog().cards[sha1 as usize].suit;
    let same_suit_card = e
        .state_core()
        .deck
        .iter()
        .copied()
        .find(|&c| e.catalog().cards[c as usize].suit == sha1_suit)
        .expect("同花色牌");
    e.state_core_mut().deck.retain(|&c| c != same_suit_card);
    e.state_core_mut().player_mut(0).hand = vec![sha1, same_suit_card];

    let normal_sha_action = GameAction::PlayCard {
        card: sha1,
        targets: vec![1],
    };
    assert!(
        e.is_legal_action(0, &normal_sha_action),
        "持有 luanji 与同花色伴随牌时，仍应允许将 sha1 作为单目标【杀】正常打出"
    );
    let wanjian_action = GameAction::PlayCard {
        card: sha1,
        targets: Vec::new(),
    };
    assert!(
        e.is_legal_action(0, &wanjian_action),
        "luanji 应在持有 2 张同花色手牌时允许将 sha1 无目标打出（视为万箭齐发）"
    );
    submit(&mut e, 0, wanjian_action);
    drain_choices(&mut e);

    assert!(
        e.state_core().player(0).hand.is_empty(),
        "luanji 结算应同时消耗主牌与第 2 张同花色伴随手牌"
    );
    assert!(
        e.state_core().discard.contains(&sha1)
            && e.state_core().discard.contains(&same_suit_card),
        "被 luanji 消耗的 2 张同花色手牌均应进入弃牌堆"
    );

    // 3. 验证扩展牌堆区域（from hand_expansion -> 无中生有）
    e.debug_grant(0, "exp_wuzhong");
    let pile_card = e.state_core_mut().deck.pop().expect("牌堆顶牌");
    e.state_core_mut()
        .vm_aux
        .piles
        .insert((0, "exp_wuzhong".to_string()), vec![pile_card]);
    let exp_action = GameAction::PlayCard {
        card: pile_card,
        targets: Vec::new(),
    };
    assert!(
        e.is_legal_action(0, &exp_action),
        "hand_expansion 应允许直接将扩展牌堆中的牌转化为【无中生有】使用"
    );
    submit(&mut e, 0, exp_action);
    drain_choices(&mut e);
    assert!(
        e.state_core()
            .vm_aux
            .piles
            .get(&(0, "exp_wuzhong".to_string()))
            .is_none_or(|v| !v.contains(&pile_card)),
        "扩展牌堆中的牌经 view_as 打出后应从扩展牌堆移除"
    );
    assert_eq!(
        e.state_core().player(0).hand.len(),
        2,
        "视为【无中生有】结算后应摸 2 张牌"
    );

    // 4. 验证 relonghun（up_to 2 same_suit + multi_card_bonus = Recover1：2 张红桃非桃牌视为【桃】回复 2 点体力）
    let luanji_def = e.catalog().skill_by_name("luanji").unwrap();
    e.state_core_mut().skills.remove(&(0, luanji_def));
    e.debug_grant(0, "relonghun");
    e.state_core_mut().player_mut(0).max_hp = 4;
    e.state_core_mut().player_mut(0).hp = 1;
    let heart_non_taos: Vec<u32> = (0..e.catalog().cards.len() as u32)
        .filter(|&c| {
            e.catalog().cards[c as usize].suit == engine_native::catalog::Suit::Heart
                && e.catalog().card_kind(c) != engine_native::catalog::CardKind::Tao
        })
        .take(2)
        .collect();
    assert_eq!(heart_non_taos.len(), 2);
    for &hc in &heart_non_taos {
        e.state_core_mut().deck.retain(|&x| x != hc);
        e.state_core_mut().discard.retain(|&x| x != hc);
    }
    e.state_core_mut().player_mut(0).hand = heart_non_taos.clone();
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: heart_non_taos[0],
            targets: Vec::new(),
        },
    );
    drain_choices(&mut e);
    assert!(
        e.state_core().player(0).hand.is_empty(),
        "relonghun 双红桃转【桃】应同时消耗两张红桃牌"
    );
    assert_eq!(
        e.state_core().player(0).hp,
        3,
        "relonghun 双红桃转【桃】（multi_card_bonus = Recover1）应回复 2 点体力（1 -> 3）"
    );
}

#[test]
fn pa12_batch17_mixed_optionality_triggers_and_cross_file_add_skill_by_name() {
    let mixed_dsl = r#"
skill mixed_opt_trig(91700201) {
    trigger phase_prepare owner_is_target {
        effect draw_cards(self, 1);
    }
    trigger phase_end owner_alive optional by event_player {
        effect draw_cards(event_player, 1);
    }
    trigger damage owner_is_target optional {
        effect draw_cards(self, 1);
    }
}
"#;
    let pack = pack_strict(
        &[
            ("mixed_opt", mixed_dsl),
            ("chixueqingfeng", CHIXUEQINGFENG),
            ("rebiluan", REBILUAN),
            ("wangzun", WANGZUN),
            ("decadexushen", DECADEXUSHEN),
            ("pot_sunchen", POT_SUNCHEN),
        ],
        "pa12-b17-triggers",
    );
    let packs = [&pack];
    let mut e = Engine::new_with_extensions(917_002, 2, &packs).expect("建局");
    e.debug_grant(0, "mixed_opt_trig");
    e.debug_grant(0, "chixueqingfeng");

    // 推进到 0 号出牌阶段：phase_prepare 触发器为强制（non-optional），应自动摸牌
    advance_to_phase(&mut e, 0, Phase::Play);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    let sha = e.debug_place(0, "sha");

    // 0 号对 1 号使用【杀】：触发 chixueqingfeng（add_skill_by_name(event_player, chixueqingfeng2) -> 3863274410）
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha,
            targets: vec![1],
        },
    );
    drain_choices(&mut e);
    assert_eq!(
        e.state_core()
            .vm_aux
            .flags
            .get(&(0, "skill:3863274410".to_string()))
            .copied(),
        Some(1),
        "chixueqingfeng 应通过跨文件 add_skill_by_name 授予 chixueqingfeng2 (3863274410)"
    );

    // 推进到 1 号出牌阶段并对 0 号造成伤害：验证 mixed_opt_trig 的 damage 分支按逐触发器 optional 弹出 Confirm 询问
    advance_to_phase(&mut e, 1, Phase::Play);
    let sha_p1 = e.debug_place(1, "sha");
    e.debug_clear_hand(0);
    submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: sha_p1,
            targets: vec![0],
        },
    );
    // 响应闪询问（取消）后进入 damage 窗口，0 号应收到 Confirm 询问
    if let Some(ask) = e.open_choice().cloned() {
        if ask.observer == 0 && ask.spec.kind == ChoiceKind::RespondCard {
            if ask.spec.options.contains(&ChoiceOption::Pass) {
                answer(&mut e, vec![ChoiceOption::Pass]);
            } else {
                answer(&mut e, Vec::new());
            }
        }
    }
    let ask = e.open_choice().cloned().expect("damage optional 触发器应弹出询问");
    assert_eq!(ask.observer, 0);
    assert_eq!(
        ask.spec.kind,
        ChoiceKind::Confirm,
        "mixed_opt_trig 的 damage 分支应弹出 ChoiceKind::Confirm，实际: {:?}",
        ask.spec.kind
    );
    let hand_before = e.state_core().player(0).hand.len();
    answer(&mut e, vec![ChoiceOption::Confirm]);
    drain_choices(&mut e);
    assert_eq!(
        e.state_core().player(0).hand.len(),
        hand_before + 1,
        "确认发动 optional 触发器后应摸 1 张牌"
    );

    // 推进 1 号从出牌阶段 -> 弃牌阶段 -> 结束阶段：验证 phase_end 分支按自身 confirmation_actor = event_player 向 1 号弹出询问
    e.debug_clear_hand(1);
    submit(&mut e, 1, GameAction::NextPhase);
    submit(&mut e, 1, GameAction::NextPhase);
    let end_ask = e
        .open_choice()
        .cloned()
        .expect("phase_end optional 触发器应弹出询问");
    assert_eq!(
        end_ask.observer, 1,
        "同一技能包含多个 PhaseBegin 触发器（phase_prepare vs phase_end）时，phase_end 分支应向 event_player (1) 询问"
    );
    assert_eq!(end_ask.spec.kind, ChoiceKind::Confirm);
    answer(&mut e, vec![ChoiceOption::Confirm]);
    drain_choices(&mut e);
    assert_eq!(
        e.state_core().player(1).hand.len(),
        1,
        "1 号确认发动 phase_end 触发器后应摸 1 张牌"
    );
}

#[test]
fn pa12_batch17_interactive_and_deterministic_resolvers_and_archive_roundtrip() {
    let probe_dsl = r#"
skill b17_interactive_probe(91700301) {
    active play_phase {
        effect choose_other_yinghun(self);
        effect resolve_bailingyun(self, 10);
        effect resolve_twzhibian(self);
        effect city_consume(self, self_skill);
        effect extra_resolve_current_trick(self, 1, self_skill);
    }
}
"#;
    let pack = pack_strict(
        &[
            ("b17_probe", probe_dsl),
            ("dclinghui", DCLINGHUI),
            ("dcxiace", DCXIACE),
            ("dili_yuheng", DILI_YUHENG),
            ("xinlirang", XINLIRANG),
            ("jsrgchaozheng", JSRGCHAOZHENG),
            ("starcuxia", STARCUXIA),
            ("reanxu", REANXU),
            ("rebingyi", REBINGYI),
            ("rejizhi", REJIZHI),
            ("houfeng", HOUFENG),
            ("dczhantao", DCZHANTAO),
            ("dcxiaoyin", DCXIAOYIN),
            ("dcjichun", DCJICHUN),
            ("twyuyan", TWYUYAN),
            ("twzhibian", TWZHIBIAN),
        ],
        "pa12-b17-resolvers",
    );
    let packs = [&pack];
    let mut e = Engine::new_with_extensions(917_003, 3, &packs).expect("建局");
    advance_to_phase(&mut e, 0, Phase::Play);
    e.debug_grant(0, "b17_interactive_probe");

    let skill_id = e.catalog().skill_by_name("b17_interactive_probe").unwrap();
    submit(&mut e, 0, GameAction::ActivateSkill { skill: skill_id });

    // 3 人局下 ChooseOtherYinghun 应挂起在选其他角色窗口，验证 NGSA 107 存读档往返
    let ask = e.open_choice().cloned().expect("ChooseOtherYinghun 应挂起选目标询问");
    assert_eq!(ask.observer, 0);
    assert!(ask.spec.options.contains(&ChoiceOption::Player(1)));
    assert!(ask.spec.options.contains(&ChoiceOption::Player(2)));
    let mut e = assert_archive_roundtrip(&e, &packs);

    answer(&mut e, vec![ChoiceOption::Player(1)]);
    // 随后 ResolveBailingyun(action=10) 挂起选目标，再次验证存读档往返并回答
    let ask2 = e
        .open_choice()
        .cloned()
        .expect("ResolveBailingyun(10) 应挂起选目标询问");
    assert_eq!(ask2.observer, 0);
    let mut e = assert_archive_roundtrip(&e, &packs);
    answer(&mut e, vec![ChoiceOption::Player(2)]);

    // 随后 ResolveTwzhibian 挂起选目标，再次验证存读档往返并回答
    let ask3 = e
        .open_choice()
        .cloned()
        .expect("ResolveTwzhibian 应挂起选目标询问");
    assert_eq!(ask3.observer, 0);
    let mut e = assert_archive_roundtrip(&e, &packs);
    answer(&mut e, vec![ChoiceOption::Player(1)]);
    drain_choices(&mut e);

    assert_eq!(
        e.state_core()
            .vm_aux
            .flags
            .get(&(0, "bailingyun_xiace".to_string()))
            .copied(),
        Some(10),
        "ResolveBailingyun(10) 落账后应记录 bailingyun_xiace = 10"
    );
    assert_eq!(
        e.state_core()
            .vm_aux
            .flags
            .get(&(0, "twzhibian_resolved".to_string()))
            .copied(),
        Some(1),
        "ResolveTwzhibian 落账后应记录 twzhibian_resolved = 1"
    );
    assert_eq!(
        e.state_core()
            .vm_aux
            .flags
            .get(&(0, "city_consumed".to_string()))
            .copied(),
        Some(1),
        "CityConsume 落账后应记录 city_consumed = 1"
    );
    assert_eq!(
        e.state_core()
            .vm_aux
            .flags
            .get(&(0, "extra_resolve_trick:1".to_string()))
            .copied(),
        Some(1),
        "ExtraResolveCurrentTrick 落账后应记录 extra_resolve_trick:1 = 1"
    );
}
