//! 视为技（view_as …）声明的语法解析。

use super::*;

impl Parser {
    pub(super) fn parse_view_as_rest(&mut self) -> Result<ViewAsAst, CompileError> {
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
}
