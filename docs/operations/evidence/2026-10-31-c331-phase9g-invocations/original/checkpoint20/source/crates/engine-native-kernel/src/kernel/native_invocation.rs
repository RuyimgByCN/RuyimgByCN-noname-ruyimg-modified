//! Finite native bodies run inside an actual named invocation in the ordinary stack.
use super::*;
use crate::event::{InvocationData,InvocationPhase as PhaseOfSkill,NativeInvocationProgress as P,NativeDamageEffect as Effect};
#[derive(Clone)]
pub struct NativeInvocation {pub owner:PlayerId,pub def:crate::ids::SkillDefId,pub window:crate::event::DamageWindowKind,pub progress:P}
pub fn advance_invocation(frame:&mut crate::event::Frame){
 let EventData::SkillInvocation(data)=&mut frame.data else{return};
 let next=match data.phase{
 PhaseOfSkill::Before if data.cancelled=>Some(PhaseOfSkill::Omitted),
 PhaseOfSkill::Before=>Some(PhaseOfSkill::Begin),
 PhaseOfSkill::Begin if data.cancelled=>Some(PhaseOfSkill::End),
 PhaseOfSkill::Begin=>Some(PhaseOfSkill::Body),PhaseOfSkill::Body=>Some(PhaseOfSkill::End),PhaseOfSkill::End=>Some(PhaseOfSkill::After),PhaseOfSkill::After|PhaseOfSkill::Omitted=>None};
 if let Some(phase)=next{data.phase=phase;frame.kind=data.phase.kind();frame.stage=if phase==PhaseOfSkill::Body{crate::event::Stage::Intrinsic}else{crate::event::Stage::Commit};frame.planned=false;frame.intrinsic_done=false;}else{frame.stage=crate::event::Stage::Done;}
}
impl NativeInvocation {
 fn effect(&self,ctx:&Ctx<'_>)->Effect{ctx.catalog.skills[self.def as usize].behavior.native_damage_effect(self.window.event_kind()).expect("validated admitted definition")}
 fn emit_invocation(&mut self,ctx:&mut Ctx<'_>,context:InvocationData)->Flow{self.progress=P::AwaitInvocation{child:ctx.res.next_event_id};ctx.emit(EventData::SkillInvocation(context));Flow::Yield}
 fn log_child(&self,ctx:&mut Ctx<'_>,begin:bool){ctx.emit(EventData::SkillLog{owner:self.owner,def:self.def,begin});}
 fn he(ctx:&Ctx<'_>,player:PlayerId)->Vec<crate::ids::CardId>{let p=ctx.core.player(player);p.hand.iter().chain(p.equips.iter()).copied().collect()}
 fn choice(player:PlayerId,cards:&[crate::ids::CardId],count:Option<u32>)->Flow{Flow::WaitChoice{observer:player,spec:ChoiceSpec{kind:ChoiceKind::SelectCards,prompt:if count.is_some(){"c331_zhangwu_source"}else{"c331_zhangwu_holder"},options:cards.iter().copied().map(ChoiceOption::Card).collect(),min:count.map(|n|n as usize).unwrap_or(1),max:count.map(|n|n as usize).unwrap_or(cards.len()),allow_cancel:true}}}
 fn selected(ctx:&Ctx<'_>,candidates:&[crate::ids::CardId])->Vec<crate::ids::CardId>{ctx.answer().iter().filter_map(|a|match a{ChoiceOption::Card(c)if candidates.contains(c)=>Some(*c),_=>None}).collect()}
 fn discard(ctx:&mut Ctx<'_>,player:PlayerId,cards:&[crate::ids::CardId]){
  let mut discarded=Vec::new();for &card in cards{let p=ctx.core.player(player);if p.hand.contains(&card)||p.equips.contains(&card){ctx.take_owned_card(player,card);ctx.to_discard(card);discarded.push(card);}}
  if !discarded.is_empty(){ctx.log(LogEvent::CardDiscarded{player,cards:discarded.clone()});ctx.emit(EventData::CardsDiscarded{player,cards:discarded});}
 }
 fn body(&mut self,ctx:&mut Ctx<'_>)->Flow{
  let id=ctx.current_frame_id();let EventData::SkillInvocation(context)=ctx.res.frame(id).expect("invocation").data.clone()else{return Flow::Done};
  match self.progress.clone(){
   P::Body=>{
    if !ctx.core.player(self.owner).alive{return Flow::Done;}
    match self.effect(ctx){
     Effect::Zhangwu=>{let cards=Self::he(ctx,self.owner);if cards.is_empty(){return Flow::Done;}self.progress=P::HolderAsk{candidates:cards.clone()};Self::choice(self.owner,&cards,None)},
     Effect::Jiding=>{let Some(source)=context.source.filter(|p|ctx.core.player(*p).alive)else{return Flow::Done};if ctx.core.deck.is_empty(){return Flow::Done;}
      self.progress=P::JidingCard{child:ctx.res.next_event_id};ctx.use_virtual_card(self.owner,CardKind::LeiSha,vec![source]);Flow::Yield},
     _=>unreachable!("only finite implemented invocation bodies")
    }
   },
   P::HolderAsk{candidates}=>{let cards=Self::selected(ctx,&candidates);if cards.is_empty(){return Flow::Done;}self.progress=P::HolderPay{cards};ctx.log(LogEvent::SkillTriggered{skill:ctx.catalog.skill_name(self.def),owner:self.owner});self.log_child(ctx,true);Flow::Yield},
   P::HolderPay{cards}=>{self.progress=P::HolderLoss{count:cards.len()as u32};if ctx.core.player(self.owner).alive{Self::discard(ctx,self.owner,&cards);}Flow::Yield},
   P::HolderLoss{count}=>{self.progress=P::HolderAfterLog{count};self.log_child(ctx,false);Flow::Yield},
   P::HolderAfterLog{count}=>{
    if !ctx.core.player(self.owner).alive{return Flow::Done;}let Some(source)=context.source.filter(|p|ctx.core.player(*p).alive)else{return Flow::Done};let cards=Self::he(ctx,source);
    if cards.len()<count as usize{self.progress=P::SourceLoss;ctx.emit(EventData::Damage(crate::machine::damage_data(Some(self.owner),source,i32::try_from(count).expect("validated bounded card payment"))));Flow::Yield}else{self.progress=P::SourceAsk{player:source,count,candidates:cards.clone()};Self::choice(source,&cards,Some(count))}
   },
   P::SourceAsk{count,candidates,..}=>{let Some(source)=context.source else{return Flow::Done};self.progress=P::SourceLoss;if !ctx.core.player(self.owner).alive||!ctx.core.player(source).alive{return Flow::Done;}let cards=Self::selected(ctx,&candidates);if cards.is_empty(){ctx.emit(EventData::Damage(crate::machine::damage_data(Some(self.owner),source,i32::try_from(count).expect("validated bounded card payment"))));}else{Self::discard(ctx,source,&cards);}Flow::Yield},
   P::SourceLoss=>Flow::Done,
   P::JidingCard{..}=>{self.progress=P::SourceLoss;if ctx.core.player(self.owner).alive{if ctx.core.skills.remove(&(self.owner,self.def)).is_some(){
 ctx.log(LogEvent::SkillToggled{player:self.owner,skill:ctx.catalog.skill_name(self.def),enabled:false});
 ctx.emit(EventData::SkillToggled{player:self.owner,skill:self.def,enabled:false,temporary:false});
}}Flow::Yield},
   _=>Flow::Done,
  }
 }
}
impl SkillMachine for NativeInvocation{
 fn fork(&self)->Box<dyn SkillMachine>{Box::new(self.clone())}
 fn snapshot(&self)->crate::snap::MachineSnapshot{crate::snap::MachineSnapshot::NativeInvocation{owner:self.owner,def:self.def,window:self.window,progress:self.progress.clone()}}
 fn step(&mut self,ctx:&mut Ctx<'_>)->Flow{
  match self.progress.clone(){
   P::Ready=>{let EventData::DamageWindow{root,..}=ctx.res.frame(ctx.current_frame_id()).expect("trigger window").data else{return Flow::Done};let d=ctx.res.frame(root).expect("damage root").damage();let context=InvocationData{owner:self.owner,def:self.def,window:ctx.current_frame_id(),source:d.source,target:d.target,phase:PhaseOfSkill::Before,cancelled:false};if self.effect(ctx)==Effect::Jiding{self.progress=P::BeforeInvocation{context};self.log_child(ctx,true);Flow::Yield}else{self.emit_invocation(ctx,context)}},
   P::BeforeInvocation{context}=>self.emit_invocation(ctx,context),
   P::AwaitInvocation{..}=>{self.progress=P::Finished;if self.effect(ctx)==Effect::Jiding{self.log_child(ctx,false);Flow::Yield}else{Flow::Done}},
   P::Finished=>Flow::Done,
   _=>self.body(ctx),
  }
 }
}
