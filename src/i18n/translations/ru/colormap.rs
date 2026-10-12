use crate::i18n::keys::ColormapKey;

pub fn text(key: ColormapKey) -> Option<&'static str> {
    Some(match key {
        ColormapKey::Sequential => "Последовательная",
        ColormapKey::Diverging => "Расходящаяся",
        ColormapKey::Cyclic => "Циклическая",
        ColormapKey::Categorical => "Категориальная",
        ColormapKey::Other => "Другая",
        ColormapKey::InterpolationLinear => "Линейная",
        ColormapKey::BlendLinearRgb => "Линейный RGB",
        ColormapKey::FilterAll => "Все",
        ColormapKey::FamilyCustom => "Свои",
        ColormapKey::FamilyTitle => "Семейство: {}",
        ColormapKey::Colormap => "Цветовая шкала",
        ColormapKey::Smooth => "Сгладить",
        ColormapKey::SmoothHover => "Смешать цвета этой палитры в непрерывный градиент (Oklab)",
        ColormapKey::Reversed => "Обратная",
        ColormapKey::SearchHint => "Поиск цветовых шкал...",
        ColormapKey::NoMatches => "Нет подходящих цветовых шкал",
        ColormapKey::ShowColorbar => "Показать легенду",
        ColormapKey::BoxTransparency => "Прозрачность рамки",
        ColormapKey::CustomColormap => "Своя цветовая шкала",
        ColormapKey::Name => "Название",
        ColormapKey::Colors => "Цвета",
        ColormapKey::ColorsHover => {
            "Hex, rgb(), hsl() или названия цветов CSS, при желании с позициями"
        }
        ColormapKey::Interpolation => "Интерполяция",
        ColormapKey::Blend => "Смешение",
        ColormapKey::Classes => "Классы",
        ColormapKey::ClassesHover => "0 оставляет градиент непрерывным",
        ColormapKey::RenameAndApply => "Переименовать и применить",
        ColormapKey::SaveAndApply => "Сохранить и применить",
        ColormapKey::SaveAsNew => "Сохранить как новую",
        ColormapKey::NameTaken => "Сохранённая цветовая шкала с таким названием уже есть",
        ColormapKey::NameRequired => "Укажите название",
        ColormapKey::Saved => "Сохранено",
        ColormapKey::BrokenHover => {
            "Не строится в этой версии: измените, чтобы исправить, или удалите"
        }
        ColormapKey::Edit => "Изменить",
        ColormapKey::Delete => "Удалить",
        ColormapKey::DeleteHover => "Удалить цветовую шкалу",
        ColormapKey::FlipOrientation => "Сменить ориентацию",
        ColormapKey::RangeMinHover => {
            "Нижняя граница диапазона (Мин.). Потяните или нажмите для ввода."
        }
        ColormapKey::RangeMaxHover => {
            "Верхняя граница диапазона (Макс.). Потяните или нажмите для ввода."
        }
        ColormapKey::LowClipTitle => "Цвет нижнего отсечения (< Мин.)",
        ColormapKey::LowClipHover => {
            "Цвет нижнего отсечения (< Мин.). Нажмите, чтобы выбрать цвет."
        }
        ColormapKey::HighClipTitle => "Цвет верхнего отсечения (> Макс.)",
        ColormapKey::HighClipHover => {
            "Цвет верхнего отсечения (> Макс.). Нажмите, чтобы выбрать цвет."
        }
        ColormapKey::HoveredValue => "Знач.: {}",
        ColormapKey::TitleHover => "Заголовок легенды. Нажмите, чтобы изменить.",
        ColormapKey::ColorPicker => "Выбор цвета",
        ColormapKey::Copy => "Копировать",
        ColormapKey::CopyHover => "Скопировать значения цвета в буфер обмена",
        ColormapKey::AlphaNotANumber => "'{}' - не число",
        ColormapKey::AlphaOutOfRange => "{} вне диапазона 0..1",
        ColormapKey::AlphaMixedForms => "используйте либо значения, либо точки position:alpha",
        ColormapKey::AlphaUnsorted => "позиции точек должны возрастать",
        ColormapKey::PrefsUnreadable => "Не удалось прочитать настройки цветовых шкал",
        ColormapKey::PrefsBackedUp => "Резервная копия сохранена в `{}`",
        ColormapKey::PrefsNotBackedUp => "Резервная копия настроек цветовых шкал не создана",
        ColormapKey::BackupsFull => "Все {} резервных копий настроек цветовых шкал заняты",
        ColormapKey::CustomNotLoaded => "Своя цветовая шкала не загружена",
        ColormapKey::CustomDoesNotBuild => "`{}` не строится: {}",
        ColormapKey::NotRegistered => "Цветовая шкала {} не зарегистрирована",
        ColormapKey::BundledUnreadable => "Не удалось прочитать встроенные цветовые шкалы",
        ColormapKey::FallingBackToGray => "{}; используется шкала серого",
    })
}
