use crate::i18n::keys::ColormapKey;

pub fn text(key: ColormapKey) -> Option<&'static str> {
    Some(match key {
        ColormapKey::Sequential => "Secuencial",
        ColormapKey::Diverging => "Divergente",
        ColormapKey::Cyclic => "Cíclico",
        ColormapKey::Categorical => "Categórico",
        ColormapKey::Other => "Otro",
        ColormapKey::InterpolationLinear => "Lineal",
        ColormapKey::BlendLinearRgb => "RGB lineal",
        ColormapKey::FilterAll => "Todos",
        ColormapKey::FamilyCustom => "Personalizados",
        ColormapKey::FamilyTitle => "Familia: {}",
        ColormapKey::Smooth => "Suavizar",
        ColormapKey::SmoothHover => {
            "Fundir los colores de esta paleta en un degradado continuo (Oklab)"
        }
        ColormapKey::Reversed => "Invertido",
        ColormapKey::SearchHint => "Buscar mapas de color...",
        ColormapKey::NoMatches => "Ningún mapa de color coincide",
        ColormapKey::ShowColorbar => "Mostrar barra de color",
        ColormapKey::ChangeColormapHover => "Haga clic para cambiar el mapa de color",
        ColormapKey::BoxTransparency => "Transparencia del recuadro",
        ColormapKey::CustomColormap => "Mapa de color personalizado",
        ColormapKey::Name => "Nombre",
        ColormapKey::Colors => "Colores",
        ColormapKey::ColorsHover => {
            "Hex, rgb(), hsl() o nombres de color CSS, opcionalmente con posiciones"
        }
        ColormapKey::Interpolation => "Interpolación",
        ColormapKey::Blend => "Mezcla",
        ColormapKey::Classes => "Clases",
        ColormapKey::ClassesHover => "0 mantiene el degradado continuo",
        ColormapKey::RenameAndApply => "Renombrar y aplicar",
        ColormapKey::SaveAndApply => "Guardar y aplicar",
        ColormapKey::SaveAsNew => "Guardar como nuevo",
        ColormapKey::NameTaken => "Ya hay un mapa de color guardado con este nombre",
        ColormapKey::NameRequired => "El nombre es obligatorio",
        ColormapKey::Saved => "Guardado",
        ColormapKey::BrokenHover => {
            "No se puede construir con esta versión: edítelo para corregirlo o elimínelo"
        }
        ColormapKey::Edit => "Editar",
        ColormapKey::Delete => "Eliminar",
        ColormapKey::DeleteHover => "Eliminar mapa de color",
        ColormapKey::FlipOrientation => "Cambiar orientación",
        ColormapKey::RangeMinHover => {
            "Extremo inferior del rango (Mín.). Arrastre para ajustar o haga clic para escribir."
        }
        ColormapKey::RangeMaxHover => {
            "Extremo superior del rango (Máx.). Arrastre para ajustar o haga clic para escribir."
        }
        ColormapKey::LowClipTitle => "Color bajo el rango (< Mín.)",
        ColormapKey::LowClipHover => {
            "Color bajo el rango (< Mín.). Haga clic para seleccionar el color."
        }
        ColormapKey::HighClipTitle => "Color sobre el rango (> Máx.)",
        ColormapKey::HighClipHover => {
            "Color sobre el rango (> Máx.). Haga clic para seleccionar el color."
        }
        ColormapKey::HoveredValue => "Val.: {}",
        ColormapKey::TitleHover => "Título de la barra de color. Haga clic para editarlo.",
        ColormapKey::ColorPicker => "Selector de color",
        ColormapKey::Copy => "Copiar",
        ColormapKey::CopyHover => "Copiar los valores del color al portapapeles",
        ColormapKey::AlphaNotANumber => "'{}' no es un número",
        ColormapKey::AlphaOutOfRange => "{} está fuera de 0..1",
        ColormapKey::AlphaMixedForms => "use valores o paradas posición:alfa, no ambos",
        ColormapKey::AlphaUnsorted => "las posiciones de las paradas deben ser crecientes",
        ColormapKey::PrefsUnreadable => "No se pueden leer las preferencias de mapas de color",
        ColormapKey::PrefsBackedUp => "Copia de seguridad guardada en `{}`",
        ColormapKey::PrefsNotBackedUp => {
            "No se hizo copia de seguridad de las preferencias de mapas de color"
        }
        ColormapKey::BackupsFull => {
            "Las {} copias de seguridad de preferencias de mapas de color están en uso"
        }
        ColormapKey::CustomNotLoaded => "No se cargó el mapa de color personalizado",
        ColormapKey::CustomDoesNotBuild => "`{}` no se puede construir: {}",
        ColormapKey::NotRegistered => "El mapa de color {} no se registró",
        ColormapKey::BundledUnreadable => "No se pueden leer los mapas de color incluidos",
        ColormapKey::FallingBackToGray => "{}; se usa la escala de grises",
    })
}
