//! Golden for XIANDING 威关银屏【绍威/涤仇】（ABI 594）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, MarkDefId, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const V_GUANYINPING: u32 = 4_134_246_215;
const DCSHAOWEI: u32 = 3_440_541_593;
const DCDICHOU: u32 = 3_304_378_096;
const DCDICHOU_EFF: u32 = 2_754_298_115;
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(skill_def: u32) -> (GameState, [PlayerId; 3]) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package())
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 20_260_803, "identity-standard").unwrap();
    let players: [PlayerId; 3] = tx.state().players.alive_by_seat().try_into().unwrap();
    let effects = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .chain(players.iter().map(|player| EffectRequest::DiscardFromHand {
            player: *player,
            count: u32::MAX,
        }))
        .chain(std::iter::once(EffectRequest::AddSkill {
            player: players[0],
            skill_def: SkillDefId::new(skill_def),
        }))
        .collect();
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = players[0];
    tx.state_mut().turn.phase = Phase::Prepare;
    tx.state_mut().revision = 1;
    (tx.commit().state, players)
}

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    zone_kind: ZoneKind,
    def: u32,
    kind: CardKind,
    suit: Suit,
) -> CardId {
    let zone = state.zones.find(zone_kind, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn skill_id(state: &GameState, owner: PlayerId, def: u32) -> engine_api::SkillId {
    state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == def)
        .unwrap()
        .id
}

fn run(state: GameState, effect: EffectRequest) -> GameState {
    let mut tx = Transaction::begin(state);
    match run_effects(&mut tx, vec![effect]) {
        Ok(_) | Err(EffectError::ChoiceRequested) => {}
        Err(error) => panic!("effect failed: {error}"),
    }
    tx.commit().state
}

fn choose(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let (state, result) = apply_command(
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
        matches!(result.result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    state
}

fn mark_count(state: &GameState, player: PlayerId, mark: u32) -> i32 {
    state
        .players
        .get(player)
        .unwrap()
        .marks
        .iter()
        .find(|(id, _)| *id == mark)
        .map_or(0, |(_, count)| *count)
}

#[test]
fn package_closes_v_guanyinping_contract() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 594);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == V_GUANYINPING)
        .unwrap();
    assert_eq!(character.name_key, "v_guanyinping");
    assert_eq!(character.skills, [DCSHAOWEI, DCDICHOU]);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == DCDICHOU_EFF));
}

#[test]
fn dcshaowei_gains_at_most_three_red_draw_cards_once_per_turn() {
    let (mut state, [owner, _, _]) = setup(DCSHAOWEI);
    let lost = inject(
        &mut state,
        owner,
        ZoneKind::Hand,
        10,
        CardKind::Shan,
        Suit::Diamond,
    );
    for (def, suit) in [
        (11, Suit::Heart),
        (12, Suit::Diamond),
        (13, Suit::Heart),
        (14, Suit::Diamond),
    ] {
        inject(
            &mut state,
            PlayerId::NONE,
            ZoneKind::DrawPile,
            def,
            CardKind::Shan,
            suit,
        );
    }
    inject(
        &mut state,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        15,
        CardKind::Shan,
        Suit::Spade,
    );
    let skill = skill_id(&state, owner, DCSHAOWEI);
    state = run(
        state,
        EffectRequest::ResolveDcshaowei {
            player: owner,
            action: 0,
            cards: vec![lost],
            skill,
        },
    );
    assert_eq!(state.hand_count(owner), 4);
    state = run(
        state,
        EffectRequest::ResolveDcshaowei {
            player: owner,
            action: 0,
            cards: vec![lost],
            skill,
        },
    );
    assert_eq!(state.hand_count(owner), 4);
}

#[test]
fn dcdichou_resolves_begin_bonus_and_end_fire_damage() {
    let (mut state, [owner, target, _]) = setup(DCDICHOU);
    let discarded = inject(
        &mut state,
        target,
        ZoneKind::Hand,
        20,
        CardKind::TrickWuzhong,
        Suit::Club,
    );
    let skill = skill_id(&state, owner, DCDICHOU);
    state = run(
        state,
        EffectRequest::AddMark {
            player: target,
            mark: MarkDefId::new(DCDICHOU),
            delta: 1,
        },
    );
    state = run(
        state,
        EffectRequest::ResolveDcdichou {
            player: owner,
            target: PlayerId::NONE,
            card: CardId::NONE,
            action: 0,
            helper_skill_def: SkillDefId::new(DCDICHOU_EFF),
            skill,
        },
    );
    state = choose(state, owner, vec![1], 1);
    state = choose(state, owner, vec![target.raw()], 2);
    state = choose(state, owner, vec![discarded.raw()], 3);
    assert_eq!(state.players.get(owner).unwrap().disabled_equip_slots[0], 1);
    assert_eq!(
        state
            .zones
            .get(state.cards.get(discarded).unwrap().zone)
            .unwrap()
            .kind,
        ZoneKind::DiscardPile
    );
    assert!(mark_count(&state, owner, DCDICHOU_EFF) > 0);

    state.turn.phase = Phase::End;
    let hp = state.players.get(target).unwrap().hp;
    state = run(
        state,
        EffectRequest::ResolveDcdichou {
            player: owner,
            target: PlayerId::NONE,
            card: CardId::NONE,
            action: 10,
            helper_skill_def: SkillDefId::new(DCDICHOU_EFF),
            skill,
        },
    );
    state = choose(state, owner, vec![target.raw()], 4);
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);
    assert_eq!(mark_count(&state, target, DCDICHOU), 0);
}

/// R01（UP37 `19d86519`）：结束阶段先清**所有**仇标记；本回合已移除仇的角色
/// 仍在候选中；候选非空时至少选一人；伤害只作用于所选者且按座位序。
#[test]
fn dcdichou_end_clears_all_marks_and_offers_history_candidate() {
    let (mut state, [owner, target, other]) = setup(DCDICHOU);
    state = run(
        state,
        EffectRequest::AddMark {
            player: target,
            mark: MarkDefId::new(DCDICHOU),
            delta: 1,
        },
    );
    state = run(
        state,
        EffectRequest::AddMark {
            player: other,
            mark: MarkDefId::new(DCDICHOU),
            delta: 2,
        },
    );
    // 本回合早些时候移除 target 的最后一点仇：历史仍需保留候选资格。
    state = run(
        state,
        EffectRequest::RemoveMark {
            player: target,
            mark: MarkDefId::new(DCDICHOU),
            amount: 0,
        },
    );
    assert_eq!(mark_count(&state, target, DCDICHOU), 0);
    // 历史必须随状态保存/恢复（跨进程存档路径）。
    let restored: GameState = serde_json::from_slice(&serde_json::to_vec(&state).unwrap()).unwrap();
    assert!(restored.turn.removed_marks.contains(&(DCDICHOU, target)));
    state = restored;
    state.turn.phase = Phase::End;
    let hp_target = state.players.get(target).unwrap().hp;
    let hp_other = state.players.get(other).unwrap().hp;
    let skill = skill_id(&state, owner, DCDICHOU);
    state = run(
        state,
        EffectRequest::ResolveDcdichou {
            player: owner,
            target: PlayerId::NONE,
            card: CardId::NONE,
            action: 10,
            helper_skill_def: SkillDefId::new(DCDICHOU_EFF),
            skill,
        },
    );
    // 选择前全量清标记（含未选者）。
    assert_eq!(mark_count(&state, target, DCDICHOU), 0);
    assert_eq!(mark_count(&state, other, DCDICHOU), 0);
    assert!(state.turn.removed_marks.is_empty());

    // 候选非空 -> 不能空选。
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (after_reject, result) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(41),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Rejected { .. }),
        "空选必须拒绝：{result:?}"
    );
    assert_eq!(after_reject.players.get(target).unwrap().hp, hp_target);
    assert_eq!(after_reject.players.get(other).unwrap().hp, hp_other);

    // 选择挂起时再次序列化/恢复，仍能按原 payload 结算。
    state = serde_json::from_slice(&serde_json::to_vec(&state).unwrap()).unwrap();
    // 已在本回合移除仇的 target 仍可被选择并受到 1 点火伤；未选者不受伤。
    state = choose(state, owner, vec![target.raw()], 42);
    assert_eq!(state.players.get(target).unwrap().hp, hp_target - 1);
    assert_eq!(state.players.get(other).unwrap().hp, hp_other);
    assert_eq!(mark_count(&state, target, DCDICHOU), 0);
    assert_eq!(mark_count(&state, other, DCDICHOU), 0);
}
