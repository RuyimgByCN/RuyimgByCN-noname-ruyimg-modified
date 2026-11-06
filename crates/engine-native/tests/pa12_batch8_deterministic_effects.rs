//! PA12 迁移批⑧（2026-11-06，零版本号变更）：
//! 1. `rvm_ops` 接入 11 个既有 ABI（<= 759）确定性非挂起 `EffectRequest` 原语：
//!    - `AddSkillUntilTurnEnd`（tag 38）：按包 `skill_def_id`（32 位 FNV）或目录索引解析，
//!      登记 `expires_end_of_turn` 或 `vm_aux.flags[(p, "skill:{raw}")]` + `"skill_turn:{raw}"`（回合末清理）；
//!    - `RemoveSkillByDef`（tag 39）：按包 `skill_def_id` 或目录索引从指定玩家移除技能并清理旗标；
//!    - `RevealHand`（tag 40）/ `RevealHandToPlayer`（tag 41）：发射 `LogEvent::CardsRevealed`；
//!    - `GainCardByKind`（tag 42）/ `GainDiscardCardByKind`（tag 43）/ `GainEquipmentFromDrawPile`（tag 44）：
//!      按牌种或装备栏从摸牌堆/弃牌堆检索并移入手牌，发射 `CardsDrawn`；
//!    - `GiveAllHand`（tag 45）：将拥有者全部手牌移交目标并统一发射 `CardsGiven` 触发窗与日志；
//!    - `DiscardAllTargetEquipment`（tag 46）：弃置目标全部装备区卡牌并更新 `vm_aux.lost`；
//!    - `GainEventCards`（tag 47）/ `GainCardIfDiscard`（tag 48）：回收事件关联实体牌仍在弃牌堆者。
//! 2. `rvm_ops::kind_tag` 修正为与 `content-compiler` 的 `card_kind_value` 完全同源的规范编号
//!    （`Sha=1`、`Tao=2`、`Shan=3`、`Jiu=4`、`Weapon=10..=Treasure=14`、`Wuzhong=20..=WangmeiZhike=42`），
//!    消除此前 `CardKind::ALL.iter().position(...)` 顺序下标与编译器编码失配导致的牌种查询与按牌种获牌偏差。
//! 3. `rvm_ops::resolve_skill_def` 支持按 `SkillDef.behavior.package_skill_id()` 反查 32 位 FNV `skill_def_id`，
//!    使 `.skill` 源码中的 `add_skill`、`add_skill_until_turn_end`、`remove_skill_by_def`、`remove_skill` 与
//!    `has_skill` 查询可精确命中同包或跨包 VM 技能定义。
//! 4. `rvm_named::phrase` 补齐 `other_target_alive_has_discardable_hand_equip`、
//!    `target_alive_has_discardable_hand_equip`、`other_target_alive`、`target_alive` 与
//!    `other_responded_sha_gainable`，修复 `Atom::TargetAlive` 未接线及目标可弃牌判定缺失。

use content_compiler::compile_source_named;
use engine_native::catalog::CardKind;
use engine_native::content::extension::{ExtensionDef, ExtensionPack, InitHook};
use engine_native::event::{DamageNature, Phase};
use engine_native::{ChoiceOption, Engine, GameAction, GameCommand, LogEvent};
use engine_native_kernel::rvm_ops::{accept_effect, dec_effects, enc_effects, run_query};
use rule_api::{CardId, EffectRequest, PlayerId, QueryRequest, QueryResponse, SkillDefId};

const WUJI: &str = include_str!("../../../content/original/ported/sp/wuji.skill");
const HUXIAO: &str = include_str!("../../../content/original/ported/sp/huxiao.skill");
const ZHICHI: &str = include_str!("../../../content/original/ported/yijiang/zhichi.skill");
const JIGONG: &str = include_str!("../../../content/original/ported/yijiang/jigong.skill");
const DCQIUSUO: &str = include_str!("../../../content/original/ported/collab/dcqiusuo.skill");
const FRIENDMANJUAN: &str =
    include_str!("../../../content/original/ported/mobile/friendmanjuan.skill");
const SPSHIJI: &str = include_str!("../../../content/original/ported/shiji/spshiji.skill");
const LY_PILICHE: &str = include_str!("../../../content/original/ported/mobile/ly_piliche.skill");
const DCLONGGONG: &str = include_str!("../../../content/original/ported/collab/dclonggong.skill");
const OLDRENXIN: &str = include_str!("../../../content/original/ported/yijiang/oldrenxin.skill");
const NAMAN: &str = include_str!("../../../content/original/ported/sp/naman.skill");
const OLHAIZHU: &str = include_str!("../../../content/original/ported/collab/olhaizhu.skill");

fn pack_for(sources: &[(&str, &str)], pack_name: &'static str) -> ExtensionPack {
    let mut vm_skills = Vec::new();
    for (id, source) in sources {
        let pkg = compile_source_named(*id, source).expect("编译真实 ported .skill");
        assert!(
            pkg.manifest.rule_vm_abi_version <= rule_vm::RULE_VM_ABI_VERSION,
            "批⑧不引入新 ABI：{} > {}",
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
            description: "PA12 批⑧ 确定性效果原语、牌种编号同源与技能 ID 解析",
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

fn confirm(e: &mut Engine, actor: u8) {
    let ch = e.open_choice().cloned().expect("应有待答确认窗");
    assert_eq!(ch.observer, actor);
    assert_eq!(ch.spec.prompt, "trigger_confirm");
    submit(
        e,
        actor,
        GameAction::Answer {
            choice_id: ch.id,
            selection: vec![ChoiceOption::Confirm],
        },
    );
}

/// 1. 批⑧新增放行的 11 个 `EffectRequest` 变体（tags 38..=48）全部通过 `accept_effect` 与 `enc_effects`/`dec_effects` 往返。
#[test]
fn pa12_batch8_codec_new_11_effects_round_trip_and_validation() {
    let p1 = PlayerId::new(1);
    let p2 = PlayerId::new(2);
    let sd = SkillDefId::new(3_796_089_056);
    let c1 = CardId::new(11);
    let c2 = CardId::new(23);

    let batch8_effects = vec![
        EffectRequest::AddSkillUntilTurnEnd {
            player: p1,
            skill_def: sd,
        },
        EffectRequest::RemoveSkillByDef {
            player: p1,
            skill_def: sd,
        },
        EffectRequest::RevealHand { player: p2 },
        EffectRequest::RevealHandToPlayer {
            player: p2,
            observer: p1,
        },
        EffectRequest::GainCardByKind {
            player: p1,
            kind: 34,
        },
        EffectRequest::GainDiscardCardByKind {
            player: p1,
            kind: 1,
        },
        EffectRequest::GainEquipmentFromDrawPile { player: p2 },
        EffectRequest::GiveAllHand {
            player: p1,
            target: p2,
        },
        EffectRequest::DiscardAllTargetEquipment {
            actor: p1,
            target: p2,
        },
        EffectRequest::GainEventCards {
            player: p1,
            cards: vec![c1, c2],
        },
        EffectRequest::GainCardIfDiscard {
            player: p1,
            card: c1,
        },
    ];
    assert_eq!(batch8_effects.len(), 11);
    for req in &batch8_effects {
        accept_effect(req, 8).unwrap_or_else(|e| panic!("accept_effect 应放行 {req:?}: {e:?}"));
    }
    let mut bytes = Vec::new();
    enc_effects(&mut bytes, &batch8_effects);
    let decoded = dec_effects(&bytes).expect("dec_effects 应完整解码 11 个新增变体");
    assert_eq!(decoded, batch8_effects);

    // 边界拒绝：非法座位（9）、非法牌种（0）、空卡牌 ID（0）、自己给自己全部手牌。
    assert!(accept_effect(
        &EffectRequest::AddSkillUntilTurnEnd {
            player: PlayerId::new(9),
            skill_def: sd,
        },
        8
    )
    .is_err());
    assert!(accept_effect(
        &EffectRequest::GainCardByKind {
            player: p1,
            kind: 0,
        },
        8
    )
    .is_err());
    assert!(accept_effect(
        &EffectRequest::GainCardIfDiscard {
            player: p1,
            card: CardId::NONE,
        },
        8
    )
    .is_err());
    assert!(accept_effect(
        &EffectRequest::GiveAllHand {
            player: p1,
            target: p1,
        },
        8
    )
    .is_err());
}

/// 2. `kind_tag` 与 `content-compiler` 的规范 `card_kind_value` 完全一致（杀=1、桃=2、闪=3、酒=4、武器=10）。
#[test]
fn pa12_batch8_kind_tag_matches_compiler_canonical_values() {
    let e = Engine::new(42, 2);
    let catalog = e.catalog();
    let core = e.state_core();

    let find_card = |kind: CardKind| -> u32 {
        catalog
            .cards
            .iter()
            .position(|c| c.kind == kind)
            .expect("标准牌堆含该牌种") as u32
    };
    let sha_cid = find_card(CardKind::Sha);
    let tao_cid = find_card(CardKind::Tao);
    let shan_cid = find_card(CardKind::Shan);
    let weapon_cid = find_card(CardKind::Weapon);

    assert_eq!(
        run_query(
            core,
            catalog,
            2,
            0,
            QueryRequest::CardAttribute {
                card: CardId::new(sha_cid + 1),
                attr: 0,
            }
        )
        .unwrap(),
        QueryResponse::Int(1),
        "Sha 规范编号应为 1（而非 ALL 下标 0）"
    );
    assert_eq!(
        run_query(
            core,
            catalog,
            2,
            0,
            QueryRequest::CardAttribute {
                card: CardId::new(tao_cid + 1),
                attr: 0,
            }
        )
        .unwrap(),
        QueryResponse::Int(2),
        "Tao 规范编号应为 2"
    );
    assert_eq!(
        run_query(
            core,
            catalog,
            2,
            0,
            QueryRequest::CardAttribute {
                card: CardId::new(shan_cid + 1),
                attr: 0,
            }
        )
        .unwrap(),
        QueryResponse::Int(3),
        "Shan 规范编号应为 3（而非 ALL 下标 1）"
    );
    assert_eq!(
        run_query(
            core,
            catalog,
            2,
            0,
            QueryRequest::CardAttribute {
                card: CardId::new(weapon_cid + 1),
                attr: 0,
            }
        )
        .unwrap(),
        QueryResponse::Int(10),
        "Weapon 规范编号应为 10"
    );
}

/// 3. 真实 `sp/wuji.skill` + `sp/huxiao.skill`：
///    结束阶段本回合造成伤害 >= 3 时，通过 32 位 `skill_def_id = 3796089056` 移除 `huxiao`、
///    加 1 点体力上限、回复 1 点体力，并移除自身 `wuji`。
#[test]
fn pa12_batch8_wuji_remove_skill_by_def_and_awaken() {
    let pack = pack_for(&[("wuji", WUJI), ("huxiao", HUXIAO)], "pa12_b8_wuji");
    let mut e = Engine::new_with_extensions(10801, 4, &[&pack]).expect("装载 wuji + huxiao");
    e.debug_grant(0, "wuji");
    e.debug_grant(0, "huxiao");

    // 推进到 0 号出牌阶段
    while !(e.turn().current == 0 && e.turn().phase == Phase::Play) {
        let cur = e.turn().current;
        submit(&mut e, cur, GameAction::NextPhase);
    }
    e.debug_clear_hand(0);
    e.debug_set_max_hp(0, 3);
    e.debug_set_hp(0, 2);

    // 反例：本回合仅造成 2 点伤害（< 3）→ 进入结束阶段时 wuji 不触发。
    e.debug_set_hp(1, 5);
    e.debug_deal_damage(Some(0), 1, 2, DamageNature::Normal);
    assert_eq!(e.state_core().player(0).turn_damage, 2);
    while !(e.turn().current == 0 && e.turn().phase == Phase::End) {
        submit(&mut e, 0, GameAction::NextPhase);
    }
    assert!(e.debug_has_skill(0, "huxiao"), "伤害 2 时不失虎啸");
    assert!(e.debug_has_skill(0, "wuji"), "伤害 2 时武继未觉醒");
    assert_eq!(e.state_core().player(0).max_hp, 3);
    assert_eq!(e.debug_hp(0), 2);

    // 正例：下一轮 0 号出牌阶段本回合造成 3 点伤害 → 进入结束阶段时 wuji 触发。
    while !(e.turn().current == 0 && e.turn().phase == Phase::Play) {
        let cur = e.turn().current;
        e.debug_clear_hand(cur);
        submit(&mut e, cur, GameAction::NextPhase);
    }
    e.debug_clear_hand(0);
    e.debug_set_max_hp(0, 3);
    e.debug_set_hp(0, 2);
    e.debug_set_max_hp(1, 6);
    e.debug_set_hp(1, 6);
    e.debug_deal_damage(Some(0), 1, 3, DamageNature::Normal);
    assert_eq!(e.state_core().player(0).turn_damage, 3);

    while !(e.turn().current == 0 && e.turn().phase == Phase::End) {
        submit(&mut e, 0, GameAction::NextPhase);
    }
    assert!(
        !e.debug_has_skill(0, "huxiao"),
        "武继应按 package_skill_id(3796089056) 移除虎啸"
    );
    assert!(
        !e.debug_has_skill(0, "wuji"),
        "武继应移除自身（觉醒）"
    );
    assert_eq!(e.state_core().player(0).max_hp, 4, "体力上限 +1");
    assert_eq!(e.debug_hp(0), 3, "回复 1 点体力");
}

/// 4. 真实 `yijiang/zhichi.skill` 与 `yijiang/jigong.skill`：
///    触发后执行 `add_skill_until_turn_end(self, skill_def)`，本回合内 `HasSkill` 查询为真，
///    回合结束后自动归零。
#[test]
fn pa12_batch8_zhichi_add_skill_until_turn_end_and_expiry() {
    let pack = pack_for(
        &[("zhichi", ZHICHI), ("jigong", JIGONG)],
        "pa12_b8_zhichi_jigong",
    );
    let mut e = Engine::new_with_extensions(10802, 4, &[&pack]).expect("装载 zhichi + jigong");
    e.debug_grant(1, "zhichi");
    e.debug_grant(0, "jigong");

    // 推进到 0 号出牌阶段（0 号触发 jigong：摸 2 张 + 获得 3671794947 至回合末）
    while !(e.turn().current == 0 && e.turn().phase == Phase::Play) {
        let cur = e.turn().current;
        if let Some(ch) = e.open_choice().cloned() {
            submit(
                &mut e,
                ch.observer,
                GameAction::Answer {
                    choice_id: ch.id,
                    selection: vec![ChoiceOption::Confirm],
                },
            );
        } else {
            e.debug_clear_hand(cur);
            submit(&mut e, cur, GameAction::NextPhase);
        }
    }
    if e.open_choice().is_some() {
        confirm(&mut e, 0);
    }
    assert_eq!(
        run_query(
            e.state_core(),
            e.catalog(),
            4,
            0,
            QueryRequest::HasSkill {
                player: PlayerId::new(1),
                skill_def: SkillDefId::new(3_671_794_947),
            }
        )
        .unwrap(),
        QueryResponse::Bool(true),
        "急攻发动后本回合内 0 号应持有临时技能 3671794947"
    );

    // 0 号受 1 点伤害（满足 1 号 zhichi 的 owner_alive_not_target）→ 1 号获得 3197261303 至回合末
    e.debug_clear_hand(0);
    e.debug_deal_damage(Some(1), 0, 1, DamageNature::Normal);

    assert_eq!(
        run_query(
            e.state_core(),
            e.catalog(),
            4,
            1,
            QueryRequest::HasSkill {
                player: PlayerId::new(2),
                skill_def: SkillDefId::new(3_197_261_303),
            }
        )
        .unwrap(),
        QueryResponse::Bool(true),
        "本回合内 1 号应持有临时技能 3197261303"
    );

    // 0 号结束本回合后，0 号与 1 号的本回合临时技能全部自动清除。
    while e.turn().current == 0 {
        e.debug_clear_hand(0);
        submit(&mut e, 0, GameAction::NextPhase);
    }
    assert_eq!(
        run_query(
            e.state_core(),
            e.catalog(),
            4,
            0,
            QueryRequest::HasSkill {
                player: PlayerId::new(1),
                skill_def: SkillDefId::new(3_671_794_947),
            }
        )
        .unwrap(),
        QueryResponse::Bool(false),
        "换回合后临时技能 3671794947 应自动清除"
    );
    assert_eq!(
        run_query(
            e.state_core(),
            e.catalog(),
            4,
            1,
            QueryRequest::HasSkill {
                player: PlayerId::new(2),
                skill_def: SkillDefId::new(3_197_261_303),
            }
        )
        .unwrap(),
        QueryResponse::Bool(false),
        "换回合后临时技能 3197261303 应自动清除"
    );
}

/// 5. 真实 `collab/dcqiusuo.skill`（`gain_card_by_kind(self, tiesuo)`）与
///    `mobile/friendmanjuan.skill`（`gain_discard_card_by_kind(self, sha)`）：
///    按规范牌种编号检索牌堆/弃牌堆并获得对应卡牌。
#[test]
fn pa12_batch8_dcqiusuo_and_friendmanjuan_gain_by_kind() {
    let pack = pack_for(
        &[("dcqiusuo", DCQIUSUO), ("friendmanjuan", FRIENDMANJUAN)],
        "pa12_b8_gain_by_kind",
    );
    let mut e = Engine::new_with_extensions(10803, 4, &[&pack]).expect("装载求索与漫卷");
    e.debug_grant(1, "friendmanjuan");

    // 放一张【杀】进弃牌堆，再令 0 号给 2 号一张牌以发射 CardsMoved 触发 1 号的 friendmanjuan。
    let sha_in_discard = e.debug_place(0, "sha");
    let _ = e.debug_discard_card(0, sha_in_discard);
    assert!(e.state_core().discard.contains(&sha_in_discard));

    let gift = e.debug_place(0, "shan");
    let p1_before = e.debug_hand(1).len();
    let _ = e.debug_give_hand_card(0, 2, gift);

    assert_eq!(e.debug_hand(1).len(), p1_before + 1);
    assert!(
        e.debug_hand(1).contains(&sha_in_discard),
        "漫卷应按规范编号 kind=1 从弃牌堆获得【杀】"
    );
    assert!(!e.state_core().discard.contains(&sha_in_discard));
    assert_eq!(e.state_core().vm_aux.marks.get(&(1, 0)).copied(), Some(1));
}

/// 6. 真实 `collab/dclonggong.skill`：
///    受到伤害前（`damage_before`）确认发动 → 防止本次伤害，并令伤害来源从摸牌堆获得一张装备牌。
#[test]
fn pa12_batch8_dclonggong_prevent_damage_and_gain_equipment() {
    let pack = pack_for(&[("dclonggong", DCLONGGONG)], "pa12_b8_dclonggong");
    let mut e = Engine::new_with_extensions(10804, 4, &[&pack]).expect("装载 dclonggong");
    e.debug_grant(0, "dclonggong");

    let equip_cid = *e
        .state_core()
        .deck
        .iter()
        .rev()
        .find(|&&cid| e.catalog().card_kind(cid).equip_slot().is_some())
        .expect("摸牌堆含装备牌");
    let p0_hp_before = e.debug_hp(0);
    let p1_hand_before = e.debug_hand(1).len();

    e.debug_deal_damage(Some(1), 0, 1, DamageNature::Normal);
    confirm(&mut e, 0);

    assert_eq!(
        e.debug_hp(0),
        p0_hp_before,
        "龙宫应防止本次伤害"
    );
    assert_eq!(
        e.debug_hand(1).len(),
        p1_hand_before + 1,
        "伤害来源应从摸牌堆获得 1 张装备牌"
    );
    assert!(
        e.debug_hand(1).contains(&equip_cid),
        "获得的应是摸牌堆顶起首张装备牌"
    );
}

/// 7. 真实 `mobile/ly_piliche.skill`：
///    `owner_source_other_target_alive_has_discardable_hand_equip` 具名短语与
///    `discard_all_target_equipment(self, event_player)` 端到端验证。
#[test]
fn pa12_batch8_ly_piliche_discard_all_target_equipment() {
    let pack = pack_for(&[("ly_piliche", LY_PILICHE)], "pa12_b8_piliche");
    let mut e = Engine::new_with_extensions(10805, 4, &[&pack]).expect("装载 ly_piliche");
    e.debug_grant(0, "ly_piliche");

    // 反例 1：目标（座位 1）手牌与装备均为空 → 不触发。
    e.debug_clear_hand(1);
    assert!(e.debug_equips(1).is_empty());
    e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    assert!(
        e.open_choice().is_none(),
        "目标无手牌无装备时霹雳车不应触发"
    );

    // 反例 2：自己对自己造成伤害（不满足 other_target_alive）→ 不触发。
    e.debug_deal_damage(Some(0), 0, 1, DamageNature::Normal);
    assert!(
        e.open_choice().is_none(),
        "自伤不满足 other_target_alive，霹雳车不应触发"
    );

    // 正例：1 号在出牌阶段挂上两件装备，0 号对 1 号造成伤害 → 确认发动 → 弃置 1 号全部装备。
    while !(e.turn().current == 1 && e.turn().phase == Phase::Play) {
        let cur = e.turn().current;
        e.debug_clear_hand(cur);
        submit(&mut e, cur, GameAction::NextPhase);
    }
    let eq1 = e.debug_place(1, "weapon2");
    let eq2 = e.debug_place(1, "horse_plus");
    submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: eq1,
            targets: vec![],
        },
    );
    submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: eq2,
            targets: vec![],
        },
    );
    e.debug_clear_hand(1);
    assert_eq!(e.debug_equips(1).len(), 2);

    e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    confirm(&mut e, 0);

    assert!(
        e.debug_equips(1).is_empty(),
        "霹雳车应弃置目标装备区全部卡牌"
    );
    assert!(e.state_core().discard.contains(&eq1));
    assert!(e.state_core().discard.contains(&eq2));
}

/// 8. 真实 `yijiang/oldrenxin.skill`（`give_all_hand` + `turn_over` + `recover_hp`）与
///    `shiji/spshiji.skill`（`reveal_hand`）：
#[test]
fn pa12_batch8_oldrenxin_give_all_hand_and_spshiji_reveal_hand() {
    let pack = pack_for(
        &[("oldrenxin", OLDRENXIN), ("spshiji", SPSHIJI)],
        "pa12_b8_renxin_shiji",
    );
    let mut e = Engine::new_with_extensions(10806, 4, &[&pack]).expect("装载 oldrenxin + spshiji");
    e.debug_grant(0, "oldrenxin");
    e.debug_grant(0, "spshiji");

    // A) oldrenxin：2 号对 1 号造成 1 点致命伤害令 1 号濒死（hp: 1 -> 0），0 号有手牌 →
    // 确认发动 oldrenxin → 0 号翻面、交出全部手牌给 1 号、1 号回复 1 点体力脱离濒死！
    let p0_cards = e.debug_hand(0);
    assert!(!p0_cards.is_empty());
    let p1_before = e.debug_hand(1).len();
    e.debug_set_hp(1, 1);

    e.debug_deal_damage(Some(2), 1, 1, DamageNature::Normal);
    confirm(&mut e, 0);

    assert!(e.debug_face_down(0), "旧仁心应将自身翻面");
    assert!(e.debug_hand(0).is_empty(), "旧仁心应交出全部手牌");
    assert_eq!(
        e.debug_hand(1).len(),
        p1_before + p0_cards.len(),
        "濒死角色应收到全部手牌"
    );
    assert_eq!(e.debug_hp(1), 1, "濒死角色应回复 1 点体力脱离濒死");

    // B) spshiji：0 号对 1 号造成伤害 → 确认发动 → 展示 1 号全部手牌（产出 CardsRevealed 日志）。
    e.debug_set_hp(1, 3);
    let log_mark = e.log().len();
    let expected_revealed = e.debug_hand(1);
    e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    confirm(&mut e, 0);
    let revealed: Vec<Vec<u32>> = e.log()[log_mark..]
        .iter()
        .filter_map(|ev| match ev {
            LogEvent::CardsRevealed { cards, .. } => Some(cards.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(
        revealed,
        vec![expected_revealed],
        "reveal_hand 应展示目标全部手牌"
    );
}

/// 9. 真实 `sp/naman.skill`（`gain_card_if_discard(self, event_card)` + `other_responded_sha_gainable`）
///    与 `collab/olhaizhu.skill`（`gain_event_cards(self)`）：
#[test]
fn pa12_batch8_naman_gain_card_if_discard_on_other_responded_sha() {
    let pack = pack_for(
        &[("naman", NAMAN), ("olhaizhu", OLHAIZHU)],
        "pa12_b8_naman_haizhu",
    );
    let mut e = Engine::new_with_extensions(10807, 4, &[&pack]).expect("装载 naman + olhaizhu");
    e.debug_grant(2, "naman");

    // 推进到 0 号出牌阶段，0 号对 1 号使用【决斗】或【南蛮入侵】，令 1 号打出一张【杀】进入弃牌堆 →
    // 2 号触发纳蛮（naman）→ 确认后从弃牌堆获得 1 号打出的【杀】！
    while !(e.turn().current == 0 && e.turn().phase == Phase::Play) {
        let cur = e.turn().current;
        submit(&mut e, cur, GameAction::NextPhase);
    }
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);
    e.debug_clear_hand(3);
    let responded_sha = e.debug_place(1, "sha");
    let juedou = e.debug_place(0, "duel");
    let p2_before = e.debug_hand(2).len();

    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: juedou,
            targets: vec![1],
        },
    );
    // 1 号响应决斗打出 responded_sha，随后 0 号放弃继续打杀结束决斗 → CardResponded 子帧弹出触发 2 号纳蛮！
    let ch_resp = e.open_choice().cloned().expect("1 号响应决斗");
    assert_eq!(ch_resp.observer, 1);
    submit(
        &mut e,
        1,
        GameAction::Answer {
            choice_id: ch_resp.id,
            selection: vec![ChoiceOption::Card(responded_sha)],
        },
    );
    let ch_duel_p0 = e.open_choice().cloned().expect("决斗轮转至 0 号响应");
    assert_eq!(ch_duel_p0.observer, 0);
    submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: ch_duel_p0.id,
            selection: vec![ChoiceOption::Pass],
        },
    );

    // 1 号打出的【杀】结算触发 2 号的纳蛮询问确认！
    confirm(&mut e, 2);

    assert_eq!(e.debug_hand(2).len(), p2_before + 1);
    assert!(
        e.debug_hand(2).contains(&responded_sha),
        "纳蛮应从弃牌堆获得其他角色打出的【杀】"
    );
    assert!(!e.state_core().discard.contains(&responded_sha));
}
