//! PA12 迁移批⑦（2026-11-06，零版本号变更）：
//! 1. `rvm_ops::codec` 补齐 `accept_effect` 已放行但编码面遗漏的 5 个 `EffectRequest` 变体
//!    （`TurnOver`=27、`PreventDamage`=28、`ModifyCurrentDamage`=29、`SkipCurrentPhase`=36、
//!    `StorageSet`=37），使 `accept_effect` 放行的全部 37 个变体 100% 往返可编码，消除
//!    `discard_hand_equip_count` 等提交期开窗挂起后续跟 `modify_current_damage`（如真实技能
//!    `sp/mingshi.skill`）在 `save()` 快照时触发 `unreachable!` 的崩溃。
//! 2. `rvm_named::phrase` 补齐主语位 `other_*` 与 `targeted_by_other` 复合短语
//!    （`targeted_by_other`、`other_dying`、`other_damage_before`/`other_damage`、
//!    `other_direct_damage_before`、`other_phase_end`、`other_phase_play`、`other_phase_begin`、
//!    `other_judge_phase`、`other_recovered`），避免主语位 `other` 被单字切分为 `Atom::OtherAlive`
//!    而遗漏 `event_player != owner` / `UserOther` 反例门。
//! 3. 批③b 可选触发放弃确认（Decline）与 `once`（scope 1）/ `once_per_game`（scope 3）语义核验：
//!    放弃 `trigger_confirm` 不写入 `vm_aux.once`，同回合/同局再次满足条件仍可触发；确认后
//!    消耗配额。

use content_compiler::compile_source_named;
use engine_native::content::extension::{ExtensionDef, ExtensionPack, InitHook};
use engine_native::event::DamageNature;
use engine_native::{ChoiceOption, Engine, GameAction, GameCommand};
use engine_native_kernel::rvm_ops::{accept_effect, dec_effects, enc_effects};
use rule_api::{CardId, EffectRequest, MarkDefId, PlayerId, SkillDefId, SkillId, ZoneId};

const MINGSHI: &str = include_str!("../../../content/original/ported/sp/mingshi.skill");
const SHIROHA_GUYING: &str =
    include_str!("../../../content/original/ported/key/shiroha_guying.skill");
const XIEMU2: &str = include_str!("../../../content/original/ported/sp/xiemu2.skill");
const DCZHANGJI: &str = include_str!("../../../content/original/ported/xianding/dczhangji.skill");
const OLBUYI: &str = include_str!("../../../content/original/ported/onlyOL/olbuyi.skill");
const LIANGFAN2: &str = include_str!("../../../content/original/ported/offline/liangfan2.skill");

fn pack_for(sources: &[(&str, &str)], pack_name: &'static str) -> ExtensionPack {
    let mut vm_skills = Vec::new();
    for (id, source) in sources {
        let pkg = compile_source_named(*id, source).expect("编译真实 ported .skill");
        assert!(
            pkg.manifest.rule_vm_abi_version <= rule_vm::RULE_VM_ABI_VERSION,
            "批⑦不引入新 ABI：{} > {}",
            pkg.manifest.rule_vm_abi_version,
            rule_vm::RULE_VM_ABI_VERSION
        );
        let out = content_native_loader::bridge_vm_skill_decls(&pkg).expect("正式桥接");
        vm_skills.extend(out.decls);
    }
    ExtensionPack {
        def: ExtensionDef {
            name: pack_name,
            version: 1,
            description: "PA12 批⑦ 编码完备、具名短语与次数门",
        },
        generals: Vec::new(),
        cards: Vec::new(),
        host_skills: Vec::new(),
        vm_skills,
        init_hook: InitHook::None,
        enabled: true,
    }
}

fn submit(e: &mut Engine, actor: u8, action: GameAction) {
    let rev = e.revision();
    let out = e.submit(GameCommand {
        actor,
        expected_revision: rev,
        dedup_id: (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64)
            | 1,
        action: action.clone(),
    });
    assert!(
        !matches!(out, engine_native::CommandOutcome::Rejected(_)),
        "命令不应被拒绝: actor={actor}, action={action:?}, out={out:?}"
    );
}

fn hand_len(e: &Engine, p: u8) -> usize {
    e.state_core().player(p).hand.len()
}

/// 1. `accept_effect` 放行的全部 37 个 `EffectRequest` 变体 100% 往返编解码通过。
#[test]
fn pa12_batch7_codec_all_37_accepted_effects_round_trip() {
    let p1 = PlayerId::new(1);
    let p2 = PlayerId::new(2);
    let s1 = SkillId::new(7);
    let sd1 = SkillDefId::new(11);
    let m1 = MarkDefId::new(13);
    let c1 = CardId::new(5);
    let z1 = ZoneId::new((1 << 8) | 3);
    let z2 = ZoneId::new((2 << 8) | 3);

    let all_37 = vec![
        EffectRequest::RecoverHp {
            source: p1,
            player: p2,
            amount: 1,
        },
        EffectRequest::DrawCards {
            player: p1,
            count: 2,
        },
        EffectRequest::DealDamage {
            source: p1,
            target: p2,
            amount: 1,
            is_lose_hp: false,
            nature: 1,
        },
        EffectRequest::ChangeMaxHp {
            player: p1,
            delta: -1,
        },
        EffectRequest::AddMark {
            player: p1,
            mark: m1,
            delta: 2,
        },
        EffectRequest::RemoveMark {
            player: p1,
            mark: m1,
            amount: 1,
        },
        EffectRequest::DiscardFromHand {
            player: p1,
            count: 1,
        },
        EffectRequest::DiscardFromHandEquip {
            player: p1,
            count: 2,
        },
        EffectRequest::AddSkill {
            player: p1,
            skill_def: sd1,
        },
        EffectRequest::RemoveSkill { skill: s1 },
        EffectRequest::GainTopCardToSkillPile {
            player: p1,
            skill: s1,
        },
        EffectRequest::GainWholeSkillPile {
            player: p1,
            skill: s1,
            pile_def: 42,
        },
        EffectRequest::GainAllSkillPileCards {
            player: p1,
            skill: s1,
        },
        EffectRequest::GainFromSkillPile {
            player: p1,
            skill: s1,
        },
        EffectRequest::DiscardSkillPile {
            player: p1,
            skill: s1,
            pile_def: 42,
        },
        EffectRequest::StoreEventCardInSkillPile {
            player: p1,
            card: c1,
            skill: s1,
        },
        EffectRequest::PlaceTopDeckToExpansion {
            player: p1,
            skill: s1,
            pile_def: 42,
            count: 3,
        },
        EffectRequest::EnterDying { player: p1 },
        EffectRequest::KillPlayer {
            player: p2,
            killer: p1,
        },
        EffectRequest::MoveCards {
            cards: vec![c1],
            to: z1,
            from: Some(z2),
        },
        EffectRequest::RevealCards {
            cards: vec![c1],
            to: vec![p1, p2],
        },
        EffectRequest::SetTurnState {
            current: p1,
            phase: 2,
        },
        EffectRequest::TakeFromHand { from: p1, to: p2 },
        EffectRequest::DiscardAllOwnJudge {
            player: p1,
            skill: s1,
        },
        EffectRequest::GainOtherSkillPileCards {
            player: p1,
            target: p2,
            skill_def: 99,
            skill: s1,
        },
        EffectRequest::DiscardHandEquipCount {
            player: p1,
            count: 1,
            skill: s1,
        },
        EffectRequest::TurnOver { player: p1 },
        EffectRequest::PreventDamage { damage_id: 0 },
        EffectRequest::ModifyCurrentDamage {
            player: p1,
            delta: -1,
            skill: s1,
        },
        EffectRequest::ResolveOlJieDanshou {
            player: p1,
            skill: s1,
        },
        EffectRequest::ChooseGiveOrRecastOneHand {
            player: p1,
            skill: s1,
            require_skill_def: 1,
            grant_skill_def: 2,
            grant_mark_def: 3,
        },
        EffectRequest::ChooseOtherTargetsBySkippedPhaseCountDamageDraw {
            player: p1,
            amount: 2,
            skill: s1,
        },
        EffectRequest::ResolveTiqi {
            player: p1,
            skill: s1,
        },
        EffectRequest::ResolveBaoshu {
            player: p1,
            skill: s1,
        },
        EffectRequest::ResolveDcbihuo {
            player: p1,
            skill: s1,
        },
        EffectRequest::SkipCurrentPhase { player: p1 },
        EffectRequest::StorageSet {
            skill: s1,
            key: 7,
            value: 9,
        },
    ];
    assert_eq!(all_37.len(), 37);
    for req in &all_37 {
        accept_effect(req, 4).unwrap_or_else(|e| panic!("accept_effect 应放行 {req:?}: {e:?}"));
    }
    let mut bytes = Vec::new();
    enc_effects(&mut bytes, &all_37);
    let decoded = dec_effects(&bytes).expect("37 个放行变体解码成功");
    assert_eq!(decoded, all_37, "37 个放行变体编解码严格往返一致");
}

/// 2. 真实 `sp/mingshi.skill`（`discard_hand_equip_count(self, 1)` + `modify_current_damage(self, -1)`）：
/// 提交期 `DiscardHandEquipCount` 打开 `rvm_discard_pick` 选牌窗时，`ModifyCurrentDamage` 留在
/// `commit_pending` 队列；此时执行 `save()` -> `restore_with_extensions()` 字节级往返并在双端选牌，
/// 伤害减 1 且状态哈希完全一致。
#[test]
fn pa12_batch7_mingshi_discard_pick_save_restore_with_pending_modify_damage() {
    let pack = pack_for(&[("sp-mingshi", MINGSHI)], "pa12-batch7-mingshi");
    let mut e = Engine::new_with_extensions(997_101, 4, &[&pack]).expect("建局");
    e.debug_grant(1, "mingshi");
    e.debug_set_hp(0, 4);
    e.debug_set_hp(1, 2); // hp(event_source=0)=4 > hp(self=1)=2
    // 确保 1 号手牌 >= 2 张，使 DiscardHandEquipCount(1) 进入玩家选牌挂起窗
    let _c1 = e.debug_place(1, "sha");
    let _c2 = e.debug_place(1, "shan");

    // 0 号对 1 号造成 2 点伤害 → 1 号 mingshi 开 trigger_confirm
    e.debug_deal_damage(Some(0), 1, 2, DamageNature::Normal);
    let c_confirm = e.open_choice().cloned().expect("mingshi 确认门");
    assert_eq!(c_confirm.spec.prompt, "trigger_confirm");
    assert_eq!(c_confirm.observer, 1);
    submit(
        &mut e,
        1,
        GameAction::Answer {
            choice_id: c_confirm.id,
            selection: vec![ChoiceOption::Confirm],
        },
    );

    // 确认后进入 rvm_discard_pick 选牌窗，此时 ModifyCurrentDamage(-1) 位于 commit_pending 队列中
    let c_pick = e.open_choice().cloned().expect("mingshi 弃牌选牌窗");
    assert_eq!(c_pick.spec.prompt, "rvm_discard_pick");
    let snap = e.save();
    let mut restored =
        Engine::restore_with_extensions(&snap, &[&pack]).expect("挂起队列含 ModifyCurrentDamage 恢复成功");
    assert_eq!(e.state_hash(), restored.state_hash(), "恢复前状态哈希一致");

    let pick_opt = c_pick.spec.options[0];
    submit(
        &mut e,
        1,
        GameAction::Answer {
            choice_id: c_pick.id,
            selection: vec![pick_opt],
        },
    );
    submit(
        &mut restored,
        1,
        GameAction::Answer {
            choice_id: c_pick.id,
            selection: vec![pick_opt],
        },
    );

    assert_eq!(
        e.debug_hp(1),
        1,
        "mingshi 弃 1 张牌并令 2 点伤害 -1，1 号仅受 1 点伤害（2 -> 1）"
    );
    assert_eq!(
        e.state_hash(),
        restored.state_hash(),
        "双端完成选牌与后续 ModifyCurrentDamage 落账后状态哈希一致"
    );
}

/// 3. 真实 `key/shiroha_guying.skill`（`owner_is_target` 伤害 -1 + `owner_source_other_damage_before` 伤害 +1）：
/// - 1 号对 2 号造成 1 点伤害：触发 `owner_source_other_damage_before`（+1），2 号受 2 点伤害；
/// - 1 号对自己造成 2 点伤害：仅触发 `owner_is_target`（-1），不触发 `owner_source_other_damage_before`
///   （`!Atom::EvPlayer` 拦截自伤），1 号仅受 1 点伤害。
#[test]
fn pa12_batch7_shiroha_guying_other_damage_before_vs_self_damage() {
    let pack = pack_for(
        &[("key-shiroha-guying", SHIROHA_GUYING)],
        "pa12-batch7-guying",
    );
    let mut e = Engine::new_with_extensions(997_102, 4, &[&pack]).expect("建局");
    e.debug_grant(1, "shiroha_guying");
    e.debug_set_hp(1, 4);
    e.debug_set_hp(2, 4);

    // ① 1 号对他人（2 号）造成 1 点伤害 → +1 = 2 点伤害
    e.debug_deal_damage(Some(1), 2, 1, DamageNature::Normal);
    assert_eq!(
        e.debug_hp(2),
        2,
        "对其他角色造成伤害触发 owner_source_other_damage_before（伤害 +1：4 -> 2）"
    );

    // ② 1 号对自己造成 2 点伤害 → 仅 owner_is_target（-1）生效，owner_source_other_damage_before 不触发
    e.debug_deal_damage(Some(1), 1, 2, DamageNature::Normal);
    assert_eq!(
        e.debug_hp(1),
        3,
        "自伤仅触发 owner_is_target（2 - 1 = 1 点伤害：4 -> 3），不触发 other_damage_before"
    );
}

/// 4. 真实 `sp/xiemu2.skill`（`owner_alive_targeted_by_other_black_card`）与 `xianding/dczhangji.skill`
/// （`owner_alive_targeted_by_other`）：
/// - 0 号持 `xiemu2`，对自己使用黑色延时锦囊【闪电】（黑桃 A，目标为自己）：因使用者非其他角色
///   （`Atom::UserOther` 为假），不触发 `xiemu2`；
/// - 0 号对 1 号（持 `xiemu2`）使用黑色【杀】：1 号成为其他角色使用的黑色牌目标，触发 `xiemu2` 摸 2 张。
#[test]
fn pa12_batch7_targeted_by_other_xiemu2_and_dczhangji_reject_self_use() {
    let pack = pack_for(
        &[("sp-xiemu2", XIEMU2), ("xianding-dczhangji", DCZHANGJI)],
        "pa12-batch7-targeted-by-other",
    );
    let mut e = Engine::new_with_extensions(997_103, 4, &[&pack]).expect("建局");
    e.debug_grant(0, "xiemu2");
    e.debug_grant(1, "xiemu2");

    // 推进到 0 号出牌阶段
    while !(e.turn().current == 0 && e.turn().phase == engine_native::event::Phase::Play) {
        let cur = e.turn().current;
        submit(&mut e, cur, GameAction::NextPhase);
    }

    // ① 0 号对自己挂黑色【闪电】（spade A）：用牌入判定区，自己成为自己使用黑色牌的目标 → xiemu2 不触发
    let shandian = e.debug_place(0, "shandian");
    let h0_before = hand_len(&e, 0);
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: shandian,
            targets: vec![],
        },
    );
    assert_eq!(
        hand_len(&e, 0),
        h0_before - 1,
        "自己对自己使用黑色牌不触发 targeted_by_other（仅消耗 1 张闪电，不额外摸 2）"
    );

    // ② 0 号对 1 号使用黑色【杀】（从牌堆挑一张黑桃/梅花杀入手）：1 号成为他人使用黑牌的目标 → xiemu2 摸 2
    let mut black_sha = None;
    for _ in 0..20 {
        let c = e.debug_place(0, "sha");
        if matches!(
            e.catalog().cards[c as usize].suit,
            engine_native::catalog::Suit::Spade | engine_native::catalog::Suit::Club
        ) {
            black_sha = Some(c);
            break;
        }
    }
    let black_sha = black_sha.expect("标准牌堆中存在黑色杀");
    let h1_before = hand_len(&e, 1);
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: black_sha,
            targets: vec![1],
        },
    );
    assert_eq!(
        hand_len(&e, 1),
        h1_before + 2,
        "1 号成为 0 号黑色【杀】的目标后立即触发 xiemu2 摸 2 张牌"
    );
}

/// 5. 真实 `onlyOL/olbuyi.skill`（`dying owner_alive_other_dying_hand_nonempty_once_per_game optional`）
/// 与 `offline/liangfan2.skill`（`damage owner_is_target_once optional`）：
/// - A: `other_dying` 反例门：持有者（1 号）自身进入濒死时不触发 `olbuyi`；
/// - B: `once`（scope 1）放弃确认不消耗本回合配额：第一次受伤放弃 `liangfan2`，同回合第二次受伤仍开确认门，
///      确认后摸 1 张，同回合第三次受伤不再开门；
/// - C: `once_per_game`（scope 3）放弃确认不消耗整局配额：他人（2 号）第一次濒死时 1 号放弃 `olbuyi`，
///      第二次濒死时 1 号仍开 `olbuyi` 确认门并使 2 号回复 1 点体力脱离濒死；跨回合后 2 号第三次濒死时
///      `olbuyi` 已消耗整局配额，不再开门。
#[test]
fn pa12_batch7_other_dying_and_once_decline_preserves_turn_and_game_quota() {
    let pack = pack_for(
        &[("onlyOL-olbuyi", OLBUYI), ("offline-liangfan2", LIANGFAN2)],
        "pa12-batch7-once-and-other-dying",
    );
    let mut e = Engine::new_with_extensions(997_104, 4, &[&pack]).expect("建局");
    e.debug_grant(1, "olbuyi");
    e.debug_grant(1, "liangfan2");
    let _hand_card = e.debug_place(1, "sha"); // 确保 1 号 hand_nonempty 成立

    // --- Part A & B1: 1 号受 1 点伤害（1 -> 0）：
    // DamageFlow 先派发 TargetDamage（触发 1 号 liangfan2 确认门，先测试放弃不消耗 once），
    // 再派发 Dying（1 号自身濒死不得触发 olbuyi 的 other_dying，直接进入 dying_save 求桃窗）。
    e.debug_set_hp(1, 1);
    let tao1 = e.debug_place(1, "tao");
    e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    let c_lf1 = e
        .open_choice()
        .cloned()
        .expect("TargetDamage 阶段先触发第一次受伤的 liangfan2 确认门");
    assert_eq!(c_lf1.spec.prompt, "trigger_confirm");
    assert_eq!(c_lf1.observer, 1);
    submit(
        &mut e,
        1,
        GameAction::Answer {
            choice_id: c_lf1.id,
            selection: Vec::new(), // 放弃 liangfan2（Cancel），不消耗 once
        },
    );
    // 紧接着进入 Dying 阶段：1 号自身濒死不得触发 olbuyi（other_dying），应直接进入求桃窗 dying_save
    let c_dying = e.open_choice().cloned().expect("1 号濒死求桃窗");
    assert_eq!(
        c_dying.spec.prompt, "dying_save",
        "持有者自身濒死不得触发 other_dying（olbuyi）"
    );
    submit(
        &mut e,
        c_dying.observer,
        GameAction::Answer {
            choice_id: c_dying.id,
            selection: vec![ChoiceOption::Card(tao1)],
        },
    );
    assert!(e.open_choice().is_none());

    // --- Part B: 同回合 1 号第二次受伤，因前次放弃未消耗 once，liangfan2 再次开门 ---
    e.debug_set_hp(1, 3);
    let h1_before = hand_len(&e, 1);
    e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    let c_lf2 = e
        .open_choice()
        .cloned()
        .expect("放弃后同回合第二次受伤仍应触发 liangfan2 确认门");
    assert_eq!(c_lf2.spec.prompt, "trigger_confirm");
    submit(
        &mut e,
        1,
        GameAction::Answer {
            choice_id: c_lf2.id,
            selection: vec![ChoiceOption::Confirm],
        },
    );
    assert_eq!(hand_len(&e, 1), h1_before + 1, "确认 liangfan2 后摸 1 张牌");

    // 同回合 1 号第三次受伤：once 已消耗，不再开门
    e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    assert!(
        e.open_choice().is_none(),
        "同回合已确认发动过 liangfan2（once），第三次受伤不再开门"
    );

    // --- Part C: 他人（2 号）进入濒死，测试 olbuyi（once_per_game）放弃与确认 ---
    e.debug_set_hp(2, 1);
    let tao2 = e.debug_place(2, "tao");
    e.debug_deal_damage(Some(0), 2, 1, DamageNature::Normal);
    let c_buyi_decline = e
        .open_choice()
        .cloned()
        .expect("2 号濒死触发 1 号 olbuyi 确认门");
    assert_eq!(c_buyi_decline.spec.prompt, "trigger_confirm");
    assert_eq!(c_buyi_decline.observer, 1);
    // 1 号第一次放弃 olbuyi → 不消耗 once_per_game，接着由 2 号用桃自救
    submit(
        &mut e,
        1,
        GameAction::Answer {
            choice_id: c_buyi_decline.id,
            selection: Vec::new(),
        },
    );
    for _ in 0..16 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
        let sel = if c.observer == 2 && c.spec.options.contains(&ChoiceOption::Card(tao2)) {
            vec![ChoiceOption::Card(tao2)]
        } else {
            Vec::new()
        };
        submit(
            &mut e,
            c.observer,
            GameAction::Answer {
                choice_id: c.id,
                selection: sel,
            },
        );
    }
    assert!(e.open_choice().is_none());
    assert_eq!(e.debug_hp(2), 1);

    // 2 号第二次进入濒死：因前次放弃未消耗 once_per_game，olbuyi 再次开门；本次选择 Confirm
    e.debug_deal_damage(Some(0), 2, 1, DamageNature::Normal);
    let c_buyi_confirm = e
        .open_choice()
        .cloned()
        .expect("放弃后 2 号第二次濒死仍触发 1 号 olbuyi 确认门");
    assert_eq!(c_buyi_confirm.spec.prompt, "trigger_confirm");
    assert_eq!(c_buyi_confirm.observer, 1);
    submit(
        &mut e,
        1,
        GameAction::Answer {
            choice_id: c_buyi_confirm.id,
            selection: vec![ChoiceOption::Confirm],
        },
    );
    assert_eq!(
        e.debug_hp(2),
        1,
        "olbuyi 确认后使 event_player（2 号）回复 1 点体力脱离濒死"
    );
    assert!(
        e.open_choice().is_none(),
        "2 号已被 olbuyi 救回，无需进入求桃窗口"
    );

    // 推进到下一名角色回合，验证 once_per_game（scope 3）跨回合仍保持已消耗
    let cur = e.turn().current;
    for _ in 0..32 {
        if e.turn().current != cur && e.open_choice().is_none() {
            break;
        }
        if let Some(c) = e.open_choice().cloned() {
            let sel = if c.spec.allow_cancel {
                Vec::new()
            } else {
                c.spec
                    .options
                    .iter()
                    .copied()
                    .take(c.spec.min.max(1))
                    .collect()
            };
            submit(
                &mut e,
                c.observer,
                GameAction::Answer {
                    choice_id: c.id,
                    selection: sel,
                },
            );
            continue;
        }
        let c_seat = e.turn().current;
        submit(&mut e, c_seat, GameAction::NextPhase);
    }
    assert_ne!(e.turn().current, cur, "已推进至下一回合");

    // 跨回合后 2 号第三次进入濒死：olbuyi（once_per_game）不再触发，直接进入求桃窗
    e.debug_set_hp(2, 1);
    e.debug_deal_damage(Some(0), 2, 1, DamageNature::Normal);
    let c_after = e.open_choice().cloned().expect("2 号第三次濒死直接进求桃窗");
    assert_eq!(
        c_after.spec.prompt, "dying_save",
        "once_per_game 跨回合仍保持已消耗，olbuyi 不再开确认门"
    );
}
