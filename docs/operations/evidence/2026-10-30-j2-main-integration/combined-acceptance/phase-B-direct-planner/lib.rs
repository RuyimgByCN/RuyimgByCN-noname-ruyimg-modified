use engine_native::{Engine, Phase, GameAction, GameCommand, ChoiceOption as O, CommandOutcome, MachineSnapshot};
use engine_native::ai::BoundedPlanner;
use engine_native_wasm::{engine_from_packs, engine_from_archive, hex_encode};
use serde_json::{json,Value};
const PACKS:&str=include_str!("/Users/ruyimg/Documents/Claude/sanguosha/noname-next/docs/operations/evidence/2026-10-30-j2-main-integration/combined-acceptance/phase-B-direct-planner/author-packs.json");
const MATRIX:&str=include_str!("/Users/ruyimg/Documents/Claude/sanguosha/noname-next/docs/operations/evidence/2026-10-30-j2-main-integration/combined-acceptance/j2-native-matrix.json");
fn archive(e:&Engine)->String{hex_encode(&engine_native::archive::encode_archive(&e.save()))}
fn public(e:&Engine,seat:u8)->Value{json!({"input":format!("{:?}",e.ai_session(seat).input()),"choice":format!("{:?}",e.ai_session(seat).choice()),"events":format!("{:?}",e.ai_session(seat).events())})}
fn submit(engine: &mut Engine, action: GameAction) {
    let outcome = engine.submit(GameCommand {
        actor: 0,
        expected_revision: engine.revision(),
        dedup_id: engine.save().last_dedup + 1,
        action,
    });
    assert!(
        !matches!(outcome, CommandOutcome::Rejected(_)),
        "{outcome:?}"
    );
}

fn answer(engine: &mut Engine, selection: Vec<O>) {
    let choice_id = engine.open_choice().unwrap().id;
    submit(
        engine,
        GameAction::Answer {
            choice_id,
            selection,
        },
    );
}

fn fixture(packs: &str, red: bool) -> Engine {
    let mut engine = engine_from_packs(4, 99107868, packs).unwrap();
    engine.assign_general_named(0, "j2-cross-author").unwrap();
    engine.debug_set_turn(0, Phase::Play, 1);
    engine.debug_set_hp(0, 4);
    engine.debug_clear_hand(0);
    let spare = engine.debug_place(0, "sha");
    engine.debug_place(0, "tao");
    engine.debug_place(0, "shan");
    assert_eq!(engine.state_core().players[1].hand.len(), 4);
    for index in 0..4 {
        let deck_index = engine
            .state_core()
            .deck
            .iter()
            .position(|id| {
                matches!(
                    engine.catalog().cards[*id as usize].suit,
                    engine_native::catalog::Suit::Heart | engine_native::catalog::Suit::Diamond
                ) == red
            })
            .unwrap();
        let core = engine.state_core_mut();
        std::mem::swap(&mut core.players[1].hand[index], &mut core.deck[deck_index]);
    }
    let skill = engine.catalog().skill_by_name("nc10_j2_cached").unwrap();
    submit(&mut engine, GameAction::ActivateSkill { skill });
    for (prompt, selection) in [
        ("rvm_danshou_option", vec![O::Tag(1)]),
        ("rvm_danshou_target", vec![O::Player(0)]),
        ("rvm_danshou_card", vec![O::Card(spare)]),
    ] {
        assert_eq!(engine.open_choice().unwrap().spec.prompt, prompt);
        answer(&mut engine, selection);
    }
    let choice = engine.open_choice().unwrap();
    assert_eq!(choice.spec.prompt, "rvm_danshou_discard");
    assert_eq!((choice.spec.min, choice.spec.options.len()), (1, 2));
    engine_from_archive(&archive(&engine), packs).unwrap();
    engine
}


fn damage(e:&Engine)->Vec<u32>{e.save().resolution.parks.iter().flat_map(|p|match &p.machine{MachineSnapshot::Rvm{commit_pending,..}=>engine_native::rvm_ops::dec_effects(commit_pending).unwrap().iter().filter_map(|e|if let rule_api::EffectRequest::DealDamage{amount,..}=e{Some(*amount)}else{None}).collect::<Vec<_>>(),_=>vec![]}).collect()}
fn plan_check(e:&Engine,packs:&str,seat:u8,budget:usize)->Value{
 let before=archive(e);let planner=BoundedPlanner::new(budget).with_policy_variant(1).with_lookahead(2);let plan=e.plan_action(seat,&planner);assert_eq!((plan.nodes_used,plan.sim_faults),(0,0));assert_eq!(archive(e),before);
 let mut restored=engine_from_archive(&before,packs).unwrap();let result=restored.apply_plan(&plan).unwrap();assert!(!matches!(result,CommandOutcome::Rejected(_)));let after=archive(&restored);engine_from_archive(&after,packs).unwrap();
 json!({"budget":budget,"fullPlan":format!("{plan:?}"),"action":format!("{:?}",plan.action),"nodes":plan.nodes_used,"simFaults":plan.sim_faults,"sourceImmutable":true,"restoredApplyValid":true,"outcome":format!("{result:?}"),"afterHash":hex_encode(&restored.state_hash()),"afterArchive":after})
}
#[cfg_attr(target_arch="wasm32",wasm_bindgen::prelude::wasm_bindgen)]
pub fn run_matrix()->String{
 let pair=[fixture(PACKS,false),fixture(PACKS,true)];assert_eq!(damage(&pair[0]),vec![1]);assert_eq!(damage(&pair[1]),vec![5]);assert_eq!(public(&pair[0],0),public(&pair[1],0));let originals=[archive(&pair[0]),archive(&pair[1])];let mut budgets=vec![];
 for budget in [0,48]{let plans=[plan_check(&pair[0],PACKS,0,budget),plan_check(&pair[1],PACKS,0,budget)];assert_eq!(plans[0]["fullPlan"],plans[1]["fullPlan"]);budgets.push(json!({"budget":budget,"variants":plans}));}
 assert_eq!([archive(&pair[0]),archive(&pair[1])],originals);
 let matrix:Value=serde_json::from_str(MATRIX).unwrap();let names=["rvm_danshou_option","rvm_danshou_target","rvm_danshou_card","rvm_danshou_discard","rvm_fenghe_card","rvm_fenghe_destination","rvm_pingkou_targets"];let mut seen=std::collections::BTreeSet::new();let mut prompt_cases=vec![];
 for scenario in matrix["scenarios"].as_array().unwrap(){let packs=scenario["packs_json"].as_str().unwrap();let mut frames=vec![("initial".to_string(),scenario["initial"].as_str().unwrap())];frames.extend(scenario["steps"].as_array().unwrap().iter().enumerate().map(|(i,s)|(format!("step{i}"),s["archive"].as_str().unwrap())));
  for (checkpoint,hex) in frames{let e=engine_from_archive(hex,packs).unwrap();let Some(c)=e.open_choice()else{continue};if !names.contains(&c.spec.prompt)||seen.contains(c.spec.prompt){continue;}let prompt=c.spec.prompt;let seat=c.observer;let before=archive(&e);let checks=[plan_check(&e,packs,seat,0),plan_check(&e,packs,seat,48)];assert_eq!(archive(&e),before);seen.insert(prompt);prompt_cases.push(json!({"scenario":scenario["name"],"checkpoint":checkpoint,"prompt":prompt,"observer":seat,"public":public(&e,seat),"sourceArchive":before,"checks":checks}));}
 }
 assert_eq!(seen.len(),7);json!({"passed":true,"entrypoint":"Engine::plan_action","paired":{"seed":99107868,"hp":4,"queuedDamage":[damage(&pair[0]),damage(&pair[1])],"public":public(&pair[0],0),"sourceArchives":originals,"budgets":budgets,"plannerCalls":4},"realSevenPrompts":prompt_cases,"sevenPromptPlannerCalls":14,"policyVariant":1,"lookaheadDepth":2,"allNodes":0,"allSimFaults":0}).to_string()
}
