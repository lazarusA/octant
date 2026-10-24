//! Spanish phrasings of the verbs.

use super::Phrases;
use super::Verb::*;

pub(super) static PHRASES: Phrases = Phrases {
    verbs: &[
        (Open, &["abre", "abrir", "carga", "cargar", "cambia a"]),
        (
            Select,
            &["selecciona", "seleccionar", "elige", "elegir", "escoge"],
        ),
        (
            Plot,
            &[
                "grafica",
                "graficar",
                "dibuja",
                "dibujar",
                "muestra",
                "mostrar",
                "visualiza",
                "visualizar",
                "traza",
                "representa",
            ],
        ),
        (
            Play,
            &[
                "reproduce",
                "reproducir",
                "inicia",
                "iniciar",
                "anima",
                "animar",
                "empieza",
                "continúa",
            ],
        ),
        (Pause, &["pausa", "pausar", "detén", "detener"]),
        (Next, &["siguiente", "paso siguiente", "adelante", "avanza"]),
        (
            Previous,
            &["anterior", "paso anterior", "atrás", "retrocede"],
        ),
        (
            Colormap,
            &["mapa de colores", "escala de colores", "paleta", "colores"],
        ),
        (
            ResetView,
            &[
                "restablece la vista",
                "restablecer la vista",
                "reinicia la vista",
                "restablecer vista",
            ],
        ),
    ],
    catalog: &["catálogo", "catalogo"],
    then: &["luego", "después", "y luego", "entonces"],
};
