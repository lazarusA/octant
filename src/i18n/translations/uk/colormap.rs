use crate::i18n::keys::ColormapKey;

pub fn text(key: ColormapKey) -> Option<&'static str> {
    Some(match key {
        ColormapKey::Sequential => "Послідовна",
        ColormapKey::Diverging => "Розбіжна",
        ColormapKey::Cyclic => "Циклічна",
        ColormapKey::Categorical => "Категоріальна",
        ColormapKey::Other => "Інша",
        ColormapKey::InterpolationLinear => "Лінійна",
        ColormapKey::BlendLinearRgb => "Лінійний RGB",
        ColormapKey::FilterAll => "Усі",
        ColormapKey::FamilyCustom => "Власні",
        ColormapKey::FamilyTitle => "Родина: {}",
        ColormapKey::Smooth => "Плавна",
        ColormapKey::SmoothHover => "Змішати кольори цієї палітри в неперервний градієнт (Oklab)",
        ColormapKey::Reversed => "Обернена",
        ColormapKey::SearchHint => "Пошук колірних шкал...",
        ColormapKey::NoMatches => "Немає відповідних колірних шкал",
        ColormapKey::ShowColorbar => "Показати кольорову смугу",
        ColormapKey::ChangeColormapHover => "Натисніть, щоб змінити колірну шкалу",
        ColormapKey::ColorSettingsHover => "Налаштування кольору",
        ColormapKey::BoxTransparency => "Прозорість рамки",
        ColormapKey::CustomColormap => "Власна колірна шкала",
        ColormapKey::Name => "Назва",
        ColormapKey::Colors => "Кольори",
        ColormapKey::ColorsHover => {
            "Hex, rgb(), hsl() або назви кольорів CSS, за бажанням із позиціями"
        }
        ColormapKey::Interpolation => "Інтерполяція",
        ColormapKey::Blend => "Змішування",
        ColormapKey::Classes => "Класи",
        ColormapKey::ClassesHover => "0 залишає градієнт неперервним",
        ColormapKey::RenameAndApply => "Перейменувати й застосувати",
        ColormapKey::SaveAndApply => "Зберегти й застосувати",
        ColormapKey::SaveAsNew => "Зберегти як нову",
        ColormapKey::NameTaken => "Збережена колірна шкала з такою назвою вже існує",
        ColormapKey::NameRequired => "Потрібна назва",
        ColormapKey::Saved => "Збережено",
        ColormapKey::BrokenHover => {
            "Не будується в цій версії: відредагуйте, щоб виправити, або видаліть"
        }
        ColormapKey::Edit => "Редагувати",
        ColormapKey::Delete => "Видалити",
        ColormapKey::DeleteHover => "Видалити колірну шкалу",
        ColormapKey::FlipOrientation => "Змінити орієнтацію",
        ColormapKey::RangeMinHover => {
            "Нижня межа діапазону (Мін.). Потягніть, щоб змінити, або клацніть, щоб ввести."
        }
        ColormapKey::RangeMaxHover => {
            "Верхня межа діапазону (Макс.). Потягніть, щоб змінити, або клацніть, щоб ввести."
        }
        ColormapKey::LowClipTitle => "Колір нижнього відсікання (< Мін.)",
        ColormapKey::LowClipHover => {
            "Колір нижнього відсікання (< Мін.). Клацніть, щоб вибрати колір."
        }
        ColormapKey::HighClipTitle => "Колір верхнього відсікання (> Макс.)",
        ColormapKey::HighClipHover => {
            "Колір верхнього відсікання (> Макс.). Клацніть, щоб вибрати колір."
        }
        ColormapKey::HoveredValue => "Знач.: {}",
        ColormapKey::TitleHover => "Заголовок кольорової смуги. Клацніть, щоб редагувати.",
        ColormapKey::ColorPicker => "Вибір кольору",
        ColormapKey::Copy => "Копіювати",
        ColormapKey::CopyHover => "Копіювати значення кольору в буфер обміну",
        ColormapKey::AlphaNotANumber => "'{}' не є числом",
        ColormapKey::AlphaOutOfRange => "{} поза межами 0..1",
        ColormapKey::AlphaMixedForms => "використовуйте або значення, або точки position:alpha",
        ColormapKey::AlphaUnsorted => "позиції точок мають зростати",
        ColormapKey::PrefsUnreadable => "Не вдалося прочитати налаштування колірних шкал",
        ColormapKey::PrefsBackedUp => "Резервну копію збережено як `{}`",
        ColormapKey::PrefsNotBackedUp => "Резервну копію налаштувань колірних шкал не створено",
        ColormapKey::BackupsFull => "Усі {} резервних копій налаштувань колірних шкал зайняті",
        ColormapKey::CustomNotLoaded => "Власну колірну шкалу не завантажено",
        ColormapKey::CustomDoesNotBuild => "`{}` не будується: {}",
        ColormapKey::NotRegistered => "Колірну шкалу {} не зареєстровано",
        ColormapKey::BundledUnreadable => "Не вдалося прочитати вбудовані колірні шкали",
        ColormapKey::FallingBackToGray => "{}; перехід на відтінки сірого",
    })
}
