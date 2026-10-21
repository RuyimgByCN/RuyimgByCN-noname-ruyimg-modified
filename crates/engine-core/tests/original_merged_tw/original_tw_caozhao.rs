//! Golden coverage for TW 曹肇「复纂/宠齐/非服」.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWFUZUAN: u32 = 3_482_762_704;
const TWCHONGQI: u32 = 4_284_388_882;
const TWFEIFU: u32 = 3_948_553_827;
const SOURCE: &str = include_str!("../../../../content/original/ported/tw/twcaozhao.skill");

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
        "{:?}",
        result.result
    );
    state
}

fn setup(seed: u64, skills: &[(usize, u32)]) -> (Transaction, Vec<PlayerId>) {
    let package = content_compiler::compile_source_named("tw-caozhao-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 387);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == TWFEIFU && skill.conversion));
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(skills.iter().map(|(index, skill)| EffectRequest::AddSkill {
        player: players[*index],
        skill_def: SkillDefId::new(*skill),
    }));
    run_effects(&mut tx, effects).unwrap();
    (tx, players)
}

fn skill(tx: &Transaction, owner: PlayerId, def: u32) -> engine_api::SkillId {
    tx.state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == def)
        .unwrap()
        .id
}

#[test]
fn fuzuan_toggles_chongqi_grants_and_feifu_gives_equipment() {
    let (mut tx, players) = setup(2_026_080_204, &[(0, TWFUZUAN), (1, TWFEIFU)]);
    let (owner, target) = (players[0], players[1]);
    let fuzuan = skill(&tx, owner, TWFUZUAN);
    let feifu = skill(&tx, target, TWFEIFU);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveTwcaozhao {
                player: owner,
                target: PlayerId::NONE,
                source: PlayerId::NONE,
                card: engine_api::CardId::NONE,
                selected_skill_def: 0,
                twfuzuan_skill_def: TWFUZUAN,
                twfeifu_skill_def: TWFEIFU,
                action: 0,
                stage: 0,
                skill: fuzuan,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = choose(tx.commit().state, owner, vec![target.raw()], false, 1);
    assert_eq!(state.skills.get(feifu).unwrap().view_as_branch, 1);

    let (mut tx, players) = setup(2_026_080_205, &[(0, TWCHONGQI)]);
    let (owner, target) = (players[0], players[1]);
    let chongqi = skill(&tx, owner, TWCHONGQI);
    let old_max_hp = tx.state().players.get(owner).unwrap().max_hp;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveTwcaozhao {
                player: owner,
                target: PlayerId::NONE,
                source: PlayerId::NONE,
                card: engine_api::CardId::NONE,
                selected_skill_def: 0,
                twfuzuan_skill_def: TWFUZUAN,
                twfeifu_skill_def: TWFEIFU,
                action: 1,
                stage: 0,
                skill: chongqi,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = choose(tx.commit().state, owner, vec![target.raw()], false, 2);
    assert_eq!(state.players.get(owner).unwrap().max_hp, old_max_hp - 1);
    for player in &players {
        assert!(state
            .players
            .get(*player)
            .unwrap()
            .skills
            .iter()
            .any(|skill| {
                state
                    .skills
                    .get(*skill)
                    .is_some_and(|skill| skill.def.raw() == TWFEIFU)
            }));
    }
    assert!(state
        .players
        .get(target)
        .unwrap()
        .skills
        .iter()
        .any(|skill| {
            state
                .skills
                .get(*skill)
                .is_some_and(|skill| skill.def.raw() == TWFUZUAN)
        }));

    let (mut tx, players) = setup(2_026_080_206, &[(0, TWFEIFU)]);
    let (owner, source) = (players[0], players[1]);
    let feifu = skill(&tx, owner, TWFEIFU);
    let card = tx.state().hand_cards(owner)[0];
    let card_state = tx.state_mut().cards.get_mut(card).unwrap();
    card_state.def = engine_api::CardDefId::NONE;
    card_state.kind = CardKind::Weapon;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveTwcaozhao {
                player: owner,
                target: owner,
                source,
                card: engine_api::CardId::NONE,
                selected_skill_def: 0,
                twfuzuan_skill_def: TWFUZUAN,
                twfeifu_skill_def: TWFEIFU,
                action: 2,
                stage: 0,
                skill: feifu,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = choose(tx.commit().state, owner, vec![card.raw()], false, 3);
    assert_eq!(state.skills.get(feifu).unwrap().view_as_branch, 1);
    assert!(state.has_pending_choice());
    let state = choose(state, source, vec![1], false, 4);
    let equip = state.zones.find(ZoneKind::Equip, source).unwrap();
    assert_eq!(state.cards.get(card).unwrap().zone, equip);
}
