use engine_native::{ChoiceOption as O, CommandOutcome, Engine, GameAction as A, GameCommand, Phase};
use engine_native::archive::{decode_archive, encode_archive};
use engine_native::catalog::{CardKind, Suit};
use engine_native::content::extension::ExtensionPack;
use engine_native::state::Role;
use serde_json::{json, Value};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

fn face(e: &Engine, id: u32) -> Value {
    let c = &e.catalog().cards[id as usize];
    let name = match c.kind {
        CardKind::Sha | CardKind::FireSha | CardKind::LeiSha => Some("sha"),
        CardKind::Shan => Some("shan"), CardKind::Tao => Some("tao"),
        CardKind::Jiu => Some("jiu"), CardKind::Wuzhong => Some("wuzhong"),
        CardKind::Duel => Some("juedou"), CardKind::Dismantle => Some("guohe"),
        CardKind::Snatch => Some("shunshou"), CardKind::Nullify => Some("wuxie"),
        CardKind::SavageAttack => Some("nanman"), CardKind::ArcheryAttack => Some("wanjian"),
        CardKind::Le => Some("lebu"), CardKind::Bing => Some("bingliang"),
        CardKind::ShanDian => Some("shandian"), CardKind::Chain => Some("tiesuo"),
        CardKind::Qinggang => Some("qinggang"),
        CardKind::Wugu => Some("wugu"), CardKind::Taoyuan => Some("taoyuan"),
        _ => match c.name {
            "诸葛连弩" => Some("zhuge"), "青釭剑" => Some("qinggang"),
            "八卦阵" => Some("bagua"), "仁王盾" => Some("renwang"),
            "赤兔" => Some("chitu"), "的卢" => Some("dilu"),
            _ => None,
        },
    };
    json!({"id":id,"name":c.name,"kind":format!("{:?}",c.kind),"upstream_name":name,
           "suit":format!("{:?}",c.suit).to_lowercase(),"rank":c.rank,
           "nature":match c.kind {CardKind::FireSha=>Some("fire"),CardKind::LeiSha=>Some("thunder"),_=>None}})
}

fn state(e: &Engine) -> Value {
    let core = e.state_core();
    let players: Vec<Value> = core.players.iter().enumerate().map(|(s,p)| {
        let skills: Vec<Value> = core.skills.iter().filter(|((owner,_),_)|usize::from(*owner)==s)
            .map(|((_,def),inst)|json!({"name":e.catalog().skill_name(*def),"enabled":inst.enabled,"once_used":inst.once_used})).collect();
        json!({"seat":s,"hp":p.hp,"max_hp":p.max_hp,"hujia":p.hujia,"face_down":p.face_down,
               "hand":p.hand.iter().map(|c|face(e,*c)).collect::<Vec<_>>(),
               "equips":p.equips.iter().map(|c|face(e,*c)).collect::<Vec<_>>(),"skills":skills})
    }).collect();
    let flags: Vec<_> = core.vm_aux.flags.iter().filter(|((_,k),_)|k.starts_with("mbcm"))
        .map(|((p,k),v)|json!({"seat":p,"key":k,"value":v})).collect();
    json!({"revision":e.revision(),"hash":format!("{:02x?}",e.state_hash()),"players":players,
           "turn":format!("{:?}",e.turn()),"jiu_active":e.turn().jiu_active,"jiu_used":e.turn().jiu_used,
           "flags":flags,"choice":e.open_choice().map(|c|json!({"id":c.id,"observer":c.observer,
               "prompt":c.spec.prompt,"kind":format!("{:?}",c.spec.kind),"min":c.spec.min,"max":c.spec.max,
               "allow_cancel":c.spec.allow_cancel,"options":c.spec.options.iter().map(|o|format!("{o:?}")).collect::<Vec<_>>()})),
           "deck_top_first":core.deck.iter().rev().map(|c|face(e,*c)).collect::<Vec<_>>(),
           "discard":core.discard.iter().map(|c|face(e,*c)).collect::<Vec<_>>(),
           "table":core.table.iter().map(|c|face(e,*c)).collect::<Vec<_>>()})
}

struct Run<'a> {
    e: Engine, pack: &'a ExtensionPack, base: &'a PathBuf, name: &'a str,
    restored: bool, dedup: u64, steps: Vec<Value>, checkpoints: Vec<Value>,
}
impl Run<'_> {
    fn cmd(&mut self, actor: u8, action: A) -> CommandOutcome {
        let request=GameCommand{actor,expected_revision:self.e.revision(),dedup_id:self.dedup,action};
        self.dedup+=1;
        let mark=self.e.log().len(); let request_debug=format!("{request:?}");
        let out=self.e.submit(request);
        let row=json!({"request":request_debug,"reply":format!("{out:?}"),"after":state(&self.e),
            "log_delta":self.e.log()[mark..].iter().map(|l|format!("{l:?}")).collect::<Vec<_>>()});
        let path=self.base.join(if self.restored {"native-restored-commands.jsonl"}else{"native-direct-commands.jsonl"});
        let mut stream=OpenOptions::new().create(true).append(true).open(path).unwrap();
        writeln!(stream,"{}",json!({"case":self.name,"step":self.steps.len(),"data":row})).unwrap();
        self.steps.push(row); out
    }
    fn checkpoint(&mut self) {
        if self.e.open_choice().is_none(){return;}
        let bytes=encode_archive(&self.e.save());
        let auth_before=self.e.save().auth_generation;
        let stage=self.checkpoints.len();
        let path=self.base.join(format!("{}-{}-pending-{stage}.ngsa",self.name,if self.restored{"restored"}else{"direct"}));
        fs::write(&path,&bytes).unwrap();
        let round=decode_archive(&bytes).map_err(|e|format!("{e:?}"))
            .and_then(|s|Engine::restore_with_extensions(&s,&[self.pack]).map_err(|e|format!("{e:?}")));
        match round {
            Ok(other)=>{
                let same_bytes=encode_archive(&other.save())==bytes;
                let same_hash=other.state_hash()==self.e.state_hash();
                let mut normalized=other.save();
                normalized.auth_generation=auth_before;
                let same_except_auth=normalized==self.e.save();
                self.checkpoints.push(json!({"file":path.file_name().unwrap().to_str(),"prompt":self.e.open_choice().unwrap().spec.prompt,
                    "bytes":bytes.len(),"roundtrip_bytes_identical":same_bytes,"state_hash_identical":same_hash,
                    "auth_before":auth_before,"auth_after":other.save().auth_generation,
                    "snapshot_equal_except_auth_generation":same_except_auth,"error":null}));
                if self.restored {self.e=other;}
            },
            Err(error)=>self.checkpoints.push(json!({"file":path.file_name().unwrap().to_str(),"error":error,
                "prompt":self.e.open_choice().unwrap().spec.prompt,"bytes":bytes.len(),"auth_before":auth_before})),
        }
    }
    fn answer(&mut self, selection: Vec<O>) {
        self.checkpoint();
        if let Some(c)=self.e.open_choice().cloned(){self.cmd(c.observer,A::Answer{choice_id:c.id,selection});}
    }
    fn settle(&mut self) {
        for _ in 0..24 {
            let Some(c)=self.e.open_choice().cloned() else {break;};
            let sel=if c.spec.options.contains(&O::Pass){vec![O::Pass]}else if c.spec.allow_cancel{vec![]}else{c.spec.options[..usize::from(c.spec.min)].to_vec()};
            self.answer(sel);
        }
    }
}

fn setup(pack:&ExtensionPack,name:&str)->Engine {
    let mut e=Engine::new_with_extensions(202611140029,4,&[pack]).unwrap();
    for p in 0..4 {e.debug_clear_hand(p);e.debug_set_max_hp(p,4);e.debug_set_hp(p,4);}
    e.state_core_mut().skills.clear();
    e.debug_set_identity_roles(vec![Role::Lord,Role::Loyalist,Role::Rebel,Role::Traitor]);
    e.debug_set_turn(0,if name.starts_with("q_"){Phase::Draw}else{Phase::Play},1);
    if name.starts_with("q_") {
        e.debug_grant(0,"mbcmqingzheng");
        if name!="q_no_hand" {e.debug_place_suit(0,Suit::Spade);e.debug_place_suit(0,Suit::Spade);e.debug_place_suit(0,Suit::Heart);}
        e.debug_place_suit(1,Suit::Club);e.debug_place_suit(1,Suit::Heart);e.debug_place_suit(1,Suit::Heart);
        e.debug_place_suit(2,Suit::Diamond);
    } else if name.starts_with("j_") {
        if name!="j_no_skill" {e.debug_grant(0,"mbcmjiushi");}
        if name.contains("damage") {e.debug_set_turn(1,Phase::Play,1);e.debug_place(1,"sha");}
        e.state_core_mut().player_mut(0).face_down=name.contains("back") || name=="j_active_facedown";
        e.debug_set_deck_top("wuzhong");
    } else {
        if name!="f_no_skill" {e.debug_grant(0,"mbcmfangzhu");}
        for p in [1,2] {e.debug_grant(p,"mbcmjiushi");e.debug_grant(p,"马术");}
        if name=="f_last_target" {e.state_core_mut().vm_aux.flags.insert((0,"mbcmfangzhu_last_target".into()),1);}
    }
    e
}

fn run_case<'a>(pack:&'a ExtensionPack,base:&'a PathBuf,name:&'a str,restored:bool)->Value {
    let mut run=Run{e:setup(pack,name),pack,base,name,restored,dedup:1,steps:vec![],checkpoints:vec![]};
    let initial=state(&run.e);
    if name.starts_with("q_") {
        run.cmd(0,A::NextPhase);
        if name!="q_no_hand" {
            run.answer(if name=="q_cancel_initial"{vec![]}else{vec![O::Confirm]});
            if name!="q_cancel_initial" {
                if name=="q_cancel_suit_attempt" {run.answer(vec![]);}
                else {
                    run.answer(vec![O::Tag(if name=="q_B"{1}else{0})]);
                    run.answer(if name=="q_cancel_target"{vec![]}else{vec![O::Player(if name=="q_target_B"{2}else{1})]});
                    if name!="q_cancel_target" {run.answer(vec![O::Tag(if name=="q_B"{1}else if name=="q_target_B"{3}else{2})]);}
                    run.settle();
                }
            }
        }
    } else if name.starts_with("j_") {
        if name.contains("damage") {
            let sha=run.e.state_core().player(1).hand[0];
            run.cmd(1,A::PlayCard{card:sha,targets:vec![0]});
            for _ in 0..12 {
                let Some(c)=run.e.open_choice().cloned()else{break;};
                if c.spec.prompt=="mbcmjiushi_turnback_confirm" {
                    run.answer(if name=="j_damage_back_cancel"{vec![]}else{vec![O::Confirm]});
                } else if c.spec.prompt=="mbcmjiushi_gain_confirm" {
                    run.answer(if name=="j_damage_back_accept_gain"{vec![O::Confirm]}else{vec![]});
                } else {let pass=c.spec.options.contains(&O::Pass);run.answer(if pass{vec![O::Pass]}else{vec![]});}
            }
        } else {
            let skill=run.e.catalog().skill_by_name("mbcmjiushi").unwrap();
            run.cmd(0,A::ActivateSkill{skill});
            if run.e.open_choice().is_some(){run.answer(if name=="j_active_gain"{vec![O::Confirm]}else{vec![]});}
        }
    } else {
        let skill=run.e.catalog().skill_by_name("mbcmfangzhu").unwrap();
        run.cmd(0,A::ActivateSkill{skill});
        if name=="f_cancel_target" {run.answer(vec![]);run.cmd(0,A::ActivateSkill{skill});}
        else if name=="f_last_target" || name=="f_self_attempt" {
            run.answer(vec![O::Player(if name=="f_last_target"{1}else{0})]);
        } else if name!="f_no_skill" {
            run.answer(vec![O::Player(if name=="f_target_B"{2}else{1})]);
            if name=="f_cancel_mode_attempt" {run.answer(vec![]);}
            else {
                run.answer(vec![O::Tag(if name=="f_skill_A"{1}else{0})]);
                run.cmd(0,A::ActivateSkill{skill});
            }
        }
    }
    let final_state=state(&run.e);let final_bytes=encode_archive(&run.e.save());
    fs::write(base.join(format!("{name}-{}-snapshot-debug.txt",if restored{"restored-final"}else{"direct-final"})),format!("{:#?}",run.e.save())).unwrap();
    fs::write(base.join(format!("{name}-{}.ngsa",if restored{"restored-final"}else{"direct-final"})),&final_bytes).unwrap();
    json!({"case":name,"mode":if restored{"restore_before_each_answer"}else{"direct"},"initial":initial,
           "commands":run.steps,"pending_archive_checks":run.checkpoints,"final":final_state,
           "final_archive_hex":final_bytes.iter().map(|b|format!("{b:02x}")).collect::<String>()})
}

fn number_after(text:&str,prefix:&str)->u64 {
    text.split_once(prefix).unwrap().1.chars().take_while(|c|c.is_ascii_digit()).collect::<String>().parse().unwrap()
}

fn continue_old(mut e:Engine,path:&std::path::Path)->Value {
    let file=path.file_name().unwrap().to_str().unwrap();
    let (case,mode)=if let Some((case,_))=file.split_once("-direct-pending-"){(case,"direct")}else{(file.split_once("-restored-pending-").unwrap().0,"restored")};
    let old:Value=serde_json::from_slice(&fs::read(path.parent().unwrap().join(format!("native-{case}.json"))).unwrap()).unwrap();
    let commands=old[mode]["commands"].as_array().unwrap();
    let choice=e.open_choice().unwrap().id;
    let marker=format!("action: Answer {{ choice_id: {choice},");
    let start=commands.iter().position(|row|row["request"].as_str().unwrap().contains(&marker)).unwrap();
    let mut replay=vec![];
    for row in &commands[start..] {
        let original=row["request"].as_str().unwrap();
        let action=if original.contains("action: Answer") {
            let body=original.split_once("selection: [").unwrap().1.split_once(']').unwrap().0;
            let selection=body.split(',').filter(|s|!s.trim().is_empty()).map(|s|{
                let s=s.trim();match s {
                    "Confirm"=>O::Confirm,"Pass"=>O::Pass,
                    _ if s.starts_with("Tag(")=>O::Tag(number_after(s,"Tag(") as u8),
                    _ if s.starts_with("Player(")=>O::Player(number_after(s,"Player(") as u8),
                    _=>panic!("unsupported old response option {s}"),
                }
            }).collect();
            A::Answer{choice_id:number_after(original,"choice_id: "),selection}
        } else if original.contains("action: ActivateSkill") {
            A::ActivateSkill{skill:number_after(original,"skill: ") as u16}
        } else {panic!("unsupported continuation command {original}");};
        let request=GameCommand{actor:number_after(original,"actor: ") as u8,expected_revision:e.revision(),dedup_id:number_after(original,"dedup_id: "),action};
        let request_debug=format!("{request:?}");let mark=e.log().len();let reply=e.submit(request);
        let log:Vec<_>=e.log()[mark..].iter().map(|l|format!("{l:?}")).collect();
        replay.push(json!({"request":request_debug,"request_equal_to_before":request_debug==original,
            "reply":format!("{reply:?}"),"reply_equal_to_before":format!("{reply:?}")==row["reply"].as_str().unwrap(),
            "log_delta":log,"log_delta_equal_to_before":serde_json::to_value(&log).unwrap()==row["log_delta"]}));
    }
    let hex=old[mode]["final_archive_hex"].as_str().unwrap();
    let bytes:Vec<u8>=(0..hex.len()).step_by(2).map(|i|u8::from_str_radix(&hex[i..i+2],16).unwrap()).collect();
    let expected=decode_archive(&bytes).unwrap();let mut actual=e.save();actual.auth_generation=expected.auth_generation;
    json!({"commands":replay,"final_snapshot_equal_except_auth_generation":actual==expected,
        "final_state_hash_equal":format!("{:02x?}",e.state_hash())==old[mode]["final"]["hash"].as_str().unwrap()})
}

fn main() {
    let base=PathBuf::from(std::env::var("L29_EVIDENCE").expect("L29_EVIDENCE"));
    let repo=PathBuf::from(std::env::var("L29_REPO").expect("L29_REPO"));
    let cards=fs::read_to_string(repo.join("packages/extensions/longxue-upstream-liuli/assets/cards.json")).unwrap();
    let pack=content_native_loader::bridge_pack(&cards).unwrap().pack;
    fs::write(base.join("native-pack-input.json"),&cards).unwrap();
    let inspect=Engine::new_with_extensions(202611140029,4,&[&pack]).unwrap();
    fs::write(base.join("native-input-metadata.json"),serde_json::to_vec_pretty(&json!({
        "seed":202611140029_u64,"seats":4,"pack_name":pack.def.name,"pack_version":pack.def.version,
        "pack_enabled":pack.enabled,"pack_cards":pack.cards.len(),"pack_host_skills":pack.host_skills.len(),
        "pack_vm_skills":pack.vm_skills.len(),"content_lock":engine_native::content::extension::content_lock_id(&[&pack]),
        "snapshot_bound_ruleset_version":inspect.save().ruleset_version
    })).unwrap()).unwrap();
    if let Ok(old_dir)=std::env::var("L29_RESTORE_INPUT_DIR") {
        let mut paths:Vec<_>=fs::read_dir(old_dir).unwrap().map(|p|p.unwrap().path())
            .filter(|p|p.extension().is_some_and(|e|e=="ngsa") && p.file_name().unwrap().to_string_lossy().contains("-pending-")).collect();
        paths.sort();
        let mut results=vec![];
        for path in paths {
            let bytes=fs::read(&path).unwrap();let old=decode_archive(&bytes).unwrap();
            let row=match Engine::restore_with_extensions(&old,&[&pack]) {
                Ok(e)=>{
                    let mut normalized=e.save();normalized.auth_generation=old.auth_generation;
                    let prompt=e.open_choice().map(|c|c.spec.prompt);let auth_after=e.save().auth_generation;
                    let same=normalized==old;let continuation=continue_old(e,&path);
                    json!({"input":path,"bytes":bytes.len(),"prompt":prompt,
                        "auth_before":old.auth_generation,"auth_after":auth_after,
                        "snapshot_equal_except_auth_generation":same,"continuation":continuation,"error":null})
                },
                Err(error)=>json!({"input":path,"bytes":bytes.len(),"error":format!("{error:?}")}),
            };
            results.push(row);
        }
        fs::write(base.join("old-ngsa-restore-results.json"),serde_json::to_vec_pretty(&results).unwrap()).unwrap();
    }
    let cases=["q_A","q_B","q_target_B","q_cancel_initial","q_cancel_target","q_cancel_suit_attempt","q_no_hand",
        "j_active_gain","j_active_cancel_gain","j_active_facedown","j_no_skill","j_damage_back_accept_gain",
        "j_damage_back_cancel","j_damage_faceup","f_skill_A","f_ban_B","f_target_B","f_cancel_target",
        "f_last_target","f_self_attempt","f_cancel_mode_attempt","f_no_skill"];
    let mut rows=vec![];
    let only=std::env::var("L29_CASES").ok();
    for name in cases.into_iter().filter(|name|only.as_ref().is_none_or(|v|v.split(',').any(|x|x==*name))) {
        let direct=run_case(&pack,&base,name,false);
        let restored=run_case(&pack,&base,name,true);
        let same=direct["final_archive_hex"]==restored["final_archive_hex"];
        let row=json!({"case":name,"direct":direct,"restored":restored,"final_ngsa_bytes_equal":same});
        fs::write(base.join(format!("native-{name}.json")),serde_json::to_vec_pretty(&row).unwrap()).unwrap();
        println!("{}",json!({"case":name,"final_ngsa_bytes_equal":same,"pending_checkpoints":row["direct"]["pending_archive_checks"].as_array().unwrap().len(),"final_choice":row["direct"]["final"]["choice"]}));
        rows.push(row);
    }
    fs::write(base.join("native-results.json"),serde_json::to_vec_pretty(&rows).unwrap()).unwrap();
}
