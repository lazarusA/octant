use crate::i18n::keys::CommonKey;

pub fn text(key: CommonKey) -> Option<&'static str> {
    Some(match key {
        CommonKey::Dataset => "Conjunto de datos",
        CommonKey::Variables => "Variables",
        CommonKey::Dimensions => "Dimensiones",
        CommonKey::Settings => "Configuración",
        CommonKey::About => "Acerca de",
        CommonKey::Cache => "Caché",
        CommonKey::Language => "Idioma",
        CommonKey::Loading => "Cargando...",
        CommonKey::CloseEsc => "Cerrar (Esc)",
        CommonKey::Cancel => "Cancelar",
        CommonKey::Save => "Guardar",
        CommonKey::Reset => "Restablecer",
        CommonKey::Move => "Mover",
        CommonKey::MoveHover => "Arrastre para mover; doble clic para restablecer",
        CommonKey::Light => "Claro",
        CommonKey::Dark => "Oscuro",
        CommonKey::SwitchToLightHover => "Cambiar al modo claro",
        CommonKey::SwitchToDarkHover => "Cambiar al modo oscuro",
        CommonKey::ToggleHeroHover => {
            "Mostrar u ocultar la pantalla de inicio (clic derecho para Acerca de)"
        }
        CommonKey::AboutOctantHover => "Acerca de Octant",
        CommonKey::ToggleVariableControlsHover => "Mostrar u ocultar el panel de variables",
        CommonKey::FetchingProgress => "Descargando {} / {} ({}%)",
        CommonKey::Abort => "Abortar",
        CommonKey::AbortHover => "Interrumpir y abortar la transferencia de datos en curso",
    })
}
