//! Spanish phrasings of the verbs.

use super::Phrases;
use super::Verb::*;

pub(super) static PHRASES: Phrases = Phrases {
    verbs: &[
        (Hide, &["oculta", "ocultar", "esconde"]),
        (Reveal, &["muestra de nuevo", "vuelve a mostrar"]),
        (Remove, &["quita", "elimina", "borra"]),
        (Range, &["rango", "rango de color", "límites"]),
        (Export, &["exporta", "exportar", "guarda la imagen"]),
        (
            Describe,
            &["describe", "qué es", "información", "háblame de"],
        ),
        (Search, &["busca", "buscar", "encuentra"]),
        (List, &["lista", "enumera", "qué variables"]),
        (Goto, &["ve a", "ir a", "salta a", "pon"]),
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
    months: [
        &["enero"],
        &["febrero"],
        &["marzo"],
        &["abril"],
        &["mayo"],
        &["junio"],
        &["julio"],
        &["agosto"],
        &["septiembre", "setiembre"],
        &["octubre"],
        &["noviembre"],
        &["diciembre"],
    ],
    axes: [
        &["tiempo", "fecha"],
        &["latitud"],
        &["longitud"],
        &["nivel", "profundidad", "altura"],
    ],
    dataset: &["conjunto de datos", "datos"],
    overlay: &["superpuesto", "superposición", "encima", "como capa"],
    well: &["pozo"],
    field: &["campo"],
    catalog: &["catálogo", "catalogo"],
    then: &["luego", "después", "y luego", "entonces"],
};
