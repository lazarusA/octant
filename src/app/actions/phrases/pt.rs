//! Portuguese phrasings of the verbs.

use super::Phrases;
use super::Verb::*;

pub(super) static PHRASES: Phrases = Phrases {
    verbs: &[
        (Goto, &["vai para", "ir para", "salta para", "põe"]),
        (
            Open,
            &[
                "abre",
                "abra",
                "abrir",
                "carrega",
                "carregue",
                "carregar",
                "muda para",
            ],
        ),
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
    months: [
        &["janeiro"],
        &["fevereiro"],
        &["março"],
        &["abril"],
        &["maio"],
        &["junho"],
        &["julho"],
        &["agosto"],
        &["setembro"],
        &["outubro"],
        &["novembro"],
        &["dezembro"],
    ],
    axes: [
        &["tempo", "data"],
        &["latitude"],
        &["longitude"],
        &["nível", "profundidade", "altura"],
    ],
    catalog: &["catálogo", "catalogo"],
    then: &["depois", "então", "e depois"],
};
