//! Content registry operations for `new` and related helpers.

use super::*;

impl ContentRegistry {
    /// Empty registry.
    pub fn new() -> Self {
        Self {
            modules: BTreeMap::new(),
            package_skills: BTreeMap::new(),
            skill_def_by_name: BTreeMap::new(),
            package_characters: BTreeMap::new(),
            package_cards: BTreeMap::new(),
            package_triggers: Vec::new(),
            package_modifiers: Vec::new(),
            package_actives: Vec::new(),
            package_view_as: Vec::new(),
            track_turn_card_history: false,
            track_turn_card_name_history: false,
            track_play_card_history: false,
            track_per_player_turn_card_count: false,
            track_per_player_play_phase_card_count: false,
            track_play_phase_card_suit_history: false,
            track_discard_pile_suit_history: false,
            track_per_player_turn_card_action_count: false,
            track_lifetime_card_action_count: false,
            track_per_player_turn_sha_use_count: false,
            track_turn_use_response_discard_count: false,
            track_discard_phase_discard_count: false,
            track_discard_phase_lost_cards: false,
            track_turn_discarded_cards: false,
            track_turn_haoyi_cards: false,
            track_ddd_chengjing_history: false,
            track_game_card_history: false,
            track_turn_red_card_target_history: false,
            track_turn_card_target_counts: false,
            track_turn_hand_card_suit_history: false,
            track_turn_used_card_suit_history: false,
            track_turn_basic_card_variant_history: false,
            track_play_card_category_history: false,
            track_play_card_category_history_all_users: false,
            track_play_phase_card_gain_history: false,
            track_owner_card_category_history: false,
            track_damage_taken_count: false,
            track_damaged_by_player_bitset: false,
            track_turn_damage_pair_counts: false,
            track_entered_dying_history: false,
            track_play_card_category_duplicates: false,
            track_sha_use_refund_provenance: false,
            track_opening_extra_play_lifecycle: false,
            track_phase_target_hp_history: false,
            track_turn_hp_change_history: false,
            track_turn_card_gain_count: false,
            track_turn_card_loss_history: false,
            track_card_target_history: false,
            track_basic_use_history: false,
            track_used_type_groups: false,
            packages: Vec::new(),
            next_trigger_id: PACKAGE_TRIGGER_ID_BASE,
            next_modifier_id: PACKAGE_MODIFIER_ID_BASE,
            combined_logic_hash: [0u8; 32],
        }
    }

    /// Loaded package count.
    pub fn package_count(&self) -> usize {
        self.packages.len()
    }

    /// Module lookup.
    pub fn module(&self, name: &str) -> Option<&VerifiedModule> {
        self.modules.get(name).or_else(|| {
            let mut matches = self.modules.values().filter(|module| module.name == name);
            let first = matches.next();
            if matches.next().is_none() {
                first
            } else {
                None
            }
        })
    }

    /// Package metas.
    pub fn packages(&self) -> &[LoadedPackageMeta] {
        &self.packages
    }

    pub(crate) fn validate_snapshot_modules(&self) -> Result<(), String> {
        use rule_vm::bytecode::effect_tag;

        for (key, module) in &self.modules {
            content_schema::validate_vm_module_hash(module)?;
            let old = rule_vm::module_uses_effect_tag(module, effect_tag::GAIN_OTHER_SKILL_PILE_CARDS)
                .map_err(|e| e.to_string())?;
            let new = rule_vm::module_uses_effect_tag(
                module,
                effect_tag::CHOOSE_OTHER_TARGETS_BY_SKIPPED_PHASE_COUNT_DAMAGE_DRAW,
            )
            .map_err(|e| e.to_string())?;
            if !old && !new {
                continue;
            }
            let abi = self.packages.iter().find_map(|package| {
                (package_module_key(&package.namespace, &package.package_id, &module.name) == *key)
                    .then_some(package.rule_vm_abi_version)
                    .flatten()
            }).ok_or_else(|| format!("module {key} has no recorded VM ABI"))?;
            if (old && (754..=757).contains(&abi)) || (new && abi < 758) {
                return Err(format!("module {key} has incompatible effect tag for ABI {abi}"));
            }
        }
        Ok(())
    }

    /// Skill metadata, if declared by a loaded package.
    pub fn skill_metadata(&self, skill_def: SkillDefId) -> Option<&SkillMetadata> {
        self.package_skills.get(&skill_def.raw())
    }

    /// Resolve a published skill name to its definition id (Phase 3 AddSkillByName).
    pub fn skill_def_by_name(&self, name: &str) -> Option<SkillDefId> {
        self.skill_def_by_name
            .get(name)
            .map(|def| SkillDefId::new(*def))
    }

    /// All loaded skill metadata in stable definition-id order.
    pub fn skills(&self) -> impl Iterator<Item = &SkillMetadata> {
        self.package_skills.values()
    }

    /// Whether any loaded package defines the given modifier target.
    pub fn has_modifier_target(&self, target: crate::query::ModifierTarget) -> bool {
        self.package_modifiers
            .iter()
            .any(|modifier| modifier.target == target)
    }

    /// Loaded immutable card metadata.
    pub fn card_metadata(&self, card_def: engine_api::CardDefId) -> Option<&PackageCardEntry> {
        self.package_cards.get(&card_def.raw())
    }

    /// Character definition declared by a loaded package.
    pub fn character(&self, character_def: u32) -> Option<&PackageCharacterEntry> {
        self.package_characters.get(&character_def)
    }

    /// All loaded character definitions in stable id order.
    pub fn characters(&self) -> impl Iterator<Item = &PackageCharacterEntry> {
        self.package_characters.values()
    }

    /// Resolve trigger definitions for a skill (package overrides bootstrap).
    pub fn defs_for_skill(&self, skill_def: SkillDefId) -> Vec<TriggerDefinition> {
        let pkg: Vec<_> = self
            .package_triggers
            .iter()
            .filter(|d| d.skill_def == skill_def)
            .cloned()
            .collect();
        if !pkg.is_empty() {
            return pkg;
        }
        bootstrap_trigger_defs()
            .into_iter()
            .filter(|d| d.skill_def == skill_def)
            .collect()
    }

    /// Find a trigger def by id (package then bootstrap).
    pub fn find_def(&self, id: TriggerDefId) -> Option<TriggerDefinition> {
        self.package_triggers
            .iter()
            .find(|d| d.id == id)
            .cloned()
            .or_else(|| bootstrap_trigger_defs().into_iter().find(|d| d.id == id))
    }

    /// Whether loaded content needs the current player's turn-wide card-use history.
    pub fn tracks_turn_card_history(&self) -> bool {
        self.track_turn_card_history
    }

    /// Whether loaded content compares consecutive logical card names this turn.
    pub fn tracks_turn_card_name_history(&self) -> bool {
        self.track_turn_card_name_history
    }

    /// Whether loaded content needs card-use history scoped to the current play phase.
    pub fn tracks_play_card_history(&self) -> bool {
        self.track_play_card_history
    }

    /// Whether loaded content needs a separate turn card-use count for each player.
    pub fn tracks_per_player_turn_card_count(&self) -> bool {
        self.track_per_player_turn_card_count
    }

    /// Whether loaded content needs a card-use count scoped to each play phase.
    pub fn tracks_per_player_play_phase_card_count(&self) -> bool {
        self.track_per_player_play_phase_card_count
    }

    /// Whether loaded content needs effective used-card suits for the current Play phase.
    pub fn tracks_play_phase_card_suit_history(&self) -> bool {
        self.track_play_phase_card_suit_history
    }

    /// Whether loaded content needs physical suits entering the discard pile this turn.
    pub fn tracks_discard_pile_suit_history(&self) -> bool {
        self.track_discard_pile_suit_history
    }

    /// Whether loaded content needs a turn used-or-responded card count for each player.
    pub fn tracks_per_player_turn_card_action_count(&self) -> bool {
        self.track_per_player_turn_card_action_count
    }

    /// Whether loaded content needs match-lifetime used-or-responded card counts per player.
    pub fn tracks_lifetime_card_action_count(&self) -> bool {
        self.track_lifetime_card_action_count
    }

    /// Whether loaded content needs a turn logical-Sha use count for each player.
    pub fn tracks_per_player_turn_sha_use_count(&self) -> bool {
        self.track_per_player_turn_sha_use_count
    }

    /// Whether loaded content needs the turn-wide physical-card discard count caused by card
    /// use and responses.
    pub fn tracks_turn_use_response_discard_count(&self) -> bool {
        self.track_turn_use_response_discard_count
    }

    /// Whether loaded content needs the current discard-phase discard count.
    pub fn tracks_discard_phase_discard_count(&self) -> bool {
        self.track_discard_phase_discard_count
    }

    /// Whether loaded content opts into discard-phase lost-card slice tracking (ABI 513).
    pub fn tracks_discard_phase_lost_cards(&self) -> bool {
        self.track_discard_phase_lost_cards
    }

    /// Whether loaded content needs all discarded physical cards for the current turn.
    pub fn tracks_turn_discarded_cards(&self) -> bool {
        self.track_turn_discarded_cards
    }

    /// Whether loaded content needs 豪义 turn-card tracking (ABI 601).
    pub fn tracks_turn_haoyi_cards(&self) -> bool {
        self.track_turn_haoyi_cards
    }

    /// Whether loaded content needs 管宁「承经」ordered use history.
    pub fn tracks_ddd_chengjing_history(&self) -> bool {
        self.track_ddd_chengjing_history
    }

    /// Whether loaded content needs each player's previous card across turn boundaries.
    pub fn tracks_game_card_history(&self) -> bool {
        self.track_game_card_history
    }

    /// Whether loaded content needs per-target distinct-red-card history for the current turn.
    pub fn tracks_turn_red_card_target_history(&self) -> bool {
        self.track_turn_red_card_target_history
    }

    /// Whether loaded content needs per-user/per-target card-use counts for the current turn.
    pub fn tracks_turn_card_target_counts(&self) -> bool {
        self.track_turn_card_target_counts
    }

    /// Whether loaded content needs distinct effective suits used from physical hand cards.
    pub fn tracks_turn_hand_card_suit_history(&self) -> bool {
        self.track_turn_hand_card_suit_history
    }

    /// Whether loaded content needs distinct effective suits among all cards used this turn.
    pub fn tracks_turn_used_card_suit_history(&self) -> bool {
        self.track_turn_used_card_suit_history
    }

    /// Whether loaded content needs distinct logical basic-card variants used this turn.
    pub fn tracks_turn_basic_card_variant_history(&self) -> bool {
        self.track_turn_basic_card_variant_history
    }

    /// Whether loaded content needs distinct logical categories used this play phase.
    pub fn tracks_play_card_category_history(&self) -> bool {
        self.track_play_card_category_history
    }

    /// Whether loaded content needs category history for every user in the current Play phase.
    pub fn tracks_play_card_category_history_all_users(&self) -> bool {
        self.track_play_card_category_history_all_users
    }

    /// Whether loaded content needs per-player Play-phase gain history.
    pub fn tracks_play_phase_card_gain_history(&self) -> bool {
        self.track_play_phase_card_gain_history
    }

    /// Whether loaded content needs categories each player used or discarded this turn.
    pub fn tracks_owner_card_category_history(&self) -> bool {
        self.track_owner_card_category_history
    }

    /// Whether loaded content needs per-player damage-taken counts this turn.
    pub fn tracks_damage_taken_count(&self) -> bool {
        self.track_damage_taken_count
    }

    /// Whether loaded content needs the per-source damaged-players bitset
    /// (ADR 0644 P3 霸政 opinion alignment).
    pub fn tracks_damaged_by_player_bitset(&self) -> bool {
        self.track_damaged_by_player_bitset
    }

    /// Whether loaded content needs source-target damage counts for the current turn.
    pub fn tracks_turn_damage_pair_counts(&self) -> bool {
        self.track_turn_damage_pair_counts
    }

    /// Whether loaded content needs any-player entered-dying history for the current turn.
    pub fn tracks_entered_dying_history(&self) -> bool {
        self.track_entered_dying_history
    }

    /// Whether loaded content needs to distinguish repeated Play-phase card categories.
    pub fn tracks_play_card_category_duplicates(&self) -> bool {
        self.track_play_card_category_duplicates
    }

    /// Whether saved proactive Sha continuations must retain count-refund provenance.
    pub fn tracks_sha_use_refund_provenance(&self) -> bool {
        self.track_sha_use_refund_provenance
    }

    /// Whether turn starts use the recoverable opening-phase queue.
    pub fn tracks_opening_extra_play_lifecycle(&self) -> bool {
        self.track_opening_extra_play_lifecycle
    }

    /// Whether loaded content needs phase-scoped target and actual-HP-change history.
    pub fn tracks_phase_target_hp_history(&self) -> bool {
        self.track_phase_target_hp_history
    }

    /// Whether loaded content needs per-player turn-scoped actual HP-change counts.
    pub fn tracks_turn_hp_change_history(&self) -> bool {
        self.track_turn_hp_change_history
    }

    /// Whether loaded content needs per-player turn card-gain counts.
    pub fn tracks_turn_card_gain_count(&self) -> bool {
        self.track_turn_card_gain_count
    }

    /// Whether loaded content needs per-player turn card-loss provenance.
    pub fn tracks_turn_card_loss_history(&self) -> bool {
        self.track_turn_card_loss_history
    }

    /// Whether loaded content needs per-player turn card-target counts.
    pub fn tracks_card_target_history(&self) -> bool {
        self.track_card_target_history
    }

    /// Whether loaded content needs lifetime basic-card use counts.
    pub fn tracks_basic_use_history(&self) -> bool {
        self.track_basic_use_history
    }

    /// Whether loaded content needs lifetime used-type bitmap.
    pub fn tracks_used_type_groups(&self) -> bool {
        self.track_used_type_groups
    }

    /// Resolve modifier definitions for a skill (package overrides bootstrap).
    pub fn modifier_defs_for_skill(&self, skill_def: SkillDefId) -> Vec<ModifierDefinition> {
        let pkg: Vec<_> = self
            .package_modifiers
            .iter()
            .filter(|d| d.skill_def == skill_def)
            .cloned()
            .collect();
        if !pkg.is_empty() {
            return pkg;
        }
        bootstrap_modifier_defs()
            .into_iter()
            .filter(|d| d.skill_def == skill_def)
            .collect()
    }

    /// Active skill defs for a skill definition (package only).
    pub fn active_defs_for_skill(&self, skill_def: SkillDefId) -> Vec<ActiveSkillDef> {
        self.package_actives
            .iter()
            .filter(|d| d.skill_def == skill_def)
            .cloned()
            .collect()
    }

    /// First active def for skill (if any).
    pub fn first_active_for_skill(&self, skill_def: SkillDefId) -> Option<ActiveSkillDef> {
        self.package_actives
            .iter()
            .find(|d| d.skill_def == skill_def)
            .cloned()
    }

    /// View-as defs for a skill definition.
    pub fn view_as_defs_for_skill(&self, skill_def: SkillDefId) -> Vec<ViewAsDef> {
        self.package_view_as
            .iter()
            .filter(|d| d.skill_def == skill_def)
            .cloned()
            .collect()
    }

    /// All view-as defs.
    pub fn all_view_as(&self) -> &[ViewAsDef] {
        &self.package_view_as
    }

    /// Load / merge a content package (idempotent if same logic_hash already present).
    pub fn load_package(&mut self, pkg: &ContentPackage) -> Result<(), ContentLoadError> {
        // ponytail: package loads are cold-path; stage deltas instead if clone cost is ever measured.
        let mut staged = self.clone();
        staged.load_package_inner(pkg)?;
        *self = staged;
        Ok(())
    }
}
