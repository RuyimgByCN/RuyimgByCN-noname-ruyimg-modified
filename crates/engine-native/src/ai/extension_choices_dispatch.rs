//! 扩展包挂起询问（`OpenChoice`）按批次子模块顺序分派的默认回退链。

use super::*;
use crate::choice::{ChoiceKind, ChoiceOption};

#[allow(clippy::too_many_arguments)]
pub(super) fn plan_extension_choice_fallback(
    catalog: &crate::catalog::Catalog,
    input: &crate::view::PolicyInput,
    id: u64,
    kind: ChoiceKind,
    prompt: &'static str,
    options: &[ChoiceOption],
    min: usize,
    max: usize,
    allow_cancel: bool,
) -> Option<GameAction> {
    extension_choices_misc::plan_misc_choice_action(
        catalog,
        input,
        id,
        prompt,
        options,
        min,
        max,
        allow_cancel,
    )
    .or_else(|| {
        extension_choices_u05_hearth::plan_hearth_choice_action(
            catalog,
            input,
            id,
            kind,
            prompt,
            options,
            min,
            max,
            allow_cancel,
        )
    })
    .or_else(|| {
        extension_choices_u05_6b::plan_hearth_6b_choice_action(
            catalog,
            input,
            id,
            kind,
            prompt,
            options,
            min,
            max,
            allow_cancel,
        )
    })
    .or_else(|| {
        extension_choices_u05_1::plan_u05_b1_choice_action(
            catalog,
            input,
            id,
            prompt,
            options,
            min,
            max,
            allow_cancel,
        )
    })
    .or_else(|| {
        extension_choices_l10::plan_l10_choice_action(
            catalog,
            input,
            id,
            prompt,
            options,
            min,
            max,
            allow_cancel,
        )
    })
    .or_else(|| {
        extension_choices_l28::plan_l28_choice_action(
            catalog,
            input,
            id,
            prompt,
            options,
            min,
            max,
            allow_cancel,
        )
    })
    .or_else(|| {
        extension_choices_u01::plan_u01_choice_action(
            catalog,
            input,
            id,
            prompt,
            options,
            min,
            max,
            allow_cancel,
        )
    })
    .or_else(|| {
        extension_choices_l11::plan_l11_choice_action(
            catalog,
            input,
            id,
            prompt,
            options,
            min,
            max,
            allow_cancel,
        )
    })
    .or_else(|| {
        extension_choices_l11_b2::plan_l11_b2_choice_action(
            catalog,
            input,
            id,
            prompt,
            options,
            min,
            max,
            allow_cancel,
        )
    })
    .or_else(|| {
        extension_choices_l11_b3::plan_l11_b3_choice_action(
            catalog,
            input,
            id,
            prompt,
            options,
            min,
            max,
            allow_cancel,
        )
    })
    .or_else(|| {
        extension_choices_l11_b4::plan_l11_b4_choice_action(
            catalog,
            input,
            id,
            prompt,
            options,
            min,
            max,
            allow_cancel,
        )
    })
    .or_else(|| {
        extension_choices_l11_b5::plan_l11_b5_choice_action(
            catalog,
            input,
            id,
            prompt,
            options,
            min,
            max,
            allow_cancel,
        )
    })
    .or_else(|| {
        extension_choices_u05_2::plan_u05_2_choice_action(
            catalog,
            input,
            id,
            prompt,
            options,
            min,
            max,
            allow_cancel,
        )
    })
    .or_else(|| {
        extension_choices_u05_ow::plan_u05_ow_choice_action(
            catalog,
            input,
            id,
            prompt,
            options,
            min,
            max,
            allow_cancel,
        )
    })
    .or_else(|| {
        extension_choices_u05_5a::plan_u05_5a_choice_action(
            catalog,
            input,
            id,
            prompt,
            options,
            min,
            max,
            allow_cancel,
        )
    })
}
