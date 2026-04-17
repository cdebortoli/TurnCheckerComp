use eframe::egui;

use crate::i18n::{I18n, I18nValue};
use crate::models::check_source_type::CheckSourceType;
use crate::models::CheckRepeatType;
use crate::ui::theme::Theme;

pub(crate) fn source_label(i18n: &I18n, source: CheckSourceType) -> String {
    match source {
        CheckSourceType::Game => i18n.t("source-game"),
        CheckSourceType::GlobalGame => i18n.t("source-global-game"),
        CheckSourceType::Blueprint => i18n.t("source-blueprint"),
        CheckSourceType::Turn => i18n.t("source-turn"),
    }
}

pub(crate) fn source_color(source: CheckSourceType, theme: &Theme) -> egui::Color32 {
    match source {
        CheckSourceType::Blueprint => theme.source_blueprint,
        CheckSourceType::Game => theme.source_game,
        CheckSourceType::GlobalGame => theme.source_global,
        CheckSourceType::Turn => theme.source_turn,
    }
}

pub(crate) fn repeat_editor_label(i18n: &I18n, repeat_case: CheckRepeatType) -> String {
    match repeat_case {
        CheckRepeatType::Everytime => i18n.t("repeat-everytime"),
        CheckRepeatType::Conditional(_) => i18n.t("repeat-conditional"),
        CheckRepeatType::Specific(_) => i18n.t("repeat-specific"),
        CheckRepeatType::Until(_) => i18n.t("repeat-until"),
    }
}

pub(crate) fn repeat_badge_label(i18n: &I18n, repeat_case: CheckRepeatType) -> String {
    match repeat_case {
        CheckRepeatType::Everytime => i18n.t("repeat-badge-every-turn"),
        CheckRepeatType::Conditional(value) => i18n.tr(
            "repeat-badge-conditional",
            &[("turn", I18nValue::from(value))],
        ),
        CheckRepeatType::Specific(value) => {
            i18n.tr("repeat-badge-specific", &[("turn", I18nValue::from(value))])
        }
        CheckRepeatType::Until(value) => {
            i18n.tr("repeat-badge-until", &[("turn", I18nValue::from(value))])
        }
    }
}

pub(crate) fn repeat_color(repeat_case: CheckRepeatType, theme: &Theme) -> egui::Color32 {
    match repeat_case {
        CheckRepeatType::Everytime => theme.repeat_everytime,
        CheckRepeatType::Conditional(_) => theme.repeat_conditional,
        CheckRepeatType::Specific(_) => theme.repeat_specific,
        CheckRepeatType::Until(_) => theme.repeat_until,
    }
}
