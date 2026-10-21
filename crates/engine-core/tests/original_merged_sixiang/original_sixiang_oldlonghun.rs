//! Golden semantics for SIXIANG 牢神赵云「旧龙魂」。

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{CardKind, ContinuationKind, GameState, Phase, Zone, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const OLDLONGHUN: u32 = 4_175_093_300;
const SUIT_OVERRIDE: u32 = 4_200_003_050;
const OLD_SHEN_ZHAOYUN: u32 = 3_575_824_543;
const OLDJUEJING: u32 = 2_511_966_880;
const OLDLONGHUN_SOURCE: &str =
    include_str!("../../../../content/original/ported/sixiang/oldlonghun.skill");
const TEST_SOURCE: &str = concat!(
    include_str!("../../../../content/original/ported/sixiang/oldlonghun.skill"),
    "\nskill oldlonghun_test_suit(4200003050) locked {\n",
    "  modifier card_suit(club) override diamond;\n",
    "}\n"
);
const CHARACTERS: &str =
    include_str!("../../../../content/original/ported/sixiang/characters.json");
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/sixiang/sixiang.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    other: PlayerId,
    skill: SkillId,
    cards: [CardId; 4],
}

fn setup(owner_is_current: bool, hp: i32, suit_override: bool) -> Setup {
    let package = content_compiler::compile_source_named("oldlonghun-test", TEST_SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, OLDLONGHUN as u64, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let current = tx.state().turn.current;
    let other = players
        .into_iter()
        .find(|player| *player != current)
        .unwrap();
    let (owner, other) = if owner_is_current {
        (current, other)
    } else {
        (other, current)
    };

    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLDLONGHUN),
    });
    if suit_override {
        effects.push(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(SUIT_OVERRIDE),
        });
    }
    run_effects(&mut tx, effects).unwrap();

    tx.state_mut().turn.current = if owner_is_current { owner } else { other };
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().players.get_mut(owner).unwrap().hp = hp;
    tx.state_mut().players.get_mut(owner).unwrap().max_hp = u32::try_from(hp.max(4)).unwrap();
    tx.state_mut().players.get_mut(other).unwrap().hp = 8;
    tx.state_mut().players.get_mut(other).unwrap().max_hp = 8;

    let cards: [CardId; 4] = tx.state().hand_cards(owner)[..4].try_into().unwrap();
    for card_id in cards {
        let card = tx.state_mut().cards.get_mut(card_id).unwrap();
        card.kind = CardKind::Tao;
        card.suit = Suit::Club;
    }
    for card_id in tx.state().hand_cards(other) {
        let card = tx.state_mut().cards.get_mut(card_id).unwrap();
        card.kind = CardKind::Tao;
        card.suit = Suit::Spade;
    }
    tx.state_mut().cards.get_mut(cards[2]).unwrap().kind = CardKind::Armor;

    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let equip = tx.state().zones.find(ZoneKind::Equip, owner).unwrap();
    let expansion = tx.state_mut().zones.alloc(Zone {
        id: engine_api::ZoneId::NONE,
        kind: ZoneKind::Expansion,
        owner,
        cards: Vec::new(),
    });
    run_effects(
        &mut tx,
        vec![
            EffectRequest::MoveCards {
                cards: vec![cards[1]],
                to: expansion,
                from: Some(hand),
            },
            EffectRequest::MoveCards {
                cards: vec![cards[2]],
                to: equip,
                from: Some(hand),
            },
        ],
    )
    .unwrap();

    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == OLDLONGHUN)
        .unwrap();
    Setup {
        state: tx.commit().state,
        owner,
        other,
        skill,
        cards,
    }
}

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    submit(
        state,
        actor,
        CommandKind::ResolveChoice {
            choice_id,
            selections,
            cancelled: false,
        },
        key,
    )
}

#[test]
fn source_contract_and_character_closure_are_exact() {
    let package =
        content_compiler::compile_source_named("oldlonghun-contract", OLDLONGHUN_SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 290);
    let rules: Vec<_> = package
        .view_as_index
        .iter()
        .filter(|rule| rule.skill_def_id == OLDLONGHUN)
        .collect();
    assert_eq!(rules.len(), 6);
    assert!(rules.iter().all(|rule| {
        rule.source_zone.as_deref() == Some("hand_expansion")
            && rule.owner_hp_min_one_cards
            && rule.multi_card_bonus.is_none()
    }));
    let signatures: Vec<_> = rules
        .iter()
        .map(|rule| {
            (
                rule.as_kind.as_str(),
                rule.window.as_str(),
                rule.filter.as_str(),
                rule.nature.as_deref(),
            )
        })
        .collect();
    assert_eq!(
        signatures,
        [
            ("tao", "play_phase", "heart", None),
            ("tao", "response", "heart", None),
            ("sha", "play_phase", "diamond", Some("fire")),
            ("sha", "response", "diamond", Some("fire")),
            ("wuxie", "response", "spade", None),
            ("shan", "response", "club", None),
        ]
    );

    let characters: serde_json::Value = serde_json::from_str(CHARACTERS).unwrap();
    let character = characters
        .as_array()
        .unwrap()
        .iter()
        .find(|character| character["id"] == OLD_SHEN_ZHAOYUN)
        .unwrap();
    assert_eq!(character["name_key"], "old_shen_zhaoyun");
    assert_eq!(character["sex"], "male");
    assert_eq!(character["group"], "shen");
    assert_eq!(character["max_hp"], 2);
    assert_eq!(
        character["skills"],
        serde_json::json!([OLDJUEJING, OLDLONGHUN])
    );

    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(committed.manifest.rule_vm_abi_version, 562);
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == OLDLONGHUN && skill.name == "oldlonghun"));
    let committed_rules: Vec<_> = committed
        .view_as_index
        .iter()
        .filter(|rule| rule.skill_def_id == OLDLONGHUN)
        .collect();
    assert_eq!(committed_rules, rules);
    let committed_character = committed
        .character_index
        .iter()
        .find(|character| character.id == OLD_SHEN_ZHAOYUN)
        .unwrap();
    assert_eq!(committed_character.max_hp, 2);
    assert_eq!(committed_character.skills, [OLDJUEJING, OLDLONGHUN]);
    assert!(!serde_json::to_string(&committed).unwrap().contains(".js"));
}

#[test]
fn play_uses_exact_hp_effective_suit_hand_and_expansion_without_bonus() {
    let Setup {
        state,
        owner,
        other: target,
        skill,
        cards,
    } = setup(true, 2, true);

    let (_, one_card) = submit(
        state.clone(),
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![target],
            card_targets: vec![cards[0]],
        },
        1,
    );
    assert!(matches!(one_card.result, CommandResult::Rejected { .. }));

    let (_, includes_equip) = submit(
        state.clone(),
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![target],
            card_targets: vec![cards[0], cards[2]],
        },
        2,
    );
    assert!(matches!(
        includes_equip.result,
        CommandResult::Rejected { .. }
    ));

    let hp = state.players.get(target).unwrap().hp;
    let (state, accepted) = submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![target],
            card_targets: vec![cards[0], cards[1]],
        },
        3,
    );
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(cards[0]).unwrap().zone, discard);
    assert_eq!(state.cards.get(cards[1]).unwrap().zone, discard);
    assert_eq!(
        state
            .zones
            .get(state.cards.get(cards[2]).unwrap().zone)
            .unwrap()
            .kind,
        ZoneKind::Equip
    );
}

#[test]
fn response_requires_current_hp_sources_and_does_not_offer_equipment() {
    let Setup {
        mut state,
        owner,
        other: attacker,
        cards,
        ..
    } = setup(false, 3, false);
    let attack = state.hand_cards(attacker)[0];
    state.cards.get_mut(attack).unwrap().kind = CardKind::Sha;
    let hp = state.players.get(owner).unwrap().hp;
    let (state, response) = submit(
        state,
        attacker,
        CommandKind::PlayCard {
            card: attack,
            targets: vec![owner],
            card_targets: Vec::new(),
        },
        10,
    );
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::RespondToSha
    );
    let physical_tokens: std::collections::BTreeSet<_> =
        cards.iter().map(|card| card.raw()).collect();
    let conversion = response
        .choice
        .unwrap()
        .options
        .iter()
        .map(|option| option.token)
        .find(|token| !physical_tokens.contains(token))
        .unwrap();

    let (state, source_choice) = resolve(state, owner, vec![conversion], 11);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::MultiCardViewAsResponse
    );
    let choice = source_choice.choice.unwrap();
    assert_eq!((choice.min_count, choice.max_count), (3, 3));
    let offered: std::collections::BTreeSet<_> =
        choice.options.iter().map(|option| option.token).collect();
    let allowed = [cards[0].raw(), cards[1].raw(), cards[3].raw()]
        .into_iter()
        .collect();
    assert_eq!(offered, allowed);
    assert!(!offered.contains(&cards[2].raw()));

    let selected = vec![cards[0].raw(), cards[1].raw(), cards[3].raw()];
    let (state, completed) = resolve(state, owner, selected, 12);
    assert!(matches!(completed.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(owner).unwrap().hp, hp);
}
