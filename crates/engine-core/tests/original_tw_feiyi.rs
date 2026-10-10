//! Golden coverage for TW 费祎「生息/宽济」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWSHENGXI: u32 = 3_391_342_890;
const TWKUANJI: u32 = 3_010_237_936;
const SOURCE: &str = include_str!("../../../content/original/ported/tw/twfeiyi.skill");

fn inject(
    tx: &mut Transaction,
    zone: ZoneKind,
    owner: PlayerId,
    kind: CardKind,
    rank: u8,
) -> CardId {
    let zone = tx.state().zones.find(zone, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_377_000 + u32::from(rank)),
        kind,
        suit: Suit::Club,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn choose(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn shengxi_creates_salt_and_gains_zhinang_while_kuanji_transfers_discard() {
    let package = content_compiler::compile_source_named("tw-feiyi-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 377);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 2_026_080_118, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, target) = (players[0], players[1]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(
        [TWSHENGXI, TWKUANJI].map(|skill_def| EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(skill_def),
        }),
    );
    run_effects(&mut tx, effects).unwrap();
    let skill = |tx: &Transaction, def| {
        tx.state()
            .skills
            .sorted_live()
            .into_iter()
            .find(|skill| skill.owner == owner && skill.def.raw() == def)
            .unwrap()
            .id
    };
    let shengxi = skill(&tx, TWSHENGXI);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveTwfeiyi {
            player: owner,
            action: 0,
            stage: 0,
            card: CardId::NONE,
            event_cards: Vec::new(),
            skill: shengxi,
        }],
    )
    .unwrap();
    assert!(tx.state().hand_cards(owner).iter().any(|card| {
        tx.state()
            .cards
            .get(*card)
            .is_some_and(|card| card.kind == CardKind::TrickTiaojiyanmei && card.rank == 6)
    }));

    let zhinang = inject(
        &mut tx,
        ZoneKind::DrawPile,
        PlayerId::NONE,
        CardKind::TrickWuzhong,
        8,
    );
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveTwfeiyi {
                player: owner,
                action: 1,
                stage: 0,
                card: CardId::NONE,
                event_cards: Vec::new(),
                skill: shengxi,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = choose(
        tx.commit().state,
        owner,
        vec![CardKind::TrickWuzhong.tag().into()],
        1,
    );
    assert!(state.hand_cards(owner).contains(&zhinang));

    let mut tx = Transaction::begin(state);
    let kuanji = skill(&tx, TWKUANJI);
    let discard_card = inject(
        &mut tx,
        ZoneKind::DiscardPile,
        PlayerId::NONE,
        CardKind::Shan,
        9,
    );
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveTwfeiyi {
                player: owner,
                action: 2,
                stage: 0,
                card: CardId::NONE,
                event_cards: vec![discard_card],
                skill: kuanji,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = choose(tx.commit().state, owner, vec![discard_card.raw()], 2);
    assert!(state.has_pending_choice());
    let state = choose(state, owner, vec![target.raw()], 3);
    assert!(state.hand_cards(target).contains(&discard_card));
}
