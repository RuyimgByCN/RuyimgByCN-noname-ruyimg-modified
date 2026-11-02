//! Internal invocation receipts; existing machine85 bytes are unchanged.
use super::*;
use crate::event::{InvocationData,InvocationPhase,NativeInvocationProgress as P};
pub(super)fn enc_context(w:&mut W,d:&InvocationData){w.u8(d.owner);w.u16(d.def);w.u32(d.window);enc_opt_pid(w,&d.source);w.u8(d.target);w.u8(d.phase.tag());w.boolean(d.cancelled);}
pub(super)fn dec_context(r:&mut R<'_>)->Result<InvocationData,ArchiveError>{Ok(InvocationData{owner:r.u8()?,def:r.u16()?,window:r.u32()?,source:dec_opt_pid(r)?,target:r.u8()?,phase:InvocationPhase::from_tag(r.u8()?).ok_or(ArchiveError::BadTag("invocation_phase",0))?,cancelled:r.boolean()?})}
pub(super)fn enc_progress(w:&mut W,p:&P){match p{
 P::Ready=>w.u8(0),P::BeforeInvocation{context}=>{w.u8(1);enc_context(w,context)},P::AwaitInvocation{child}=>{w.u8(2);w.u32(*child)},P::Finished=>w.u8(3),P::Body=>w.u8(4),
 P::HolderAsk{candidates}=>{w.u8(5);w.len(candidates.len());for c in candidates{w.u32(*c)}},P::HolderPay{cards}=>{w.u8(6);w.len(cards.len());for c in cards{w.u32(*c)}},P::HolderLoss{count}=>{w.u8(7);w.u32(*count)},P::HolderAfterLog{count}=>{w.u8(8);w.u32(*count)},P::SourceAsk{player,count,candidates}=>{w.u8(9);w.u8(*player);w.u32(*count);w.len(candidates.len());for c in candidates{w.u32(*c)}},P::SourceLoss=>w.u8(10),P::JidingCard{child}=>{w.u8(11);w.u32(*child)}
}}
pub(super)fn dec_progress(r:&mut R<'_>)->Result<P,ArchiveError>{Ok(match r.u8()?{
0=>P::Ready,1=>P::BeforeInvocation{context:dec_context(r)?},2=>P::AwaitInvocation{child:r.u32()?},3=>P::Finished,4=>P::Body,5=>P::HolderAsk{candidates:r.vec(|r|r.u32())?},6=>P::HolderPay{cards:r.vec(|r|r.u32())?},7=>P::HolderLoss{count:r.u32()?},8=>P::HolderAfterLog{count:r.u32()?},9=>P::SourceAsk{player:r.u8()?,count:r.u32()?,candidates:r.vec(|r|r.u32())?},10=>P::SourceLoss,11=>P::JidingCard{child:r.u32()?},t=>return Err(ArchiveError::BadTag("native_invocation_progress",t))})}
