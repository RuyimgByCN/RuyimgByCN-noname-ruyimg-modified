//! 内容包的逻辑哈希与资源哈希（锁定、签名与升级比对的输入）。

use super::*;

impl ContentPackage {
    /// Compute package logic hash from semantic fields only.
    pub fn compute_logic_hash(&self) -> [u8; 32] {
        let mut h = blake3::Hasher::new();
        h.update(b"noname-next/npack-logic/v1");
        h.update(&self.manifest.package_format_version.to_le_bytes());
        h.update(&self.manifest.content_schema_version.to_le_bytes());
        h.update(&self.manifest.rule_vm_abi_version.to_le_bytes());
        h.update(self.manifest.namespace.as_bytes());
        h.update(self.manifest.package_id.as_bytes());
        h.update(self.manifest.version.as_bytes());
        h.update(self.compiler_version.as_bytes());
        if self.has_native_mechanisms() {
            h.update(b"npack-native-mechanisms/1");
            h.update(&self.native_mechanisms_hash());
        }
        if !self.manifest.dependencies.is_empty() {
            h.update(b"dependencies/v1");
            let mut dependencies = self.manifest.dependencies.clone();
            dependencies.sort();
            for dependency in dependencies {
                h.update(dependency.namespace.as_bytes());
                h.update(b"\0");
                h.update(dependency.package_id.as_bytes());
                h.update(b"\0");
                h.update(dependency.version.as_bytes());
                h.update(&dependency.logic_hash);
            }
        }
        for m in &self.modules {
            h.update(&m.logic_hash);
            h.update(m.name.as_bytes());
            h.update(&m.code);
        }
        for skill in &self.skill_index {
            h.update(&skill.skill_def_id.to_le_bytes());
            h.update(skill.name.as_bytes());
            h.update(&[u8::from(skill.locked)]);
            if skill.lord_only {
                h.update(b"lord_only/v1");
            }
            if skill.non_gainable {
                h.update(b"non_gainable/v1");
            }
            if skill.conversion {
                h.update(b"conversion/v1");
            }
        }
        for character in &self.character_index {
            h.update(&character.id.to_le_bytes());
            h.update(character.name_key.as_bytes());
            h.update(&character.max_hp.to_le_bytes());
            h.update(&character.initial_hp.to_le_bytes());
            for skill in &character.skills {
                h.update(&skill.to_le_bytes());
            }
            if !character.sex.is_empty()
                || !character.group.is_empty()
                || character.is_lord
                || !character.tags.is_empty()
            {
                h.update(b"character-metadata/v1");
                h.update(character.sex.as_bytes());
                h.update(b"\0");
                h.update(character.group.as_bytes());
                h.update(&[u8::from(character.is_lord)]);
                let mut tags = character.tags.clone();
                tags.sort();
                for tag in tags {
                    h.update(tag.as_bytes());
                    h.update(b"\0");
                }
            }
            // Additive segment: clan memberships. Hash-neutral when empty so non-clan
            // packages keep their existing logic_hash.
            if !character.clans.is_empty() {
                h.update(b"character-clans/v1");
                let mut clans = character.clans.clone();
                clans.sort();
                for clan in clans {
                    h.update(clan.as_bytes());
                    h.update(b"\0");
                }
            }
        }
        for card in &self.card_index {
            h.update(&card.card_def_id.to_le_bytes());
            h.update(card.name.as_bytes());
            h.update(card.kind.as_bytes());
            if card.name_length != 0 || card.gift {
                h.update(b"card-rule-metadata/v1");
                h.update(&[card.name_length, u8::from(card.gift)]);
            }
            h.update(&[card.weapon_range]);
            for skill in &card.equip_skills {
                h.update(&skill.to_le_bytes());
            }
            h.update(&card.distance_from.to_le_bytes());
            h.update(&card.distance_to.to_le_bytes());
            h.update(&card.on_leave_equip_skill_def.to_le_bytes());
        }
        for (position, t) in self.trigger_index.iter().enumerate() {
            h.update(&t.trigger_def_id.to_le_bytes());
            h.update(&t.skill_def_id.to_le_bytes());
            h.update(t.event.as_bytes());
            h.update(t.filter.as_bytes());
            h.update(&[u8::from(t.optional)]);
            if !t.choice_actor.is_owner() {
                h.update(b"trigger-choice-actor/v1");
                h.update(&[t.choice_actor.tag()]);
            }
            h.update(t.module.as_bytes());
            h.update(&t.entry_pc.to_le_bytes());
            if t.has_nondefault_order() {
                h.update(b"trigger-order/v1");
                h.update(&(position as u64).to_le_bytes());
                h.update(&t.trigger_def_id.to_le_bytes());
                h.update(&t.skill_def_id.to_le_bytes());
                for value in [&t.event, &t.module] {
                    h.update(&(value.len() as u64).to_le_bytes());
                    h.update(value.as_bytes());
                }
                h.update(&t.entry_pc.to_le_bytes());
                h.update(&[t.timing.tag()]);
                h.update(&t.priority.to_le_bytes());
                h.update(&[u8::from(t.order_selectable)]);
            }
        }
        for m in &self.modifier_index {
            h.update(&m.skill_def_id.to_le_bytes());
            h.update(m.target.as_bytes());
            if let Some(selector) = &m.selector {
                h.update(b"selector/v1");
                h.update(selector.as_bytes());
            }
            h.update(m.mode.as_bytes());
            if let Some(source) = &m.value_source {
                h.update(b"value-source/v1");
                h.update(source.as_bytes());
            }
            h.update(&m.value.to_le_bytes());
        }
        for a in &self.active_index {
            h.update(&a.skill_def_id.to_le_bytes());
            h.update(a.window.as_bytes());
            h.update(a.module.as_bytes());
            h.update(&a.entry_pc.to_le_bytes());
            if let Some(selection) = &a.selection {
                h.update(&[1]);
                h.update(selection.zone.as_bytes());
                h.update(&[
                    selection.min,
                    selection.max,
                    u8::from(selection.allow_cancel),
                ]);
                if selection.same_suit {
                    h.update(b"active-selection-same-suit/v1");
                }
                if selection.same_color {
                    h.update(b"active-selection-same-color/v1");
                }
                if selection.distinct_categories {
                    h.update(b"active-selection-distinct-categories/v1");
                }
                if let Some(filter) = &selection.filter {
                    h.update(b"active-selection-filter/v1");
                    h.update(filter.as_bytes());
                }
                if selection.unused_category_by_skill_this_turn {
                    h.update(b"active-selection-unused-category-this-turn/v1");
                }
                if selection.unused_category_by_skill_this_play_phase {
                    h.update(b"active-selection-unused-category-this-play-phase/v1");
                }
                if selection.unused_category_by_owner_history_this_turn {
                    h.update(b"active-selection-unused-category-owner-history-this-turn/v1");
                }
                if let Some(sum) = selection.sum_effective_number_exact {
                    h.update(b"active-selection-sum-effective-number-exact/v1");
                    h.update(&sum.to_le_bytes());
                }
            } else {
                h.update(&[0]);
            }
            if let Some(targets) = &a.targets {
                h.update(b"active-targets/v1");
                h.update(&[targets.min, targets.max]);
                if !targets.allow_self {
                    h.update(b"active-targets-other/v1");
                }
                if targets.lowest_hp {
                    h.update(b"active-targets-lowest-hp/v1");
                }
                if targets.highest_hp {
                    h.update(b"active-targets-highest-hp/v1");
                }
                if targets.unlinked {
                    h.update(b"active-targets-unlinked/v1");
                }
                if targets.linked {
                    h.update(b"active-targets-linked/v1");
                }
                if let Some(mark) = targets.max_by_owner_mark {
                    h.update(b"active-targets-max-by-owner-mark/v1");
                    h.update(&mark.to_le_bytes());
                }
                if targets.max_by_owner_lost_hp {
                    h.update(b"active-targets-max-by-owner-lost-hp/v1");
                }
                if targets.max_by_owner_hp {
                    h.update(b"active-targets-max-by-owner-hp/v1");
                }
                if targets.wounded {
                    h.update(b"active-targets-wounded/v1");
                }
                if targets.male {
                    h.update(b"active-targets-male/v1");
                }
                if targets.hp_at_least_owner {
                    h.update(b"active-targets-hp-at-least-owner/v1");
                }
                if targets.hp_gt_owner {
                    h.update(b"active-targets-hp-gt-owner/v1");
                }
                if targets.hp_lte_owner {
                    h.update(b"active-targets-hp-lte-owner/v1");
                }
                if let Some(limit) = targets.hp_gt {
                    h.update(b"active-targets-hp-gt/v1");
                    h.update(&[limit]);
                }
                if let Some(limit) = targets.max_hp_gt {
                    h.update(b"active-targets-max-hp-gt/v1");
                    h.update(&[limit]);
                }
                if targets.can_compare {
                    h.update(b"active-targets-can-compare/v1");
                }
                if let Some(kind) = &targets.can_use_on_owner {
                    h.update(b"active-targets-can-use-on-owner/v1");
                    h.update(kind.as_bytes());
                }
                if targets.second_can_use_juedou_on_first {
                    h.update(b"active-targets-second-can-use-juedou-on-first/v1");
                }
                if targets.equipment_pair_with_armor {
                    h.update(b"active-targets-equipment-pair-with-armor/v1");
                }
                if targets.has_equip {
                    h.update(b"active-targets-has-equip/v1");
                }
                if targets.has_discardable_equip {
                    h.update(b"active-targets-has-discardable-equip/v1");
                }
                if targets.has_gainable_equip {
                    h.update(b"active-targets-has-gainable-equip/v1");
                }
                if targets.has_gainable_hand_equip {
                    h.update(b"active-targets-has-gainable-hand-equip/v1");
                }
                if targets.in_attack_range {
                    h.update(b"active-targets-in-attack-range/v1");
                }
                if targets.owner_in_target_attack_range {
                    h.update(b"active-owner-in-target-attack-range/v1");
                }
                if targets.hand_equip_nonempty {
                    h.update(b"active-targets-hand-equip-nonempty/v1");
                }
                if targets.has_discardable_hand_equip {
                    h.update(b"active-targets-has-discardable-hand-equip/v1");
                }
                if targets.unused_by_skill_this_turn {
                    h.update(b"active-targets-unused-by-skill-this-turn/v1");
                }
                if targets.unused_by_skill_this_play_phase {
                    h.update(b"active-targets-unused-by-skill-this-play-phase/v1");
                }
                if targets.match_selected_card_count {
                    h.update(b"active-targets-match-selected-card-count/v1");
                }
                if targets.selected_cards_match_target_equip_count {
                    h.update(b"active-targets-selected-cards-match-target-equip-count/v1");
                }
            }
            if let Some(uses) = a.uses_per_turn {
                h.update(b"active-uses-per-turn/v1");
                h.update(&[uses]);
            }
            if let Some(uses) = a.uses_per_play_phase {
                h.update(b"active-uses-per-play-phase/v1");
                h.update(&[uses]);
            }
            if let Some(source) = &a.uses_per_turn_source {
                h.update(b"active-uses-per-turn-source/v1");
                h.update(source.as_bytes());
            }
            if let Some((mark, count)) = a.mark_at_least {
                h.update(b"active-owner-mark-at-least/v1");
                h.update(&mark.to_le_bytes());
                h.update(&count.to_le_bytes());
            }
            if let Some(skill) = a.owner_missing_skill {
                h.update(b"active-owner-missing-skill/v1");
                h.update(&skill.to_le_bytes());
            }
            if a.hand_gt_hp {
                h.update(b"active-owner-hand-gt-hp/v1");
            }
            if a.hand_gt_max_hp {
                h.update(b"active-owner-hand-gt-max-hp/v1");
            }
            if a.hand_limit_positive {
                h.update(b"active-owner-hand-limit-positive/v1");
            }
            if a.equip_nonempty {
                h.update(b"active-owner-equip-nonempty/v1");
            }
            if a.hand_nonempty {
                h.update(b"active-owner-hand-nonempty/v1");
            }
            if let Some(tag) = a.hand_missing_gaintag {
                h.update(b"active-owner-hand-missing-gaintag/v1");
                h.update(&tag.to_le_bytes());
            }
            if a.face_up {
                h.update(b"active-owner-face-up/v1");
            }
            if a.owner_unlinked {
                h.update(b"active-owner-unlinked/v1");
            }
            if a.once_per_game {
                h.update(b"active-once-per-game/v1");
            }
        }
        for v in &self.view_as_index {
            h.update(&v.skill_def_id.to_le_bytes());
            h.update(v.as_kind.as_bytes());
            h.update(v.window.as_bytes());
            h.update(v.filter.as_bytes());
            if let Some(zone) = &v.source_zone {
                h.update(b"source-zone/v1");
                h.update(zone.as_bytes());
            }
            if let Some(uses) = v.uses_per_turn {
                h.update(b"view-as-uses-per-turn/v1");
                h.update(&[uses]);
            }
            if v.ignore_distance {
                h.update(b"view-as-ignore-distance/v1");
            }
            if v.red_unrespondable {
                h.update(b"view-as-red-unrespondable/v1");
            }
            if v.black_no_use_count {
                h.update(b"view-as-black-no-use-count/v1");
            }
            if v.no_use_count {
                h.update(b"view-as-no-use-count/v1");
            }
            if v.target_lacks_source_suit_unrespondable {
                h.update(b"view-as-target-lacks-source-suit-unrespondable/v1");
            }
            if v.refund_use_count_if_no_damage {
                h.update(b"view-as-refund-use-count-if-no-damage/v1");
            }
            if v.min_cards != 1 {
                h.update(b"view-as-min-cards/v1");
                h.update(&[v.min_cards]);
            }
            if v.max_cards != 1 {
                h.update(b"view-as-max-cards/v1");
                h.update(&[v.max_cards]);
            }
            if v.owner_hp_min_one_cards {
                h.update(b"view-as-owner-hp-min-one-cards/v1");
            }
            if v.owner_basic_uses_plus_one_cards {
                h.update(b"view-as-owner-basic-uses-plus-one-cards/v1");
            }
            if v.once_per_turn_per_branch {
                h.update(b"view-as-once-per-turn-per-branch/v1");
            }
            if v.same_suit {
                h.update(b"view-as-same-suit/v1");
            }
            if let Some(nature) = &v.nature {
                h.update(b"view-as-nature/v1");
                h.update(nature.as_bytes());
            }
            if let Some(bonus) = &v.multi_card_bonus {
                h.update(b"view-as-multi-card-bonus/v1");
                h.update(bonus.as_bytes());
            }
            if let Some(branch) = v.branch {
                h.update(b"view-as-branch/v1");
                h.update(&[branch]);
            }
            if v.once_per_round {
                h.update(b"view-as-once-per-round/v1");
            }
            if let Some(branch) = v.next_branch {
                h.update(b"view-as-next-branch/v1");
                h.update(&[branch]);
            }
            if let Some(cost) = &v.cost {
                h.update(b"view-as-cost/v1");
                h.update(cost.as_bytes());
            }
        }
        *h.finalize().as_bytes()
    }

    /// Compute the presentation/resource hash; empty presentation data keeps the legacy zero hash.
    pub fn compute_asset_hash(&self) -> [u8; 32] {
        let legacy_digest = self.compute_legacy_asset_hash();
        let voices: Vec<_> = self
            .definition_voice_assets()
            .into_iter()
            .filter(|(_, _, voices)| !voices.is_empty())
            .collect();
        if voices.is_empty() {
            return legacy_digest;
        }
        // A fixed-size digest prevents legacy strings from absorbing voice declarations.
        let mut h = blake3::Hasher::new();
        h.update(b"noname-next/npack-definition-voice-assets/v1");
        h.update(&legacy_digest);
        h.update(&serde_json::to_vec(&voices).expect("definition voices are serializable"));
        *h.finalize().as_bytes()
    }

    fn compute_legacy_asset_hash(&self) -> [u8; 32] {
        let has_character_assets = self.character_index.iter().any(|character| {
            !character.sort_key.is_empty()
                || character.portrait_asset.is_some()
                || !character.skin_assets.is_empty()
        });
        if self.translations.is_empty()
            && self.asset_manifest.is_none()
            && self.asset_paths.is_empty()
            && !has_character_assets
            && self.skin_index().is_empty()
        {
            return [0u8; 32];
        }

        let mut h = blake3::Hasher::new();
        h.update(b"noname-next/npack-assets/v1");
        for (locale, entries) in &self.translations {
            h.update(locale.as_bytes());
            h.update(b"\0");
            for (key, value) in entries {
                h.update(key.as_bytes());
                h.update(b"\0");
                h.update(value.as_bytes());
                h.update(b"\0");
            }
        }
        if let Some(manifest) = &self.asset_manifest {
            let bytes = serde_json::to_vec(manifest).expect("asset manifest is serializable");
            h.update(&bytes);
        }
        for (asset_id, path) in &self.asset_paths {
            h.update(asset_id.as_bytes());
            h.update(b"\0");
            h.update(path.as_bytes());
            h.update(b"\0");
        }
        let mut characters: Vec<_> = self.character_index.iter().collect();
        characters.sort_by_key(|character| character.id);
        for character in characters {
            if character.sort_key.is_empty()
                && character.portrait_asset.is_none()
                && character.skin_assets.is_empty()
            {
                continue;
            }
            h.update(&character.id.to_le_bytes());
            h.update(character.sort_key.as_bytes());
            if let Some(asset_id) = &character.portrait_asset {
                h.update(asset_id.as_bytes());
            }
            let mut skins = character.skin_assets.clone();
            skins.sort();
            for asset_id in skins {
                h.update(asset_id.as_bytes());
                h.update(b"\0");
            }
        }
        if !self.skin_index().is_empty() {
            h.update(b"skin-index/v1");
            let mut skin_index = self.skin_index().clone();
            skin_index.skins.sort_by_key(|skin| skin.id);
            for skin in &mut skin_index.skins {
                if let PackageSkinPresentation::Spine { dependencies, .. } = &mut skin.presentation
                {
                    dependencies.sort();
                }
            }
            skin_index
                .aliases
                .sort_by_key(|alias| (alias.character_id, alias.source_character_id));
            h.update(
                &serde_json::to_vec(&skin_index).expect("skin index is serializable presentation"),
            );
        }
        *h.finalize().as_bytes()
    }
}
