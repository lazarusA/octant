//! English phrasings of the verbs.

use super::Phrases;
use super::Verb::*;

pub(super) static PHRASES: Phrases = Phrases {
    verbs: &[
        (Select, &["select", "choose", "pick"]),
        (
            Plot,
            &["plot", "show", "draw", "display", "visualize", "visualise"],
        ),
        (Play, &["play", "start", "animate", "resume"]),
        (Pause, &["pause", "stop"]),
        (Next, &["next", "next step", "forward", "step forward"]),
        (
            Previous,
            &["previous", "previous step", "back", "step back", "go back"],
        ),
        (
            Colormap,
            &[
                "colormap",
                "color map",
                "colour map",
                "colors",
                "colours",
                "palette",
            ],
        ),
        (
            ResetView,
            &["reset view", "reset the view", "reset zoom", "reset camera"],
        ),
    ],
    then: &["then", "and then", "after that"],
};
