//! `Toasts`: merging, bounding, hover pausing, expiry and fading.

use std::time::Duration;

use web_time::Instant;

use super::model::{Notice, Severity};
use super::queue::{FADE, MAX_TOASTS, Toasts, opacity};

fn notice(severity: Severity, title: &str) -> Notice {
    Notice::new(severity, title, "detail")
}

fn titles(toasts: &Toasts) -> Vec<&str> {
    toasts
        .items()
        .iter()
        .map(|t| t.notice.title.as_str())
        .collect()
}

#[test]
fn a_repeated_notice_counts_and_restarts_its_time() {
    let mut toasts = Toasts::default();
    let t0 = Instant::now();
    toasts.push(notice(Severity::Error, "Couldn't load data"));
    toasts.tick(t0, &[]);
    toasts.tick(t0 + Duration::from_secs(10), &[]);
    toasts.push(notice(Severity::Error, "Couldn't load data"));
    let toast = &toasts.items()[0];
    assert_eq!((toasts.items().len(), toast.count), (1, 2));
    assert_eq!(toast.remaining, Severity::Error.lifetime());
}

#[test]
fn notices_differing_in_detail_stay_apart() {
    let mut toasts = Toasts::default();
    toasts.push(Notice::new(
        Severity::Warning,
        "Coordinates unavailable",
        "a",
    ));
    toasts.push(Notice::new(
        Severity::Warning,
        "Coordinates unavailable",
        "b",
    ));
    assert_eq!(toasts.items().len(), 2);
}

#[test]
fn a_full_stack_drops_info_first_then_warnings_then_errors() {
    let mut toasts = Toasts::default();
    toasts.push(notice(Severity::Error, "e1"));
    toasts.push(notice(Severity::Success, "s1"));
    toasts.push(notice(Severity::Warning, "w1"));
    toasts.push(notice(Severity::Error, "e2"));
    toasts.push(notice(Severity::Error, "e3"));
    assert_eq!(titles(&toasts), ["e1", "w1", "e2", "e3"], "info goes first");
    toasts.push(notice(Severity::Error, "e4"));
    assert_eq!(titles(&toasts), ["e1", "e2", "e3", "e4"], "then warnings");
    toasts.push(notice(Severity::Error, "e5"));
    assert_eq!(
        titles(&toasts),
        ["e2", "e3", "e4", "e5"],
        "then the oldest error"
    );
    assert_eq!(toasts.items().len(), MAX_TOASTS);
}

#[test]
fn toasts_expire_after_their_lifetime() {
    let mut toasts = Toasts::default();
    let t0 = Instant::now();
    toasts.push(notice(Severity::Success, "Saved"));
    toasts.push(notice(Severity::Error, "Export failed"));
    toasts.tick(t0, &[]);
    toasts.tick(t0 + Severity::Success.lifetime(), &[]);
    assert_eq!(titles(&toasts), ["Export failed"]);
    toasts.tick(t0 + Severity::Error.lifetime(), &[]);
    assert!(toasts.is_empty());
}

#[test]
fn hovering_pauses_a_toast() {
    let mut toasts = Toasts::default();
    let t0 = Instant::now();
    toasts.push(notice(Severity::Info, "a"));
    toasts.push(notice(Severity::Info, "b"));
    toasts.tick(t0, &[]);
    toasts.tick(t0 + Duration::from_secs(10), &[true, false]);
    assert_eq!(titles(&toasts), ["a"], "the hovered toast stays");
    assert_eq!(toasts.items()[0].remaining, Severity::Info.lifetime());
}

#[test]
fn dismissing_removes_one_toast() {
    let mut toasts = Toasts::default();
    toasts.push(notice(Severity::Info, "a"));
    toasts.push(notice(Severity::Info, "b"));
    let id = toasts.items()[0].id;
    toasts.dismiss(id);
    assert_eq!(titles(&toasts), ["b"]);
}

#[test]
fn repaints_wait_for_the_fade_then_run_every_frame() {
    let mut toasts = Toasts::default();
    assert_eq!(toasts.next_repaint(), None);
    let t0 = Instant::now();
    toasts.push(notice(Severity::Info, "a"));
    toasts.tick(t0, &[]);
    assert_eq!(
        toasts.next_repaint(),
        Some(Severity::Info.lifetime() - FADE)
    );
    toasts.tick(t0 + Severity::Info.lifetime() - FADE / 2, &[]);
    assert_eq!(toasts.next_repaint(), Some(Duration::ZERO));
}

#[test]
fn opacity_falls_over_the_fade() {
    assert_eq!(opacity(Duration::from_secs(3)), 1.0);
    assert_eq!(opacity(FADE / 2), 0.5);
    assert_eq!(opacity(Duration::ZERO), 0.0);
}

#[test]
fn reports_reach_the_app_in_order() {
    super::report(Severity::Warning, "toast test report 1", "");
    super::report(Severity::Error, "toast test report 2", "why");
    let drained: Vec<_> = super::drain_reports()
        .into_iter()
        .filter(|n| n.title.starts_with("toast test report"))
        .collect();
    assert_eq!(drained.len(), 2);
    assert_eq!(drained[1].detail, "why");
}
