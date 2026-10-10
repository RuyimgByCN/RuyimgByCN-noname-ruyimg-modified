//! Golden for KEY「游凤」的无来源动态转换、分支计次与装备栏废除语义。

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const YOUFENG: u32 = 2_842_065_522;
const SOURCE: &str = include_str!("../../../content/original/ported/key/chihaya_youfeng.skill");

fn package() -> content_schema::ContentPackage {
    content_compiler::compile_source_named("key-youfeng", SOURCE).unwrap()
}

fn started_state() -> engine_core::state::GameState {
    let mut engine = Engine::new();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_718,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&package()).unwrap();
    engine.state().clone()
}

fn command(actor: PlayerId, kind: CommandKind) -> Command {
    Command {
        actor,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind,
    }
}

fn add_youfeng(tx: &mut Transaction, owner: PlayerId) -> engine_api::SkillId {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(YOUFENG),
        }],
    )
    .unwrap();
    tx.state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == YOUFENG)
        .unwrap()
}

fn other(state: &engine_core::state::GameState, player: PlayerId) -> PlayerId {
    state
        .players
        .iter_live()
        .find(|candidate| candidate.id != player)
        .unwrap()
        .id
}

fn neutralize_hands(tx: &mut Transaction) {
    let cards: Vec<CardId> = tx
        .state()
        .players
        .iter_live()
        .flat_map(|player| tx.state().hand_cards(player.id))
        .collect();
    for card in cards {
        tx.state_mut().cards.get_mut(card).unwrap().kind = CardKind::Other;
    }
}

fn inject_hand(tx: &mut Transaction, player: PlayerId, kind: CardKind, suit: Suit) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::NONE,
        kind,
        suit,
        rank: 7,
        weapon_range: if kind == CardKind::Weapon { 3 } else { 0 },
        zone: hand,
        owner: player,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn assert_accepted(result: &engine_core::command::PipelineResult) {
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{:?}",
        result.result
    );
}

#[test]
fn package_declares_two_source_free_branches_and_abi_17() {
    let package = package();
    assert_eq!(package.manifest.rule_vm_abi_version, 17);
    assert_eq!(package.view_as_index.len(), 2);
    let yin = package
        .view_as_index
        .iter()
        .find(|rule| rule.branch == Some(0))
        .unwrap();
    assert_eq!(yin.as_kind, "ordinary_trick");
    assert_eq!(yin.source_zone.as_deref(), Some("none"));
    assert!(yin.once_per_round);
    assert_eq!(yin.next_branch, Some(1));
    assert_eq!(yin.cost.as_deref(), Some("gain_max_hp_1"));
    let yang = package
        .view_as_index
        .iter()
        .find(|rule| rule.branch == Some(1))
        .unwrap();
    assert_eq!(yang.as_kind, "basic");
    assert_eq!(yang.next_branch, Some(0));
    assert_eq!(yang.cost.as_deref(), Some("disable_equip_slot"));
}

#[test]
fn proactive_yin_then_yang_each_once_per_round_and_slot_stays_disabled() {
    let state = started_state();
    let owner = state.turn.current;
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.phase = Phase::Play;
    neutralize_hands(&mut tx);
    let skill = add_youfeng(&mut tx, owner);
    let weapon = inject_hand(&mut tx, owner, CardKind::Weapon, Suit::Spade);
    let state = tx.commit().state;

    let (state, equip_result) = apply_command(
        state,
        command(
            owner,
            CommandKind::PlayCard {
                card: weapon,
                targets: vec![],
                card_targets: vec![],
            },
        ),
    );
    assert_accepted(&equip_result);
    let before_max = state.players.get(owner).unwrap().max_hp;

    let (state, yin) = apply_command(
        state,
        command(
            owner,
            CommandKind::UseSkillCard {
                skill,
                card_kind: CardKind::TrickWuzhong.tag(),
                targets: vec![],
                equip_slot: 0,
            },
        ),
    );
    assert_accepted(&yin);
    assert_eq!(state.players.get(owner).unwrap().max_hp, before_max + 1);
    let instance = state.skills.get(skill).unwrap();
    assert_eq!(instance.view_as_branch, 1);
    assert_eq!(instance.view_as_used_branches & 1, 1);

    let (state, yang) = apply_command(
        state,
        command(
            owner,
            CommandKind::UseSkillCard {
                skill,
                card_kind: CardKind::Jiu.tag(),
                targets: vec![],
                equip_slot: 1,
            },
        ),
    );
    assert_accepted(&yang);
    assert_eq!(state.players.get(owner).unwrap().disabled_equip_slots[0], 1);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(weapon).unwrap().zone, discard);
    let instance = state.skills.get(skill).unwrap();
    assert_eq!(instance.view_as_branch, 0);
    assert_eq!(instance.view_as_used_branches & 0b11, 0b11);

    let (_, repeated) = apply_command(
        state.clone(),
        command(
            owner,
            CommandKind::UseSkillCard {
                skill,
                card_kind: CardKind::TrickWuzhong.tag(),
                targets: vec![],
                equip_slot: 0,
            },
        ),
    );
    assert!(matches!(repeated.result, CommandResult::Rejected { .. }));

    let mut next_round = state;
    next_round.turn.round = next_round.turn.round.saturating_add(1);
    let (_, available_again) = apply_command(
        next_round,
        command(
            owner,
            CommandKind::UseSkillCard {
                skill,
                card_kind: CardKind::TrickWuzhong.tag(),
                targets: vec![],
                equip_slot: 0,
            },
        ),
    );
    assert_accepted(&available_again);
}

#[test]
fn yang_can_answer_sha_after_selecting_an_enabled_slot() {
    let state = started_state();
    let source = state.turn.current;
    let target = other(&state, source);
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.phase = Phase::Play;
    neutralize_hands(&mut tx);
    let skill = add_youfeng(&mut tx, target);
    tx.state_mut().skills.get_mut(skill).unwrap().view_as_branch = 1;
    let sha = inject_hand(&mut tx, source, CardKind::Sha, Suit::Spade);
    let hp = tx.state().players.get(target).unwrap().hp;
    let state = tx.commit().state;

    let (state, played) = apply_command(
        state,
        command(
            source,
            CommandKind::PlayCard {
                card: sha,
                targets: vec![target],
                card_targets: vec![],
            },
        ),
    );
    assert_accepted(&played);
    let response = played.choice.unwrap();
    assert_eq!(response.observer, target);
    let token = response
        .options
        .iter()
        .map(|option| option.token)
        .find(|token| *token != engine_core::content::BAGUA_RESPONSE_TOKEN)
        .unwrap();

    let (state, chose_response) = apply_command(
        state,
        command(
            target,
            CommandKind::ResolveChoice {
                choice_id: response.choice_id,
                selections: vec![token],
                cancelled: false,
            },
        ),
    );
    assert_accepted(&chose_response);
    let slot_choice = chose_response.choice.unwrap();
    assert_eq!(slot_choice.observer, target);

    let (state, chose_slot) = apply_command(
        state,
        command(
            target,
            CommandKind::ResolveChoice {
                choice_id: slot_choice.choice_id,
                selections: vec![1],
                cancelled: false,
            },
        ),
    );
    assert_accepted(&chose_slot);
    assert_eq!(state.players.get(target).unwrap().hp, hp);
    assert_eq!(
        state.players.get(target).unwrap().disabled_equip_slots[0],
        1
    );
    assert_eq!(state.skills.get(skill).unwrap().view_as_branch, 0);
}

#[test]
fn yin_can_respond_with_wuxie_and_nullify_the_trick() {
    let state = started_state();
    let source = state.turn.current;
    let responder = other(&state, source);
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.phase = Phase::Play;
    neutralize_hands(&mut tx);
    let skill = add_youfeng(&mut tx, responder);
    let wuzhong = inject_hand(&mut tx, source, CardKind::TrickWuzhong, Suit::Heart);
    let before_hand = tx.state().hand_count(source);
    let before_max = tx.state().players.get(responder).unwrap().max_hp;
    let state = tx.commit().state;

    let (state, played) = apply_command(
        state,
        command(
            source,
            CommandKind::PlayCard {
                card: wuzhong,
                targets: vec![],
                card_targets: vec![],
            },
        ),
    );
    assert_accepted(&played);
    let choice = played.choice.unwrap();
    assert_eq!(choice.observer, responder);
    let token = choice.options[0].token;

    let (state, wuxie) = apply_command(
        state,
        command(
            responder,
            CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections: vec![token],
                cancelled: false,
            },
        ),
    );
    assert_accepted(&wuxie);
    assert!(wuxie.choice.is_none());
    assert_eq!(state.hand_count(source), before_hand - 1);
    assert_eq!(state.players.get(responder).unwrap().max_hp, before_max + 1);
    assert_eq!(state.skills.get(skill).unwrap().view_as_branch, 1);
}

#[test]
fn yang_can_self_save_while_dying() {
    let state = started_state();
    let player = state.turn.current;
    let mut tx = Transaction::begin(state);
    neutralize_hands(&mut tx);
    let skill = add_youfeng(&mut tx, player);
    tx.state_mut().skills.get_mut(skill).unwrap().view_as_branch = 1;
    {
        let owner = tx.state_mut().players.get_mut(player).unwrap();
        owner.hp = 0;
        owner.dying = true;
    }
    let source = other(tx.state(), player);
    assert!(run_effects(
        &mut tx,
        vec![EffectRequest::RequestDyingSave { player, source }],
    )
    .is_err());
    let committed = tx.commit();
    let save_choice = committed.output.choice.unwrap();
    let token = save_choice
        .options
        .iter()
        .map(|option| option.token)
        .find(|token| *token != 0)
        .unwrap();

    let (state, selected_save) = apply_command(
        committed.state,
        command(
            player,
            CommandKind::ResolveChoice {
                choice_id: save_choice.choice_id,
                selections: vec![token],
                cancelled: false,
            },
        ),
    );
    assert_accepted(&selected_save);
    let slot_choice = selected_save.choice.unwrap();
    let (state, selected_slot) = apply_command(
        state,
        command(
            player,
            CommandKind::ResolveChoice {
                choice_id: slot_choice.choice_id,
                selections: vec![1],
                cancelled: false,
            },
        ),
    );
    assert_accepted(&selected_slot);
    let owner = state.players.get(player).unwrap();
    assert_eq!(owner.hp, 1);
    assert!(!owner.dying);
    assert_eq!(owner.disabled_equip_slots[0], 1);
}
