//! English phrasings of the verbs.

use super::Phrases;
use super::Verb::*;

pub(super) static PHRASES: Phrases = Phrases {
    verbs: &[
        (Goto, &["go to", "jump to", "set"]),
        (Open, &["open", "load", "switch to"]),
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
    months: [
        &["january", "jan"],
        &["february", "feb"],
        &["march"],
        &["april", "apr"],
        &["may"],
        &["june", "jun"],
        &["july", "jul"],
        &["august", "aug"],
        &["september", "sep", "sept"],
        &["october", "oct"],
        &["november", "nov"],
        &["december", "dec"],
    ],
    axes: [
        &["time", "date"],
        &["latitude"],
        &["longitude"],
        &["level", "depth", "height"],
    ],
    catalog: &["catalog", "catalogue"],
    then: &["then", "and then", "after that"],
};
