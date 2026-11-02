use super::*;
pub(super) fn push_native_invocation(
    h: &mut HashSink,
    p: &engine_native_kernel::event::NativeInvocationProgress,
) {
    use engine_native_kernel::event::NativeInvocationProgress as P;
    match p {
        P::YuanceAsk => push_u8(h, 12),
        P::JidianAsk { targets } => {
            push_u8(h, 13);
            push_players(h, targets);
        }
        P::AwaitJudge {
            child,
            target,
            purpose,
        } => {
            push_u8(h, 14);
            push_u32(h, *child);
            push_u8(h, *target);
            push_u8(
                h,
                match purpose {
                    engine_native_kernel::event::JudgePurpose::Yuance(option) => option.tag(),
                    engine_native_kernel::event::JudgePurpose::Jidian => 2,
                },
            );
        }
        P::BodyDone => push_u8(h, 15),
        P::JidianAfterLog { target } => {
            push_u8(h, 16);
            push_u8(h, *target);
        }
        P::QinglongAsk {
            target,
            hand,
            equip,
        } => {
            push_u8(h, 17);
            push_u8(h, *target);
            push_cards(h, hand);
            push_cards(h, equip)
        }
        P::QinglongLog {
            target,
            card,
            public,
        } => {
            push_u8(h, 18);
            push_u8(h, *target);
            push_u32(h, *card);
            push_u8(h, u8::from(*public))
        }
        P::QinglongLoss {
            target,
            card,
            public,
        } => {
            push_u8(h, 19);
            push_u8(h, *target);
            push_u32(h, *card);
            push_u8(h, u8::from(*public))
        }
        P::Ready => push_u8(h, 0),
        P::BeforeInvocation { context } => {
            push_u8(h, 1);
            push_event_data(h, &EventData::SkillInvocation(context.clone()));
        }
        P::AwaitInvocation { child, admission } => {
            push_event_data(h, &EventData::SkillInvocation(admission.clone()));
            push_u8(h, 2);
            push_u32(h, *child);
        }
        P::Finished => push_u8(h, 3),
        P::Body => push_u8(h, 4),
        P::HolderAsk { candidates } => {
            push_u8(h, 5);
            push_cards(h, candidates);
        }
        P::HolderPay { cards } => {
            push_u8(h, 6);
            push_cards(h, cards);
        }
        P::HolderLoss { count } => {
            push_u8(h, 7);
            push_u32(h, *count);
        }
        P::HolderAfterLog { count } => {
            push_u8(h, 8);
            push_u32(h, *count);
        }
        P::SourceAsk {
            player,
            count,
            candidates,
        } => {
            push_u8(h, 9);
            push_u8(h, *player);
            push_u32(h, *count);
            push_cards(h, candidates);
        }
        P::SourceLoss => push_u8(h, 10),
        P::JidingCard { child } => {
            push_u8(h, 11);
            push_u32(h, *child);
        }
    }
}
