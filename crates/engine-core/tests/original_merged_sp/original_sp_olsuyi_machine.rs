//! Golden for SP 俗义 olsuyi (ABI 739, tag 760):
//! 出牌阶段限一次，选一名其他角色：牌堆顶向下检索第一张其可装备的装备
//! 牌令其获得并装备；然后对其视为使用一张【杀】。active 全管道驱动。

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const OLSUYI: u32 = 4_134_784_601;
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/sp/sp.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn started(seed: u64) -> (GameState, Vec<PlayerId>) {
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package())
        .unwrap();
    let players = tx.state().players.alive_by_seat();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    (tx.commit().state, players)
}

fn add_skill(state: &mut GameState, player: PlayerId, def: u32) -> engine_api::SkillId {
    let mut tx = Transaction::begin(state.clone());
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
    *state = tx.commit().state;
    state
        .players
        .get(player)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| state.skills.get(*skill).unwrap().def.raw() == def)
        .unwrap()
}

fn inject_draw_top(state: &mut GameState, kind: CardKind, tag: u32) -> CardId {
    let draw = state
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(96_000 + tag),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 1,
        zone: draw,
        owner: PlayerId::NONE,
        face_up: false,
    });
    state.zones.get_mut(draw).unwrap().cards.push(card);
    card
}

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

#[test]
fn package_publishes_olsuyi_active_contract() {
    let package = package();
    assert_eq!(package.manifest.rule_vm_abi_version, 745);
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == OLSUYI)
        .expect("olsuyi active entry");
    assert_eq!(active.uses_per_turn, Some(1));
    assert!(active
        .targets
        .as_ref()
        .is_some_and(|targets| { !targets.allow_self && targets.min == 1 && targets.max == 1 }));
}

#[test]
fn olsuyi_equips_target_from_deck_then_uses_virtual_sha() {
    let (mut state, players) = started(2_026_095_001);
    let owner = players[0];
    let target = players[1];
    state.turn.current = owner;
    state.turn.phase = Phase::Play;
    let skill = add_skill(&mut state, owner, OLSUYI);
    let weapon = inject_draw_top(&mut state, CardKind::Weapon, 1);
    let hp_before = state.players.get(target).unwrap().hp;

    let (state, accepted) = submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![target],
            card_targets: Vec::new(),
        },
        1,
    );
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    // 装备牌离开牌堆、为目标装备（目标手牌区不留）。
    let weapon_state = state.cards.get(weapon).unwrap();
    assert_eq!(weapon_state.owner, target);
    assert_ne!(
        weapon_state.zone,
        state
            .zones
            .find(ZoneKind::DrawPile, PlayerId::NONE)
            .unwrap()
    );
    assert_ne!(
        weapon_state.zone,
        state.zones.find(ZoneKind::Hand, target).unwrap()
    );
    // 虚拟杀完整结算：目标无闪 → 掉 1 体力。
    assert_eq!(state.players.get(target).unwrap().hp, hp_before - 1);

    // uses_per_turn 1：本回合再次发动被权威拒绝。
    let (_, rejected) = submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![target],
            card_targets: Vec::new(),
        },
        2,
    );
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
}

#[test]
fn olsuyi_without_equipment_in_deck_still_uses_virtual_sha() {
    let (mut state, players) = started(2_026_095_002);
    let owner = players[0];
    let target = players[1];
    state.turn.current = owner;
    state.turn.phase = Phase::Play;
    let skill = add_skill(&mut state, owner, OLSUYI);
    inject_draw_top(&mut state, CardKind::Tao, 2); // 非装备牌 → 跳过装备段
    let hp_before = state.players.get(target).unwrap().hp;

    let (state, accepted) = submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![target],
            card_targets: Vec::new(),
        },
        1,
    );
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, hp_before - 1);
}
