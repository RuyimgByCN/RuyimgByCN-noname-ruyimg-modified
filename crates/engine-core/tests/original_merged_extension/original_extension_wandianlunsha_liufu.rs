//! Golden for archived extension 玩点论杀 刘馥【筑城/夺气】（ABI 610）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const ZHUCHENG: u32 = 2_400_116_759;
const DUOQI: u32 = 2_671_845_513;
const SOURCE: &str = include_str!("../../../../content/original/ported/wandianlunsha/liufu.skill");

fn inject(state: &mut GameState, owner: PlayerId, kind: CardKind, serial: u32) -> CardId {
    let zone = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(96_100 + serial),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn skill(state: &GameState, owner: PlayerId, def: u32) -> engine_api::SkillId {
    state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == def)
        .unwrap()
        .id
}

fn choose(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> GameState {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
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
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    state
}

#[test]
fn zhucheng_stores_lost_hp_top_cards_defends_sha_and_duoqi_ends_play() {
    let package = content_compiler::compile_source_named("wandianlunsha-liufu", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 610);
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 20_260_803, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let attacker = players[1];
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
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(ZHUCHENG),
        },
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(DUOQI),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let participant = tx.state_mut().players.get_mut(owner).unwrap();
    participant.max_hp = 3;
    participant.hp = 1;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::End;
    let zhucheng = skill(tx.state(), owner, ZHUCHENG);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveWdbLiufu {
            player: owner,
            target: PlayerId::NONE,
            card: CardId::NONE,
            zhucheng_skill_def: SkillDefId::new(ZHUCHENG),
            action: 0,
            skill: zhucheng,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().skill_pile_cards(owner, ZHUCHENG).len(), 2);
    tx.state_mut().players.get_mut(owner).unwrap().hp = 3;

    tx.state_mut().turn.current = attacker;
    tx.state_mut().turn.phase = Phase::Play;
    let sha = inject(tx.state_mut(), attacker, CardKind::Sha, 1);
    let payment = inject(tx.state_mut(), attacker, CardKind::Shan, 2);
    let payment_two = inject(tx.state_mut(), attacker, CardKind::Tao, 3);
    let hp = tx.state().players.get(owner).unwrap().hp;
    let state = tx.commit().state;
    let (state, result) = apply_command(
        state,
        Command {
            actor: attacker,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: sha,
                targets: vec![owner],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let state = choose(state, owner, vec![1], false, 2);
    assert_eq!(state.resolution.continuation_kind, ContinuationKind::Custom);
    let state = choose(
        state,
        attacker,
        vec![payment.raw(), payment_two.raw()],
        false,
        3,
    );
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let state = choose(state, owner, vec![0], false, 4);
    assert_eq!(state.players.get(owner).unwrap().hp, hp - 1);
    assert_eq!(state.skill_pile_cards(owner, ZHUCHENG).len(), 2);

    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = attacker;
    tx.state_mut().turn.phase = Phase::Play;
    let duoqi = skill(tx.state(), owner, DUOQI);
    let pile_card = tx.state().skill_pile_cards(owner, ZHUCHENG)[0];
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveWdbLiufu {
                player: owner,
                target: attacker,
                card: CardId::NONE,
                zhucheng_skill_def: SkillDefId::new(ZHUCHENG),
                action: 2,
                skill: duoqi,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = choose(tx.commit().state, owner, vec![pile_card.raw()], false, 5);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let state = choose(state, owner, vec![0], false, 6);
    assert_ne!(state.turn.phase, Phase::Play);
    assert_eq!(state.skill_pile_cards(owner, ZHUCHENG).len(), 1);
}
