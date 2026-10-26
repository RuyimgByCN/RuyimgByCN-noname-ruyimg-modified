//! Recursive-descent parser.

use crate::ast::*;
use crate::lexer::{lex, Token, TokenKind};
use crate::CompileError;
use content_schema::Span;

/// Parse source into AST.
pub fn parse_source(source: &str) -> Result<SourceFile, CompileError> {
    let tokens = lex(source)?;
    let mut p = Parser { tokens, i: 0 };
    p.parse_file()
}

struct Parser {
    tokens: Vec<Token>,
    i: usize,
}

impl Parser {
    fn peek(&self) -> &Token {
        &self.tokens[self.i.min(self.tokens.len() - 1)]
    }

    fn pos(&self) -> usize {
        self.peek().pos
    }

    fn bump(&mut self) -> Token {
        let t = self.tokens[self.i].clone();
        if self.i + 1 < self.tokens.len() {
            self.i += 1;
        }
        t
    }

    fn expect_ident(&mut self) -> Result<String, CompileError> {
        match self.bump().kind {
            TokenKind::Ident(s) => Ok(s),
            _ => Err(CompileError::Parse {
                pos: self.pos(),
                message: "expected identifier".into(),
            }),
        }
    }

    fn expect(&mut self, kind: TokenKind) -> Result<(), CompileError> {
        let t = self.bump();
        if std::mem::discriminant(&t.kind) == std::mem::discriminant(&kind) {
            Ok(())
        } else {
            Err(CompileError::Parse {
                pos: t.pos,
                message: format!("expected {kind:?}, got {:?}", t.kind),
            })
        }
    }

    fn parse_file(&mut self) -> Result<SourceFile, CompileError> {
        let mut skills = Vec::new();
        while !matches!(self.peek().kind, TokenKind::Eof) {
            skills.push(self.parse_skill()?);
        }
        Ok(SourceFile { skills })
    }

    fn parse_skill(&mut self) -> Result<SkillAst, CompileError> {
        let start = self.pos() as u32;
        let kw = self.expect_ident()?;
        if kw != "skill" {
            return Err(CompileError::Parse {
                pos: start as usize,
                message: "expected skill".into(),
            });
        }
        let name = self.expect_ident()?;
        self.expect(TokenKind::LParen)?;
        let def_id = match self.bump().kind {
            TokenKind::Int(n) if u32::try_from(n).is_ok() => n as u32,
            _ => {
                return Err(CompileError::Parse {
                    pos: self.pos(),
                    message: "expected skill def id".into(),
                });
            }
        };
        self.expect(TokenKind::RParen)?;
        let mut locked = false;
        let mut lord_only = false;
        let mut non_gainable = false;
        let mut conversion = false;
        loop {
            match &self.peek().kind {
                TokenKind::Ident(value) if value == "locked" && !locked => {
                    locked = true;
                    self.bump();
                }
                TokenKind::Ident(value) if value == "lord_only" && !lord_only => {
                    lord_only = true;
                    self.bump();
                }
                TokenKind::Ident(value) if value == "non_gainable" && !non_gainable => {
                    non_gainable = true;
                    self.bump();
                }
                TokenKind::Ident(value) if value == "conversion" && !conversion => {
                    conversion = true;
                    self.bump();
                }
                _ => break,
            }
        }
        self.expect(TokenKind::LBrace)?;
        let mut items = Vec::new();
        while !matches!(self.peek().kind, TokenKind::RBrace | TokenKind::Eof) {
            items.push(self.parse_skill_item()?);
        }
        self.expect(TokenKind::RBrace)?;
        Ok(SkillAst {
            name,
            def_id,
            locked,
            lord_only,
            non_gainable,
            conversion,
            items,
            span: Span {
                start,
                end: self.pos() as u32,
            },
        })
    }

    fn parse_skill_item(&mut self) -> Result<SkillItemAst, CompileError> {
        let id = self.expect_ident()?;
        match id.as_str() {
            "trigger" => Ok(SkillItemAst::Trigger(self.parse_trigger_rest()?)),
            "active" => Ok(SkillItemAst::Active(self.parse_active_rest()?)),
            "view_as" => Ok(SkillItemAst::ViewAs(self.parse_view_as_rest()?)),
            "modifier" => Ok(SkillItemAst::Modifier(self.parse_modifier_rest()?)),
            _ => Err(CompileError::Parse {
                pos: self.pos(),
                message: format!("unknown skill item {id}"),
            }),
        }
    }

    fn parse_view_as_rest(&mut self) -> Result<ViewAsAst, CompileError> {
        let start = self.pos() as u32;
        let as_kind = self.expect_ident()?;
        let window = self.expect_ident()?;
        let filter = self.expect_ident()?;
        let mut source_zone = None;
        let mut uses_per_turn = None;
        let mut ignore_distance = false;
        let mut red_unrespondable = false;
        let mut black_no_use_count = false;
        let mut no_use_count = false;
        let mut target_lacks_source_suit_unrespondable = false;
        let mut refund_use_count_if_no_damage = false;
        let mut min_cards = 1;
        let mut max_cards = 1;
        let mut owner_hp_min_one_cards = false;
        let mut owner_basic_uses_plus_one_cards = false;
        let mut once_per_turn_per_branch = false;
        let mut source_card_count_seen = false;
        let mut same_suit = false;
        let mut nature = None;
        let mut multi_card_bonus = None;
        let mut branch = None;
        let mut once_per_round = false;
        let mut next_branch = None;
        let mut cost = None;
        let mut mark_cost = None;
        let mut post_draw = 0;
        while let TokenKind::Ident(option) = self.peek().kind.clone() {
            match option.as_str() {
                "from" => {
                    if source_zone.is_some() {
                        return Err(CompileError::Parse {
                            pos: self.pos(),
                            message: "duplicate view_as source zone".into(),
                        });
                    }
                    self.bump();
                    source_zone = Some(self.expect_ident()?);
                }
                "uses_per_turn" => {
                    if uses_per_turn.is_some() {
                        return Err(CompileError::Parse {
                            pos: self.pos(),
                            message: "duplicate view_as use limit".into(),
                        });
                    }
                    self.bump();
                    uses_per_turn = Some(self.expect_selection_count()?);
                }
                "ignore_distance" => {
                    if ignore_distance {
                        return Err(CompileError::Parse {
                            pos: self.pos(),
                            message: "duplicate view_as ignore_distance".into(),
                        });
                    }
                    self.bump();
                    ignore_distance = true;
                }
                "red_unrespondable" => {
                    if red_unrespondable {
                        return Err(CompileError::Parse {
                            pos: self.pos(),
                            message: "duplicate view_as red_unrespondable".into(),
                        });
                    }
                    self.bump();
                    red_unrespondable = true;
                }
                "black_no_use_count" => {
                    if black_no_use_count {
                        return Err(CompileError::Parse {
                            pos: self.pos(),
                            message: "duplicate view_as black_no_use_count".into(),
                        });
                    }
                    self.bump();
                    black_no_use_count = true;
                }
                "no_use_count" => {
                    if no_use_count {
                        return Err(CompileError::Parse {
                            pos: self.pos(),
                            message: "duplicate view_as no_use_count".into(),
                        });
                    }
                    self.bump();
                    no_use_count = true;
                }
                "target_lacks_source_suit_unrespondable" => {
                    if target_lacks_source_suit_unrespondable {
                        return Err(CompileError::Parse {
                            pos: self.pos(),
                            message: "duplicate target_lacks_source_suit_unrespondable".into(),
                        });
                    }
                    self.bump();
                    target_lacks_source_suit_unrespondable = true;
                }
                "refund_use_count_if_no_damage" => {
                    if refund_use_count_if_no_damage {
                        return Err(CompileError::Parse {
                            pos: self.pos(),
                            message: "duplicate view_as refund_use_count_if_no_damage".into(),
                        });
                    }
                    self.bump();
                    refund_use_count_if_no_damage = true;
                }
                "up_to" => {
                    if source_card_count_seen {
                        return Err(CompileError::Parse {
                            pos: self.pos(),
                            message: "duplicate view_as source-card limit".into(),
                        });
                    }
                    self.bump();
                    max_cards = self.expect_selection_count()?;
                    source_card_count_seen = true;
                }
                "exactly" => {
                    if source_card_count_seen {
                        return Err(CompileError::Parse {
                            pos: self.pos(),
                            message: "duplicate view_as source-card limit".into(),
                        });
                    }
                    self.bump();
                    let count = self.expect_selection_count()?;
                    min_cards = count;
                    max_cards = count;
                    source_card_count_seen = true;
                }
                "at_least" => {
                    if source_card_count_seen {
                        return Err(CompileError::Parse {
                            pos: self.pos(),
                            message: "duplicate view_as source-card limit".into(),
                        });
                    }
                    self.bump();
                    min_cards = self.expect_selection_count()?;
                    max_cards = u8::MAX;
                    source_card_count_seen = true;
                }
                "same_suit" => {
                    if same_suit {
                        return Err(CompileError::Parse {
                            pos: self.pos(),
                            message: "duplicate view_as same_suit".into(),
                        });
                    }
                    self.bump();
                    same_suit = true;
                }
                "owner_hp_min_one_cards" => {
                    if source_card_count_seen {
                        return Err(CompileError::Parse {
                            pos: self.pos(),
                            message: "duplicate view_as source-card limit".into(),
                        });
                    }
                    self.bump();
                    owner_hp_min_one_cards = true;
                    source_card_count_seen = true;
                }
                "owner_basic_uses_plus_one_cards" => {
                    if source_card_count_seen {
                        return Err(CompileError::Parse {
                            pos: self.pos(),
                            message: "duplicate view_as source-card limit".into(),
                        });
                    }
                    self.bump();
                    owner_basic_uses_plus_one_cards = true;
                    source_card_count_seen = true;
                }
                "once_per_turn_per_branch" => {
                    if once_per_turn_per_branch {
                        return Err(CompileError::Parse {
                            pos: self.pos(),
                            message: "duplicate view_as once_per_turn_per_branch".into(),
                        });
                    }
                    self.bump();
                    once_per_turn_per_branch = true;
                }
                "nature" => {
                    if nature.is_some() {
                        return Err(CompileError::Parse {
                            pos: self.pos(),
                            message: "duplicate view_as nature".into(),
                        });
                    }
                    self.bump();
                    nature = Some(self.expect_ident()?);
                }
                "multi_card_bonus" => {
                    if multi_card_bonus.is_some() {
                        return Err(CompileError::Parse {
                            pos: self.pos(),
                            message: "duplicate view_as multi-card bonus".into(),
                        });
                    }
                    self.bump();
                    multi_card_bonus = Some(self.expect_ident()?);
                }
                "branch" => {
                    if branch.is_some() {
                        return Err(CompileError::Parse {
                            pos: self.pos(),
                            message: "duplicate view_as branch".into(),
                        });
                    }
                    self.bump();
                    branch = Some(self.expect_selection_count()?);
                }
                "once_per_round" => {
                    if once_per_round {
                        return Err(CompileError::Parse {
                            pos: self.pos(),
                            message: "duplicate view_as once_per_round".into(),
                        });
                    }
                    self.bump();
                    once_per_round = true;
                }
                "next_branch" => {
                    if next_branch.is_some() {
                        return Err(CompileError::Parse {
                            pos: self.pos(),
                            message: "duplicate view_as next_branch".into(),
                        });
                    }
                    self.bump();
                    next_branch = Some(self.expect_selection_count()?);
                }
                "cost" => {
                    if cost.is_some() {
                        return Err(CompileError::Parse {
                            pos: self.pos(),
                            message: "duplicate view_as cost".into(),
                        });
                    }
                    self.bump();
                    cost = Some(self.expect_ident()?);
                }
                "mark_cost" => {
                    if mark_cost.is_some() {
                        return Err(CompileError::Parse {
                            pos: self.pos(),
                            message: "duplicate view_as mark cost".into(),
                        });
                    }
                    self.bump();
                    mark_cost = Some(match self.bump().kind {
                        TokenKind::Int(value) if u32::try_from(value).is_ok() => value as u32,
                        _ => {
                            return Err(CompileError::Parse {
                                pos: self.pos(),
                                message: "expected mark skill def id".into(),
                            });
                        }
                    });
                }
                "post_draw" => {
                    if post_draw != 0 {
                        return Err(CompileError::Parse {
                            pos: self.pos(),
                            message: "duplicate view_as post draw".into(),
                        });
                    }
                    self.bump();
                    post_draw = self.expect_selection_count()?;
                    if post_draw == 0 {
                        return Err(CompileError::Parse {
                            pos: self.pos(),
                            message: "view_as post draw must be positive".into(),
                        });
                    }
                }
                _ => break,
            }
        }
        self.expect(TokenKind::Semi)?;
        Ok(ViewAsAst {
            as_kind,
            window,
            filter,
            source_zone,
            uses_per_turn,
            ignore_distance,
            red_unrespondable,
            black_no_use_count,
            no_use_count,
            target_lacks_source_suit_unrespondable,
            refund_use_count_if_no_damage,
            min_cards,
            max_cards,
            owner_hp_min_one_cards,
            owner_basic_uses_plus_one_cards,
            once_per_turn_per_branch,
            same_suit,
            nature,
            multi_card_bonus,
            branch,
            once_per_round,
            next_branch,
            cost,
            mark_cost,
            post_draw,
            span: Span {
                start,
                end: self.pos() as u32,
            },
        })
    }

    fn parse_active_rest(&mut self) -> Result<ActiveAst, CompileError> {
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

    fn expect_selection_count(&mut self) -> Result<u8, CompileError> {
        match self.bump().kind {
            TokenKind::Int(value) if (0..=u8::MAX as i64).contains(&value) => Ok(value as u8),
            _ => Err(CompileError::Parse {
                pos: self.pos(),
                message: "expected selection count 0..255".into(),
            }),
        }
    }

    fn expect_selection_sum(&mut self) -> Result<u16, CompileError> {
        match self.bump().kind {
            TokenKind::Int(value) if (1..=u16::MAX as i64).contains(&value) => Ok(value as u16),
            _ => Err(CompileError::Parse {
                pos: self.pos(),
                message: "expected positive selection sum 1..65535".into(),
            }),
        }
    }

    fn parse_trigger_rest(&mut self) -> Result<TriggerAst, CompileError> {
        let start = self.pos() as u32;
        let event = self.expect_ident()?;
        let filter = self.expect_ident()?;
        let optional = matches!(&self.peek().kind, TokenKind::Ident(value) if value == "optional");
        if optional {
            self.bump();
        }
        let choice_actor = if optional
            && matches!(&self.peek().kind, TokenKind::Ident(value) if value == "by")
        {
            self.bump();
            match self.expect_ident()?.as_str() {
                "owner" => content_schema::TriggerChoiceActor::Owner,
                "event_player" => content_schema::TriggerChoiceActor::EventPlayer,
                "event_source" => content_schema::TriggerChoiceActor::EventSource,
                actor => {
                    return Err(CompileError::Parse {
                        pos: self.pos(),
                        message: format!(
                            "unknown optional trigger actor {actor}; expected owner, event_player, or event_source"
                        ),
                    });
                }
            }
        } else {
            content_schema::TriggerChoiceActor::Owner
        };
        self.expect(TokenKind::LBrace)?;
        let body = self.parse_block_body()?;
        self.expect(TokenKind::RBrace)?;
        Ok(TriggerAst {
            event,
            filter,
            optional,
            choice_actor,
            body,
            span: Span {
                start,
                end: self.pos() as u32,
            },
        })
    }

    fn parse_modifier_rest(&mut self) -> Result<ModifierAst, CompileError> {
        let start = self.pos() as u32;
        let target = self.expect_ident()?;
        let selector = if self.peek().kind == TokenKind::LParen {
            self.bump();
            let selector = self.expect_ident()?;
            self.expect(TokenKind::RParen)?;
            Some(selector)
        } else {
            None
        };
        let mode = self.expect_ident()?;
        let (value, value_source) = match self.bump().kind {
            TokenKind::Int(n) if i32::try_from(n).is_ok() => (n as i32, None),
            TokenKind::Ident(source) => (1, Some(source)),
            TokenKind::Minus => match self.bump().kind {
                TokenKind::Ident(source) => (-1, Some(source)),
                _ => {
                    return Err(CompileError::Parse {
                        pos: self.pos(),
                        message: "expected modifier value source after '-'".into(),
                    });
                }
            },
            _ => {
                return Err(CompileError::Parse {
                    pos: self.pos(),
                    message: "expected modifier value".into(),
                });
            }
        };
        self.expect(TokenKind::Semi)?;
        Ok(ModifierAst {
            target,
            selector,
            mode,
            value_source,
            value,
            span: Span {
                start,
                end: self.pos() as u32,
            },
        })
    }

    fn parse_block_body(&mut self) -> Result<Vec<StmtAst>, CompileError> {
        let mut body = Vec::new();
        while !matches!(self.peek().kind, TokenKind::RBrace | TokenKind::Eof) {
            body.push(self.parse_stmt()?);
        }
        Ok(body)
    }

    fn parse_stmt(&mut self) -> Result<StmtAst, CompileError> {
        let start = self.pos() as u32;
        let id = self.expect_ident()?;
        match id.as_str() {
            "effect" => {
                let name = self.expect_ident()?;
                self.expect(TokenKind::LParen)?;
                let args = self.parse_arg_list()?;
                self.expect(TokenKind::RParen)?;
                self.expect(TokenKind::Semi)?;
                Ok(StmtAst::Effect {
                    name,
                    args,
                    span: Span {
                        start,
                        end: self.pos() as u32,
                    },
                })
            }
            "if" => {
                let cond = self.parse_expr()?;
                self.expect(TokenKind::LBrace)?;
                let then_body = self.parse_block_body()?;
                self.expect(TokenKind::RBrace)?;
                let else_body = if matches!(self.peek().kind, TokenKind::Ident(ref s) if s == "else")
                {
                    self.bump();
                    self.expect(TokenKind::LBrace)?;
                    let b = self.parse_block_body()?;
                    self.expect(TokenKind::RBrace)?;
                    b
                } else {
                    Vec::new()
                };
                Ok(StmtAst::If {
                    cond,
                    then_body,
                    else_body,
                    span: Span {
                        start,
                        end: self.pos() as u32,
                    },
                })
            }
            "loop" => {
                let times = match self.bump().kind {
                    TokenKind::Int(n) if u32::try_from(n).is_ok() => n as u32,
                    _ => {
                        return Err(CompileError::Parse {
                            pos: self.pos(),
                            message: "expected loop count".into(),
                        });
                    }
                };
                // Cap at compile time hard max
                if times > 1_000_000 {
                    return Err(CompileError::Parse {
                        pos: start as usize,
                        message: "loop bound too large (max 1000000)".into(),
                    });
                }
                self.expect(TokenKind::LBrace)?;
                let body = self.parse_block_body()?;
                self.expect(TokenKind::RBrace)?;
                Ok(StmtAst::Loop {
                    times,
                    body,
                    span: Span {
                        start,
                        end: self.pos() as u32,
                    },
                })
            }
            "let" => {
                let name = self.expect_ident()?;
                self.expect(TokenKind::Assign)?;
                let value = self.parse_expr()?;
                self.expect(TokenKind::Semi)?;
                Ok(StmtAst::Let {
                    name,
                    value,
                    span: Span {
                        start,
                        end: self.pos() as u32,
                    },
                })
            }
            _ => Err(CompileError::Parse {
                pos: start as usize,
                message: format!("unknown statement {id}"),
            }),
        }
    }

    fn parse_arg_list(&mut self) -> Result<Vec<ExprAst>, CompileError> {
        let mut args = Vec::new();
        if matches!(self.peek().kind, TokenKind::RParen) {
            return Ok(args);
        }
        loop {
            args.push(self.parse_expr()?);
            if matches!(self.peek().kind, TokenKind::Comma) {
                self.bump();
                continue;
            }
            break;
        }
        Ok(args)
    }

    fn parse_expr(&mut self) -> Result<ExprAst, CompileError> {
        self.parse_and()
    }

    fn parse_and(&mut self) -> Result<ExprAst, CompileError> {
        let mut left = self.parse_cmp()?;
        while matches!(self.peek().kind, TokenKind::AmpAmp) {
            self.bump();
            let right = self.parse_cmp()?;
            left = ExprAst::BinOp {
                op: "&&".into(),
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_cmp(&mut self) -> Result<ExprAst, CompileError> {
        let mut left = self.parse_add()?;
        loop {
            match &self.peek().kind {
                TokenKind::EqEq => {
                    self.bump();
                    let right = self.parse_add()?;
                    left = ExprAst::BinOp {
                        op: "==".into(),
                        left: Box::new(left),
                        right: Box::new(right),
                    };
                }
                TokenKind::Lt => {
                    self.bump();
                    let right = self.parse_add()?;
                    left = ExprAst::BinOp {
                        op: "<".into(),
                        left: Box::new(left),
                        right: Box::new(right),
                    };
                }
                TokenKind::Gt => {
                    self.bump();
                    let right = self.parse_add()?;
                    left = ExprAst::BinOp {
                        op: ">".into(),
                        left: Box::new(left),
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }
        Ok(left)
    }

    fn parse_add(&mut self) -> Result<ExprAst, CompileError> {
        let mut left = self.parse_mul()?;
        loop {
            match &self.peek().kind {
                TokenKind::Plus => {
                    self.bump();
                    let right = self.parse_mul()?;
                    left = ExprAst::BinOp {
                        op: "+".into(),
                        left: Box::new(left),
                        right: Box::new(right),
                    };
                }
                TokenKind::Minus => {
                    self.bump();
                    let right = self.parse_mul()?;
                    left = ExprAst::BinOp {
                        op: "-".into(),
                        left: Box::new(left),
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }
        Ok(left)
    }

    fn parse_mul(&mut self) -> Result<ExprAst, CompileError> {
        let mut left = self.parse_primary()?;
        while matches!(self.peek().kind, TokenKind::Star) {
            self.bump();
            let right = self.parse_primary()?;
            left = ExprAst::BinOp {
                op: "*".into(),
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_primary(&mut self) -> Result<ExprAst, CompileError> {
        match self.bump().kind {
            TokenKind::Int(n) if i32::try_from(n).is_ok() => Ok(ExprAst::Int(n as i32)),
            TokenKind::Int(n) if u32::try_from(n).is_ok() => Ok(ExprAst::U32(n as u32)),
            TokenKind::Int(_) => Err(CompileError::Parse {
                pos: self.pos(),
                message: "integer expression out of u32 range".into(),
            }),
            TokenKind::Ident(name) => {
                if matches!(self.peek().kind, TokenKind::LParen) {
                    self.bump();
                    let args = self.parse_arg_list()?;
                    self.expect(TokenKind::RParen)?;
                    Ok(ExprAst::Call { name, args })
                } else {
                    Ok(ExprAst::Ident(name))
                }
            }
            TokenKind::LParen => {
                let e = self.parse_expr()?;
                self.expect(TokenKind::RParen)?;
                Ok(e)
            }
            other => Err(CompileError::Parse {
                pos: self.pos(),
                message: format!("expected expression, got {other:?}"),
            }),
        }
    }
}
