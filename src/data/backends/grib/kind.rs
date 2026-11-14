//! What separates messages of one parameter at one level type into variables: their
//! statistics, derived ensemble product, probability, percentile, anomaly and wave band
//! (gribberish's `message_kind`, `python/src/dataset.rs`). Plain instantaneous fields are
//! `instant`.

use gribberish::message_metadata::MessageMetadata;
use gribberish::templates::product::tables::ProbabilityType;

/// The kind of the message `meta` describes, as one token.
pub fn kind(meta: &MessageMetadata) -> String {
    let mut tokens: Vec<String> = Vec::new();
    if let Some(stat) = &meta.statistical_process {
        let hours = (meta.forecast_end_date)
            .map(|end| end.signed_duration_since(meta.forecast_date).num_hours())
            .filter(|h| *h > 0);
        tokens.push(match hours {
            Some(h) => format!("{}{h}h", stat.abbv()),
            None => stat.abbv(),
        });
    }
    if let Some(derived) = &meta.derived_forecast_type {
        tokens.push(derived.abbv());
    }
    // Shortcut: thresholds and percentiles make variables of their own, not a dimension;
    // stack them along one if files with many thresholds (NBM) call for it.
    if let Some(prob) = &meta.probability_type {
        tokens.push(match threshold(meta, prob) {
            Some(t) => format!("{}{t}", prob.abbv()),
            None => prob.abbv(),
        });
    } else if let Some(p) = meta.percentile_value {
        tokens.push(format!("pctl{p}"));
    }
    if meta.is_anomaly {
        tokens.push("anom".into());
    }
    if let Some((lower, upper)) = meta.wave_period_range {
        let bound = |v: Option<f64>| v.map_or(String::new(), |v| format!("{v:.0}"));
        tokens.push(format!("per{}-{}s", bound(lower), bound(upper)));
    }
    tokens.retain(|t| !t.is_empty());
    match tokens.is_empty() {
        true => "instant".into(),
        false => tokens.join("_"),
    }
}

/// The limit that tells probability messages of one field apart: "above/below upper limit"
/// products carry it in the upper limit, the others in the lower one; between-limit
/// products in both.
fn threshold(meta: &MessageMetadata, prob: &ProbabilityType) -> Option<String> {
    let (lower, upper) = (meta.probability_lower_limit, meta.probability_upper_limit);
    match prob {
        ProbabilityType::BetweenLimits | ProbabilityType::BetweenLimitsInclusive => {
            Some(format!("{}_{}", short(lower?), short(upper?)))
        }
        ProbabilityType::AboveUpperLimit | ProbabilityType::BelowUpperLimit => {
            upper.or(lower).map(short)
        }
        _ => lower.or(upper).map(short),
    }
}

/// `v` without the noise of its decimal scaling (`0.0127`, not `0.012700000000000001`).
fn short(v: f64) -> String {
    ((v * 1e6).round() / 1e6).to_string()
}
