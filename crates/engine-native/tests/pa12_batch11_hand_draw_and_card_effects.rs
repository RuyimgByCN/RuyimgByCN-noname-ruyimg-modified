//! PA12 迁移批⑪（2026-11-06，零版本号变更）：
//! 1. `rvm_ops/query.rs`：`locate` 为处理区 `core.table` 返回 `CardZone == 6`（`Processing`）；
//!    `HasSkill` 在 `skill_disabled_raw:{raw}` 置位时返回 `Bool(false)`。
//! 2. `rvm_ops/commit.rs`：`detach` 支持从 `core.table` 摘牌；`MoveCards` 移入手牌（`kind == 3`）
//!    补记 `LogEvent::CardsDrawn`；向 `commit_combat` 透传当前执行技能名 `executing`。
//! 3. `rvm_ops` 与 `rvm_ops/codec.rs` 接入 13 个既有 ABI（<= 759）确定性效果原语（tags 71..=83）：
//!    - `AdjustHandTo`（tag 71）：将目标手牌数调整至 N（不足补摸、超出从手牌首起弃置）；
//!    - `DrawByPlayPhaseCardCategoryCount`（tag 72）：按本出牌阶段已使用过的牌类别数（基本/锦囊/装备）摸牌；
//!    - `DrawEachBySeat`（tag 73）：令两名角色按当前回合逆时针座次各摸 N 张牌；
//!    - `SetDrawPhaseFromBottom`（tag 74）：令当前摸牌阶段改为从牌堆底（`deck[0]`）摸牌；
//!    - `DrawCardsFromBottom`（tag 75）：从牌堆底直接摸 N 张牌并触发 `CardsDrawn`；
//!    - `CancelIncomingCardUse`（tag 76）：从当前 `CardUsed` 目标集中移除目标角色并将桌面/弃牌堆中的实体牌收入其手牌；
//!    - `ExcludeSavedShaTarget`（tag 77）：从当前【杀】目标集中移除指定角色（若目标集清空则将桌面载体牌移入弃牌堆）；
//!    - `TakeOneCard`（tag 78）：确定性获得来源角色一张手牌或装备牌（或仅装备牌）；
//!    - `TakeOneHandCard`（tag 79）：确定性获得来源角色一张手牌（缺省来源由 `EngineVmHost` 从事件对手补齐）；
//!    - `DisableSkillByDefUntilTurnEnd`（tag 80）：按 `skill_def` 禁用技能至回合结束（回合结束自动恢复 `enabled = true`）；
//!    - `BanActiveUntilTurnEnd`（tag 81）：封禁技能本回合主动发动与触发过闸；
//!    - `SetNonLockedSkillSuppression`（tag 82）：启停非锁定技压制源标记；
//!    - `SuppressNonLockedSkillsUntilTurnEnd`（tag 83）：标记目标角色非锁定技失效至回合结束。
//! 4. `rvm_named` 修正 `Atom::InTargets`（当 `targets` 非空时只看 `targets.contains(&owner)`，不再误把使用者 `event_player` 当目标），
//!    并补齐 `"target_alive_has_equip"`（`Atom::TargetEquipNonEmpty`）与 `"other_alive_per_damage_point"`（`Atom::SourceNotTarget`）。

use content_compiler::compile_source_named;
use engine_native::content::extension::{ExtensionDef, ExtensionPack, InitHook};
use engine_native::event::Phase;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand};
use engine_native_kernel::rvm_ops::{accept_effect, dec_effects, enc_effects, run_query};
use rule_api::{CardId, EffectRequest, PlayerId, QueryRequest, QueryResponse, SkillDefId, SkillId};

const DCJIANXIONG: &str = include_str!("../../../content/original/ported/collab/dcjianxiong.skill");
const NATULIE: &str = include_str!("../../../content/original/ported/collab/natulie.skill");
const DCCHENYONG: &str = include_str!("../../../content/original/ported/xianding/dcchenyong.skill");
const WANGXI: &str = include_str!("../../../content/original/ported/refresh/wangxi.skill");
const NZRY_CUNMU: &str = include_str!("../../../content/original/ported/shenhua/nzry_cunmu.skill");
const DUCHI: &str = include_str!("../../../content/original/ported/newjiang/duchi.skill");
const DANLAO: &str = include_str!("../../../content/original/ported/sp/danlao.skill");
const ZHUZHOU: &str = include_str!("../../../content/original/ported/collab/zhuzhou.skill");
const NSCANGJIAN: &str = include_str!("../../../content/original/ported/diy/nscangjian.skill");
const STDYIBING: &str = include_str!("../../../content/original/ported/sixiang/stdyibing.skill");

fn pack_for(sources: &[(&str, &str)], pack_name: &'static str) -> ExtensionPack {
    let mut vm_skills = Vec::new();
    for (id, source) in sources {
        let mut pkg = compile_source_named(*id, source).expect("编译真实 ported .skill");
        assert!(
            pkg.manifest.rule_vm_abi_version <= rule_vm::RULE_VM_ABI_VERSION,
            "批⑪不引入新 ABI：{} > {}",
            pkg.manifest.rule_vm_abi_version,
            rule_vm::RULE_VM_ABI_VERSION
        );
        if !pkg.modifier_index.is_empty() {
            pkg.modifier_index.clear();
            pkg.manifest.logic_hash = pkg.compute_logic_hash();
        }
        let out = content_native_loader::bridge_vm_skill_decls(&pkg).expect("正式桥接");
        vm_skills.extend(out.decls);
    }
    ExtensionPack {
        def: ExtensionDef {
            name: pack_name,
            version: 1,
            description: "PA12 批⑪ 手牌调整、底牌摸取、目标排除与禁用原语",
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
        !matches!(out, CommandOutcome::Rejected(_)),
        "submit({actor}, {action:?}) 失败: {out:?}"
    );
}

fn drain_confirm(e: &mut Engine) {
    for _ in 0..80 {
        let Some(ask) = e.open_choice().cloned() else {
            return;
        };
        if ask.spec.options.contains(&ChoiceOption::Confirm) {
            submit(
                e,
                ask.observer,
                GameAction::Answer {
                    choice_id: ask.id,
                    selection: vec![ChoiceOption::Confirm],
                },
            );
        } else if ask.spec.options.contains(&ChoiceOption::Pass) {
            submit(
                e,
                ask.observer,
                GameAction::Answer {
                    choice_id: ask.id,
                    selection: vec![ChoiceOption::Pass],
                },
            );
        } else if ask.spec.allow_cancel {
            submit(
                e,
                ask.observer,
                GameAction::Answer {
                    choice_id: ask.id,
                    selection: vec![],
                },
            );
        } else {
            let n = ask.spec.min.max(1) as usize;
            let selection: Vec<ChoiceOption> = ask.spec.options.iter().take(n).cloned().collect();
            submit(
                e,
                ask.observer,
                GameAction::Answer {
                    choice_id: ask.id,
                    selection,
                },
            );
        }
    }
}

fn submit_to_phase(e: &mut Engine, seat: u8, want: Phase) {
    for _ in 0..80 {
        drain_confirm(e);
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

#[test]
fn pa12_batch11_codec_roundtrip_71_to_83() {
    let effects = vec![
        EffectRequest::AdjustHandTo {
            player: PlayerId::new(1),
            count: 2,
        },
        EffectRequest::DrawByPlayPhaseCardCategoryCount {
            player: PlayerId::new(1),
            skill: SkillId::new(7),
        },
        EffectRequest::DrawEachBySeat {
            first: PlayerId::new(1),
            second: PlayerId::new(2),
            count: 1,
            skill: SkillId::new(8),
        },
        EffectRequest::SetDrawPhaseFromBottom {
            player: PlayerId::new(1),
        },
        EffectRequest::DrawCardsFromBottom {
            player: PlayerId::new(2),
            count: 1,
        },
        EffectRequest::CancelIncomingCardUse {
            player: PlayerId::new(1),
            source: PlayerId::new(2),
            card: CardId::new(15),
            skill: SkillId::new(9),
        },
        EffectRequest::ExcludeSavedShaTarget {
            card: CardId::new(16),
            target: PlayerId::new(3),
        },
        EffectRequest::TakeOneCard {
            from: PlayerId::new(2),
            to: PlayerId::new(1),
            equip_only: true,
        },
        EffectRequest::TakeOneHandCard {
            from: PlayerId::new(3),
            to: PlayerId::new(1),
        },
        EffectRequest::DisableSkillByDefUntilTurnEnd {
            player: PlayerId::new(1),
            skill_def: SkillDefId::new(3312200979),
        },
        EffectRequest::BanActiveUntilTurnEnd {
            player: PlayerId::new(1),
            skill: SkillId::new(10),
        },
        EffectRequest::SetNonLockedSkillSuppression {
            player: PlayerId::new(1),
            enabled: true,
            skill: SkillId::new(11),
        },
        EffectRequest::SuppressNonLockedSkillsUntilTurnEnd {
            player: PlayerId::new(2),
        },
    ];
    for req in &effects {
        accept_effect(req, 4).expect("批⑪ 13 项新变体均应通过 accept_effect");
    }
    let mut buf = Vec::new();
    enc_effects(&mut buf, &effects);
    let decoded = dec_effects(&buf).expect("dec_effects 应成功解码 tags 71..=83");
    assert_eq!(decoded, effects, "tags 71..=83 编解码必须 100% 往返一致");

    assert!(accept_effect(
        &EffectRequest::DrawEachBySeat {
            first: PlayerId::new(1),
            second: PlayerId::new(2),
            count: 0,
            skill: SkillId::NONE,
        },
        4
    )
    .is_err());
    assert!(accept_effect(
        &EffectRequest::DrawCardsFromBottom {
            player: PlayerId::new(1),
            count: 0,
        },
        4
    )
    .is_err());
    assert!(accept_effect(
        &EffectRequest::CancelIncomingCardUse {
            player: PlayerId::new(1),
            source: PlayerId::new(2),
            card: CardId::NONE,
            skill: SkillId::NONE,
        },
        4
    )
    .is_err());
    assert!(accept_effect(
        &EffectRequest::ExcludeSavedShaTarget {
            card: CardId::NONE,
            target: PlayerId::new(2),
        },
        4
    )
    .is_err());
    assert!(accept_effect(
        &EffectRequest::DisableSkillByDefUntilTurnEnd {
            player: PlayerId::new(1),
            skill_def: SkillDefId::NONE,
        },
        4
    )
    .is_err());
    assert!(accept_effect(
        &EffectRequest::BanActiveUntilTurnEnd {
            player: PlayerId::new(1),
            skill: SkillId::NONE,
        },
        4
    )
    .is_err());
}

#[test]
fn pa12_batch11_dcjianxiong_table_zone_gain_and_progressive_draw() {
    let pack = pack_for(&[("dcjianxiong", DCJIANXIONG)], "pa12-batch11-dcjianxiong");
    let mut e = Engine::new_with_extensions(911_001, 4, &[&pack]).expect("建局");
    e.debug_grant(1, "dcjianxiong");

    submit_to_phase(&mut e, 0, Phase::Play);
    for s in 0..4u8 {
        e.state_core_mut().player_mut(s).hand.clear();
        e.state_core_mut().player_mut(s).hp = 4;
        e.state_core_mut().player_mut(s).max_hp = 4;
    }

    // 0 号装备【方天画戟】并对 [1, 2] 打出最后一张手牌【杀】：
    // 对 1 号结算伤害时 extra_targets 仍含 2 号，advance_target 返回 Flow::Yield 暂不弃置 sha1，
    // 因此在 1 号受伤触发 dcjianxiong 确认窗期间，sha1 仍在处理区 core.table（CardZone == 6）。
    let fangtian = e.debug_place(0, "fangtian");
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: fangtian,
            targets: vec![],
        },
    );
    drain_confirm(&mut e);

    let sha1 = e.debug_place(0, "sha");
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha1,
            targets: vec![1, 2],
        },
    );
    // 此时应停在 1 号 dcjianxiong 确认窗，且 sha1 仍在处理区 core.table（CardZone == 6）。
    let ask = e.open_choice().cloned().expect("dcjianxiong 确认窗");
    assert_eq!(ask.observer, 1);
    let zone_in_table = run_query(
        e.state_core(),
        e.catalog(),
        4,
        1,
        QueryRequest::CardZone {
            card: CardId::new(sha1),
        },
    )
    .expect("查询处理区卡牌区域");
    assert_eq!(
        zone_in_table,
        QueryResponse::Int(6),
        "方天画戟多目标【杀】对首目标结算伤害窗口内，造成伤害的实体牌仍在 core.table，CardZone 应为 6（Processing）"
    );

    submit(
        &mut e,
        1,
        GameAction::Answer {
            choice_id: ask.id,
            selection: vec![ChoiceOption::Confirm],
        },
    );
    drain_confirm(&mut e);

    assert!(
        e.state_core().player(1).hand.contains(&sha1),
        "经典奸雄：1 号应从处理区获得造成伤害的【杀】"
    );
    assert!(
        !e.state_core().discard.contains(&sha1) && !e.state_core().table.contains(&sha1),
        "经典奸雄：获得的【杀】不应再落入弃牌堆或残留桌面"
    );
    assert_eq!(
        e.state_core().player(1).hand.len(),
        2,
        "经典奸雄首次发动：获得伤害牌 1 张 + 摸 1 张牌 = 2 张手牌"
    );
    assert_eq!(
        e.state_core()
            .vm_aux
            .marks
            .get(&(1, 3695091838))
            .copied()
            .unwrap_or(0),
        1,
        "经典奸雄首次发动后标记 3695091838 应 +1"
    );

    // 再次造成伤害：验证渐进摸牌（marks == 1 -> 本次摸 2 张牌）且标记增至 2。
    let duel = e.debug_place(0, "duel");
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: duel,
            targets: vec![1],
        },
    );
    drain_confirm(&mut e);
    assert_eq!(
        e.state_core()
            .vm_aux
            .marks
            .get(&(1, 3695091838))
            .copied()
            .unwrap_or(0),
        2,
        "经典奸雄第二次发动后标记 3695091838 应为 2"
    );
    assert_eq!(
        e.state_core().player(1).hand.len(),
        4,
        "经典奸雄第二次发动：原 2 张手牌 + 渐进摸 2 张牌 = 4 张手牌"
    );
}

#[test]
fn pa12_batch11_natulie_adjust_hand_to_draw_and_discard() {
    let pack = pack_for(&[("natulie", NATULIE)], "pa12-batch11-natulie");
    let mut e = Engine::new_with_extensions(911_002, 4, &[&pack]).expect("建局");
    e.debug_grant(0, "natulie");

    submit_to_phase(&mut e, 0, Phase::Play);
    for s in 0..4u8 {
        e.state_core_mut().player_mut(s).hand.clear();
    }
    // 1) 手牌数 < 2 时补摸至 2：0 号仅 1 张【杀】，打出后手牌为 0 → 发动烈补摸至 2 张。
    let sha = e.debug_place(0, "sha");
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha,
            targets: vec![1],
        },
    );
    drain_confirm(&mut e);
    assert_eq!(
        e.state_core().player(0).hand.len(),
        2,
        "烈（AdjustHandTo）：打出基本牌后手牌 0 < 2，应补摸至 2 张"
    );

    // 2) 手牌数 > 2 时弃置至 2：0 号再放入 3 张手牌 + 1 张【桃】（共 6 张），受伤后吃【桃】剩 5 张 → 发动烈弃置 3 张剩 2 张。
    e.state_core_mut().player_mut(0).hp = 2;
    e.state_core_mut().player_mut(0).max_hp = 4;
    let _c1 = e.debug_place(0, "shan");
    let _c2 = e.debug_place(0, "shan");
    let _c3 = e.debug_place(0, "duel");
    let tao = e.debug_place(0, "tao");
    assert_eq!(e.state_core().player(0).hand.len(), 6);
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: tao,
            targets: vec![],
        },
    );
    drain_confirm(&mut e);
    assert_eq!(
        e.state_core().player(0).hand.len(),
        2,
        "烈（AdjustHandTo）：使用【桃】后手牌 5 > 2，应弃置 3 张调整至 2 张"
    );
}

#[test]
fn pa12_batch11_dcchenyong_draw_by_play_phase_card_category_count() {
    let pack = pack_for(&[("dcchenyong", DCCHENYONG)], "pa12-batch11-dcchenyong");
    let mut e = Engine::new_with_extensions(911_003, 4, &[&pack]).expect("建局");
    e.debug_grant(0, "dcchenyong");

    submit_to_phase(&mut e, 0, Phase::Play);
    for s in 0..4u8 {
        e.state_core_mut().player_mut(s).hand.clear();
    }
    // 在本出牌阶段依次使用：基本牌（sha）、锦囊牌（duel）、装备牌（qinggang）共 3 类。
    let sha = e.debug_place(0, "sha");
    let duel = e.debug_place(0, "duel");
    let qinggang = e.debug_place(0, "qinggang");

    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha,
            targets: vec![1],
        },
    );
    drain_confirm(&mut e);
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: duel,
            targets: vec![2],
        },
    );
    drain_confirm(&mut e);
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: qinggang,
            targets: vec![],
        },
    );
    drain_confirm(&mut e);
    assert_eq!(e.state_core().player(0).hand.len(), 0);

    // 推进至结束阶段（phase_end） → 触发沉勇，按已使用的 3 类牌摸 3 张牌。
    submit_to_phase(&mut e, 0, Phase::End);
    drain_confirm(&mut e);
    assert_eq!(
        e.state_core().player(0).hand.len(),
        3,
        "沉勇：本出牌阶段使用过基本、锦囊、装备 3 类牌，结束阶段应摸 3 张牌"
    );
}

#[test]
fn pa12_batch11_wangxi_draw_each_by_seat() {
    let pack = pack_for(&[("wangxi", WANGXI)], "pa12-batch11-wangxi");
    let mut e = Engine::new_with_extensions(911_004, 4, &[&pack]).expect("建局");
    e.debug_grant(1, "wangxi");

    submit_to_phase(&mut e, 0, Phase::Play);
    for s in 0..4u8 {
        e.state_core_mut().player_mut(s).hand.clear();
        e.state_core_mut().player_mut(s).hp = 4;
    }
    let sha = e.debug_place(0, "sha");
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha,
            targets: vec![1],
        },
    );
    drain_confirm(&mut e);

    assert_eq!(
        e.state_core().player(0).hand.len(),
        1,
        "忘隙：伤害来源 0 号应摸 1 张牌"
    );
    assert_eq!(
        e.state_core().player(1).hand.len(),
        1,
        "忘隙：受伤持有者 1 号应摸 1 张牌"
    );
}

#[test]
fn pa12_batch11_nzry_cunmu_and_duchi_draw_from_bottom() {
    let pack = pack_for(
        &[("nzry_cunmu", NZRY_CUNMU), ("duchi", DUCHI)],
        "pa12-batch11-bottom-draw",
    );
    let mut e = Engine::new_with_extensions(911_005, 4, &[&pack]).expect("建局");
    e.debug_grant(1, "nzry_cunmu");
    e.debug_grant(2, "duchi");

    submit_to_phase(&mut e, 0, Phase::Play);
    for s in 0..4u8 {
        e.state_core_mut().player_mut(s).hand.clear();
    }

    // 1) 督持（duchi）：0 号对 2 号出【决斗】（无距离限制）→ 2 号从牌堆底（deck[0]）摸 1 张牌。
    let duel = e.debug_place(0, "duel");
    let bottom_for_duchi = e.state_core().deck[0];
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: duel,
            targets: vec![2],
        },
    );
    drain_confirm(&mut e);
    assert!(
        e.state_core().player(2).hand.contains(&bottom_for_duchi),
        "督持（DrawCardsFromBottom）：成为他人用牌目标时应从牌堆底（deck[0]）摸 1 张牌"
    );

    // 2) 寸目（nzry_cunmu）：推进到 1 号摸牌阶段前记录牌堆底两张牌，验证 1 号摸牌阶段从牌堆底摸 2 张。
    let bottom1 = e.state_core().deck[0];
    let bottom2 = e.state_core().deck[1];
    submit_to_phase(&mut e, 1, Phase::Play);
    assert_eq!(
        e.state_core().player(1).hand,
        vec![bottom1, bottom2],
        "寸目（SetDrawPhaseFromBottom）：1 号摸牌阶段应从牌堆底依次摸入 deck[0] 与 deck[1]"
    );
    assert_eq!(
        e.state_core()
            .vm_aux
            .flags
            .get(&(1, "draw_phase_bottom".to_string()))
            .copied()
            .unwrap_or(0),
        0,
        "寸目：摸牌阶段结束后 draw_phase_bottom 临时标记应被清除"
    );

    // 3) 边界场景 A：DrawCardsFromBottom 在牌堆为空时自动洗入弃牌堆并从新牌堆底摸牌。
    //    推进到 3 号出牌阶段，清空牌堆并将指定牌放入弃牌堆，3 号对 2 号出【决斗】触发督持。
    submit_to_phase(&mut e, 3, Phase::Play);
    for s in 0..4u8 {
        e.state_core_mut().player_mut(s).hand.clear();
    }
    let duel2 = e.debug_place(3, "duel");
    let moved_deck: Vec<u32> = e.state_core_mut().deck.drain(..).collect();
    e.state_core_mut().discard.extend(moved_deck);
    assert!(e.state_core().deck.is_empty() && !e.state_core().discard.is_empty());
    submit(
        &mut e,
        3,
        GameAction::PlayCard {
            card: duel2,
            targets: vec![2],
        },
    );
    drain_confirm(&mut e);
    assert_eq!(
        e.state_core().player(2).hand.len(),
        1,
        "督持（DrawCardsFromBottom 空牌堆重洗）：牌堆为空时应洗入弃牌堆并从新牌堆底摸 1 张牌"
    );

    // 4) 边界场景 B：SetDrawPhaseFromBottom 叠加摸牌数修正（draw_phase_bonus = +1，共摸 3 张）且牌堆仅剩 1 张触发中途重洗。
    submit_to_phase(&mut e, 1, Phase::Judge);
    for s in 0..4u8 {
        e.state_core_mut().player_mut(s).hand.clear();
    }
    let only_bottom = e.state_core().deck[0];
    let rest_to_discard: Vec<u32> = e.state_core_mut().deck.drain(1..).collect();
    e.state_core_mut().discard.extend(rest_to_discard);
    assert_eq!(e.state_core().deck.len(), 1);
    e.state_core_mut()
        .vm_aux
        .flags
        .insert((1, "draw_phase_bonus".to_string()), 1);
    submit_to_phase(&mut e, 1, Phase::Play);
    assert_eq!(
        e.state_core().player(1).hand.len(),
        3,
        "寸目 + 摸牌修正 + 中途洗牌：1 号应完整摸满 3 张牌"
    );
    assert_eq!(
        e.state_core().player(1).hand[0],
        only_bottom,
        "寸目 + 中途洗牌：第 1 张摸入的应为洗牌前唯一的牌堆底牌"
    );
}

#[test]
fn pa12_batch11_danlao_cancel_incoming_card_use() {
    let pack = pack_for(&[("danlao", DANLAO)], "pa12-batch11-danlao");
    let mut e = Engine::new_with_extensions(911_006, 4, &[&pack]).expect("建局");
    e.debug_grant(1, "danlao");

    submit_to_phase(&mut e, 0, Phase::Play);
    for s in 0..4u8 {
        e.state_core_mut().player_mut(s).hand.clear();
        e.state_core_mut().player_mut(s).hp = 4;
    }

    let savage = e.debug_place(0, "savage");
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: savage,
            targets: vec![],
        },
    );
    drain_confirm(&mut e);

    assert_eq!(
        e.state_core().player(1).hp,
        4,
        "啖酪（CancelIncomingCardUse）：多目标锦囊【南蛮入侵】对 1 号无效，1 号不受伤害"
    );
    assert!(
        e.state_core().player(1).hand.contains(&savage) && e.state_core().player(1).hand.len() == 2,
        "啖酪：1 号摸 1 张牌并获得处理区中的【南蛮入侵】（共 2 张手牌）"
    );
    assert!(
        !e.state_core().discard.contains(&savage),
        "啖酪：已被 1 号收入手牌的【南蛮入侵】在 AOE 收尾时不应再进入弃牌堆"
    );
    assert_eq!(
        e.state_core().player(2).hp,
        3,
        "啖酪：未被排除的 2 号仍正常结算【南蛮入侵】受 1 点伤害"
    );
    assert_eq!(
        e.state_core().player(3).hp,
        3,
        "啖酪：未被排除的 3 号仍正常结算【南蛮入侵】受 1 点伤害"
    );

    // 边界场景：多名目标（1 号与 2 号）同时持有啖酪（danlao）——
    // 1 号先发动啖酪从处理区拿走实体牌并免伤，2 号后发动啖酪仍应从目标集中移除并摸 1 张牌，
    // 且不会从 1 号手牌重复复制实体牌，仅 3 号受 1 点伤害。
    e.debug_grant(2, "danlao");
    for s in 0..4u8 {
        e.state_core_mut().player_mut(s).hand.clear();
        e.state_core_mut().player_mut(s).hp = 4;
    }
    let wanjian = e.debug_place(0, "archery");
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: wanjian,
            targets: vec![],
        },
    );
    drain_confirm(&mut e);
    assert_eq!(e.state_core().player(1).hp, 4, "1 号啖酪免伤");
    assert_eq!(e.state_core().player(2).hp, 4, "2 号啖酪同样应从目标集移除并免伤");
    assert_eq!(e.state_core().player(3).hp, 3, "未持啖酪的 3 号受 1 点伤害");
    assert!(
        e.state_core().player(1).hand.contains(&wanjian)
            && !e.state_core().player(2).hand.contains(&wanjian)
            && !e.state_core().discard.contains(&wanjian),
        "万箭齐发实体牌只应被首名发动的 1 号获得一次，不得在 2 号手牌或弃牌堆重复出现"
    );
    assert_eq!(
        e.state_core().player(2).hand.len(),
        1,
        "2 号啖酪摸 1 张牌（不重复获得已被 1 号拿走的实体牌）"
    );
}

#[test]
fn pa12_batch11_zhuzhou_nscangjian_stdyibing_take_cards() {
    let pack = pack_for(
        &[
            ("zhuzhou", ZHUZHOU),
            ("nscangjian", NSCANGJIAN),
            ("stdyibing", STDYIBING),
        ],
        "pa12-batch11-take-cards",
    );
    let mut e = Engine::new_with_extensions(911_007, 4, &[&pack]).expect("建局");
    e.debug_grant(0, "nscangjian");
    e.debug_grant(2, "zhuzhou");
    e.debug_grant(3, "stdyibing");

    submit_to_phase(&mut e, 0, Phase::Play);
    for s in 0..4u8 {
        e.state_core_mut().player_mut(s).hand.clear();
        e.state_core_mut().player_mut(s).equips.clear();
    }
    e.state_core_mut().player_mut(1).hp = 2;
    let p1_hand = e.debug_place(1, "duel");
    let p1_equip = e.debug_place(1, "renwang");
    e.state_core_mut().player_mut(1).hand.retain(|c| *c != p1_equip);
    e.state_core_mut().player_mut(1).equips.push(p1_equip);

    // 1) 0 号对 1 号造成 1 点伤害（1 号存活 hp=1）：
    //    - 2 号发动诛咒（TakeOneCard hand_equip）：令伤害来源 0 号获得 1 号手牌 p1_hand；
    //    - 0 号发动藏剑（TakeOneCard equip）：获得 1 号装备区 p1_equip。
    let sha1 = e.debug_place(0, "sha");
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha1,
            targets: vec![1],
        },
    );
    drain_confirm(&mut e);

    assert!(
        e.state_core().player(0).hand.contains(&p1_hand),
        "诛咒（TakeOneCard hand_equip）：伤害来源 0 号应获得受伤者 1 号的手牌"
    );
    assert!(
        e.state_core().player(0).hand.contains(&p1_equip)
            && e.state_core().player(1).equips.is_empty(),
        "藏剑（TakeOneCard equip）：0 号造成伤害后应获得存活目标 1 号装备区的牌"
    );

    // 2) 推进到 2 号出牌阶段，给 1 号（hp=1）两张手牌（第一张在 TargetDamage 窗口被 2 号 zhuzhou 拿走，第二张在随后 Dying 触发窗被 3 号遗秉 stdyibing 拿走）。
    submit_to_phase(&mut e, 2, Phase::Play);
    for s in 0..4u8 {
        e.state_core_mut().player_mut(s).hand.clear();
    }
    let _first_hand = e.debug_place(1, "sha");
    let dying_hand = e.debug_place(1, "wuzhong");
    assert_eq!(e.state_core().player(1).hand.len(), 2);
    let sha2 = e.debug_place(2, "sha");
    submit(
        &mut e,
        2,
        GameAction::PlayCard {
            card: sha2,
            targets: vec![1],
        },
    );
    drain_confirm(&mut e);

    assert!(
        e.state_core().player(3).hand.contains(&dying_hand),
        "遗秉（TakeOneHandCard）：1 号濒死结算时，3 号应获得其手牌"
    );
}

#[test]
fn pa12_batch11_exclude_saved_sha_disable_ban_and_suppression_effects() {
    let wxdlqianqian =
        include_str!("../../../content/original/ported/offline/wxdlqianqian.skill");
    let rewansha = include_str!("../../../content/original/ported/refresh/rewansha.skill");
    let aux_dsl = r#"
skill aux_exclude_sha(9911001) {
  trigger card_targeted owner_targeted_by_other_sha {
    effect exclude_current_sha_target(event_card, self);
  }
}

skill aux_ban_active_and_suppress(9911002) {
  active play_phase {
    effect draw_cards(self, 1);
  }
  trigger card_used owner_is_source {
    effect ban_active_until_turn_end(self, self_skill);
    effect suppress_non_locked_skills_until_turn_end(self);
  }
}
"#;
    let pack = pack_for(
        &[
            ("wxdlqianqian", wxdlqianqian),
            ("rewansha", rewansha),
            ("aux_batch11", aux_dsl),
        ],
        "pa12-batch11-direct-effects",
    );
    let mut e = Engine::new_with_extensions(911_008, 4, &[&pack]).expect("建局");
    e.debug_grant(0, "wxdlqianqian");
    e.debug_grant(0, "rewansha");
    e.debug_grant(0, "aux_ban_active_and_suppress");
    e.debug_grant(1, "aux_exclude_sha");

    submit_to_phase(&mut e, 0, Phase::Play);
    for s in 0..4u8 {
        e.state_core_mut().player_mut(s).hand.clear();
        e.state_core_mut().player_mut(s).hp = 4;
    }

    // 1) 用牌前：0 号持有 wxdlqianqian(3312200979) 为启用态，且可发动主动技 aux_ban_active_and_suppress。
    assert_eq!(
        run_query(
            e.state_core(),
            e.catalog(),
            4,
            0,
            QueryRequest::HasSkill {
                player: PlayerId::new(1),
                skill_def: SkillDefId::new(3312200979),
            },
        )
        .expect("HasSkill 查询"),
        QueryResponse::Bool(true)
    );
    let aux_skill = e
        .catalog()
        .skill_by_name("aux_ban_active_and_suppress")
        .expect("skill_by_name");
    submit(
        &mut e,
        0,
        GameAction::ActivateSkill { skill: aux_skill },
    );
    drain_confirm(&mut e);
    assert_eq!(
        e.state_core().player(0).hand.len(),
        1,
        "被封禁前发动主动技应成功摸 1 张牌"
    );

    // 2) 0 号对 1 号出【杀】：
    //    - 0 号触发 wxdlqianqian -> DisableSkillByDefUntilTurnEnd(3312200979)；
    //    - 0 号触发 aux_ban_active_and_suppress -> BanActiveUntilTurnEnd + SuppressNonLockedSkillsUntilTurnEnd；
    //    - 1 号触发 aux_exclude_sha -> ExcludeSavedShaTarget 将 1 号从【杀】目标集移除并将【杀】移入弃牌堆。
    let sha = e.debug_place(0, "sha");
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha,
            targets: vec![1],
        },
    );
    drain_confirm(&mut e);

    assert_eq!(
        e.state_core().player(1).hp,
        4,
        "ExcludeSavedShaTarget：1 号从【杀】目标集移除后不受伤害"
    );
    assert!(
        e.state_core().discard.contains(&sha) && !e.state_core().table.contains(&sha),
        "ExcludeSavedShaTarget：目标集清空后桌面【杀】应移入弃牌堆"
    );
    assert_eq!(
        run_query(
            e.state_core(),
            e.catalog(),
            4,
            0,
            QueryRequest::HasSkill {
                player: PlayerId::new(1),
                skill_def: SkillDefId::new(3312200979),
            },
        )
        .expect("HasSkill 查询"),
        QueryResponse::Bool(false),
        "DisableSkillByDefUntilTurnEnd：本回合内 HasSkill(3312200979) 应返回 false"
    );
    assert_eq!(
        e.state_core()
            .vm_aux
            .flags
            .get(&(0, "suppress_non_locked_turn".to_string()))
            .copied()
            .unwrap_or(0),
        1,
        "SuppressNonLockedSkillsUntilTurnEnd：应置位 suppress_non_locked_turn"
    );

    // 封禁后本回合再次发动主动技应被拒绝。
    let rev = e.revision();
    let banned_out = e.submit(GameCommand {
        actor: 0,
        expected_revision: rev,
        dedup_id: 991100299,
        action: GameAction::ActivateSkill { skill: aux_skill },
    });
    assert!(
        matches!(banned_out, CommandOutcome::Rejected(_)),
        "BanActiveUntilTurnEnd：本回合封禁后再次发动主动技必须被拒绝"
    );

    // 3) 0 号回合内：
    //    3a) 先令持【桃】的 2 号进入濒死（触发 rewansha 的 dying -> SetNonLockedSkillSuppression(true)），
    //        2 号使用【桃】自救脱离濒死后触发 rewansha 的 dying_after -> SetNonLockedSkillSuppression(false)，
    //        验证 non_locked_suppression_source 在回合中途即被清除；
    //    3b) 再令无【桃】的 2 号濒死并阵亡，验证 SetNonLockedSkillSuppression(true) 再次置位，
    //        以及 0 号回合结束后 DisableSkillByDefUntilTurnEnd / BanActiveUntilTurnEnd / SuppressNonLockedSkillsUntilTurnEnd 全部清理恢复。
    e.state_core_mut().player_mut(2).hp = 1;
    let tao = e.debug_place(2, "tao");
    let duel_save = e.debug_place(0, "duel");
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: duel_save,
            targets: vec![2],
        },
    );
    // 2 号在决斗响应窗选择 Pass → 受到 1 点伤害进入濒死 → rewansha dying 触发置 non_locked_suppression_source = 1 → 停在求桃窗。
    for _ in 0..20 {
        let Some(ask) = e.open_choice().cloned() else {
            break;
        };
        if ask.observer == 2 && ask.spec.options.contains(&ChoiceOption::Card(tao)) {
            assert_eq!(
                e.state_core()
                    .vm_aux
                    .flags
                    .get(&(0, "non_locked_suppression_source".to_string()))
                    .copied()
                    .unwrap_or(0),
                1,
                "界完杀（dying）：2 号求桃期间 non_locked_suppression_source 应为 1"
            );
            submit(
                &mut e,
                2,
                GameAction::Answer {
                    choice_id: ask.id,
                    selection: vec![ChoiceOption::Card(tao)],
                },
            );
        } else if ask.spec.options.contains(&ChoiceOption::Pass) {
            submit(
                &mut e,
                ask.observer,
                GameAction::Answer {
                    choice_id: ask.id,
                    selection: vec![ChoiceOption::Pass],
                },
            );
        } else {
            submit(
                &mut e,
                ask.observer,
                GameAction::Answer {
                    choice_id: ask.id,
                    selection: vec![ChoiceOption::Confirm],
                },
            );
        }
    }
    drain_confirm(&mut e);
    assert!(
        e.state_core().player(2).alive && e.state_core().player(2).hp == 1,
        "2 号吃【桃】自救后应存活且体力为 1"
    );
    assert_eq!(
        e.state_core()
            .vm_aux
            .flags
            .get(&(0, "non_locked_suppression_source".to_string()))
            .copied()
            .unwrap_or(0),
        0,
        "界完杀（dying_after -> SetNonLockedSkillSuppression(false)）：脱离濒死后 non_locked_suppression_source 应立即移除"
    );

    let duel = e.debug_place(0, "duel");
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: duel,
            targets: vec![2],
        },
    );
    drain_confirm(&mut e);
    assert!(!e.state_core().player(2).alive, "2 号应阵亡");
    assert_eq!(
        e.state_core()
            .vm_aux
            .flags
            .get(&(0, "non_locked_suppression_source".to_string()))
            .copied()
            .unwrap_or(0),
        1,
        "界完杀（SetNonLockedSkillSuppression）：0 号回合内角色濒死触发 dying 后 non_locked_suppression_source 应置为 1"
    );

    // 结束 0 号回合 → 进入 1 号回合，检查回合级禁用与压制标记全部复原。
    submit_to_phase(&mut e, 1, Phase::Play);
    assert_eq!(
        run_query(
            e.state_core(),
            e.catalog(),
            4,
            0,
            QueryRequest::HasSkill {
                player: PlayerId::new(1),
                skill_def: SkillDefId::new(3312200979),
            },
        )
        .expect("回合结束后 HasSkill 查询"),
        QueryResponse::Bool(true),
        "DisableSkillByDefUntilTurnEnd：回合结束后技能应恢复启用"
    );
    assert_eq!(
        e.state_core()
            .vm_aux
            .flags
            .get(&(0, "suppress_non_locked_turn".to_string()))
            .copied()
            .unwrap_or(0),
        0,
        "SuppressNonLockedSkillsUntilTurnEnd：回合结束后压制标记应清除"
    );
    assert_eq!(
        e.state_core()
            .vm_aux
            .flags
            .get(&(0, "skill_banned_turn:aux_ban_active_and_suppress".to_string()))
            .copied()
            .unwrap_or(0),
        0,
        "BanActiveUntilTurnEnd：回合结束后封禁标记应清除"
    );
    assert_eq!(
        e.state_core()
            .vm_aux
            .flags
            .get(&(0, "non_locked_suppression_source".to_string()))
            .copied()
            .unwrap_or(0),
        0,
        "SetNonLockedSkillSuppression：回合结束后 non_locked_suppression_source 应自动归零"
    );
}
