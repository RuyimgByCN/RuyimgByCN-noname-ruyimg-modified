//! Golden tests for KEY「转运」rewritten from the legacy JavaScript semantics.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{ContinuationKind, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{damage_nature, EffectRequest};

const HISAKO: u32 = 2_387_604_022;
const HISAKO_YINBAO: u32 = 3_078_155_020;
const HISAKO_ZHUANYUN: u32 = 3_818_579_103;
const SOURCE: &str = concat!(
    include_str!("../../../content/original/ported/key/hisako_yinbao.skill"),
    "\n",
    include_str!("../../../content/original/ported/key/hisako_zhuanyun.skill")
);
const KEY: &[u8] = include_bytes!("../../../content/original/ported/key/key.npack.json");

fn started() -> (Transaction, PlayerId, Vec<PlayerId>) {
    let package = content_compiler::compile_source_named("hisako_test", SOURCE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(20_260_722),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: 20_260_722,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let others = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(HISAKO_YINBAO),
        },
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(HISAKO_ZHUANYUN),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, others)
}

#[test]
fn package_declares_abi72_modifier_and_current_aggregate_character() {
    let compiled = content_compiler::compile_source_named("hisako_zhuanyun", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 72);
    assert!(compiled.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == HISAKO_ZHUANYUN
            && modifier.target == "judgment_best_from_draw_pile"
            && modifier.mode == "override"
            && modifier.value == 1
    }));

    let committed = content_schema::ContentPackage::from_json_bytes(KEY).unwrap();
    assert_eq!(committed.manifest.rule_vm_abi_version, 691);
    let character = committed
        .character_index
        .iter()
        .find(|character| character.id == HISAKO)
        .unwrap();
    assert_eq!(character.max_hp, 3);
    assert_eq!(character.sex, "female");
    assert_eq!(character.group, "key");
    assert_eq!(character.skills, vec![HISAKO_YINBAO, HISAKO_ZHUANYUN]);
}

#[test]
fn yinbao_uses_the_best_card_across_the_draw_pile_instead_of_the_top_card() {
    let (mut tx, owner, others) = started();
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let draw_cards = tx.state().zones.get(draw).unwrap().cards.clone();
    assert!(draw_cards.len() >= 2);
    for card in &draw_cards {
        tx.state_mut().cards.get_mut(*card).unwrap().suit = Suit::Heart;
    }
    let favorable = draw_cards[draw_cards.len() - 2];
    let ordinary_top = *draw_cards.last().unwrap();
    tx.state_mut().cards.get_mut(favorable).unwrap().suit = Suit::Spade;

    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: others[0],
                target: owner,
                amount: 1,
                is_lose_hp: false,
                nature: damage_nature::NORMAL,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let choice_id = ChoiceId::new(tx.state().resolution.pending_choice_id);
    let (state, output) = apply_command(
        tx.commit().state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::JudgeDamageOtherOnSuit
    );
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(favorable).unwrap().zone, discard);
    assert_eq!(state.cards.get(ordinary_top).unwrap().zone, draw);
    assert_eq!(
        state.zones.get(draw).unwrap().cards.last(),
        Some(&ordinary_top)
    );
}
