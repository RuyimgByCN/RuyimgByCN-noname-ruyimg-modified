//! Fixed tags for named policy identity in direct native extension locks.
use super::*;

impl NamedProgram {
    pub(crate) fn hash_semantics(&self, h: &mut blake3::Hasher) {
        h.update(&(self.groups.len() as u64).to_le_bytes());
        for group in &self.groups {
            h.update(&(group.len() as u64).to_le_bytes());
            for clause in group {
                h.update(&[u8::from(clause.neg)]);
                match &clause.atom {
                    Atom::EvPlayer => { h.update(&0u16.to_le_bytes()); }
                    Atom::EvActor => { h.update(&1u16.to_le_bytes()); }
                    Atom::OwnerAlive => { h.update(&2u16.to_le_bytes()); }
                    Atom::InTargets => { h.update(&3u16.to_le_bytes()); }
                    Atom::UserOther => { h.update(&4u16.to_le_bytes()); }
                    Atom::Wounded => { h.update(&5u16.to_le_bytes()); }
                    Atom::Healthy => { h.update(&6u16.to_le_bytes()); }
                    Atom::Linked => { h.update(&7u16.to_le_bytes()); }
                    Atom::Unlinked => { h.update(&8u16.to_le_bytes()); }
                    Atom::FaceDown => { h.update(&9u16.to_le_bytes()); }
                    Atom::FaceUp => { h.update(&10u16.to_le_bytes()); }
                    Atom::HandEmpty => { h.update(&11u16.to_le_bytes()); }
                    Atom::HandNonEmpty => { h.update(&12u16.to_le_bytes()); }
                    Atom::HandBelowMax => { h.update(&13u16.to_le_bytes()); }
                    Atom::HandAboveHp => { h.update(&14u16.to_le_bytes()); }
                    Atom::HandAtLeastHp => { h.update(&15u16.to_le_bytes()); }
                    Atom::HandAtLeast(n) => { h.update(&16u16.to_le_bytes()); h.update(&[*n]); }
                    Atom::HpBelow(n) => { h.update(&17u16.to_le_bytes()); h.update(&[*n]); }
                    Atom::HpEq(n) => { h.update(&18u16.to_le_bytes()); h.update(&[*n]); }
                    Atom::JudgeNonEmpty => { h.update(&19u16.to_le_bytes()); }
                    Atom::EquipNonEmpty => { h.update(&20u16.to_le_bytes()); }
                    Atom::EquipEmpty => { h.update(&21u16.to_le_bytes()); }
                    Atom::HandOrEquip => { h.update(&22u16.to_le_bytes()); }
                    Atom::HandOrEquipEmpty => { h.update(&23u16.to_le_bytes()); }
                    Atom::OtherAlive => { h.update(&24u16.to_le_bytes()); }
                    Atom::OtherHand => { h.update(&25u16.to_le_bytes()); }
                    Atom::OtherWounded => { h.update(&26u16.to_le_bytes()); }
                    Atom::CardSha => { h.update(&27u16.to_le_bytes()); }
                    Atom::CardTrick => { h.update(&28u16.to_le_bytes()); }
                    Atom::CardBasic => { h.update(&29u16.to_le_bytes()); }
                    Atom::CardEquipKind => { h.update(&30u16.to_le_bytes()); }
                    Atom::CardJuedou => { h.update(&31u16.to_le_bytes()); }
                    Atom::CardRed => { h.update(&32u16.to_le_bytes()); }
                    Atom::CardBlack => { h.update(&33u16.to_le_bytes()); }
                    Atom::CardSpade => { h.update(&34u16.to_le_bytes()); }
                    Atom::CardHeart => { h.update(&35u16.to_le_bytes()); }
                    Atom::CardClub => { h.update(&36u16.to_le_bytes()); }
                    Atom::CardDiamond => { h.update(&37u16.to_le_bytes()); }
                    Atom::CardJiu => { h.update(&38u16.to_le_bytes()); }
                    Atom::CardShan => { h.update(&39u16.to_le_bytes()); }
                    Atom::CardNanman => { h.update(&40u16.to_le_bytes()); }
                    Atom::CardWanjian => { h.update(&41u16.to_le_bytes()); }
                    Atom::CardWuzhong => { h.update(&42u16.to_le_bytes()); }
                    Atom::CardDelayed => { h.update(&43u16.to_le_bytes()); }
                    Atom::Fire => { h.update(&44u16.to_le_bytes()); }
                    Atom::Thunder => { h.update(&45u16.to_le_bytes()); }
                    Atom::Damage => { h.update(&46u16.to_le_bytes()); }
                    Atom::PhasePlay => { h.update(&47u16.to_le_bytes()); }
                    Atom::PhaseDraw => { h.update(&48u16.to_le_bytes()); }
                    Atom::PhaseDiscard => { h.update(&49u16.to_le_bytes()); }
                    Atom::PhaseJudge => { h.update(&50u16.to_le_bytes()); }
                    Atom::PhaseEnd => { h.update(&51u16.to_le_bytes()); }
                    Atom::PhaseBegin => { h.update(&52u16.to_le_bytes()); }
                    Atom::TurnDmg0 => { h.update(&53u16.to_le_bytes()); }
                    Atom::TurnDmg => { h.update(&54u16.to_le_bytes()); }
                    Atom::TurnDmgAtLeast(n) => { h.update(&55u16.to_le_bytes()); h.update(&[*n]); }
                    Atom::BtePos => { h.update(&56u16.to_le_bytes()); }
                    Atom::SourceAlive => { h.update(&57u16.to_le_bytes()); }
                    Atom::DistLe1 => { h.update(&58u16.to_le_bytes()); }
                    Atom::TiedHand => { h.update(&59u16.to_le_bytes()); }
                    Atom::TiedHp => { h.update(&60u16.to_le_bytes()); }
                    Atom::TiedEquip => { h.update(&61u16.to_le_bytes()); }
                    Atom::NotUniqueHp => { h.update(&62u16.to_le_bytes()); }
                    Atom::Chain => { h.update(&63u16.to_le_bytes()); }
                    Atom::Direct => { h.update(&64u16.to_le_bytes()); }
                    Atom::Dying => { h.update(&65u16.to_le_bytes()); }
                    Atom::Death => { h.update(&66u16.to_le_bytes()); }
                    Atom::DiscardEvt => { h.update(&67u16.to_le_bytes()); }
                    Atom::Moved => { h.update(&68u16.to_le_bytes()); }
                    Atom::Responded => { h.update(&69u16.to_le_bytes()); }
                    Atom::Used => { h.update(&70u16.to_le_bytes()); }
                    Atom::OutsideTurn => { h.update(&71u16.to_le_bytes()); }
                    Atom::InTurn => { h.update(&72u16.to_le_bytes()); }
                    Atom::Lost => { h.update(&73u16.to_le_bytes()); }
                    Atom::LostAtLeast(n) => { h.update(&74u16.to_le_bytes()); h.update(&[*n]); }
                    Atom::Batch => { h.update(&75u16.to_le_bytes()); }
                    Atom::PileNonEmpty => { h.update(&76u16.to_le_bytes()); }
                    Atom::PileEmpty => { h.update(&77u16.to_le_bytes()); }
                    Atom::MarkPos => { h.update(&78u16.to_le_bytes()); }
                    Atom::DrawNotFixed => { h.update(&79u16.to_le_bytes()); }
                    Atom::Once { scope } => { h.update(&80u16.to_le_bytes()); h.update(&[*scope]); }
                    Atom::FlagSet(s) => { h.update(&81u16.to_le_bytes()); h.update(&(s.len() as u64).to_le_bytes()); h.update(s.as_bytes()); }
                    Atom::FlagUnset(s) => { h.update(&82u16.to_le_bytes()); h.update(&(s.len() as u64).to_le_bytes()); h.update(s.as_bytes()); }
                    Atom::Tag(s) => { h.update(&83u16.to_le_bytes()); h.update(&(s.len() as u64).to_le_bytes()); h.update(s.as_bytes()); }
                    Atom::FirstHit => { h.update(&84u16.to_le_bytes()); }
                    Atom::SecondHit => { h.update(&85u16.to_le_bytes()); }
                    Atom::AfterFirst => { h.update(&86u16.to_le_bytes()); }
                    Atom::OddHand => { h.update(&87u16.to_le_bytes()); }
                    Atom::Lord => { h.update(&88u16.to_le_bytes()); }
                    Atom::Female => { h.update(&89u16.to_le_bytes()); }
                    Atom::Male => { h.update(&90u16.to_le_bytes()); }
                    Atom::SexSame => { h.update(&91u16.to_le_bytes()); }
                    Atom::GroupSame => { h.update(&92u16.to_le_bytes()); }
                    Atom::Transformed => { h.update(&93u16.to_le_bytes()); }
                    Atom::MinHp => { h.update(&94u16.to_le_bytes()); }
                    Atom::TargetAlive => { h.update(&95u16.to_le_bytes()); }
                    Atom::TargetHandNonEmpty => { h.update(&96u16.to_le_bytes()); }
                    Atom::TargetHandEmpty => { h.update(&97u16.to_le_bytes()); }
                    Atom::CanDiscard(n) => { h.update(&98u16.to_le_bytes()); h.update(&[*n]); }
                    Atom::CountAtLeast(n) => { h.update(&99u16.to_le_bytes()); h.update(&[*n]); }
                    Atom::SingleTarget => { h.update(&100u16.to_le_bytes()); }
                    Atom::MultiTarget => { h.update(&101u16.to_le_bytes()); }
                    Atom::MovedFromOwner => { h.update(&102u16.to_le_bytes()); }
                    Atom::Gained => { h.update(&103u16.to_le_bytes()); }
                    Atom::FactionIs(n) => { h.update(&104u16.to_le_bytes()); h.update(&[*n]); }
                    Atom::ShaOrTrick => { h.update(&105u16.to_le_bytes()); }
                    Atom::ShaOrJuedou => { h.update(&106u16.to_le_bytes()); }
                    Atom::BasicOrTrick => { h.update(&107u16.to_le_bytes()); }
                    Atom::ElementalOrTrick => { h.update(&108u16.to_le_bytes()); }
                    Atom::RedOrJiu => { h.update(&109u16.to_le_bytes()); }
                    Atom::JuedouOrRedSha => { h.update(&110u16.to_le_bytes()); }
                }
            }
        }
        h.update(&(self.consume.len() as u64).to_le_bytes());
        for (key, scope) in &self.consume { h.update(&(key.len() as u64).to_le_bytes()); h.update(key.as_bytes()); h.update(&[*scope]); }
        h.update(&[u8::from(self.count_hits)]);
    }
}
