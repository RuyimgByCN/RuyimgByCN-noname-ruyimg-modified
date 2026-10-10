//! Golden coverage for TW 樊稠「兴乱」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWXINGLUAN: u32 = 3_047_451_649;
const TW_FANCHOU: u32 = 3_054_071_833;
const SOURCE: &str = include_str!("../../../content/original/ported/tw/twxingluan.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/tw/tw.npack.json");

fn inject(tx: &mut Transaction, kind: CardKind, def: u32) -> CardId {
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Heart,
        rank: 7,
        weapon_range: u8::from(kind == CardKind::Weapon) * 2,
        zone: draw,
        owner: PlayerId::NONE,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(draw).unwrap().cards.push(card);
    card
}

fn choose(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, output) = apply_command(
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
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn distributes_one_category_with_cap_then_applies_owner_share_threshold() {
    let compiled = content_compiler::compile_source_named("twxingluan-test", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 491);
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(committed.manifest.rule_vm_abi_version, 733);
    assert_eq!(
        committed
            .character_index
            .iter()
            .find(|character| character.id == TW_FANCHOU)
            .unwrap()
            .skills,
        [TWXINGLUAN]
    );

    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    bootstrap_match(&mut tx, 3, 2_026_080_208, "identity-standard").unwrap();
    let players: [PlayerId; 3] = tx.state().players.alive_by_seat().try_into().unwrap();
    let removed = tx
        .state()
        .zones
        .find(ZoneKind::Removed, PlayerId::NONE)
        .unwrap();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for kind in [ZoneKind::DrawPile, ZoneKind::DiscardPile] {
        let zone = tx.state().zones.find(kind, PlayerId::NONE).unwrap();
        let cards = tx.state().zones.get(zone).unwrap().cards.clone();
        if !cards.is_empty() {
            effects.push(EffectRequest::MoveCards {
                cards,
                to: removed,
                from: Some(zone),
            });
        }
    }
    effects.push(EffectRequest::AddSkill {
        player: players[0],
        skill_def: SkillDefId::new(TWXINGLUAN),
    });
    run_effects(&mut tx, effects).unwrap();
    let basic = [
        inject(&mut tx, CardKind::Sha, 99_101),
        inject(&mut tx, CardKind::Shan, 99_102),
        inject(&mut tx, CardKind::Tao, 99_103),
        inject(&mut tx, CardKind::Jiu, 99_104),
    ];
    let trick = inject(&mut tx, CardKind::TrickJuedou, 99_105);
    let equip = inject(&mut tx, CardKind::Weapon, 99_106);
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == players[0] && skill.def.raw() == TWXINGLUAN)
        .unwrap()
        .id;
    let hp: Vec<_> = players
        .iter()
        .map(|player| tx.state().players.get(*player).unwrap().hp)
        .collect();
    assert_eq!(
        run_effects(
            &mut tx,
            vec![
                EffectRequest::RevealChooseCategoryDistributeSelfThresholdHpLoss {
                    player: players[0],
                    reveal_count: 6,
                    recipient_cap: 3,
                    skill,
                }
            ],
        ),
        Err(EffectError::ChoiceRequested)
    );

    let state = choose(tx.commit().state, players[0], vec![1], 1);
    let state = choose(state, players[0], vec![basic[0].raw()], 2);
    let state = choose(state, players[0], vec![players[0].raw()], 3);
    let state = choose(state, players[0], vec![basic[1].raw(), basic[2].raw()], 4);
    let state = choose(state, players[0], vec![players[1].raw()], 5);
    let state = choose(state, players[0], vec![basic[3].raw()], 6);
    let state = choose(state, players[0], vec![players[2].raw()], 7);

    assert!(!state.has_pending_choice());
    for (index, player) in players.iter().enumerate() {
        assert_eq!(state.players.get(*player).unwrap().hp, hp[index] - 1);
    }
    assert!(state.hand_cards(players[0]).contains(&basic[0]));
    assert!(state.hand_cards(players[1]).contains(&basic[1]));
    assert!(state.hand_cards(players[1]).contains(&basic[2]));
    assert!(state.hand_cards(players[2]).contains(&basic[3]));
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(trick).unwrap().zone, discard);
    assert_eq!(state.cards.get(equip).unwrap().zone, discard);
}
