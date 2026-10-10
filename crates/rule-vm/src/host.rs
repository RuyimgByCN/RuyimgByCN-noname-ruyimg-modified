//! Host interface — VM never touches GameState directly.

use rule_api::{CapabilitySet, EffectRequest, HostTrap, QueryRequest, QueryResponse, RelationTag};

/// Context the host provides for skill execution.
#[derive(Debug, Clone, Copy, Default)]
pub struct HostContext {
    /// Current skill instance raw id.
    pub skill_id: u32,
    /// Skill owner player raw id.
    pub self_id: u32,
    /// Event primary player raw id.
    pub event_player: u32,
    /// Event source player raw id.
    pub event_source: u32,
    /// Event card raw id (e.g. the used card for `CardUsed`); 0 if none.
    pub event_card: u32,
    /// Damage amount bound to a damage event; 0 for other events.
    pub event_amount: u32,
}

/// Calls the VM may make into the host.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostCall {
    /// Emit effect.
    Effect(EffectRequest),
    /// Run query.
    Query(QueryRequest),
}

/// Host implemented by engine-core (or CollectingHost in tests).
pub trait VmHost {
    /// Capability set granted to this module.
    fn capabilities(&self) -> &CapabilitySet;

    /// Skill / event context.
    fn context(&self) -> HostContext;

    /// Card inputs selected for an active skill invocation.
    fn selected_cards(&self) -> &[u32] {
        &[]
    }

    /// Player targets selected for an active skill invocation.
    fn selected_targets(&self) -> &[u32] {
        &[]
    }

    /// Physical cards captured from the triggering domain event.
    fn event_cards(&self) -> &[u32] {
        &[]
    }

    /// Player targets captured from the triggering domain event.
    fn event_targets(&self) -> &[u32] {
        &[]
    }

    /// Emit a typed effect (queued by engine; not applied inside VM).
    fn emit_effect(&mut self, req: EffectRequest) -> Result<(), HostTrap>;

    /// Run a typed query (read-only).
    fn run_query(&mut self, req: QueryRequest) -> Result<QueryResponse, HostTrap>;
}

/// Test / offline host that records effects and answers queries with stubs.
#[derive(Debug, Clone)]
pub struct CollectingHost {
    /// Caps.
    pub caps: CapabilitySet,
    /// Context.
    pub ctx: HostContext,
    /// Recorded effects.
    pub effects: Vec<EffectRequest>,
    /// Active-skill card inputs exposed to selection-aware effects.
    pub selected_cards: Vec<u32>,
    /// Active-skill player targets exposed to target-aware effects.
    pub selected_targets: Vec<u32>,
    /// Physical cards captured from a triggering domain event.
    pub event_cards: Vec<u32>,
    /// Player targets captured from a triggering domain event (Phase 3: event_target_count).
    pub event_targets: Vec<u32>,
    /// Optional query handler: default distance=1, range=1, hand=4.
    pub query_stub_int: i32,
    /// Optional stub answer for `Relation` queries; when set, overrides the int stub.
    pub query_stub_relation: Option<RelationTag>,
}

impl Default for CollectingHost {
    fn default() -> Self {
        Self {
            caps: CapabilitySet::standard_skill(),
            ctx: HostContext::default(),
            effects: Vec::new(),
            selected_cards: Vec::new(),
            selected_targets: Vec::new(),
            event_cards: Vec::new(),
            event_targets: Vec::new(),
            query_stub_int: 1,
            query_stub_relation: None,
        }
    }
}

impl VmHost for CollectingHost {
    fn capabilities(&self) -> &CapabilitySet {
        &self.caps
    }

    fn context(&self) -> HostContext {
        self.ctx
    }

    fn selected_cards(&self) -> &[u32] {
        &self.selected_cards
    }

    fn selected_targets(&self) -> &[u32] {
        &self.selected_targets
    }

    fn event_cards(&self) -> &[u32] {
        &self.event_cards
    }

    fn event_targets(&self) -> &[u32] {
        &self.event_targets
    }

    fn emit_effect(&mut self, req: EffectRequest) -> Result<(), HostTrap> {
        if !self.caps.emit_effects {
            return Err(HostTrap::CapabilityDenied {
                name: "emit_effects".into(),
            });
        }
        self.effects.push(req);
        Ok(())
    }

    fn run_query(&mut self, req: QueryRequest) -> Result<QueryResponse, HostTrap> {
        if !self.caps.run_queries {
            return Err(HostTrap::CapabilityDenied {
                name: "run_queries".into(),
            });
        }
        if let (QueryRequest::Relation { .. }, Some(tag)) = (&req, self.query_stub_relation) {
            return Ok(QueryResponse::Relation(tag));
        }
        Ok(QueryResponse::Int(self.query_stub_int))
    }
}
