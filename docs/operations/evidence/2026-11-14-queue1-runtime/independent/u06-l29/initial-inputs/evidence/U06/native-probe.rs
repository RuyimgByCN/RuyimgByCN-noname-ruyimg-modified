//! Runtime comparison fixtures, not a shipped test or equivalence signoff.
//! Upstream: 欢乐卡牌/card/skill.js:5-31,53-128; card/card.js:4-15,81-108.
use engine_native::archive::{decode_archive, encode_archive};
use engine_native::catalog::{CardKind, Suit};
use engine_native::content::extension::ExtensionPack;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};
use serde_json::{json, Value};
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn submit(e: &mut Engine, actor: u8, action: GameAction) -> CommandOutcome {
    e.submit(GameCommand { actor, expected_revision: e.revision(),
        dedup_id: DEDUP.fetch_add(1, Ordering::Relaxed), action })
}

fn accepted(e: &mut Engine, actor: u8, action: GameAction) -> CommandOutcome {
    let out = submit(e, actor, action);
    assert!(!matches!(out, CommandOutcome::Rejected(_)), "{out:?}");
    out
}

fn fresh(pack: &ExtensionPack) -> Engine {
    let mut e = Engine::new_with_extensions(2026111406, 8, &[pack]).unwrap();
    e.state_core_mut().skills.clear();
    for seat in 0..8 {
        let p = e.state_core_mut().player_mut(seat);
        let cards: Vec<_> = p.hand.drain(..).chain(p.equips.drain(..)).chain(p.judge.drain(..)).collect();
        p.hp = 8;
        p.max_hp = 8;
        e.state_core_mut().deck.extend(cards);
    }
    e.debug_set_turn(0, Phase::Play, 1);
    e
}

fn remove(e: &mut Engine, card: u32) {
    e.state_core_mut().deck.retain(|&c| c != card);
    e.state_core_mut().discard.retain(|&c| c != card);
    e.state_core_mut().table.retain(|&c| c != card);
    for p in &mut e.state_core_mut().players {
        p.hand.retain(|&c| c != card);
        p.equips.retain(|&c| c != card);
        p.judge.retain(|&c| c != card);
    }
}

fn hand(e: &mut Engine, owner: u8, card: u32) {
    remove(e, card);
    e.state_core_mut().player_mut(owner).hand.push(card);
}

fn named(e: &Engine, name: &str) -> u32 {
    e.catalog().cards.iter().position(|d| d.name == name).unwrap() as u32
}

fn face_card(e: &Engine, kind: CardKind, suit: Suit, rank: u8) -> u32 {
    e.catalog().cards.iter().enumerate()
        .find(|(_, d)| d.kind == kind && d.suit == suit && d.rank == rank)
        .map(|(id, _)| id as u32).expect("exact fixture face must exist")
}

fn equip(e: &mut Engine, seat: u8, name: &str) -> u32 {
    let card = named(e, name);
    hand(e, seat, card);
    e.debug_set_turn(seat, Phase::Play, 1);
    accepted(e, seat, GameAction::PlayCard { card, targets: vec![] });
    assert!(e.debug_equips(seat).contains(&card));
    card
}

fn face(e: &Engine, card: u32) -> Value {
    let d = &e.catalog().cards[card as usize];
    json!({"id":card,"name":d.name,"kind":format!("{:?}",d.kind),
        "suit":format!("{:?}",d.suit).to_lowercase(),"rank":d.rank})
}

fn players(e: &Engine) -> Value {
    json!((0..8).map(|seat| {
        let p = e.state_core().player(seat);
        let skills: Vec<_> = e.state_core().skills.range((seat,0)..(seat,u16::MAX))
            .map(|((_,d),s)| json!({"name":e.catalog().skills[*d as usize].name,
                "expires_end_of_turn":s.expires_end_of_turn})).collect();
        json!({"seat":seat,"hp":p.hp,"max_hp":p.max_hp,
            "hand":p.hand.iter().map(|&c|face(e,c)).collect::<Vec<_>>(),
            "equips":p.equips.iter().map(|&c|face(e,c)).collect::<Vec<_>>(),"skills":skills})
    }).collect::<Vec<_>>())
}

fn answer(e: &mut Engine, pack: &ExtensionPack, selection: Vec<ChoiceOption>, trace: &mut Vec<Value>) {
    let c = e.open_choice().expect("choice").clone();
    let bytes = encode_archive(&e.save());
    let mut restored = Engine::restore_with_extensions(&decode_archive(&bytes).unwrap(), &[pack]).unwrap();
    assert_eq!(e.state_hash(),restored.state_hash());
    let before = e.log().len();
    let out = submit(e,c.observer,GameAction::Answer { choice_id:c.id, selection:selection.clone() });
    let resumed = submit(&mut restored,c.observer,GameAction::Answer { choice_id:c.id, selection:selection.clone() });
    assert_eq!(format!("{out:?}"),format!("{resumed:?}"));
    assert_eq!(e.state_hash(),restored.state_hash());
    assert_eq!(&e.log()[before..],restored.log());
    trace.push(json!({"prompt":c.spec.prompt,"observer":c.observer,
        "options":format!("{:?}",c.spec.options),"min":c.spec.min,"max":c.spec.max,
        "allow_cancel":c.spec.allow_cancel,"selection":format!("{selection:?}"),
        "outcome":format!("{out:?}"),"rejected":matches!(out,CommandOutcome::Rejected(_)),
        "archive_bytes":bytes.len(),"resumed_hash_and_log_equal":true}));
}

fn result(case: &str, e: &Engine, initial: Value, trace: Vec<Value>) {
    let choice = e.open_choice().map(|c|json!({"prompt":c.spec.prompt,
        "allow_cancel":c.spec.allow_cancel,"options":format!("{:?}",c.spec.options)}));
    println!("{}",json!({"case":case,"initial":initial,"players":players(e),
        "settled":e.open_choice().is_none(),"pending":choice,"choices":trace,
        "turn":format!("{:?}",e.turn()),
        "discard":e.state_core().discard.iter().map(|&c|face(e,c)).collect::<Vec<_>>(),
        "log":e.log().iter().map(|x|format!("{x:?}")).collect::<Vec<_>>() }));
}

fn monkey(pack: &ExtensionPack) {
    for branch in ["confirm","cancel","self"] {
        let mut e=fresh(pack);
        equip(&mut e,0,"hnl_monkey_3");
        let actor=if branch=="self" {0} else {1};
        e.debug_set_turn(actor,Phase::Play,1);
        e.debug_set_hp(actor,2);
        let tao=face_card(&e,CardKind::Tao,Suit::Heart,3);
        hand(&mut e,actor,tao);
        let initial=json!({"players":players(&e),"actor":actor,"attack":face(&e,tao)});
        accepted(&mut e,actor,GameAction::PlayCard{card:tao,targets:vec![]});
        let mut trace=vec![];
        if branch!="self" {
            assert_eq!(e.open_choice().unwrap().spec.prompt,"huanle_monkey_confirm");
            answer(&mut e,pack,vec![if branch=="confirm" {ChoiceOption::Confirm} else {ChoiceOption::Pass}],&mut trace);
        }
        assert!(e.open_choice().is_none());
        result(&format!("monkey-{branch}"),&e,initial,trace);
    }
}

fn kuwu(pack: &ExtensionPack) {
    for branch in ["hand","equip","empty","duel"] {
        let mut e=fresh(pack);
        equip(&mut e,0,"hnl_kuwu_0");
        let tao=face_card(&e,CardKind::Tao,Suit::Heart,3);
        let monkey=named(&e,"hnl_monkey_3");
        if branch!="empty" {
            equip(&mut e,1,"hnl_monkey_3");
            hand(&mut e,1,tao);
        }
        e.debug_set_turn(0,Phase::Play,1);
        let attack=if branch=="duel" {face_card(&e,CardKind::Duel,Suit::Spade,1)}
            else {face_card(&e,CardKind::Sha,Suit::Spade,7)};
        hand(&mut e,0,attack);
        let initial=json!({"players":players(&e),"actor":0,"attack":face(&e,attack)});
        accepted(&mut e,0,GameAction::PlayCard{card:attack,targets:vec![1]});
        let mut trace=vec![];
        if branch=="hand" || branch=="equip" {
            assert_eq!(e.open_choice().unwrap().spec.prompt,"kuwu_discard");
            answer(&mut e,pack,vec![],&mut trace);
            assert!(trace.last().unwrap()["rejected"].as_bool().unwrap());
            answer(&mut e,pack,vec![ChoiceOption::Card(if branch=="hand" {tao} else {monkey})],&mut trace);
        }
        assert!(e.open_choice().is_none());
        result(&format!("kuwu-{branch}"),&e,initial,trace);
    }
}

fn xuelun(pack: &ExtensionPack) {
    for branch in ["skill-a","skill-b","target-b","target-cancel","skill-cancel","no-skills"] {
        let mut e=fresh(pack);
        equip(&mut e,0,"hnl_xuelunyang_4");
        if branch!="no-skills" {
            e.debug_grant(1,"马术");
            e.debug_grant(1,"咆哮");
            e.debug_grant(2,"武圣");
        }
        e.debug_set_turn(7,Phase::End,1);
        let initial=json!({"players":players(&e),"actor":0,"trigger":"phaseBegin"});
        accepted(&mut e,7,GameAction::NextPhase);
        let mut trace=vec![];
        let mut selected_skill=None;
        if branch!="no-skills" {
            assert_eq!(e.open_choice().unwrap().spec.prompt,"huanle_xuelunyang_target");
            let target=if branch=="target-b" {2} else {1};
            answer(&mut e,pack,if branch=="target-cancel" {vec![]} else {vec![ChoiceOption::Player(target)]},&mut trace);
            if branch!="target-cancel" {
                assert_eq!(e.open_choice().unwrap().spec.prompt,"huanle_xuelunyang_skill");
                let idx=if branch=="skill-b" {1} else {0};
                let skills: Vec<_> =e.state_core().skills.range((target,0)..(target,u16::MAX))
                    .map(|((_,d),_)|e.catalog().skills[*d as usize].name).collect();
                if branch!="skill-cancel" {selected_skill=Some(skills[idx]);}
                answer(&mut e,pack,if branch=="skill-cancel" {vec![]} else {vec![ChoiceOption::Tag(idx as u8)]},&mut trace);
            }
        }
        if branch=="skill-cancel" {assert!(e.open_choice().is_some());}
        else {assert!(e.open_choice().is_none());}
        let initial=json!({"fixture":initial,"selected_skill":selected_skill});
        result(&format!("xuelun-{branch}"),&e,initial,trace);
    }
}

fn main() {
    let root=std::env::var("NONAME_REPO_ROOT").unwrap();
    let asset=std::fs::read_to_string(format!("{root}/packages/extensions/huanle-upstream-noname/assets/cards.json")).unwrap();
    let pack=content_native_loader::bridge_pack(&asset.replace("\r\n","\n")).unwrap().pack;
    monkey(&pack);
    kuwu(&pack);
    xuelun(&pack);
}
