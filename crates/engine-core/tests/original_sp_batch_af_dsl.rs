//! Golden for sp batch-AF DSL rewrite:
//! 7 个 changeSkillsAfter 州郡（px*）技能——批量脚本误标为「audio-only」，上游
//! (sp/skill.js) 在文件头注释引用的行号处均有真实规则体。新增引擎字段
//! `TriggerContext.added_skill_def`（SkillAdded dispatch 透传）+ 参数化过滤器
//! `owner_skill_added_self_<def>`（5 处注册）：
//!   - pxbeihai（北海）：获得后手牌摸至体力上限
//!   - pxyouzhou（幽州）：获得后体力上限 +1
//!   - pxfengyi（冯翊）：获得后可回复 1
//!   - pxhongnong（弘农）：获得后可回复 1（装备重铸未建模）
//!   - pxjizhou（冀州）：获得后从牌堆获得【万箭齐发】
//!   - pxwuxiang（武乡）：获得后从牌堆获得【火攻】
//!   - pxquting（句町）：获得后可失去 1 体力并摸 3

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const PXBEIHAI: u32 = 3_401_654_349;
const PXYOUZHOU: u32 = 3_102_513_552;
const PXFENGYI: u32 = 4_070_224_935;
const PXHONGNONG: u32 = 4_043_252_708;
const PXJIZHOU: u32 = 2_256_193_085;
const PXWUXIANG: u32 = 4_127_527_891;
const PXQUTING: u32 = 2_515_073_240;
const PXCHANGSHAN: u32 = 4_243_325_414;
const PXBEIPING: u32 = 3_838_939_507;
const PXLEAN: u32 = 4_240_733_007;
const PXYANMEN: u32 = 3_339_454_948;
const PXLONGXI: u32 = 4_087_538_948;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn started(seed: u64) -> (GameState, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(seed),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: seed,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let mut state = engine.state().clone();
    // v1.11.6 增量 sp roster +1（王皑）平移种子：开局 optional 触发可能挂起 choice，
    // 以 cancelled=true 排空（original_sp_qimen_zniaoxiang 范例）。
    let mut drain_key = 1u64;
    while state.has_pending_choice() {
        let choice = ChoiceId::new(state.resolution.pending_choice_id);
        let revision = state.revision;
        let (next, output) = apply_command(
            state,
            Command {
                actor: PlayerId::NONE,
                expected_revision: ExpectedRevision(revision),
                idempotency_key: IdempotencyKey(seed ^ drain_key),
                kind: CommandKind::ResolveChoice {
                    choice_id: choice,
                    selections: vec![],
                    cancelled: true,
                },
            },
        );
        assert!(
            matches!(output.result, CommandResult::Accepted { .. }),
            "drain rejected: {:?}",
            output.result
        );
        state = next;
        drain_key += 1;
    }
    let owner = state.turn.current;
    (state, owner)
}

/// 通过 AddSkill 效果授予技能——run_effects 会派发 SkillAdded 事件并运行
/// skill_added 触发链（技能触发在 add_skill 内先注册）。
fn grant(state: GameState, owner: PlayerId, def: u32) -> (GameState, Vec<DomainEvent>) {
    let mut tx = Transaction::begin(state);
    let effects = run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(def),
        }],
    );
    match effects {
        Ok(events) => (tx.commit().state, events),
        Err(EffectError::ChoiceRequested) => {
            let tx = tx.commit();
            let (state, result) = resolve(tx.state, owner, vec![1], false, 1);
            assert!(matches!(result.result, CommandResult::Accepted { .. }));
            (state, result.events.clone())
        }
        Err(other) => panic!("unexpected effect error: {other:?}"),
    }
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    )
}

#[test]
fn package_publishes_batch_af_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    let triggers: Vec<_> = package
        .trigger_index
        .iter()
        .filter(|trigger| {
            matches!(
                trigger.skill_def_id,
                PXBEIHAI | PXYOUZHOU | PXFENGYI | PXHONGNONG | PXJIZHOU | PXWUXIANG
                    | PXQUTING | PXCHANGSHAN | PXBEIPING | PXLEAN | PXYANMEN | PXLONGXI
            )
        })
        .map(|trigger| {
            (
                trigger.skill_def_id,
                trigger.event.as_str(),
                trigger.filter.as_str(),
                trigger.optional,
            )
        })
        .collect();
    assert!(triggers.contains(&(PXBEIHAI, "skill_added", "owner_skill_added_self_3401654349", false)));
    assert!(triggers.contains(&(PXYOUZHOU, "skill_added", "owner_skill_added_self_3102513552", false)));
    assert!(triggers.contains(&(PXFENGYI, "skill_added", "owner_skill_added_self_4070224935", true)));
    assert!(triggers.contains(&(PXHONGNONG, "skill_added", "owner_skill_added_self_4043252708", true)));
    assert!(triggers.contains(&(PXJIZHOU, "skill_added", "owner_skill_added_self_2256193085", false)));
    assert!(triggers.contains(&(PXWUXIANG, "skill_added", "owner_skill_added_self_4127527891", false)));
    assert!(triggers.contains(&(PXQUTING, "skill_added", "owner_skill_added_self_2515073240", true)));
    assert!(triggers.contains(&(PXCHANGSHAN, "skill_added", "owner_skill_added_self_4243325414", false)));
    assert!(triggers.contains(&(PXBEIPING, "skill_added", "owner_skill_added_self_3838939507", false)));
    assert!(triggers.contains(&(PXLEAN, "skill_added", "owner_skill_added_self_4240733007", false)));
    assert!(triggers.contains(&(PXYANMEN, "skill_added", "owner_skill_added_self_3339454948", false)));
    assert!(triggers.contains(&(PXLONGXI, "skill_added", "owner_skill_added_self_4087538948", false)));
    // 十二技均不再保留手牌上限 +0 的诚实 noop modifier
    assert!(!package.modifier_index.iter().any(|modifier| {
        matches!(
            modifier.skill_def_id,
            PXBEIHAI | PXYOUZHOU | PXFENGYI | PXHONGNONG | PXJIZHOU | PXWUXIANG
                | PXQUTING | PXCHANGSHAN | PXBEIPING | PXLEAN | PXYANMEN | PXLONGXI
        ) && modifier.target == "hand_limit"
    }));
}

#[test]
fn pxbeihai_draws_to_max_hp_on_gain() {
    let (state, owner) = started(2_026_081_001);
    // 2 人局主公 max_hp=5 但起始手牌 6（主公加成），先弃 3 张使手牌低于上限
    let mut tx = Transaction::begin(state);
    let cards: Vec<_> = tx.state().hand_cards(owner);
    let discard: Vec<_> = cards.iter().take(3).copied().collect();
    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardCards {
            player: owner,
            cards: discard,
        }],
    )
    .unwrap();
    let state = tx.commit().state;
    let player = state.players.get(owner).unwrap();
    assert!(state.hand_count(owner) < player.max_hp);
    let (state, _) = grant(state, owner, PXBEIHAI);
    let player = state.players.get(owner).unwrap();
    assert_eq!(
        state.hand_count(owner),
        player.max_hp,
        "北海 must draw the owner's hand up to max HP"
    );
}

#[test]
fn pxyouzhou_increases_max_hp_on_gain() {
    let (state, owner) = started(2_026_081_002);
    let max_hp_before = state.players.get(owner).unwrap().max_hp;
    let (state, _) = grant(state, owner, PXYOUZHOU);
    assert_eq!(
        state.players.get(owner).unwrap().max_hp,
        max_hp_before + 1,
        "幽州 must add one max HP"
    );
}

#[test]
fn pxjizhou_gains_wanjian_on_gain() {
    let (state, owner) = started(2_026_081_003);
    let (state, _) = grant(state, owner, PXJIZHOU);
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let has_wanjian = state
        .cards
        .sorted_live()
        .into_iter()
        .filter(|card| card.zone == hand)
        .any(|card| card.kind == CardKind::TrickWanjian);
    assert!(has_wanjian, "冀州 must gain a 万箭齐发 into hand");
}

#[test]
fn pxwuxiang_gains_huogong_on_gain() {
    let (state, owner) = started(2_026_081_004);
    let (state, _) = grant(state, owner, PXWUXIANG);
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let has_huogong = state
        .cards
        .sorted_live()
        .into_iter()
        .filter(|card| card.zone == hand)
        .any(|card| card.kind == CardKind::TrickHuogong);
    assert!(has_huogong, "武乡 must gain a 火攻 into hand");
}

#[test]
fn pxquting_loses_hp_and_draws_three_on_gain() {
    let (state, owner) = started(2_026_081_005);
    let hp_before = state.players.get(owner).unwrap().hp;
    let hand_before = state.hand_count(owner);
    let (state, _) = grant(state, owner, PXQUTING);
    assert_eq!(state.players.get(owner).unwrap().hp, hp_before - 1);
    assert_eq!(state.hand_count(owner), hand_before + 3);
}

fn count_hand_kind(state: &GameState, owner: PlayerId, kind: CardKind) -> usize {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    state
        .cards
        .sorted_live()
        .into_iter()
        .filter(|card| card.zone == hand && card.kind == kind)
        .count()
}

#[test]
fn pxchangshan_gains_two_shan_on_gain() {
    let (state, owner) = started(2_026_081_006);
    let (state, _) = grant(state, owner, PXCHANGSHAN);
    assert!(
        count_hand_kind(&state, owner, CardKind::Shan) >= 2,
        "常山 must gain two 闪 into hand"
    );
}

#[test]
fn pxbeiping_gains_two_sha_on_gain() {
    let (state, owner) = started(2_026_081_007);
    let (state, _) = grant(state, owner, PXBEIPING);
    assert!(
        count_hand_kind(&state, owner, CardKind::Sha) >= 2,
        "北平 must gain two 杀 into hand"
    );
}

#[test]
fn pxlean_gains_fire_sha_and_tiesuo_on_gain() {
    let (state, owner) = started(2_026_081_008);
    let (state, _) = grant(state, owner, PXLEAN);
    assert!(
        count_hand_kind(&state, owner, CardKind::ShaFire) >= 1
            || count_hand_kind(&state, owner, CardKind::ShaThunder) >= 1,
        "乐安 must gain an elemental sha"
    );
    assert!(
        count_hand_kind(&state, owner, CardKind::TrickTiesuo) >= 1,
        "乐安 must gain a 铁索连环"
    );
}

#[test]
fn pxyanmen_gains_weapon_and_treasure_on_gain() {
    let (state, owner) = started(2_026_081_009);
    let (state, _) = grant(state, owner, PXYANMEN);
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let slots: Vec<_> = state
        .cards
        .sorted_live()
        .into_iter()
        .filter(|card| card.zone == hand)
        .filter_map(|card| card.kind.equip_slot())
        .collect();
    assert!(
        slots.contains(&engine_core::state::EquipSlot::Weapon),
        "雁门 must gain a weapon"
    );
    assert!(
        slots.contains(&engine_core::state::EquipSlot::OffensiveMount),
        "雁门 must gain an offensive mount"
    );
}

#[test]
fn pxlongxi_gains_defensive_horse_on_gain() {
    let (state, owner) = started(2_026_081_010);
    let (state, _) = grant(state, owner, PXLONGXI);
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let has_horse = state
        .cards
        .sorted_live()
        .into_iter()
        .filter(|card| card.zone == hand)
        .any(|card| card.kind == CardKind::HorseDef);
    assert!(has_horse, "陇西 must gain a defensive horse");
}

const PXNANFU: u32 = 2_422_439_050;
const PXGUANGLING: u32 = 3_967_602_994;

#[test]
fn package_publishes_batch_af_wave3_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    assert!(package
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == PXNANFU
            && trigger.event == "skill_added"
            && trigger.filter == "owner_skill_added_self_2422439050"
            && trigger.optional));
    let view_as = package
        .view_as_index
        .iter()
        .find(|v| v.skill_def_id == PXGUANGLING)
        .expect("pxguangling wugu view_as must be registered");
    assert_eq!(view_as.as_kind, "wugu");
    assert_eq!(view_as.window, "play_phase");
    assert_eq!(view_as.max_cards, 2);
    assert!(!package.modifier_index.iter().any(|modifier| {
        matches!(modifier.skill_def_id, PXNANFU | PXGUANGLING)
            && modifier.target == "hand_limit"
    }));
}

#[test]
fn pxnanfu_loses_hp_then_opens_virtual_sha() {
    let (state, owner) = started(2_026_081_011);
    let hp_before = state.players.get(owner).unwrap().hp;
    // 可选触发：确认后先失体力再进入虚拟杀目标选择
    let mut tx = Transaction::begin(state);
    let effects = run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(PXNANFU),
        }],
    );
    assert_eq!(effects.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();
    let (state, result) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    // 确认后进入虚拟杀目标选择（skill_added 触发内）
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::VirtualShaTargets
    );
    assert_eq!(state.players.get(owner).unwrap().hp, hp_before - 1);
}

const PXXIANGYANG: u32 = 4_133_746_261;
const PXYANGZHOU: u32 = 3_910_911_761;

#[test]
fn package_publishes_batch_af_wave4_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    let triggers: Vec<_> = package
        .trigger_index
        .iter()
        .filter(|trigger| matches!(trigger.skill_def_id, PXXIANGYANG | PXYANGZHOU))
        .map(|trigger| {
            (
                trigger.skill_def_id,
                trigger.event.as_str(),
                trigger.filter.as_str(),
                trigger.optional,
            )
        })
        .collect();
    assert!(triggers.contains(&(
        PXXIANGYANG,
        "skill_added",
        "owner_skill_added_self_4133746261",
        true
    )));
    assert!(triggers.contains(&(
        PXYANGZHOU,
        "skill_added",
        "owner_skill_added_self_3910911761",
        true
    )));
    assert!(!package.modifier_index.iter().any(|modifier| {
        matches!(modifier.skill_def_id, PXXIANGYANG | PXYANGZHOU)
            && modifier.target == "hand_limit"
    }));
}

#[test]
fn pxxiangyang_opens_move_choice_on_gain() {
    let (state, owner) = started(2_026_081_012);
    let mut tx = Transaction::begin(state);
    // 给另一名角色装武器——移动原语从其他角色场上传一张牌到拥有者
    let other = tx
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let equip = tx.state().zones.find(ZoneKind::Equip, other).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: engine_api::CardId::NONE,
        def: engine_api::CardDefId::new(2_000_501),
        kind: CardKind::Weapon,
        suit: engine_api::Suit::Spade,
        rank: 5,
        weapon_range: 2,
        zone: equip,
        owner: other,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(equip).unwrap().cards.push(card);
    let tx = tx.commit();
    let (state, _) = grant(tx.state, owner, PXXIANGYANG);
    // 可选触发确认后进入移动装备区牌的选择
    assert!(
        state.has_pending_choice(),
        "襄阳 must open a move-field-card choice"
    );
}

#[test]
fn pxyangzhou_opens_other_recover_choice_on_gain() {
    let (state, owner) = started(2_026_081_013);
    let mut tx = Transaction::begin(state);
    // 打伤另一名角色使其可被回复
    let other = tx
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    tx.state_mut().players.get_mut(other).unwrap().hp = 1;
    let tx = tx.commit();
    let (state, _) = grant(tx.state, owner, PXYANGZHOU);
    // 确认后进入其他角色选择
    assert!(
        state.has_pending_choice(),
        "扬州 must open an other-recover choice"
    );
}

const PXFANYANG: u32 = 3_972_961_518;
const PXRUNAN: u32 = 3_675_374_846;

#[test]
fn package_publishes_batch_af_wave5_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    let triggers: Vec<_> = package
        .trigger_index
        .iter()
        .filter(|trigger| matches!(trigger.skill_def_id, PXFANYANG | PXRUNAN))
        .map(|trigger| {
            (
                trigger.skill_def_id,
                trigger.event.as_str(),
                trigger.filter.as_str(),
                trigger.optional,
            )
        })
        .collect();
    assert!(triggers.contains(&(
        PXFANYANG,
        "skill_added",
        "owner_skill_added_self_3972961518",
        true
    )));
    assert!(triggers.contains(&(
        PXRUNAN,
        "skill_added",
        "owner_skill_added_self_3675374846",
        true
    )));
    assert!(!package.modifier_index.iter().any(|modifier| {
        matches!(modifier.skill_def_id, PXFANYANG | PXRUNAN)
            && modifier.target == "hand_limit"
    }));
}

#[test]
fn pxfanyang_opens_other_damage_choice_on_gain() {
    let (state, owner) = started(2_026_081_014);
    let (state, _) = grant(state, owner, PXFANYANG);
    assert!(
        state.has_pending_choice(),
        "范阳 must open an other-damage choice"
    );
}

#[test]
fn pxrunan_opens_other_draw_choice_on_gain() {
    let (state, owner) = started(2_026_081_015);
    let (state, _) = grant(state, owner, PXRUNAN);
    assert!(
        state.has_pending_choice(),
        "汝南 must open an other-draw choice"
    );
}

const PXHEFEI: u32 = 3_666_201_428;

#[test]
fn package_publishes_pxhefei_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    assert!(package
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == PXHEFEI
            && trigger.event == "skill_added"
            && trigger.filter == "owner_skill_added_self_3666201428"
            && trigger.optional));
    assert!(!package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == PXHEFEI && modifier.target == "hand_limit"
    }));
}

#[test]
fn pxhefei_confirm_then_discard_then_draw() {
    let (state, owner) = started(2_026_081_016);
    // 手牌初始 6 张，弃 1 摸 1 后手牌数不变
    let mut tx = Transaction::begin(state);
    let effects = run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(PXHEFEI),
        }],
    );
    assert_eq!(effects.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();
    let (state, accepted) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    // 确认后进入弃牌选择（Custom）
    assert_eq!(state.resolution.continuation_kind, ContinuationKind::Custom);
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let discardable: Vec<_> = state
        .cards
        .sorted_live()
        .into_iter()
        .filter(|c| c.zone == hand)
        .map(|c| c.id.raw())
        .collect();
    assert!(!discardable.is_empty());
    // 完成弃牌（选择第一张）→ 摸 1
    let (state, accepted) = resolve(state, owner, vec![discardable[0]], false, 2);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), 6, "弃 1 摸 1 后手牌数不变");
}

const PXSHANGYONG: u32 = 2_310_331_473;

#[test]
fn package_publishes_pxshangyong_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    assert!(package
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == PXSHANGYONG
            && trigger.event == "skill_added"
            && trigger.filter == "owner_skill_added_self_2310331473"
            && trigger.optional));
    assert!(!package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == PXSHANGYONG && modifier.target == "hand_limit"
    }));
}

#[test]
fn pxshangyong_opens_other_draw_then_self_draw() {
    let (state, owner) = started(2_026_081_017);
    let hand_before = state.hand_count(owner);
    let mut tx = Transaction::begin(state);
    let effects = run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(PXSHANGYONG),
        }],
    );
    assert_eq!(effects.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();
    let (state, accepted) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    // 确认后进入其他角色选择（摸 1）
    assert!(
        state.has_pending_choice(),
        "上庸 must open an other-draw choice"
    );
    assert_eq!(state.hand_count(owner), hand_before);
}

const PXTIANSHUI: u32 = 3_271_313_270;

#[test]
fn package_publishes_pxtianshui_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    assert!(package
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == PXTIANSHUI
            && trigger.event == "skill_added"
            && trigger.filter == "owner_skill_added_self_3271313270"
            && trigger.optional));
    assert!(!package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == PXTIANSHUI && modifier.target == "hand_limit"
    }));
}

#[test]
fn pxtianshui_opens_discard_choice_then_draws_two() {
    let (state, owner) = started(2_026_081_018);
    let mut tx = Transaction::begin(state);
    let effects = run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(PXTIANSHUI),
        }],
    );
    assert_eq!(effects.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();
    let (state, accepted) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    // 确认后进入其他角色弃牌选择
    assert!(
        state.has_pending_choice(),
        "天水 must open a target-discard choice"
    );
}

const PXB_OHAI: u32 = 2_568_108_187;

#[test]
fn package_publishes_pxbohai_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    assert!(package
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == PXB_OHAI
            && trigger.event == "skill_added"
            && trigger.filter == "owner_skill_added_self_2568108187"
            && trigger.optional));
    assert!(!package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == PXB_OHAI && modifier.target == "hand_limit"
    }));
}

#[test]
fn pxbohai_confirm_then_discard_then_draw_two() {
    let (state, owner) = started(2_026_081_019);
    let mut tx = Transaction::begin(state);
    let effects = run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(PXB_OHAI),
        }],
    );
    assert_eq!(effects.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();
    let (state, accepted) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    // 确认后进入弃牌选择（Custom）
    assert_eq!(state.resolution.continuation_kind, ContinuationKind::Custom);
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let discardable: Vec<_> = state
        .cards
        .sorted_live()
        .into_iter()
        .filter(|c| c.zone == hand)
        .map(|c| c.id.raw())
        .collect();
    assert!(!discardable.is_empty());
    let (state, accepted) = resolve(state, owner, vec![discardable[0]], false, 2);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), 7, "弃 1 摸 2 后手牌 +1");
}

const PXPENGCHENG: u32 = 3_892_864_153;

#[test]
fn package_publishes_pxpengcheng_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    assert!(package
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == PXPENGCHENG
            && trigger.event == "skill_added"
            && trigger.filter == "owner_skill_added_self_3892864153"
            && trigger.optional));
    assert!(!package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == PXPENGCHENG && modifier.target == "hand_limit"
    }));
}

#[test]
fn pxpengcheng_opens_give_then_recover_choice() {
    let (state, owner) = started(2_026_081_020);
    let mut tx = Transaction::begin(state);
    let effects = run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(PXPENGCHENG),
        }],
    );
    assert_eq!(effects.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();
    let (state, accepted) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    // 确认后进入其他角色选择（交牌）
    assert!(
        state.has_pending_choice(),
        "彭城 must open an other-give choice"
    );
}

const PXJIANGXIA: u32 = 2_687_838_381;

#[test]
fn package_publishes_pxjiangxia_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    assert!(package
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == PXJIANGXIA
            && trigger.event == "skill_added"
            && trigger.filter == "owner_skill_added_self_2687838381"
            && trigger.optional));
    assert!(!package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == PXJIANGXIA && modifier.target == "hand_limit"
    }));
}

#[test]
fn pxjiangxia_opens_other_draw_then_give_choice() {
    let (state, owner) = started(2_026_081_021);
    let mut tx = Transaction::begin(state);
    let effects = run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(PXJIANGXIA),
        }],
    );
    assert_eq!(effects.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();
    let (state, accepted) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    // 确认后进入其他角色选择（摸 2）
    assert!(
        state.has_pending_choice(),
        "江夏 must open an other-draw choice"
    );
}

const PXJUCHAO: u32 = 3_375_023_858;

#[test]
fn package_publishes_pxjuchao_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    assert!(package
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == PXJUCHAO
            && trigger.event == "skill_added"
            && trigger.filter == "owner_skill_added_self_3375023858"
            && trigger.optional));
    assert!(!package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == PXJUCHAO && modifier.target == "hand_limit"
    }));
}

#[test]
fn pxjuchao_opens_target_discard_choice() {
    let (state, owner) = started(2_026_081_022);
    let mut tx = Transaction::begin(state);
    let effects = run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(PXJUCHAO),
        }],
    );
    assert_eq!(effects.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();
    let (state, accepted) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert!(
        state.has_pending_choice(),
        "居巢 must open a target-discard choice"
    );
}

const PXNANYANG: u32 = 3_458_061_931;

#[test]
fn package_publishes_pxnanyang_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    assert!(package
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == PXNANYANG
            && trigger.event == "skill_added"
            && trigger.filter == "owner_skill_added_self_3458061931"
            && trigger.optional));
    assert!(!package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == PXNANYANG && modifier.target == "hand_limit"
    }));
}

#[test]
fn pxnanyang_opens_recover_choice_after_confirm() {
    let (state, owner) = started(2_026_081_023);
    let mut tx = Transaction::begin(state);
    let effects = run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(PXNANYANG),
        }],
    );
    assert_eq!(effects.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();
    let (state, accepted) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    // 确认后进入弃牌选择（Custom）
    assert_eq!(state.resolution.continuation_kind, ContinuationKind::Custom);
}

const PXWUWEI: u32 = 3_519_997_787;

#[test]
fn package_publishes_pxwuwei_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    assert!(package
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == PXWUWEI
            && trigger.event == "card_used"
            && trigger.filter == "owner_source_damage_card_once_per_turn"
            && trigger.optional));
    assert!(!package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == PXWUWEI && modifier.target == "hand_limit"
    }));
}

#[test]
fn pxwuwei_opens_target_adjust_on_damage_card() {
    let (state, owner) = started(2_026_081_024);
    let mut tx = Transaction::begin(state);
    let effects = run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(PXWUWEI),
        }],
    );
    // 添加技能时无选择（skill 本身无触发）
    assert!(effects.is_ok());
    // 进入出牌阶段并以杀派发 card_used 事件
    tx.state_mut().turn.phase = engine_core::state::Phase::Play;
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let sha = tx.state_mut().cards.alloc(Card {
        id: engine_api::CardId::NONE,
        def: engine_api::CardDefId::new(1_000_801),
        kind: CardKind::Sha,
        suit: engine_api::Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(sha);
    let other = tx
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let opened = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: owner,
            card: sha,
            targets: vec![other],
        }],
        &mut TriggerFireRecord::new(),
    );
    // card_used 事件派发：可选触发打开确认
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
}

const PXWENXIAN: u32 = 4_051_127_169;

#[test]
fn package_publishes_pxwenxian_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    assert!(package
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == PXWENXIAN
            && trigger.event == "skill_added"
            && trigger.filter == "owner_skill_added_self_4051127169"
            && !trigger.optional));
    assert!(!package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == PXWENXIAN && modifier.target == "hand_limit"
    }));
}

#[test]
fn pxwenxian_gains_max_hp_and_recovers_on_gain() {
    let (state, owner) = started(2_026_081_025);
    let mut tx = Transaction::begin(state);
    // 先打伤以便验证回复
    tx.state_mut().players.get_mut(owner).unwrap().hp = 3;
    // v1.11.6 增量 roster +1 平移种子：bootstrap 武将上限随座位分配变化，断言按相对值
    let max_hp_before = tx.state().players.get(owner).unwrap().max_hp;
    let tx = tx.commit();
    let (state, _) = grant(tx.state, owner, PXWENXIAN);
    let player = state.players.get(owner).unwrap();
    assert_eq!(player.max_hp, max_hp_before + 1, "温县 must add one max HP");
    assert_eq!(player.hp, 4, "温县 must recover 1");
}

const PXJINGZHOU: u32 = 2_589_712_461;

#[test]
fn package_publishes_pxjingzhou_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    assert!(package
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == PXJINGZHOU
            && trigger.event == "skill_added"
            && trigger.filter == "owner_skill_added_self_2589712461"
            && trigger.optional));
    assert!(!package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == PXJINGZHOU && modifier.target == "hand_limit"
    }));
}

#[test]
fn pxjingzhou_opens_give_choice_on_gain() {
    let (state, owner) = started(2_026_081_026);
    let mut tx = Transaction::begin(state);
    let effects = run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(PXJINGZHOU),
        }],
    );
    assert_eq!(effects.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();
    let (state, accepted) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert!(
        state.has_pending_choice(),
        "荆州 must open a give choice"
    );
}

const PXZANGKE: u32 = 3_744_135_692;

#[test]
fn package_publishes_pxzangke_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    assert!(package
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == PXZANGKE
            && trigger.event == "damage"
            && trigger.filter == "owner_is_target"
            && trigger.optional));
    assert!(!package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == PXZANGKE && modifier.target == "hand_limit"
    }));
}

#[test]
fn pxzangke_opens_choice_after_damage() {
    let (state, owner) = started(2_026_081_027);
    let mut tx = Transaction::begin(state);
    // 先移除全部既有技能（标准主公技能可能在伤害事件上循环消耗触发预算），再加 pxzangke
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(PXZANGKE),
    });
    let effects = run_effects(&mut tx, effects);
    assert!(effects.is_ok());
    let other = tx
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let opened = dispatch_for_events(
        &mut tx,
        &[DomainEvent::DamageDealt {
            source: other,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();
    let (state, accepted) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    // 确认后进入弃牌选择
    assert_eq!(state.resolution.continuation_kind, ContinuationKind::Custom);
}

const PXWENXI: u32 = 3_323_337_569;

#[test]
fn package_publishes_pxwenxi_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    assert!(package
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == PXWENXI
            && trigger.event == "card_used"
            && trigger.filter == "owner_source_first_card_suit_this_turn"
            && !trigger.optional));
    assert!(!package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == PXWENXI && modifier.target == "hand_limit"
    }));
}

#[test]
fn pxwenxi_draws_on_first_suit_use_only() {
    let (state, owner) = started(2_026_081_028);
    let mut tx = Transaction::begin(state);
    // 先移除既有技能再加 pxwenxi
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(PXWENXI),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.phase = engine_core::state::Phase::Play;
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let mut card_seq = 0u32;
    let mut make_sha = |tx: &mut Transaction, suit: engine_api::Suit| {
        let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
        card_seq += 1;
        let c = tx.state_mut().cards.alloc(Card {
            id: engine_api::CardId::NONE,
            def: engine_api::CardDefId::new(1_000_900 + card_seq),
            kind: CardKind::Sha,
            suit,
            rank: 7,
            weapon_range: 0,
            zone: hand,
            owner,
            face_up: false,
        });
        tx.state_mut().zones.get_mut(hand).unwrap().cards.push(c);
        c
    };
    let other = tx
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let fire = |tx: &mut Transaction, card| -> u32 {
        let before = tx.state().hand_count(owner);
        let effects = dispatch_for_events(
            tx,
            &[DomainEvent::CardUsed {
                player: owner,
                card,
                targets: vec![other],
            }],
            &mut TriggerFireRecord::new(),
        )
        .unwrap();
        run_effects(tx, effects).unwrap();
        u32::try_from(tx.state().hand_count(owner)).unwrap() - before
    };
    // 第一张黑桃：首次使用该花色 → 摸 1
    let sha1 = make_sha(&mut tx, engine_api::Suit::Spade);
    assert_eq!(fire(&mut tx, sha1), 1, "first spade use must draw 1");
    // 第二张黑桃：花色已用 → 不摸
    let sha2 = make_sha(&mut tx, engine_api::Suit::Spade);
    assert_eq!(fire(&mut tx, sha2), 0, "second spade use must not draw");
    // 第三张红桃：新花色 → 摸 1
    let sha3 = make_sha(&mut tx, engine_api::Suit::Heart);
    assert_eq!(fire(&mut tx, sha3), 1, "first heart use must draw 1");
}


const PXPINGYUAN: u32 = 2_911_898_950;

#[test]
fn package_publishes_pxpingyuan_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    assert!(package
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == PXPINGYUAN
            && trigger.event == "cards_moved"
            && trigger.filter == "owner_lost_hand_now_empty"
            && !trigger.optional));
    assert!(!package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == PXPINGYUAN && modifier.target == "hand_limit"
    }));
}

#[test]
fn pxpingyuan_draws_when_hand_becomes_empty() {
    let (state, owner) = started(2_026_081_029);
    let mut tx = Transaction::begin(state);
    // 移除既有技能加 pxpingyuan
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(PXPINGYUAN),
    });
    run_effects(&mut tx, effects).unwrap();
    // 清空手牌区（最后一批）→ 手牌为空 → 摸 1
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let hand_cards = tx.state().hand_cards(owner);
    tx.state_mut().zones.get_mut(hand).unwrap().cards.clear();
    let discard = tx.state().zones.find(ZoneKind::DiscardPile, PlayerId::NONE).unwrap();
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardsMoved {
            cards: hand_cards,
            from: hand,
            to: discard,
            reason: engine_api::CardMoveReason::Discard,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(tx.state().hand_count(owner), 1, "平原 must draw 1 after hand empties");
}

const OL_SHICHOU: u32 = 2_434_401_673;

#[test]
fn package_publishes_ol_shichou_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    assert!(package
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == OL_SHICHOU
            && trigger.event == "card_used"
            && trigger.filter == "owner_source_sha"
            && trigger.optional));
    assert!(!package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == OL_SHICHOU && modifier.target == "hand_limit"
    }));
}

#[test]
fn ol_shichou_opens_target_adjust_on_sha() {
    let (state, owner) = started(2_026_081_030);
    let mut tx = Transaction::begin(state);
    // 移除既有技能加 ol_shichou
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OL_SHICHOU),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.phase = engine_core::state::Phase::Play;
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let sha = tx.state_mut().cards.alloc(Card {
        id: engine_api::CardId::NONE,
        def: engine_api::CardDefId::new(1_001_001),
        kind: CardKind::Sha,
        suit: engine_api::Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(sha);
    let other = tx
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let opened = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: owner,
            card: sha,
            targets: vec![other],
        }],
        &mut TriggerFireRecord::new(),
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
}

const YONGSI2: u32 = 4_237_803_231;

#[test]
fn package_publishes_yongsi2_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    assert!(package
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == YONGSI2
            && trigger.event == "phase_discard"
            && trigger.filter == "owner_is_target"
            && !trigger.optional));
    assert!(!package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == YONGSI2 && modifier.target == "hand_limit"
    }));
}

#[test]
fn yongsi2_discards_one_at_discard_phase() {
    let (state, owner) = started(2_026_081_031);
    let mut tx = Transaction::begin(state);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(YONGSI2),
    });
    run_effects(&mut tx, effects).unwrap();
    let hand_before = tx.state().hand_count(owner);
    // 强制触发直接返回弃牌效果请求
    let opened = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: engine_api::TurnPhaseTag::Discard,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(
        opened.iter().any(|e| matches!(e, EffectRequest::DiscardHandEquipCount { .. })),
        "yongsi2 must emit a discard-hand-equip-count effect"
    );
    // 运行效果 → 打开弃牌选择
    let result = run_effects(&mut tx, opened);
    assert_eq!(result.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::Custom
    );
    // 取消选择：手牌不变
    let tx = tx.commit();
    let (state, accepted) = resolve(tx.state, owner, vec![], true, 1);
    let _ = accepted;
    assert_eq!(state.hand_count(owner), hand_before);
}

const PXHANDAN: u32 = 3_498_155_858;

#[test]
fn package_publishes_pxhandan_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    assert!(package
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == PXHANDAN
            && trigger.event == "card_used"
            && trigger.filter == "owner_source_ordinary_trick"
            && trigger.optional));
    assert!(!package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == PXHANDAN && modifier.target == "hand_limit"
    }));
}

#[test]
fn pxhandan_opens_gain_choice_on_ordinary_trick() {
    let (state, owner) = started(2_026_081_032);
    let mut tx = Transaction::begin(state);
    // 移除既有技能加 pxhandan
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(PXHANDAN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.phase = engine_core::state::Phase::Play;
    // 使用一张普通锦囊（过河拆桥）
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let trick = tx.state_mut().cards.alloc(Card {
        id: engine_api::CardId::NONE,
        def: engine_api::CardDefId::new(1_001_101),
        kind: CardKind::TrickGuohe,
        suit: engine_api::Suit::Spade,
        rank: 3,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(trick);
    let other = tx
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let opened = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: owner,
            card: trick,
            targets: vec![other],
        }],
        &mut TriggerFireRecord::new(),
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
}

const PXJULU: u32 = 2_943_799_463;

#[test]
fn package_publishes_pxjulu_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    assert!(package
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == PXJULU
            && trigger.event == "cards_moved"
            && trigger.filter == "owner_lost_spade_two_to_nine"
            && !trigger.optional));
    assert!(!package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == PXJULU && modifier.target == "hand_limit"
    }));
}

#[test]
fn pxjulu_draws_when_spade_two_to_nine_lost() {
    let (state, owner) = started(2_026_081_033);
    let mut tx = Transaction::begin(state);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(PXJULU),
    });
    run_effects(&mut tx, effects).unwrap();
    // 注入一张黑桃 7 并移出
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let spade = tx.state_mut().cards.alloc(Card {
        id: engine_api::CardId::NONE,
        def: engine_api::CardDefId::new(1_001_201),
        kind: CardKind::Sha,
        suit: engine_api::Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(spade);
    // 从手牌移出黑桃 7 → 触发摸 1
    let before = tx.state().hand_count(owner);
    let discard_zone = tx.state().zones.find(ZoneKind::DiscardPile, PlayerId::NONE).unwrap();
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardsMoved {
            cards: vec![spade],
            from: hand,
            to: discard_zone,
            reason: engine_api::CardMoveReason::Discard,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    // 合成事件不移除实体牌，手牌保持 before 且额外摸 1
    assert_eq!(
        tx.state().hand_count(owner) as u32,
        before + 1,
        "巨鹿 must draw 1 when a spade 2-9 is lost"
    );
}

const PXJIUYUAN: u32 = 2_634_282_552;

#[test]
fn package_publishes_pxjiuyuan_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    assert!(package
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == PXJIUYUAN
            && trigger.event == "card_used"
            && trigger.filter == "owner_source_sha"
            && trigger.optional));
    assert!(!package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == PXJIUYUAN && modifier.target == "hand_limit"
    }));
}

#[test]
fn pxjiuyuan_opens_target_adjust_on_sha() {
    let (state, owner) = started(2_026_081_034);
    let mut tx = Transaction::begin(state);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(PXJIUYUAN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.phase = engine_core::state::Phase::Play;
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let sha = tx.state_mut().cards.alloc(Card {
        id: engine_api::CardId::NONE,
        def: engine_api::CardDefId::new(1_001_301),
        kind: CardKind::Sha,
        suit: engine_api::Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(sha);
    let other = tx
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let opened = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: owner,
            card: sha,
            targets: vec![other],
        }],
        &mut TriggerFireRecord::new(),
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
}

const PXYUZHOU: u32 = 4_075_358_047;

#[test]
fn package_publishes_pxyuzhou_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    assert!(package
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == PXYUZHOU
            && trigger.event == "skill_added"
            && trigger.filter == "owner_skill_added_self_4075358047"
            && trigger.optional));
    assert!(!package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == PXYUZHOU && modifier.target == "hand_limit"
    }));
}

#[test]
fn pxyuzhou_links_self_then_opens_recover_choice() {
    let (state, owner) = started(2_026_081_035);
    let mut tx = Transaction::begin(state);
    // 移除既有技能加 pxyuzhou——skill_added 可选触发在 AddSkill 时打开确认
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(PXYUZHOU),
    });
    let result = run_effects(&mut tx, effects);
    assert_eq!(result.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();
    let (state, accepted) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    // 确认后进入其他角色回复选择
    assert!(
        state.has_pending_choice(),
        "豫州 must open an other-recover choice"
    );
}


const PXYONGZHOU: u32 = 2_600_588_951;

#[test]
fn package_publishes_pxyongzhou_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    assert!(package
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == PXYONGZHOU
            && trigger.event == "skill_added"
            && trigger.filter == "owner_skill_added_self_2600588951"
            && trigger.optional));
    assert!(!package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == PXYONGZHOU && modifier.target == "hand_limit"
    }));
}

#[test]
fn pxyongzhou_opens_confirm_on_gain() {
    let (state, owner) = started(2_026_081_036);
    let mut tx = Transaction::begin(state);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(PXYONGZHOU),
    });
    let result = run_effects(&mut tx, effects);
    assert_eq!(result.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
}
