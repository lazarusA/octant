//! Spanish phrasings of the verbs.

use super::Phrases;
use super::Verb::*;

pub(super) static PHRASES: Phrases = Phrases {
    verbs: &[
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
    then: &["luego", "después", "y luego", "entonces"],
};
