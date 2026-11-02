//! Recursive-descent parser.

use crate::ast::*;
use crate::lexer::{lex, Token, TokenKind};
use crate::CompileError;
use content_schema::Span;
mod active;
mod view_as;

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
        let mut charlotte = None;
        let mut persevere = None;
        let mut forever = None;
        let mut temporary = None;

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
                TokenKind::Ident(value) if value == "charlotte" && charlotte.is_none() => { charlotte = Some(true); self.bump(); }
                TokenKind::Ident(value) if value == "persevere" && persevere.is_none() => { persevere = Some(true); self.bump(); }
                TokenKind::Ident(value) if value == "forever" && forever.is_none() => { forever = Some(true); self.bump(); }
                TokenKind::Ident(value) if value == "temporary" && temporary.is_none() => { temporary = Some(true); self.bump(); }
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
            charlotte, persevere, forever, temporary,
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
