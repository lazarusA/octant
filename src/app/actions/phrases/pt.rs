//! Portuguese phrasings of the verbs.

use super::Phrases;
use super::Verb::*;

pub(super) static PHRASES: Phrases = Phrases {
    verbs: &[
        (
            Select,
            &["seleciona", "selecione", "selecionar", "escolhe", "escolha"],
        ),
        (
            Plot,
            &[
                "plota",
                "plote",
                "plotar",
                "mostra",
                "mostre",
                "mostrar",
                "desenha",
                "desenhe",
                "exibe",
                "exiba",
                "visualiza",
                "visualize",
            ],
        ),
        (
            Play,
            &[
                "reproduz",
                "reproduzir",
                "toca",
                "inicia",
                "inicie",
                "anima",
                "anime",
                "começa",
                "comece",
            ],
        ),
        (Pause, &["pausa", "pausar", "pare", "parar"]),
        (
            Next,
            &["próximo", "próximo passo", "seguinte", "avança", "avance"],
        ),
        (Previous, &["anterior", "passo anterior", "volta", "volte"]),
        (
            Colormap,
            &["mapa de cores", "escala de cores", "paleta", "cores"],
        ),
        (
            ResetView,
            &[
                "redefine a vista",
                "redefinir a vista",
                "restaura a vista",
                "restaurar a vista",
            ],
        ),
    ],
    then: &["depois", "então", "e depois"],
};
