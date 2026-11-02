//! A named invocation owns its lifecycle; continuation stages own only operation receipts.
use super::*;
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum InvocationPhase {Before,Begin,Body,End,After,Omitted}
impl InvocationPhase {
 pub fn tag(self)->u8{match self{Self::Before=>0,Self::Begin=>1,Self::Body=>2,Self::End=>3,Self::After=>4,Self::Omitted=>5}}
 pub fn from_tag(t:u8)->Option<Self>{Some(match t{0=>Self::Before,1=>Self::Begin,2=>Self::Body,3=>Self::End,4=>Self::After,5=>Self::Omitted,_=>return None})}
 pub fn kind(self)->EventKind{match self{Self::Before=>EventKind::SkillBefore,Self::Begin=>EventKind::SkillBegin,Self::Body=>EventKind::SkillInvocation,Self::End=>EventKind::SkillEnd,Self::After=>EventKind::SkillAfter,Self::Omitted=>EventKind::SkillOmitted}}
}
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct InvocationData {pub owner:PlayerId,pub def:SkillDefId,pub window:EventId,pub source:Option<PlayerId>,pub target:PlayerId,pub phase:InvocationPhase,pub cancelled:bool}
#[derive(Debug,Clone,PartialEq,Eq)]
pub enum NativeInvocationProgress {
 Ready, BeforeInvocation{context:InvocationData}, AwaitInvocation{child:EventId}, Finished,
 Body, HolderAsk{candidates:Vec<CardId>}, HolderPay{cards:Vec<CardId>}, HolderLoss{count:u32}, HolderAfterLog{count:u32},
 SourceAsk{player:PlayerId,count:u32,candidates:Vec<CardId>}, SourceLoss, JidingCard{child:EventId},
}
pub fn uses_native_invocation(effect:NativeDamageEffect)->bool {matches!(effect,NativeDamageEffect::Zhangwu|NativeDamageEffect::Jiding)}
