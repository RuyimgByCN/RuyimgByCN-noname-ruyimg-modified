//! 主动技（active play_phase …）声明的语法解析：发动条件、目标与选牌约束。

use super::*;

impl Parser {
    pub(super) fn parse_active_rest(&mut self) -> Result<ActiveAst, CompileError> {
        let start = self.pos() as u32;
        let window = self.expect_ident()?;
        let mut selection = None;
        let mut targets = None;
        let mut uses_per_turn = None;
        let mut uses_per_turn_source = None;
        let mut uses_per_play_phase = None;
        let mut mark_at_least = None;
        let mut owner_missing_skill = None;
        let mut hand_gt_hp = false;
        let mut hand_gt_max_hp = false;
        let mut hand_limit_positive = false;
        let mut equip_nonempty = false;
        let mut hand_nonempty = false;
        let mut face_up = false;
        let mut owner_unlinked = false;
        let mut once_per_game = false;
        let mut hand_missing_gaintag = None;
        while let TokenKind::Ident(option) = self.peek().kind.clone() {
            match option.as_str() {
                "select_hand" | "select_he" => {
                    if selection.is_some() {
                        return Err(CompileError::Parse {
                            pos: self.pos(),
                            message: "duplicate active card selection".into(),
                        });
                    }
                    let zone = if option == "select_hand" {
                        "hand"
                    } else {
                        "hand_equip"
                    };
                    self.bump();
                    let min = self.expect_selection_count()?;
                    let max = self.expect_selection_count()?;
                    let mut allow_cancel = false;
                    let mut same_suit = false;
                    let mut same_color = false;
                    let mut distinct_categories = false;
                    let mut distinct_suits = false;
                    let mut filter = None;
                    let mut unused_category_by_skill_this_turn = false;
                    let mut unused_category_by_skill_this_play_phase = false;
                    let mut unused_category_by_owner_history_this_turn = false;
                    let mut sum_effective_number_exact = None;
                    loop {
                        match &self.peek().kind {
                            TokenKind::Ident(value) if value == "allow_cancel" => {
                                if allow_cancel {
                                    return Err(CompileError::Parse {
                                        pos: self.pos(),
                                        message: "duplicate allow_cancel constraint".into(),
                                    });
                                }
                                allow_cancel = true;
                                self.bump();
                            }
                            TokenKind::Ident(value) if value == "same_suit" => {
                                if same_suit {
                                    return Err(CompileError::Parse {
                                        pos: self.pos(),
                                        message: "duplicate same_suit constraint".into(),
                                    });
                                }
                                same_suit = true;
                                self.bump();
                            }
                            TokenKind::Ident(value) if value == "same_color" => {
                                if same_color {
                                    return Err(CompileError::Parse {
                                        pos: self.pos(),
                                        message: "duplicate same_color constraint".into(),
                                    });
                                }
                                same_color = true;
                                self.bump();
                            }
                            TokenKind::Ident(value) if value == "distinct_categories" => {
                                if distinct_categories {
                                    return Err(CompileError::Parse {
                                        pos: self.pos(),
                                        message: "duplicate distinct_categories constraint".into(),
                                    });
                                }
                                distinct_categories = true;
                                self.bump();
                            }
                            TokenKind::Ident(value) if value == "distinct_suits" => {
                                if distinct_suits {
                                    return Err(CompileError::Parse {
                                        pos: self.pos(),
                                        message: "duplicate distinct_suits constraint".into(),
                                    });
                                }
                                distinct_suits = true;
                                self.bump();
                            }
                            TokenKind::Ident(value)
                                if matches!(
                                    value.as_str(),
                                    "card_equip"
                                        | "card_armor"
                                        | "card_sha"
                                        | "card_basic"
                                        | "card_nonbasic"
                                        | "card_non_equip"
                                        | "card_red"
                                        | "card_black"
                                        | "card_heart"
                                ) =>
                            {
                                if filter.is_some() {
                                    return Err(CompileError::Parse {
                                        pos: self.pos(),
                                        message: "duplicate active card filter".into(),
                                    });
                                }
                                filter = Some(value.clone());
                                self.bump();
                            }
                            TokenKind::Ident(value)
                                if value == "unused_category_by_skill_this_turn" =>
                            {
                                if unused_category_by_skill_this_turn {
                                    return Err(CompileError::Parse {
                                        pos: self.pos(),
                                        message: "duplicate unused-category constraint".into(),
                                    });
                                }
                                unused_category_by_skill_this_turn = true;
                                self.bump();
                            }
                            TokenKind::Ident(value)
                                if value == "unused_category_by_skill_this_play_phase" =>
                            {
                                if unused_category_by_skill_this_play_phase {
                                    return Err(CompileError::Parse {
                                        pos: self.pos(),
                                        message: "duplicate play-phase unused-category constraint"
                                            .into(),
                                    });
                                }
                                unused_category_by_skill_this_play_phase = true;
                                self.bump();
                            }
                            TokenKind::Ident(value)
                                if value == "unused_category_by_owner_history_this_turn" =>
                            {
                                if unused_category_by_owner_history_this_turn {
                                    return Err(CompileError::Parse {
                                        pos: self.pos(),
                                        message:
                                            "duplicate owner-history unused-category constraint"
                                                .into(),
                                    });
                                }
                                unused_category_by_owner_history_this_turn = true;
                                self.bump();
                            }
                            TokenKind::Ident(value) if value == "sum_effective_number_exact" => {
                                if sum_effective_number_exact.is_some() {
                                    return Err(CompileError::Parse {
                                        pos: self.pos(),
                                        message: "duplicate sum_effective_number_exact constraint"
                                            .into(),
                                    });
                                }
                                self.bump();
                                sum_effective_number_exact = Some(self.expect_selection_sum()?);
                            }
                            _ => break,
                        }
                    }
                    selection = Some(ActiveSelectionAst {
                        zone: zone.into(),
                        min,
                        max,
                        allow_cancel,
                        same_suit,
                        same_color,
                        distinct_categories,
                        distinct_suits,
                        filter,
                        unused_category_by_skill_this_turn,
                        unused_category_by_skill_this_play_phase,
                        unused_category_by_owner_history_this_turn,
                        sum_effective_number_exact,
                    });
                }
                "targets" => {
                    if targets.is_some() {
                        return Err(CompileError::Parse {
                            pos: self.pos(),
                            message: "duplicate active target contract".into(),
                        });
                    }
                    self.bump();
                    let min = self.expect_selection_count()?;
                    let max = self.expect_selection_count()?;
                    let mut allow_self = true;
                    let mut lowest_hp = false;
                    let mut highest_hp = false;
                    let mut unlinked = false;
                    let mut linked = false;
                    let mut max_by_owner_mark = None;
                    let mut max_by_owner_lost_hp = false;
                    let mut max_by_owner_hp = false;
                    let mut wounded = false;
                    let mut male = false;
                    let mut hp_at_least_owner = false;
                    let mut hp_lte_owner = false;
                    let mut hp_gt_owner = false;
                    let mut hp_gt = None;
                    let mut max_hp_gt = None;
                    let mut hand_gt_owner = false;
                    let mut hand_lte_owner = false;
                    let mut pair_hand_difference_lte_three = false;
                    let mut hand_lt = None;
                    let mut hand_nonempty = false;
                    let mut can_compare = false;
                    let mut can_use_on_owner = None;
                    let mut second_can_use_juedou_on_first = false;
                    let mut equipment_pair_with_armor = false;
                    let mut has_equip = false;
                    let mut has_discardable_equip = false;
                    let mut has_gainable_equip = false;
                    let mut has_gainable_hand_equip = false;
                    let mut in_attack_range = false;
                    let mut owner_in_target_attack_range = false;
                    let mut hand_equip_nonempty = false;
                    let mut has_discardable_hand_equip = false;
                    let mut unused_by_skill_this_turn = false;
                    let mut unused_by_skill_this_play_phase = false;
                    let mut match_selected_card_count = false;
                    let mut selected_cards_match_target_equip_count = false;
                    loop {
                        match &self.peek().kind {
                            TokenKind::Ident(value) if value == "other" => {
                                allow_self = false;
                                self.bump();
                            }
                            TokenKind::Ident(value) if value == "lowest_hp" => {
                                lowest_hp = true;
                                self.bump();
                            }
                            TokenKind::Ident(value) if value == "highest_hp" => {
                                highest_hp = true;
                                self.bump();
                            }
                            TokenKind::Ident(value) if value == "unlinked" => {
                                unlinked = true;
                                self.bump();
                            }
                            TokenKind::Ident(value) if value == "linked" => {
                                linked = true;
                                self.bump();
                            }
                            TokenKind::Ident(value) if value == "max_by_owner_mark" => {
                                if max_by_owner_mark.is_some() {
                                    return Err(CompileError::Parse {
                                        pos: self.pos(),
                                        message: "duplicate max_by_owner_mark target constraint"
                                            .into(),
                                    });
                                }
                                self.bump();
                                max_by_owner_mark = Some(match self.bump().kind {
                                    TokenKind::Int(value) if u32::try_from(value).is_ok() => {
                                        value as u32
                                    }
                                    _ => {
                                        return Err(CompileError::Parse {
                                            pos: self.pos(),
                                            message:
                                                "max_by_owner_mark requires an unsigned mark id"
                                                    .into(),
                                        });
                                    }
                                });
                            }
                            TokenKind::Ident(value) if value == "max_by_owner_lost_hp" => {
                                if max_by_owner_lost_hp {
                                    return Err(CompileError::Parse {
                                        pos: self.pos(),
                                        message: "duplicate max_by_owner_lost_hp target constraint"
                                            .into(),
                                    });
                                }
                                max_by_owner_lost_hp = true;
                                self.bump();
                            }
                            TokenKind::Ident(value) if value == "max_by_owner_hp" => {
                                if max_by_owner_hp {
                                    return Err(CompileError::Parse {
                                        pos: self.pos(),
                                        message: "duplicate max_by_owner_hp target constraint"
                                            .into(),
                                    });
                                }
                                max_by_owner_hp = true;
                                self.bump();
                            }
                            TokenKind::Ident(value) if value == "wounded" => {
                                wounded = true;
                                self.bump();
                            }
                            TokenKind::Ident(value) if value == "male" => {
                                male = true;
                                self.bump();
                            }
                            TokenKind::Ident(value) if value == "hp_at_least_owner" => {
                                hp_at_least_owner = true;
                                self.bump();
                            }
                            TokenKind::Ident(value) if value == "hp_gt_owner" => {
                                hp_gt_owner = true;
                                self.bump();
                            }
                            TokenKind::Ident(value) if value == "max_hp_gt" => {
                                if max_hp_gt.is_some() {
                                    return Err(CompileError::Parse {
                                        pos: self.pos(),
                                        message: "duplicate max_hp_gt target constraint".into(),
                                    });
                                }
                                self.bump();
                                max_hp_gt = Some(self.expect_selection_count()?);
                            }
                            TokenKind::Ident(value) if value == "hp_lte_owner" => {
                                hp_lte_owner = true;
                                self.bump();
                            }
                            TokenKind::Ident(value) if value == "hp_gt" => {
                                if hp_gt.is_some() {
                                    return Err(CompileError::Parse {
                                        pos: self.pos(),
                                        message: "duplicate hp_gt target constraint".into(),
                                    });
                                }
                                self.bump();
                                hp_gt = Some(self.expect_selection_count()?);
                            }
                            TokenKind::Ident(value) if value == "hand_gt_owner" => {
                                hand_gt_owner = true;
                                self.bump();
                            }
                            TokenKind::Ident(value) if value == "hand_lte_owner" => {
                                hand_lte_owner = true;
                                self.bump();
                            }
                            TokenKind::Ident(value)
                                if value == "pair_hand_difference_lte_three" =>
                            {
                                pair_hand_difference_lte_three = true;
                                self.bump();
                            }
                            TokenKind::Ident(value) if value == "hand_lt" => {
                                if hand_lt.is_some() {
                                    return Err(CompileError::Parse {
                                        pos: self.pos(),
                                        message: "duplicate hand_lt target constraint".into(),
                                    });
                                }
                                self.bump();
                                hand_lt = Some(self.expect_selection_count()?);
                            }
                            TokenKind::Ident(value) if value == "hand_nonempty" => {
                                hand_nonempty = true;
                                self.bump();
                            }
                            TokenKind::Ident(value) if value == "can_compare" => {
                                can_compare = true;
                                self.bump();
                            }
                            TokenKind::Ident(value) if value == "can_use_on_owner" => {
                                if can_use_on_owner.is_some() {
                                    return Err(CompileError::Parse {
                                        pos: self.pos(),
                                        message: "duplicate can_use_on_owner target constraint"
                                            .into(),
                                    });
                                }
                                self.bump();
                                can_use_on_owner = Some(self.expect_ident()?);
                            }
                            TokenKind::Ident(value)
                                if value == "second_can_use_juedou_on_first" =>
                            {
                                second_can_use_juedou_on_first = true;
                                self.bump();
                            }
                            TokenKind::Ident(value) if value == "equipment_pair_with_armor" => {
                                equipment_pair_with_armor = true;
                                self.bump();
                            }
                            TokenKind::Ident(value) if value == "has_equip" => {
                                has_equip = true;
                                self.bump();
                            }
                            TokenKind::Ident(value) if value == "has_discardable_equip" => {
                                has_discardable_equip = true;
                                self.bump();
                            }
                            TokenKind::Ident(value) if value == "has_gainable_equip" => {
                                has_gainable_equip = true;
                                self.bump();
                            }
                            TokenKind::Ident(value) if value == "has_gainable_hand_equip" => {
                                has_gainable_hand_equip = true;
                                self.bump();
                            }
                            TokenKind::Ident(value) if value == "in_attack_range" => {
                                in_attack_range = true;
                                self.bump();
                            }
                            TokenKind::Ident(value) if value == "owner_in_target_attack_range" => {
                                owner_in_target_attack_range = true;
                                self.bump();
                            }
                            TokenKind::Ident(value) if value == "hand_equip_nonempty" => {
                                hand_equip_nonempty = true;
                                self.bump();
                            }
                            TokenKind::Ident(value) if value == "has_discardable_hand_equip" => {
                                has_discardable_hand_equip = true;
                                self.bump();
                            }
                            TokenKind::Ident(value) if value == "unused_by_skill_this_turn" => {
                                if unused_by_skill_this_turn {
                                    return Err(CompileError::Parse {
                                        pos: self.pos(),
                                        message: "duplicate unused-target constraint".into(),
                                    });
                                }
                                unused_by_skill_this_turn = true;
                                self.bump();
                            }
                            TokenKind::Ident(value)
                                if value == "unused_by_skill_this_play_phase" =>
                            {
                                if unused_by_skill_this_play_phase {
                                    return Err(CompileError::Parse {
                                        pos: self.pos(),
                                        message: "duplicate play-phase unused-target constraint"
                                            .into(),
                                    });
                                }
                                unused_by_skill_this_play_phase = true;
                                self.bump();
                            }
                            TokenKind::Ident(value) if value == "match_selected_card_count" => {
                                if match_selected_card_count {
                                    return Err(CompileError::Parse {
                                        pos: self.pos(),
                                        message: "duplicate match-selected-card-count constraint"
                                            .into(),
                                    });
                                }
                                match_selected_card_count = true;
                                self.bump();
                            }
                            TokenKind::Ident(value)
                                if value == "selected_cards_match_target_equip_count" =>
                            {
                                if selected_cards_match_target_equip_count {
                                    return Err(CompileError::Parse {
                                        pos: self.pos(),
                                        message:
                                            "duplicate selected-cards-match-target-equip-count constraint"
                                                .into(),
                                    });
                                }
                                selected_cards_match_target_equip_count = true;
                                self.bump();
                            }
                            _ => break,
                        }
                    }
                    targets = Some(ActiveTargetsAst {
                        min,
                        max,
                        allow_self,
                        lowest_hp,
                        highest_hp,
                        unlinked,
                        linked,
                        max_by_owner_mark,
                        max_by_owner_lost_hp,
                        max_by_owner_hp,
                        wounded,
                        male,
                        hp_at_least_owner,
                        hp_gt_owner,
                        hp_lte_owner,
                        hp_gt,
                        max_hp_gt,
                        hand_gt_owner,
                        hand_lte_owner,
                        pair_hand_difference_lte_three,
                        hand_lt,
                        hand_nonempty,
                        can_compare,
                        can_use_on_owner,
                        second_can_use_juedou_on_first,
                        equipment_pair_with_armor,
                        has_equip,
                        has_discardable_equip,
                        has_gainable_equip,
                        has_gainable_hand_equip,
                        in_attack_range,
                        owner_in_target_attack_range,
                        hand_equip_nonempty,
                        has_discardable_hand_equip,
                        unused_by_skill_this_turn,
                        unused_by_skill_this_play_phase,
                        match_selected_card_count,
                        selected_cards_match_target_equip_count,
                    });
                }
                "uses_per_turn" => {
                    if uses_per_turn.is_some()
                        || uses_per_turn_source.is_some()
                        || uses_per_play_phase.is_some()
                    {
                        return Err(CompileError::Parse {
                            pos: self.pos(),
                            message: "duplicate active use limit".into(),
                        });
                    }
                    self.bump();
                    match self.peek().kind.clone() {
                        TokenKind::Ident(source)
                            if matches!(
                                source.as_str(),
                                "owner_hp" | "owner_lost_hp_min_one" | "living_rebel_count"
                            ) =>
                        {
                            self.bump();
                            uses_per_turn_source = Some(source);
                        }
                        _ => uses_per_turn = Some(self.expect_selection_count()?),
                    }
                }
                "uses_per_play_phase" => {
                    if uses_per_turn.is_some()
                        || uses_per_turn_source.is_some()
                        || uses_per_play_phase.is_some()
                    {
                        return Err(CompileError::Parse {
                            pos: self.pos(),
                            message: "duplicate active use limit".into(),
                        });
                    }
                    self.bump();
                    uses_per_play_phase = Some(self.expect_selection_count()?);
                }
                "mark_at_least" => {
                    if mark_at_least.is_some() {
                        return Err(CompileError::Parse {
                            pos: self.pos(),
                            message: "duplicate active mark_at_least precondition".into(),
                        });
                    }
                    self.bump();
                    let mut next_u32 = || match self.bump().kind {
                        TokenKind::Int(value) if u32::try_from(value).is_ok() => Ok(value as u32),
                        _ => Err(CompileError::Parse {
                            pos: self.pos(),
                            message: "mark_at_least requires unsigned integers".into(),
                        }),
                    };
                    let mark = next_u32()?;
                    let count = next_u32()?;
                    mark_at_least = Some((mark, count));
                }
                "owner_missing_skill" => {
                    if owner_missing_skill.is_some() {
                        return Err(CompileError::Parse {
                            pos: self.pos(),
                            message: "duplicate active owner_missing_skill precondition".into(),
                        });
                    }
                    self.bump();
                    owner_missing_skill = Some(match self.bump().kind {
                        TokenKind::Int(value) if u32::try_from(value).is_ok() => value as u32,
                        _ => {
                            return Err(CompileError::Parse {
                                pos: self.pos(),
                                message:
                                    "owner_missing_skill requires an unsigned skill definition"
                                        .into(),
                            });
                        }
                    });
                }
                "hand_gt_hp" => {
                    if hand_gt_hp {
                        return Err(CompileError::Parse {
                            pos: self.pos(),
                            message: "duplicate active hand_gt_hp precondition".into(),
                        });
                    }
                    self.bump();
                    hand_gt_hp = true;
                }
                "hand_gt_max_hp" => {
                    if hand_gt_max_hp {
                        return Err(CompileError::Parse {
                            pos: self.pos(),
                            message: "duplicate active hand_gt_max_hp precondition".into(),
                        });
                    }
                    self.bump();
                    hand_gt_max_hp = true;
                }
                "hand_limit_positive" => {
                    if hand_limit_positive {
                        return Err(CompileError::Parse {
                            pos: self.pos(),
                            message: "duplicate active hand_limit_positive precondition".into(),
                        });
                    }
                    self.bump();
                    hand_limit_positive = true;
                }
                "equip_nonempty" => {
                    if equip_nonempty {
                        return Err(CompileError::Parse {
                            pos: self.pos(),
                            message: "duplicate active equip_nonempty precondition".into(),
                        });
                    }
                    self.bump();
                    equip_nonempty = true;
                }
                "hand_nonempty" => {
                    if hand_nonempty {
                        return Err(CompileError::Parse {
                            pos: self.pos(),
                            message: "duplicate active hand_nonempty precondition".into(),
                        });
                    }
                    self.bump();
                    hand_nonempty = true;
                }
                "face_up" => {
                    if face_up {
                        return Err(CompileError::Parse {
                            pos: self.pos(),
                            message: "duplicate active face_up precondition".into(),
                        });
                    }
                    self.bump();
                    face_up = true;
                }
                "hand_missing_gaintag" => {
                    if hand_missing_gaintag.is_some() {
                        return Err(CompileError::Parse {
                            pos: self.pos(),
                            message: "duplicate active hand_missing_gaintag precondition".into(),
                        });
                    }
                    self.bump();
                    match self.bump().kind {
                        TokenKind::Int(value) if u32::try_from(value).is_ok() => {
                            hand_missing_gaintag = Some(value as u32);
                        }
                        _ => {
                            return Err(CompileError::Parse {
                                pos: self.pos(),
                                message: "hand_missing_gaintag requires an unsigned integer".into(),
                            });
                        }
                    }
                }
                "owner_unlinked" => {
                    if owner_unlinked {
                        return Err(CompileError::Parse {
                            pos: self.pos(),
                            message: "duplicate active owner_unlinked precondition".into(),
                        });
                    }
                    self.bump();
                    owner_unlinked = true;
                }
                "once_per_game" => {
                    if once_per_game {
                        return Err(CompileError::Parse {
                            pos: self.pos(),
                            message: "duplicate active once_per_game limit".into(),
                        });
                    }
                    self.bump();
                    once_per_game = true;
                }
                _ => break,
            }
        }
        self.expect(TokenKind::LBrace)?;
        let body = self.parse_block_body()?;
        self.expect(TokenKind::RBrace)?;
        Ok(ActiveAst {
            window,
            targets,
            uses_per_turn,
            uses_per_turn_source,
            uses_per_play_phase,
            mark_at_least,
            owner_missing_skill,
            hand_gt_hp,
            hand_gt_max_hp,
            hand_limit_positive,
            equip_nonempty,
            hand_nonempty,
            hand_missing_gaintag,
            face_up,
            owner_unlinked,
            once_per_game,
            selection,
            body,
            span: Span {
                start,
                end: self.pos() as u32,
            },
        })
    }

    pub(super) fn expect_selection_count(&mut self) -> Result<u8, CompileError> {
        match self.bump().kind {
            TokenKind::Int(value) if (0..=u8::MAX as i64).contains(&value) => Ok(value as u8),
            _ => Err(CompileError::Parse {
                pos: self.pos(),
                message: "expected selection count 0..255".into(),
            }),
        }
    }

    pub(super) fn expect_selection_sum(&mut self) -> Result<u16, CompileError> {
        match self.bump().kind {
            TokenKind::Int(value) if (1..=u16::MAX as i64).contains(&value) => Ok(value as u16),
            _ => Err(CompileError::Parse {
                pos: self.pos(),
                message: "expected positive selection sum 1..65535".into(),
            }),
        }
    }
}
