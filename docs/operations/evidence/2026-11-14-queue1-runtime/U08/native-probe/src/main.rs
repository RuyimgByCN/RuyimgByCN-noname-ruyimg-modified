use engine_native::{ChoiceOption as O, CommandOutcome, Engine, GameAction as A, GameCommand, Phase};
use engine_native::archive::{decode_archive, encode_archive};
use engine_native::catalog::{CardKind};
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
        CardKind::Qinggang => Some("qinggang"), CardKind::Zhuge => Some("zhuge"),
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
               "judge":p.judge.iter().map(|c|face(e,*c)).collect::<Vec<_>>(),"equips":p.equips.iter().map(|c|face(e,*c)).collect::<Vec<_>>(),"skills":skills})
    }).collect();
    let flags: Vec<_> = core.vm_aux.flags.iter().filter(|((_,k),_)|["yixin","bolehuiyan","guifu"].iter().any(|p|k.starts_with(p)))
        .map(|((p,k),v)|json!({"seat":p,"key":k,"value":v})).collect();
    json!({"revision":e.revision(),"hash":format!("{:02x?}",e.state_hash()),"players":players,
           "turn":format!("{:?}",e.turn()),"jiu_active":e.turn().jiu_active,"jiu_used":e.turn().jiu_used,
           "flags":flags,"once":format!("{:?}",core.vm_aux.once),"choice":e.open_choice().map(|c|json!({"id":c.id,"observer":c.observer,
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

}

fn equip(e: &mut Engine, seat: u8, name: &str) {
    let id = e.debug_place(seat, name);
    e.state_core_mut().player_mut(seat).hand.retain(|c| *c != id);
    e.state_core_mut().player_mut(seat).equips.push(id);
}
fn setup(pack: &ExtensionPack, name: &str) -> Engine {
    let mut e=Engine::new_with_extensions(202611140008,8,&[pack]).unwrap();
    for p in 0..8 {e.debug_clear_hand(p);e.debug_set_max_hp(p,4);e.debug_set_hp(p,4);}
    e.state_core_mut().skills.clear();
    e.debug_set_identity_roles(vec![Role::Lord,Role::Loyalist,Role::Rebel,Role::Traitor,Role::Loyalist,Role::Rebel,Role::Rebel,Role::Rebel]);
    e.debug_set_turn(0,Phase::Play,1);
    if name.starts_with("y_") {
        if name!="y_no_skill" {e.debug_grant(0,"yixin");}
        e.debug_place(0,"sha");e.debug_place(0,"shan");
        if name!="y_two_cards"{e.debug_place(0,"tao");}
        if name=="y_reuse" {e.debug_place(0,"sha");e.debug_place(0,"shan");}
        if name!="y_no_wound"{e.debug_set_hp(1,2);e.debug_set_max_hp(2,6);e.debug_set_hp(2,1);}
        if name=="y_self"{e.debug_set_hp(0,2);}
    } else if name.starts_with("g_") {
        if name!="g_no_skill"{e.debug_grant(0,"guifu");}
        if name=="g_only_own" {equip(&mut e,0,"qinggang");}
        else if name!="g_no_equip"{equip(&mut e,1,"qinggang");equip(&mut e,2,"zhuge");}
    } else {
        if name!="b_no_skill"{e.debug_grant(0,"bolehuiyan");}
        let actor=if name=="b_owner_attacker"{0}else{1};
        let target=if name=="b_owner_attacker"{1}else if name=="b_owner_target"{0}else{2};
        e.debug_set_turn(actor,Phase::Play,1);
        e.debug_place(actor,"sha");
        if name!="b_owner_attacker"{e.debug_place(0,"sha");}
        e.debug_place(0,"tao");
        if name!="b_empty_target" && target!=0 {e.debug_place(target,if ["b_miss","b_wrong_A","b_wrong_B"].contains(&name){"shan"}else{"tao"});}
        if ["b_repeat","b_cancel_then_retry"].contains(&name){e.debug_place(actor,"sha");equip(&mut e,actor,"zhuge");}
    }
    let names=["wuzhong","tao","qinggang","sha","dismantle","shan","zhuge","jiu"];
    let ids:Vec<_>=names.iter().rev().filter(|n|!((name.starts_with("g_") && !["g_no_equip","g_only_own"].contains(&name) && ["qinggang","zhuge"].contains(n)) || (name=="g_only_own" && **n=="qinggang") || (["b_repeat","b_cancel_then_retry"].contains(&name) && **n=="zhuge"))).map(|name|e.debug_set_deck_top(name)).collect();
    let all=std::mem::take(&mut e.state_core_mut().deck);
    for c in all {if !ids.contains(&c){e.state_core_mut().discard.push(c);}}
    e.state_core_mut().deck=ids;
    let core=e.state_core();let mut all=core.deck.clone();all.extend(core.discard.iter());all.extend(core.table.iter());
    for p in &core.players {all.extend(p.hand.iter());all.extend(p.equips.iter());all.extend(p.judge.iter());}
    all.sort_unstable();assert_eq!(all,(0..e.catalog().cards.len() as u32).collect::<Vec<_>>(),"initial physical conservation {name}");
    e
}
fn run_case(pack:&ExtensionPack,base:&PathBuf,name:&str,restored:bool)->Value {
    let mut run=Run{e:setup(pack,name),pack,base,name,restored,dedup:1,steps:vec![],checkpoints:vec![]};
    let initial=state(&run.e);
    if name.starts_with("y_") || name.starts_with("g_") {
        let skill=run.e.catalog().skill_by_name(if name.starts_with("y_"){ "yixin" } else { "guifu" }).unwrap();
        run.cmd(0,A::ActivateSkill{skill});
        for _ in 0..12 {
            let Some(c)=run.e.open_choice().cloned()else{break;};
            let sel=match c.spec.prompt {
                "yixin:discard_two"=>if name=="y_cancel_cards"{vec![]}else{let cards=run.e.state_core().player(0).hand.clone();let start=usize::from(name=="y_cost_B");vec![O::Card(cards[start]),O::Card(cards[start+1])]},
                "yixin:choose_target"=>if name=="y_cancel_target"{vec![]}else{vec![O::Player(if name=="y_target_B"{2}else if name=="y_self"{0}else{1})]},
                "guifu:choose_target"=>if name=="g_cancel_target"{vec![]}else{vec![O::Player(if name=="g_target_B"{2}else{1})]},
                "guifu:choose_equip"=>vec![c.spec.options[0]],
                _=>panic!("unexpected {name} {}",c.spec.prompt),
            };
            run.answer(sel);
        }
        if name.contains("cancel") || name.ends_with("reuse") {
            run.cmd(0,A::ActivateSkill{skill});
            if run.e.open_choice().is_some(){run.answer(vec![]);}
        }
    } else {
        let actor=if name=="b_owner_attacker"{0}else{1};
        let target=if name=="b_owner_attacker"{1}else if name=="b_owner_target"{0}else{2};
        let attacks=if ["b_repeat","b_cancel_then_retry"].contains(&name){2}else{1};
        for attack in 0..attacks {
            let sha=run.e.state_core().player(actor).hand.iter().copied().find(|c|run.e.catalog().card_kind(*c)==CardKind::Sha).unwrap();
            run.cmd(actor,A::PlayCard{card:sha,targets:vec![target]});
            for _ in 0..12 {
                let Some(c)=run.e.open_choice().cloned() else{break;};
                let sel=match c.spec.prompt {
                    "bolehuiyan:predict"=>if name=="b_cancel_predict" || (name=="b_cancel_then_retry"&&attack==0){vec![]}else{vec![O::Tag(if ["b_miss","b_wrong_miss","b_cancel_discard"].contains(&name){2}else{1})]},
                    "bolehuiyan:discard"=>if name=="b_cancel_discard"{vec![]}else{vec![c.spec.options[usize::from(name=="b_wrong_B")]]},
                    "respond_shan"=>if ["b_miss","b_wrong_A","b_wrong_B"].contains(&name){vec![c.spec.options.iter().copied().find(|o|matches!(o,O::Card(_))).unwrap()]}else{vec![]},
                    "trigger_confirm"=>vec![O::Confirm],
                    _=>panic!("unexpected {name} {}",c.spec.prompt),
                };
                run.answer(sel);
                if name=="b_cancel_discard"&&c.spec.prompt=="bolehuiyan:discard"{break;}
            }
        }
    }
    let bytes=encode_archive(&run.e.save());let mode=if restored{"restored"}else{"direct"};
    fs::write(base.join(format!("{name}-{mode}-final.ngsa")),&bytes).unwrap();
    fs::write(base.join(format!("{name}-{mode}-final-snapshot-debug.txt")),format!("{:#?}",run.e.save())).unwrap();
    json!({"case":name,"mode":if restored{"restore_before_each_answer"}else{"direct"},"initial":initial,"commands":run.steps,"pending_archive_checks":run.checkpoints,"final":state(&run.e),"final_archive_hex":bytes.iter().map(|b|format!("{b:02x}")).collect::<String>()})
}
fn main(){
    let base=PathBuf::from(std::env::var("U08_EVIDENCE").unwrap());let repo=PathBuf::from(std::env::var("U08_REPO").unwrap());
    let input=fs::read_to_string(repo.join("packages/extensions/hero-kill-upstream-noname/assets/cards.json")).unwrap().replace("\r\n","\n");
    let mut pack=content_native_loader::bridge_pack(&input).unwrap().pack;
    content_native_loader::attach_formal_package_upstream_rules("upstream-noname","hero-kill",&mut pack,&[]);
    fs::write(base.join("native-pack-input.json"),input).unwrap();
    let inspect=Engine::new_with_extensions(202611140008,8,&[&pack]).unwrap();
    fs::write(base.join("native-catalog.json"),serde_json::to_vec_pretty(&inspect.catalog().cards.iter().enumerate().map(|(i,_)|face(&inspect,i as u32)).collect::<Vec<_>>()).unwrap()).unwrap();
    fs::write(base.join("native-bound-snapshot-debug.txt"),format!("{:#?}",inspect.save())).unwrap();
    fs::write(base.join("native-input-metadata.json"),serde_json::to_vec_pretty(&json!({"seed":202611140008_u64,"seats":8,"pack_name":pack.def.name,"pack_version":pack.def.version,"host_skills":pack.host_skills.len(),"vm_skills":pack.vm_skills.len(),"pack_cards":pack.cards.iter().map(|c|c.name).collect::<Vec<_>>(),"content_lock":engine_native::content::extension::content_lock_id(&[&pack]),"snapshot_bound_ruleset_version":inspect.save().ruleset_version,"catalog_was_mutated":false,"pack_placeholder_cards_added":false})).unwrap()).unwrap();
    let cases=["y_target_A","y_target_B","y_cost_B","y_self","y_cancel_cards","y_cancel_target","y_two_cards","y_no_wound","y_reuse","y_no_skill","g_target_A","g_target_B","g_cancel_target","g_no_equip","g_only_own","g_reuse","g_no_skill","b_hit","b_miss","b_wrong_A","b_wrong_B","b_wrong_miss","b_cancel_predict","b_cancel_discard","b_empty_target","b_owner_attacker","b_owner_target","b_under_control","b_repeat","b_cancel_then_retry","b_no_skill"];
    let mut results=vec![];
    for name in cases {
        let direct=run_case(&pack,&base,name,false);let restored=run_case(&pack,&base,name,true);
        let row=json!({"case":name,"final_ngsa_bytes_equal":direct["final_archive_hex"]==restored["final_archive_hex"],"direct":direct,"restored":restored});
        fs::write(base.join(format!("native-{name}.json")),serde_json::to_vec_pretty(&row).unwrap()).unwrap();
        println!("{}",json!({"case":name,"pending":row["direct"]["pending_archive_checks"].as_array().unwrap().len(),"final_choice":row["direct"]["final"]["choice"]}));results.push(row);
    }
    fs::write(base.join("native-results.json"),serde_json::to_vec_pretty(&results).unwrap()).unwrap();
}
