//! Opening datasets by name and the commands waiting on them.

use super::resolve::{Resolution, Vocabulary};
use super::source_names::SourceRef;
use super::{AppAction, Done};
use crate::app::OctantApp;
use crate::app::test_support::memory_app;

/// Waits for the inspection under way, then runs the commands waiting on it.
fn settle(app: &mut OctantApp) -> Vec<super::Outcome> {
    let ctx = egui::Context::default();
    let deadline = web_time::Instant::now() + std::time::Duration::from_secs(10);
    while app.metadata_rx.is_some() {
        assert!(web_time::Instant::now() < deadline, "never settled");
        app.poll_inspection(&ctx);
        std::thread::yield_now();
    }
    app.resume_commands()
}

#[test]
fn samples_and_catalog_entries_are_one_dataset_by_any_of_their_names() {
    let vocab = Vocabulary::new(None, &[]);
    let Resolution::Resolved(SourceRef::Entry(entry)) = vocab.source("seasfire") else {
        panic!("seasfire is a catalog entry");
    };
    assert_eq!(
        vocab.source("SeasFire Cube"),
        Resolution::Resolved(SourceRef::Entry(entry))
    );
    // The landing pill and the catalog entry of one store are one choice.
    let Resolution::Resolved(SourceRef::Entry(entry)) = vocab.source("procedural 4d") else {
        panic!("the procedural pill is a catalog entry");
    };
    assert_eq!(entry.store, "procedural://volume4d");
}

#[test]
fn the_commands_after_open_wait_for_the_dataset() {
    let (mut app, _) = memory_app();
    let first = app.run_utterance("open procedural 4d, then plot");
    assert!(
        matches!(first.as_slice(), [Ok(Done::Opening(_))]),
        "{first:?}"
    );
    let rest = settle(&mut app);
    assert!(
        matches!(rest.as_slice(), [Ok(Done::Plotted { .. })]),
        "{rest:?}"
    );
    assert_eq!(app.selected.store_target, "procedural://volume4d");
}

#[test]
fn a_dataset_that_fails_to_load_drops_the_commands_after_it() {
    let (mut app, _) = memory_app();
    let open = AppAction::OpenSource {
        source: "/no/such/raster.tif".into(),
    };
    assert!(matches!(app.dispatch(open), Ok(Done::Opening(_))));
    // Said while it loads: waits behind it.
    assert_eq!(app.run_utterance("plot"), vec![]);
    let rest = settle(&mut app);
    assert!(rest.is_empty(), "{rest:?}");
    assert!(app.selected.metadata.is_none(), "it failed");
}

#[test]
fn open_catalog_shows_it() {
    let (mut app, _) = memory_app();
    assert_eq!(
        app.run_utterance("open the catalog"),
        vec![Ok(Done::CatalogOpened)]
    );
    assert!(app.layout.show_catalog_window);
}
