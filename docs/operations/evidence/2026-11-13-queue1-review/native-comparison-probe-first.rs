//! One-off audit probe; copied to a temporary Cargo example, never shipped.
//! Expected upstream paths: 3D精选/character/skill.js:6,244,6058;
//! wuxing/extension.js:62,173. Raw choices and actual card faces are retained.
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use content_native_loader::{bridge_pack, decision34_pack9_for_coexistence};
use engine_native::archive::{decode_archive, encode_archive};
use engine_native::catalog::{CardKind, Suit};
use engine_native::content::extension::ExtensionPack;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};
use serde_json::{json, Value};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn submit(e: &mut Engine, actor: u8, action: GameAction) -> CommandOutcome {
    let out = e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: DEDUP.fetch_add(1, Ordering::Relaxed),
        action,
    });
    assert!(!matches!(out, CommandOutcome::Rejected(_)), "{out:?}");
    out
}

fn pack(name: &str, asset: &str) -> ExtensionPack {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions").join(name).join("assets").join(asset);
    bridge_pack(&std::fs::read_to_string(path).unwrap().replace("\r\n", "\n"))
        .unwrap().pack
}

fn fresh(packs: &[&ExtensionPack], seats: usize, skill: Option<&str>, actor: u8) -> Engine {
    let mut e = Engine::new_with_extensions(2026111301, seats, packs).unwrap();
    e.state_core_mut().skills.clear();
    for seat in 0..seats as u8 {
        let p = e.state_core_mut().player_mut(seat);
        let cards: Vec<_> = p.hand.drain(..).chain(p.equips.drain(..)).chain(p.judge.drain(..)).collect();
        p.hp = 8;
        p.max_hp = 8;
        p.wunature = None;
        e.state_core_mut().deck.extend(cards);
    }
    e.state_core_mut().card_natures.clear();
    e.debug_set_turn(actor, Phase::Play, 1);
    if let Some(skill) = skill { e.debug_grant(0, skill); }
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

fn card(e: &Engine, kind: CardKind, suit: Suit, rank: u8, reserved: &[u32]) -> u32 {
    let candidates: Vec<_> = e.catalog().cards.iter().enumerate()
        .filter(|(id, d)| *id != 0 && d.kind == kind && !reserved.contains(&(*id as u32)))
        .collect();
    // Exact faces may be absent from the compact native deck; the JSON records
    // actual faces, and the browser must use these faces for an exact fixture.
    candidates.iter().find(|(_, d)| d.suit == suit && d.rank == rank)
        .or_else(|| candidates.iter().find(|(_, d)| d.suit == suit))
        .or_else(|| candidates.first()).expect("card kind available").0 as u32
}

fn hand(e: &mut Engine, seat: u8, card: u32) {
    remove(e, card);
    e.state_core_mut().player_mut(seat).hand.push(card);
}

fn face(e: &Engine, card: u32) -> Value {
    let d = &e.catalog().cards[card as usize];
    json!({"id":card,"name":d.name,"kind":format!("{:?}",d.kind),
        "suit":format!("{:?}",d.suit),"rank":d.rank,
        "wunature":e.state_core().card_natures.get(&card)})
}

fn answer(e: &mut Engine, packs: &[&ExtensionPack], selection: Vec<ChoiceOption>, trace: &mut Vec<Value>) {
    let c = e.open_choice().expect("open choice").clone();
    let bytes = encode_archive(&e.save());
    let mut restored = Engine::restore_with_extensions(&decode_archive(&bytes).unwrap(), packs).unwrap();
    assert_eq!(e.state_hash(), restored.state_hash());
    let selected = format!("{selection:?}");
    submit(e, c.observer, GameAction::Answer { choice_id: c.id, selection: selection.clone() });
    submit(&mut restored, c.observer, GameAction::Answer { choice_id: c.id, selection });
    assert_eq!(e.state_hash(), restored.state_hash(), "pending resume: {}",c.spec.prompt);
    trace.push(json!({"prompt":c.spec.prompt,"observer":c.observer,
        "options":format!("{:?}",c.spec.options),"min":c.spec.min,"max":c.spec.max,
        "allow_cancel":c.spec.allow_cancel,"selection":selected,"archive_bytes":bytes.len(),
        "restored_before_and_after_hash_equal":true}));
}

fn core_responses(e: &mut Engine, packs: &[&ExtensionPack], trace: &mut Vec<Value>) {
    for _ in 0..16 {
        let Some(c) = e.open_choice() else { return; };
        assert!(c.spec.options.contains(&ChoiceOption::Pass) || c.spec.allow_cancel,
            "unexpected forced core response: {}",c.spec.prompt);
        let selection = if c.spec.options.contains(&ChoiceOption::Pass) {
            vec![ChoiceOption::Pass]
        } else { vec![] };
        answer(e,packs,selection,trace);
    }
    panic!("core response did not settle");
}

fn top_four_sha(e: &mut Engine, reserved: &mut Vec<u32>) -> Vec<u32> {
    let mut top = vec![];
    for _ in 0..4 {
        let c = card(e,CardKind::Sha,Suit::Diamond,2,reserved);
        reserved.push(c);
        remove(e,c);
        top.push(c);
    }
    for &c in top.iter().rev() { e.state_core_mut().deck.push(c); }
    top
}

fn result(case: &str, e: &Engine, fixtures: Value, trace: Vec<Value>) {
    assert!(e.open_choice().is_none(),"unanswered choice in {case}");
    let players: Vec<_> = e.state_core().players.iter().enumerate().map(|(seat,p)| {
        let flag = |key: &str| e.state_core().vm_aux.flags.get(&(seat as u8,key.to_string())).copied().unwrap_or(0);
        json!({"seat":seat,"hp":p.hp,"hand":p.hand.iter().map(|&c|face(e,c)).collect::<Vec<_>>(),
            "equips":p.equips.iter().map(|&c|face(e,c)).collect::<Vec<_>>(),
            "juxian_ban":flag("dddjuxian_ban"),"xuanlun_triggered":flag("dddxuanlun_triggered")})
    }).collect();
    println!("{}",json!({"case":case,"fixtures":fixtures,"choices":trace,"players":players,
        "deck_top":e.state_core().deck.iter().rev().take(4).map(|&c|face(e,c)).collect::<Vec<_>>(),
        "discard":e.state_core().discard.iter().map(|&c|face(e,c)).collect::<Vec<_>>(),
        "log":e.log().iter().map(|x|format!("{x:?}")).collect::<Vec<_>>() }));
}

fn u01(packs: &[&ExtensionPack]) {
    for branch in ["sha","tao","cancel"] {
        let mut e = fresh(packs,8,Some("dddyuanzhen"),0);
        let duel = card(&e,CardKind::Duel,Suit::Spade,1,&[]);
        let sha = card(&e,CardKind::Sha,Suit::Club,7,&[duel]);
        let tao = card(&e,CardKind::Tao,Suit::Heart,3,&[duel,sha]);
        hand(&mut e,0,duel);hand(&mut e,2,sha);hand(&mut e,2,tao);
        let top = top_four_sha(&mut e,&mut vec![duel,sha,tao]);
        let fixtures = json!({"attack":face(&e,duel),"first":face(&e,sha),"second":face(&e,tao),
            "top":top.iter().map(|&c|face(&e,c)).collect::<Vec<_>>()});
        let mut trace = vec![];
        submit(&mut e,0,GameAction::PlayCard{card:duel,targets:vec![2]});
        assert_eq!(e.open_choice().unwrap().spec.prompt,"dddyuanzhen_discard");
        answer(&mut e,packs,match branch {"sha"=>vec![ChoiceOption::Card(sha)],
            "tao"=>vec![ChoiceOption::Card(tao)],_=>vec![]},&mut trace);
        core_responses(&mut e,packs,&mut trace);
        result(&format!("yuanzhen-{branch}"),&e,fixtures,trace);
    }
    {
        let mut e = fresh(packs,8,Some("dddjuxian"),0);
        let paid = card(&e,CardKind::Sha,Suit::Spade,7,&[]);
        let sha = card(&e,CardKind::Sha,Suit::Club,7,&[paid]);
        let equip = card(&e,CardKind::HorseMinus,Suit::Heart,5,&[paid,sha]);
        hand(&mut e,0,paid);hand(&mut e,2,sha);remove(&mut e,equip);
        e.state_core_mut().player_mut(2).equips.push(equip);
        let fixtures = json!({"paid":face(&e,paid),"first":face(&e,sha),"equip":face(&e,equip)});
        let mut trace = vec![];
        let skill = e.catalog().skill_by_name("dddjuxian").unwrap();
        submit(&mut e,0,GameAction::ActivateSkill{skill});
        answer(&mut e,packs,vec![ChoiceOption::Tag(0)],&mut trace);
        answer(&mut e,packs,vec![ChoiceOption::Card(paid)],&mut trace);
        answer(&mut e,packs,vec![ChoiceOption::Player(2)],&mut trace);
        result("juxian-top-no-gain-choice",&e,fixtures,trace);
    }
    for confirm in [true,false] {
        let mut e = fresh(packs,8,Some("dddxuanlun"),2);
        let duel = card(&e,CardKind::Duel,Suit::Spade,1,&[]);
        hand(&mut e,2,duel);
        let top = top_four_sha(&mut e,&mut vec![duel]);
        let fixtures = json!({"attack":face(&e,duel),"top":top.iter().map(|&c|face(&e,c)).collect::<Vec<_>>()});
        let mut trace = vec![];
        submit(&mut e,2,GameAction::PlayCard{card:duel,targets:vec![0]});
        assert_eq!(e.open_choice().unwrap().spec.prompt,"dddxuanlun_confirm");
        answer(&mut e,packs,if confirm {vec![ChoiceOption::Confirm]} else {vec![]},&mut trace);
        core_responses(&mut e,packs,&mut trace);
        result(if confirm {"xuanlun-confirm"}else{"xuanlun-cancel"},&e,fixtures,trace);
    }
}

fn main() {
    let d3 = pack("d3-upstream-noname","character.json");
    let p9 = decision34_pack9_for_coexistence(&[&d3]);
    u01(&[&d3,&p9]);
}
