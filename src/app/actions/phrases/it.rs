//! Italian phrasings of the verbs.

use super::Phrases;
use super::Verb::*;

pub(super) static PHRASES: Phrases = Phrases {
    verbs: &[
        (Goto, &["vai a", "salta a", "imposta"]),
        (Open, &["apri", "aprire", "carica", "caricare", "passa a"]),
        (Select, &["seleziona", "selezionare", "scegli", "scegliere"]),
        (
            Plot,
            &[
                "mostra",
                "mostrare",
                "traccia",
                "disegna",
                "visualizza",
                "grafica",
            ],
        ),
        (Play, &["riproduci", "avvia", "anima", "inizia"]),
        (Pause, &["pausa", "metti in pausa", "ferma", "stop"]),
        (
            Next,
            &["successivo", "prossimo", "avanti", "passo successivo"],
        ),
        (Previous, &["precedente", "indietro", "passo precedente"]),
        (
            Colormap,
            &[
                "mappa di colori",
                "mappa colori",
                "scala di colori",
                "tavolozza",
                "palette",
                "colori",
            ],
        ),
        (
            ResetView,
            &[
                "reimposta la vista",
                "ripristina la vista",
                "reimposta vista",
            ],
        ),
    ],
    months: [
        &["gennaio"],
        &["febbraio"],
        &["marzo"],
        &["aprile"],
        &["maggio"],
        &["giugno"],
        &["luglio"],
        &["agosto"],
        &["settembre"],
        &["ottobre"],
        &["novembre"],
        &["dicembre"],
    ],
    axes: [
        &["tempo", "data"],
        &["latitudine"],
        &["longitudine"],
        &["livello", "profondità", "altezza"],
    ],
    catalog: &["catalogo"],
    then: &["poi", "quindi", "e poi", "dopo"],
};
