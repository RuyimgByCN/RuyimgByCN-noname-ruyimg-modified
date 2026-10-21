//! Golden coverage for TW 祖茂「替巾」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWTIJIN: u32 = 3_420_559_729;
const SOURCE: &str = include_str!("../../../../content/original/ported/tw/twzumao.skill");

fn inject(state: &mut GameState, owner: PlayerId, kind: CardKind, def: u32) -> CardId {
    let zone = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
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

fn choose(state: GameState, actor: PlayerId, selection: u32, key: u64) -> GameState {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![selection],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn redirects_in_range_sha_then_discards_from_its_user_after_resolution() {
    let package = content_compiler::compile_source_named("tw-zumao-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 392);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    bootstrap_match(&mut tx, 3, 2_026_080_211, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, source, original) = (players[0], players[1], players[2]);
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
            skill_def: SkillDefId::new(TWTIJIN),
        },
        EffectRequest::SetTurnState {
            current: source,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let sha = inject(tx.state_mut(), source, CardKind::Sha, 99_021);
    let discarded = inject(tx.state_mut(), source, CardKind::Other, 99_022);
    let owner_hp = tx.state().players.get(owner).unwrap().hp;
    let state = tx.commit().state;
    let (state, result) = apply_command(
        state,
        Command {
            actor: source,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: sha,
                targets: vec![original],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    // 当前实现：替巾直接重定向（无确认段）；结算后唯一 choice 是弃使用者一张牌。
    let state = choose(state, owner, discarded.raw(), 2);
    assert_eq!(state.players.get(owner).unwrap().hp, owner_hp - 1);
    assert!(!state.has_pending_choice());
    assert_eq!(
        state
            .zones
            .get(state.cards.get(discarded).unwrap().zone)
            .unwrap()
            .kind,
        ZoneKind::DiscardPile
    );
    let skill = state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == TWTIJIN)
        .unwrap();
    assert!(skill.state_blob.is_empty());
}
