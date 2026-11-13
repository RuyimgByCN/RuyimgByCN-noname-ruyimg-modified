use engine_native::{ChoiceOption as O, CommandOutcome, Engine, GameAction as A, GameCommand, Phase};
use engine_native::archive::{decode_archive, encode_archive};
use engine_native::catalog::CardKind;
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
    let flags: Vec<_> = core.vm_aux.flags.iter().filter(|((_,k),_)|["reguhuo","rangjie","k_danggu","k_mowang"].iter().any(|p|k.starts_with(p)))
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
    fn settle(&mut self) {
        for _ in 0..24 {
            let Some(c)=self.e.open_choice().cloned() else {break;};
            let sel=if c.spec.options.contains(&O::Pass){vec![O::Pass]}else if c.spec.allow_cancel{vec![]}else{c.spec.options[..usize::from(c.spec.min)].to_vec()};
            self.answer(sel);
        }
    }
}

fn setup(pack:&ExtensionPack,name:&str)->Engine {
    let mut e=Engine::new_with_extensions(202611140010,4,&[pack]).unwrap();
    for p in 0..4 {e.debug_clear_hand(p);e.debug_set_max_hp(p,4);e.debug_set_hp(p,4);}
    e.state_core_mut().skills.clear();e.debug_set_identity_roles(vec![Role::Lord,Role::Loyalist,Role::Rebel,Role::Traitor]);
    e.debug_set_turn(0,if name.starts_with("k_"){Phase::Prepare}else{Phase::Play},1);
    if name.starts_with("g_") {
        if name!="g_no_skill"{e.debug_grant(0,"reguhuo");}
        if name!="g_no_hand"{e.debug_place(0,if name.contains("true")||name=="g_tao"{"sha"}else{"tao"});}
        if name.contains("cancel"){e.debug_place(0,"sha");}
        if name.contains("true_question"){e.debug_place(2,"tao");}
        if name=="g_tao"{e.debug_set_hp(0,2);}
        if name=="g_response"{e.debug_set_turn(1,Phase::Play,1);e.debug_place(1,"sha");}
    } else if name.starts_with("r_") {
        if name!="r_no_skill"{e.debug_grant(0,"rangjie");}
        e.debug_set_turn(1,Phase::Play,1);e.debug_place(1,"sha");
        if name.contains("move"){e.debug_place(1,"zhuge");}
        if name=="r_double"{e.debug_place(1,"jiu");}
    } else {
        if name!="k_no_skill"{e.debug_grant(0,"k_danggu");}
        if name=="k_after_initial_no_trigger"{e.state_core_mut().vm_aux.flags.insert((0,"k_danggu_inited".into()),1);}
    }
    let names=["wuzhong","tao","qinggang","sha","dismantle","shan","zhuge","jiu"];
    let ids:Vec<_>=names.iter().rev().filter(|n| !(name.contains("move") && **n=="zhuge")).map(|name|e.debug_set_deck_top(name)).collect();
    let all=std::mem::take(&mut e.state_core_mut().deck);
    for c in all {if !ids.contains(&c){e.state_core_mut().discard.push(c);}}
    e.state_core_mut().deck=ids;
    e
}

fn run_case(pack:&ExtensionPack,base:&PathBuf,name:&str,restored:bool)->Value {
    let mut run=Run{e:setup(pack,name),pack,base,name,restored,dedup:1,steps:vec![],checkpoints:vec![]};let initial=state(&run.e);
    if name.starts_with("g_") {
        let skill=run.e.catalog().skill_by_name("reguhuo").unwrap();
        if name=="g_response" {
            let c=run.e.state_core().player(1).hand[0];run.cmd(1,A::PlayCard{card:c,targets:vec![0]});
            if run.e.open_choice().is_some(){run.checkpoint();run.cmd(0,A::ActivateSkill{skill});run.settle();}
        } else {
            run.cmd(0,A::ActivateSkill{skill});
            if !["g_no_hand","g_no_skill"].contains(&name) {
                let carrier=run.e.state_core().player(0).hand[0];
                run.answer(if name=="g_cancel_card"{vec![]}else{vec![O::Card(carrier)]});
                if name=="g_cancel_card"{run.cmd(0,A::ActivateSkill{skill});}
                else {
                    run.answer(if name=="g_cancel_name"{vec![]}else{vec![O::Tag(if name=="g_tao"{4}else{0})]});
                    if name=="g_cancel_name"{run.cmd(0,A::ActivateSkill{skill});}
                    else {
                        if run.e.open_choice().is_some_and(|c|c.spec.prompt=="reguhuo_targets"){
                            run.answer(if name=="g_cancel_target"{vec![]}else{vec![O::Player(if name=="g_tao"{0}else{1})]});
                        }
                        if name!="g_cancel_target"{run.settle();}
                    }
                }
            }
        }
    } else if name.starts_with("r_") {
        if name.contains("move") {
            let eq=run.e.state_core().player(1).hand.iter().copied().find(|c|run.e.catalog().card_kind(*c)==CardKind::Zhuge).unwrap();
            run.cmd(1,A::PlayCard{card:eq,targets:vec![]});
        }
        if name=="r_double"{
            let c=run.e.state_core().player(1).hand.iter().copied().find(|c|run.e.catalog().card_kind(*c)==CardKind::Jiu).unwrap();
            run.cmd(1,A::PlayCard{card:c,targets:vec![]});
        }
        let sha=run.e.state_core().player(1).hand.iter().copied().find(|c|run.e.catalog().card_kind(*c)==CardKind::Sha).unwrap();
        run.cmd(1,A::PlayCard{card:sha,targets:vec![if name=="r_other_target"{2}else{0}]});
        for _ in 0..18 {
            let Some(c)=run.e.open_choice().cloned()else{break;};
            let selection=match c.spec.prompt {
                "trigger_confirm"=>vec![O::Confirm],
                "rangjie_choice"=>vec![O::Tag(if name=="r_cancel_choice"{2}else if name.contains("move"){1}else{0})],
                "rangjie_type"=>if name=="r_cancel_type"{vec![]}else{vec![O::Tag(if name=="r_basic"||name=="r_double"{0}else if name=="r_equip"{2}else{1})]},
                "rangjie_move_card"=>vec![c.spec.options[0]],
                "rangjie_move_target"=>vec![O::Player(if name=="r_move_B"{2}else{0})],
                _=>if c.spec.options.contains(&O::Pass){vec![O::Pass]}else{vec![]},
            };
            run.answer(selection);if name=="r_cancel_type"&&c.spec.prompt=="rangjie_type"{break;}
        }
    } else {
        run.cmd(0,A::NextPhase);
        if let Some(c)=run.e.open_choice().cloned(){
            if c.spec.prompt=="k_danggu_partner"{run.answer(if name=="k_cancel"{vec![]}else{vec![c.spec.options[if name=="k_B"&&c.spec.options.len()>1{1}else{0}]]});}
        }
    }
    let bytes=encode_archive(&run.e.save());
    fs::write(base.join(format!("{name}-{}-final.ngsa",if restored{"restored"}else{"direct"})),&bytes).unwrap();
    fs::write(base.join(format!("{name}-{}-final-snapshot-debug.txt",if restored{"restored"}else{"direct"})),format!("{:#?}",run.e.save())).unwrap();
    json!({"case":name,"mode":if restored{"restore_before_each_answer"}else{"direct"},"initial":initial,"commands":run.steps,"pending_archive_checks":run.checkpoints,
        "final":state(&run.e),"final_archive_hex":bytes.iter().map(|b|format!("{b:02x}")).collect::<String>()})
}

fn run_formal_followup(pack:&ExtensionPack,base:&PathBuf,name:&str,restored:bool)->Value {
    let original=if name=="k_A_followup"{"k_A"}else{"k_B"};
    let mut run=Run{e:setup(pack,original),pack,base,name,restored,dedup:1,steps:vec![],checkpoints:vec![]};
    let initial=state(&run.e);let mut stages=vec![];
    run.cmd(0,A::NextPhase);
    if let Some(c)=run.e.open_choice().cloned(){
        assert_eq!(c.spec.prompt,"k_danggu_partner");
        run.answer(vec![c.spec.options[if original=="k_B"{1}else{0}]]);
    }
    stages.push(json!({"label":"after_actual_clique_selection","state":state(&run.e)}));
    for index in 0..2 {
        run.cmd(0,A::NextPhase);run.settle();
        stages.push(json!({"label":format!("after_next_phase_{index}"),"state":state(&run.e)}));
        let prefix=format!("{name}-{}-after-next-phase-{index}",if restored{"restored"}else{"direct"});
        fs::write(base.join(format!("{prefix}.ngsa")),encode_archive(&run.e.save())).unwrap();
        fs::write(base.join(format!("{prefix}-snapshot-debug.txt")),format!("{:#?}",run.e.save())).unwrap();
    }
    let bytes=encode_archive(&run.e.save());
    fs::write(base.join(format!("{name}-{}-final.ngsa",if restored{"restored"}else{"direct"})),&bytes).unwrap();
    fs::write(base.join(format!("{name}-{}-final-snapshot-debug.txt",if restored{"restored"}else{"direct"})),format!("{:#?}",run.e.save())).unwrap();
    json!({"case":name,"mode":if restored{"restore_before_each_answer"}else{"direct"},"initial":initial,"stages":stages,"commands":run.steps,"pending_archive_checks":run.checkpoints,
        "final":state(&run.e),"final_archive_hex":bytes.iter().map(|b|format!("{b:02x}")).collect::<String>()})
}

fn main(){
    let base=PathBuf::from(std::env::var("L10_EVIDENCE").unwrap());let repo=PathBuf::from(std::env::var("L10_REPO").unwrap());
    let input=fs::read_to_string(repo.join("packages/extensions/epic-cards-upstream-liuli/assets/cards.json")).unwrap();
    let mut pack=content_native_loader::bridge_pack(&input).unwrap().pack;
    let bridge_counts=json!({"host_skills":pack.host_skills.len(),"vm_skills":pack.vm_skills.len(),"generals":pack.generals.len(),"cards":pack.cards.len()});
    content_native_loader::attach_formal_package_upstream_rules("upstream-liuli","epic-cards",&mut pack,&[]);
    fs::write(base.join("formal-registration.json"),serde_json::to_vec_pretty(&json!({"route":"bridge_pack then attach_formal_package_upstream_rules(upstream-liuli,epic-cards,prior_packs=[]) without injected entities",
        "bridge_counts":bridge_counts,"host_skills":pack.host_skills.iter().map(|s|json!({"name":s.name,"behavior":format!("{:?}",s.behavior)})).collect::<Vec<_>>(),
        "vm_skills":pack.vm_skills.iter().map(|s|json!({"name":s.name,"module":s.module.name,"trigger":format!("{:?}",s.trigger),"filter":format!("{:?}",s.filter)})).collect::<Vec<_>>(),
        "generals":pack.generals.iter().map(|g|json!({"name":g.name,"skills":g.skills,"max_hp":g.max_hp})).collect::<Vec<_>>(),
        "cards":pack.cards.len()})).unwrap()).unwrap();
    fs::write(base.join("native-pack-input.json"),input).unwrap();
    let inspect=Engine::new_with_extensions(202611140010,4,&[&pack]).unwrap();
    let companion_names=["scstaoluan","scschiyan","scszimou","scspicai","scsyaozhuo","scsxiaolu","scskuiji","scschihe","scsniqu","scsanruo"];
    fs::write(base.join("formal-catalog.json"),serde_json::to_vec_pretty(&json!({"catalog_skills":inspect.catalog().skills.iter().enumerate().map(|(id,s)|json!({"id":id,"name":s.name})).collect::<Vec<_>>(),
        "companion_lookup":companion_names.iter().map(|s|json!({"name":s,"id":inspect.catalog().skill_by_name(s)})).collect::<Vec<_>>(),"catalog_card_count":inspect.catalog().cards.len()})).unwrap()).unwrap();
    fs::write(base.join("native-input-metadata.json"),serde_json::to_vec_pretty(&json!({"seed":202611140010_u64,"seats":4,
        "pack_name":pack.def.name,"pack_version":pack.def.version,"host_skills":pack.host_skills.len(),"vm_skills":pack.vm_skills.len(),
        "content_lock":engine_native::content::extension::content_lock_id(&[&pack]),"snapshot_bound_ruleset_version":inspect.save().ruleset_version})).unwrap()).unwrap();
    if std::env::var_os("L10_FOLLOWUP_ONLY").is_some() {
        let mut results=vec![];
        for name in ["k_A_followup","k_B_followup"] {
            let direct=run_formal_followup(&pack,&base,name,false);let restored=run_formal_followup(&pack,&base,name,true);
            results.push(json!({"case":name,"direct":direct,"restored":restored}));
        }
        fs::write(base.join("formal-followup-results.json"),serde_json::to_vec_pretty(&results).unwrap()).unwrap();
        println!("{}",json!({"formal_followup_cases":results.len()}));return;
    }
    let cases=["g_sha_ally","g_sha_question_fake","g_true_question_discard","g_true_question_losehp","g_tao","g_cancel_card","g_cancel_name","g_cancel_target","g_no_hand","g_no_skill","g_response",
        "r_basic","r_trick","r_equip","r_move_A","r_move_B","r_cancel_choice","r_cancel_type","r_no_skill","r_other_target","r_double",
        "k_A","k_B","k_cancel","k_after_initial_no_trigger","k_no_skill"];
    let mut results=vec![];
    for name in cases{
        let direct=run_case(&pack,&base,name,false);let restored=run_case(&pack,&base,name,true);
        let row=json!({"case":name,"final_ngsa_bytes_equal":direct["final_archive_hex"]==restored["final_archive_hex"],"direct":direct,"restored":restored});
        fs::write(base.join(format!("native-{name}.json")),serde_json::to_vec_pretty(&row).unwrap()).unwrap();
        println!("{}",json!({"case":name,"pending":row["direct"]["pending_archive_checks"].as_array().unwrap().len(),"final_choice":row["direct"]["final"]["choice"]}));results.push(row);
    }
    fs::write(base.join("native-results.json"),serde_json::to_vec_pretty(&results).unwrap()).unwrap();
}
