//! Owner-visible ask identity, not a root certificate or permission to resume.
use super::*;
impl Engine {
    pub(super) fn choice_source_skill(
        &self,
        open: &crate::choice::OpenChoice,
    ) -> Option<&'static str> {
        let resolution = &self.state.resolution;
        let consumer = crate::resolution::ready_park_index(resolution)?;
        let mut producer = None;
        for (index, park) in resolution.parks.iter().enumerate() {
            let snapshot = park.machine.snapshot();
            if let Some(ask) = crate::snap::active_ask(&snapshot, &self.state.core, &self.catalog) {
                if producer.is_some()
                    || ask.observer != open.observer
                    || ask.prompt != open.spec.prompt
                    || ask.kind != open.spec.kind
                    || ask.options != open.spec.options
                    || ask.min != open.spec.min
                    || ask.max != open.spec.max
                    || ask.allow_cancel != open.spec.allow_cancel
                {
                    return None;
                }
                let source =
                    if matches!(snapshot, crate::snap::MachineSnapshot::OptionalGate { .. }) {
                        ask.source_skill
                    } else {
                        None
                    };
                producer = Some((index, source));
            }
        }
        let (index, source) = producer?;
        if index != consumer {
            return None;
        }
        let name = source?;
        self.view_for(open.observer).players[open.observer as usize]
            .skills
            .contains(&name)
            .then_some(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ChoiceView, MachineSnapshot, Phase};
    fn fixture() -> Engine {
        let mut engine = Engine::new(2306, 3);
        engine.debug_set_turn(0, Phase::Play, 1);
        for seat in 0..3 {
            engine.debug_clear_hand(seat);
        }
        engine.debug_grant(1, "遗计");
        engine.debug_deal_damage(Some(0), 1, 1, crate::event::DamageNature::Normal);
        engine
    }
    fn source(engine: &Engine) -> Option<&'static str> {
        let Some(ChoiceView::Own { source_skill, .. }) = engine.choice_view_for(Some(1)) else {
            panic!("own ask")
        };
        source_skill
    }
    fn restored(snapshot: crate::StateSnapshot) -> Engine {
        Engine::restore(
            &crate::archive::decode_archive(&crate::archive::encode_archive(&snapshot)).unwrap(),
        )
        .unwrap()
    }
    #[test]
    fn source_requires_actual_ready_consumer_not_just_a_unique_active_ask() {
        let engine = fixture();
        assert_eq!(source(&engine), Some("遗计"));
        let mut saved = engine.save();
        let mut earlier = saved.resolution.parks.last().unwrap().clone();
        earlier.machine = MachineSnapshot::JiJi {
            owner: 1,
            stage: crate::snap::JiJiStageSnap::Draw,
            target: None,
            given: 0,
        };
        saved.resolution.parks.insert(0, earlier);
        assert_eq!(
            source(&restored(saved)),
            None,
            "restore remains accepted but the first consumer is not the ask producer"
        );
        let mut saved = engine.save();
        saved.resolution.parks.last_mut().unwrap().resume_depth = 0;
        assert_eq!(source(&restored(saved)), None);
    }
    #[test]
    fn source_wrong_shape_multiple_asks_and_unowned_defs_stay_unknown() {
        let mut engine = fixture();
        engine
            .state
            .resolution
            .open_choice
            .as_mut()
            .unwrap()
            .spec
            .max = 2;
        assert_eq!(source(&engine), None);
        let engine = fixture();
        for name in ["刚烈", "奸雄"] {
            let mut saved = engine.save();
            let MachineSnapshot::OptionalGate { def, .. } =
                &mut saved.resolution.parks.last_mut().unwrap().machine
            else {
                panic!("gate")
            };
            *def = engine.catalog.skill_by_name(name).unwrap();
            // C binds the parked trigger to its window; this forged snapshot now rejects.
            assert!(Engine::restore(&saved).is_err());
            let index = crate::resolution::ready_park_index(&engine.state.resolution).unwrap();
            let mut probe = fixture();
            probe.state.resolution.parks[index].machine = saved.resolution.parks[index]
                .machine
                .restore(&probe.catalog)
                .unwrap();
            assert_eq!(source(&probe), None);
        }
        let mut engine = fixture();
        let index = crate::resolution::ready_park_index(&engine.state.resolution).unwrap();
        let p = &engine.state.resolution.parks[index];
        let duplicate = crate::resolution::Park {
            resume_depth: p.resume_depth,
            slot: p.slot,
            machine: p.machine.fork(),
            awaiting_answer: p.awaiting_answer,
        };
        engine.state.resolution.parks.push(duplicate);
        assert_eq!(source(&engine), None);
    }
    #[test]
    fn source_uses_only_the_actual_direct_inner_ask() {
        let engine = fixture();
        let mut saved = engine.save();
        let index = crate::resolution::ready_park_index(&engine.state.resolution).unwrap();
        let inner = saved.resolution.parks[index].machine.clone();
        let MachineSnapshot::OptionalGate { event, .. } = &inner else {
            panic!("gate")
        };
        saved.resolution.parks[index].machine = MachineSnapshot::OptionalGate {
            owner: 1,
            def: engine.catalog.skill_by_name("刚烈").unwrap(),
            event: event.clone(),
            asked: true,
            inner: Some(Box::new(inner)),
        };
        assert_eq!(
            source(&restored(saved)),
            Some("遗计"),
            "outer unowned name cannot replace inner source"
        );
        let mut saved = engine.save();
        let MachineSnapshot::OptionalGate { inner, .. } =
            &mut saved.resolution.parks[index].machine
        else {
            panic!("gate")
        };
        *inner = Some(Box::new(MachineSnapshot::JiJi {
            owner: 1,
            stage: crate::snap::JiJiStageSnap::GiveCard,
            target: None,
            given: 0,
        }));
        let mut engine = engine;
        engine.state.resolution.parks[index].machine = saved.resolution.parks[index]
            .machine
            .restore(&engine.catalog)
            .unwrap();
        assert_eq!(
            source(&engine),
            None,
            "inner different ask cannot borrow the outer name"
        );
    }
    #[test]
    fn source_catalog_alias_is_not_resolved_by_first_match() {
        use crate::catalog::{
            Catalog, QueryCtx, SkillBehavior, SkillDef, SkillLimits, SkillMachine,
        };
        struct Alias;
        impl SkillBehavior for Alias {
            fn filter(&self, _: &QueryCtx<'_>, _: u8, _: &crate::EventData) -> bool {
                false
            }
            fn start(&self, _: u8, _: &crate::EventData) -> Box<dyn SkillMachine> {
                panic!("not executed")
            }
        }
        let mut engine = fixture();
        let mut catalog = crate::content::standard::standard_catalog();
        catalog.skills.push(SkillDef {
            name: "遗计",
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(Alias),
        });
        engine.catalog = std::sync::Arc::new(Catalog::build(catalog.cards, catalog.skills));
        assert_eq!(source(&engine), None);
    }
}
