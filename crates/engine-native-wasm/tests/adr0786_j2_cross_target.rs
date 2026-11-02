//! J2 formal author packages: native oracle and byte-complete wasm32 replay vectors.
#[path = "adr0786_j2_cross_target/cases.rs"]
mod cases;
use engine_native::{ChoiceOption as O, CommandOutcome, Engine, GameAction, GameCommand};
use engine_native_wasm::test_support::{engine_from_archive, engine_from_packs, hex_encode};
use serde_json::{json, Value};

fn package(source: &str, name: &str) -> String {
    let mut package = content_compiler::compile_source_named(name, source).unwrap();
    let declarations = content_native_loader::bridge_vm_skill_decls(&package).unwrap();
    let generals = [("j2-cross-author",false),("j2-cross-watcher",true)].iter().map(|(name, watcher)| json!({"name":name,"hp":4,"gender":"male","faction":"qun","skills":declarations.decls.iter().filter(|d|d.name.starts_with("j2_watcher")==*watcher).map(|d|json!({"name":d.name})).collect::<Vec<_>>()})).collect::<Vec<_>>();
    let characters = json!({"pack":{"name":name,"version":1},"generals":generals});
    let character_bytes = serde_json::to_vec(&characters).unwrap();
    let cards=(name=="danshou-equipment-theft-not-gift").then(||json!({"pack":{"name":name,"version":1,"description":"J2 theft equipment","card_prefix":"j2-theft-"},"cards":[{"name":"gear","kind":"weapon","range":2,"suit":"spade","rank":7,"equip_skills":["j2_theft_gear"]}],"skills":[{"name":"j2_theft_gear","mapping":"host:sha_damage_force_discard"}]}));
    let mut entries = vec![
        json!({"asset_id":"j2:characters","source_hash":blake3::hash(&character_bytes).as_bytes(),"kind":"Data","image_variants":[],"audio_variants":[],"priority":"Lazy","depends_on":[],"platforms":[]}),
    ];
    package
        .asset_paths
        .insert("j2:characters".into(), "assets/character.json".into());
    if let Some(cards) = &cards {
        entries.push(json!({"asset_id":"j2:cards","source_hash":blake3::hash(&serde_json::to_vec(cards).unwrap()).as_bytes(),"kind":"Data","image_variants":[],"audio_variants":[],"priority":"Lazy","depends_on":[],"platforms":[]}));
        package
            .asset_paths
            .insert("j2:cards".into(), "assets/cards.json".into());
    }
    package.asset_manifest=Some(serde_json::from_value(json!({"schema_version":1,"package_id":name,"total_compressed_size":0,"total_decoded_size":0,"entries":entries})).unwrap());
    package.manifest.asset_hash = package.compute_asset_hash();
    package.manifest.logic_hash = package.compute_logic_hash();
    package.validate().unwrap();
    let mut payload = serde_json::to_value(package).unwrap();
    payload["characters"] = characters;
    if let Some(cards) = cards {
        payload["cards"] = cards;
    }
    json!([payload]).to_string()
}

fn apply(engine: &mut Engine, command: &Value) -> CommandOutcome {
    let actor = command["actor"].as_u64().unwrap() as u8;
    let action = match command["type"].as_str().unwrap() {
        "nextPhase" => GameAction::NextPhase,
        "activateSkill" => GameAction::ActivateSkill {
            skill: command["skill"].as_u64().unwrap() as u16,
        },
        "playCard" => GameAction::PlayCard {
            card: command["card"].as_u64().unwrap() as u32,
            targets: command["targets"]
                .as_array()
                .unwrap()
                .iter()
                .map(|n| n.as_u64().unwrap() as u8)
                .collect(),
        },
        "answer" => {
            let options = engine.open_choice().unwrap().spec.options.clone();
            GameAction::Answer {
                choice_id: command["choice_id"].as_u64().unwrap(),
                selection: command["selections"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|n| options[n.as_u64().unwrap() as usize])
                    .collect(),
            }
        }
        other => panic!("unknown command {other}"),
    };
    let revision = engine.revision();
    engine.submit(GameCommand {
        actor,
        expected_revision: revision,
        dedup_id: revision.wrapping_mul(2).max(1),
        action,
    })
}

fn outcome(out: &CommandOutcome) -> &'static str {
    match out {
        CommandOutcome::Accepted { .. } => "accepted",
        CommandOutcome::Pending { .. } => "pending",
        CommandOutcome::Rejected(_) => "rejected",
        CommandOutcome::Suspended => "suspended",
    }
}

fn choice(engine: &Engine) -> Value {
    let Some(c) = engine.open_choice() else {
        return Value::Null;
    };
    let options: Vec<_> = c
        .spec
        .options
        .iter()
        .map(|o| match o {
            O::Card(c) => json!({"card":c}),
            O::Player(p) => json!({"player":p}),
            O::Tag(t) => json!({"tag":t}),
            O::Skill(def) => json!({"skill":engine.catalog().skill_name(*def)}),
            O::Kind(k) => json!({"kind":k}),
            O::Control(option) => json!(option.label()),
            O::HiddenCard(_) => json!("hand"),
            O::Confirm => json!("confirm"),
            O::Pass => json!("pass"),
        })
        .collect();
    let native_kind = engine_native::view::choice_kind_name(c.spec.kind);
    json!({"id":c.id,"observer":c.observer,"prompt":c.spec.prompt,"native_kind":native_kind,"kind":engine_api::choice_kind_from_native(native_kind).expect("canonical native choice kind"),"min":c.spec.min,"max":c.spec.max,"allowCancel":c.spec.allow_cancel,"options":options})
}

fn rejected_probes(engine: &mut Engine) {
    let Some(c) = engine.open_choice().cloned() else {
        return;
    };
    let snapshot = engine.save();
    let log = engine.log().to_vec();
    let dedup = engine.revision().wrapping_mul(2).max(1);
    for (actor, id, selection) in [
        ((c.observer + 1) % 4, c.id, vec![c.spec.options[0]]),
        (c.observer, c.id + 1, vec![c.spec.options[0]]),
        (c.observer, c.id, vec![O::Tag(255)]),
    ] {
        assert!(matches!(
            engine.submit(GameCommand {
                actor,
                expected_revision: engine.revision(),
                dedup_id: dedup,
                action: GameAction::Answer {
                    choice_id: id,
                    selection
                }
            }),
            CommandOutcome::Rejected(_)
        ));
        assert_eq!(engine.save(), snapshot);
        assert_eq!(engine.log(), log);
    }
}

struct Trace {
    engine: Engine,
    packs: String,
    initial: String,
    steps: Vec<Value>,
    invalid_archives: Vec<String>,
}
impl Trace {
    fn start(source: &str, name: &str) -> Self {
        let packs = package(source, name);
        let mut engine = engine_from_packs(4, 99107860, &packs).unwrap();
        engine.assign_general_named(0, "j2-cross-author").unwrap();
        Self {
            engine,
            packs,
            initial: String::new(),
            steps: vec![],
            invalid_archives: vec![],
        }
    }
    fn begin(&mut self) {
        self.steps.clear();
        self.initial = hex_encode(&engine_native::archive::encode_archive(&self.engine.save()));
        self.engine = engine_from_archive(&self.initial, &self.packs).unwrap();
    }
    fn advance_to_play(&mut self, seat: u8) {
        for _ in 0..40 {
            if self.engine.turn().current == seat
                && self.engine.turn().phase == engine_native::Phase::Play
            {
                return;
            }
            if self.engine.open_choice().is_some() {
                self.default_answer();
            } else {
                self.step(json!({"type":"nextPhase","actor":self.engine.turn().current}));
            }
        }
        panic!("did not reach Play");
    }
    fn step(&mut self, command: Value) {
        let before_choice = choice(&self.engine);
        let before = hex_encode(&engine_native::archive::encode_archive(&self.engine.save()));
        let mut restored = engine_from_archive(&before, &self.packs).unwrap();
        rejected_probes(&mut self.engine);
        rejected_probes(&mut restored);
        let cursor = self.engine.log().len();
        let out = apply(&mut self.engine, &command);
        let resumed = apply(&mut restored, &command);
        assert_ne!(outcome(&out), "rejected", "{command}: {out:?}");
        assert_eq!(outcome(&out), outcome(&resumed));
        assert_eq!(self.engine.save().core, restored.save().core);
        assert_eq!(self.engine.save().resolution, restored.save().resolution);
        assert_eq!(&self.engine.log()[cursor..], restored.log());
        let codes = self.engine.public_event_codes_from(cursor);
        assert_eq!(codes, restored.public_event_codes_from(0));
        let after = hex_encode(&engine_native::archive::encode_archive(&self.engine.save()));
        if command["type"] == "answer" {
            let snapshot = self.engine.save();
            let log = self.engine.log().to_vec();
            // The accepted command's original dedup key cannot settle the choice a second time.
            let old_key = snapshot.last_dedup;
            assert!(matches!(
                self.engine.submit(GameCommand {
                    actor: command["actor"].as_u64().unwrap() as u8,
                    expected_revision: self.engine.revision(),
                    dedup_id: old_key,
                    action: GameAction::Answer {
                        choice_id: command["choice_id"].as_u64().unwrap(),
                        selection: vec![]
                    }
                }),
                CommandOutcome::Rejected(_)
            ));
            assert_eq!(self.engine.save(), snapshot);
            assert_eq!(self.engine.log(), log);
        }
        self.steps.push(json!({"command":command,"before_choice":before_choice,"outcome":outcome(&out),"choice":choice(&self.engine),"hash":hex_encode(&self.engine.state_hash()),"codes":codes,"archive":after,"resumed_archive":hex_encode(&engine_native::archive::encode_archive(&restored.save()))}));
    }
    fn activate(&mut self, name: &str) {
        let skill = self.engine.catalog().skill_by_name(name).unwrap();
        self.step(json!({"type":"activateSkill","actor":0,"skill":skill}));
    }
    fn answer(&mut self, selection: Vec<O>) {
        let c = self.engine.open_choice().unwrap();
        let indices: Vec<_> = selection
            .iter()
            .map(|o| {
                c.spec
                    .options
                    .iter()
                    .position(|option| option == o)
                    .expect("legal choice")
            })
            .collect();
        self.step(
            json!({"type":"answer","actor":c.observer,"choice_id":c.id,"selections":indices}),
        );
    }
    fn default_answer(&mut self) {
        let c = self.engine.open_choice().unwrap();
        let selected = if c.spec.options.contains(&O::Pass) {
            vec![O::Pass]
        } else if c.spec.prompt == "trigger_confirm" {
            vec![O::Confirm]
        } else if c.spec.allow_cancel {
            vec![]
        } else {
            c.spec.options.iter().take(c.spec.min).copied().collect()
        };
        self.answer(selected);
    }
    fn drain(&mut self) {
        for _ in 0..40 {
            if self.engine.open_choice().is_none() {
                return;
            }
            self.default_answer();
        }
        panic!("unbounded choices");
    }
    fn finish(self, name: &str) -> Value {
        assert!(self.engine.open_choice().is_none());
        json!({"name":name,"packs_json":self.packs,"initial":self.initial,"steps":self.steps,"invalid_archives":self.invalid_archives})
    }
}

#[test]
fn j2_author_primitives_native_and_every_checkpoint_replay_vectors() {
    let scenarios = cases::scenarios();
    assert!(scenarios.len() >= 16);
    let steps: usize = scenarios
        .iter()
        .map(|s| s["steps"].as_array().unwrap().len())
        .sum();
    if let Some(path) = std::env::var_os("J2_CROSS_NATIVE_OUT") {
        std::fs::write(
            path,
            serde_json::to_vec_pretty(&json!({"scenarios":scenarios,"steps":steps})).unwrap(),
        )
        .unwrap();
    }
    eprintln!(
        "J2_NATIVE_MATRIX scenarios={} steps={steps}",
        scenarios.len()
    );
}
