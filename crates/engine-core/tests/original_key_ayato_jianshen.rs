//! Golden for KEY「僭神」的神属性【杀】与铁索隔离语义。

use engine_api::{
    Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::query::effective_card_nature;
use engine_core::state::{CardKind, Phase};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{damage_nature, EffectRequest};

const AYATO_JIANSHEN: u32 = 3_383_654_000;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/key/key.npack.json");

fn isolated() -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package
        .modifier_index
        .iter()
        .any(|modifier| modifier.skill_def_id == AYATO_JIANSHEN
            && modifier.target == "card_nature"
            && modifier.value == i32::from(damage_nature::KAMI)));

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(AYATO_JIANSHEN as u64),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: AYATO_JIANSHEN as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));

    let players = engine.state().players.alive_by_seat();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: players[0],
        skill_def: SkillDefId::new(AYATO_JIANSHEN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    (tx, players[0], players[1], players[2])
}

fn set_linked(tx: &mut Transaction, player: PlayerId) {
    let marks = &mut tx.state_mut().players.get_mut(player).unwrap().marks;
    marks.retain(|(mark, _)| *mark != engine_core::content::MARK_CHAIN);
    marks.push((engine_core::content::MARK_CHAIN, 1));
    marks.sort_by_key(|(mark, _)| *mark);
}

fn is_linked(tx: &Transaction, player: PlayerId) -> bool {
    tx.state()
        .players
        .get(player)
        .unwrap()
        .marks
        .iter()
        .any(|(mark, count)| *mark == engine_core::content::MARK_CHAIN && *count > 0)
}

#[test]
fn all_logical_sha_variants_become_kami_but_other_cards_keep_their_nature() {
    let (mut tx, owner, _, _) = isolated();
    let card = tx.state().hand_cards(owner)[0];
    tx.state_mut().cards.get_mut(card).unwrap().kind = CardKind::Tao;
    let physical = tx.state().cards.get(card).unwrap();

    for logical in [CardKind::Sha, CardKind::ShaFire, CardKind::ShaThunder] {
        assert_eq!(
            effective_card_nature(tx.state(), owner, physical, logical),
            damage_nature::KAMI
        );
    }
    assert_eq!(
        effective_card_nature(tx.state(), owner, physical, CardKind::Tao),
        damage_nature::NORMAL
    );
}

#[test]
fn played_kami_sha_damages_only_its_linked_target_without_unchaining() {
    let (mut tx, owner, target, chained_other) = isolated();
    for player in [target, chained_other] {
        set_linked(&mut tx, player);
        for card in tx.state().hand_cards(player) {
            tx.state_mut().cards.get_mut(card).unwrap().kind = CardKind::Other;
        }
    }
    let sha = tx.state().hand_cards(owner)[0];
    tx.state_mut().cards.get_mut(sha).unwrap().kind = CardKind::Sha;
    let target_hp = tx.state().players.get(target).unwrap().hp;
    let other_hp = tx.state().players.get(chained_other).unwrap().hp;

    let (after, pipe) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: sha,
                targets: vec![target],
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    assert_eq!(after.players.get(target).unwrap().hp, target_hp - 1);
    assert_eq!(after.players.get(chained_other).unwrap().hp, other_hp);
    let after = Transaction::begin(after);
    assert!(is_linked(&after, target));
    assert!(is_linked(&after, chained_other));
    assert!(pipe.events.iter().any(|event| matches!(
        event,
        DomainEvent::DamageDealt {
            source,
            target: damaged,
            amount: 1,
            nature,
            ..
        } if *source == owner && *damaged == target && *nature == damage_nature::KAMI
    )));
}

#[test]
fn normal_and_kami_damage_do_not_enter_the_elemental_chain_path() {
    for nature in [damage_nature::NORMAL, damage_nature::KAMI] {
        let (mut tx, owner, target, chained_other) = isolated();
        set_linked(&mut tx, target);
        set_linked(&mut tx, chained_other);
        let target_hp = tx.state().players.get(target).unwrap().hp;
        let other_hp = tx.state().players.get(chained_other).unwrap().hp;

        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: owner,
                target,
                amount: 1,
                is_lose_hp: false,
                nature,
            }],
        )
        .unwrap();
        assert_eq!(tx.state().players.get(target).unwrap().hp, target_hp - 1);
        assert_eq!(tx.state().players.get(chained_other).unwrap().hp, other_hp);
        assert!(is_linked(&tx, target));
        assert!(is_linked(&tx, chained_other));
    }
}
