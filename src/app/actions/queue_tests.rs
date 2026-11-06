//! The queue runs without the voice bar: link commands run with it closed,
//! a closed bar drops only voice commands, a new link replaces the last.

use super::Origin;
use crate::app::OctantApp;
use crate::app::actions::AppAction;
use crate::app::actions::tests::active_app;

fn plot_t2m() -> Vec<AppAction> {
    vec![
        AppAction::SelectVariable { path: "t2m".into() },
        AppAction::Plot { plot_type: None },
    ]
}

fn origins(app: &OctantApp) -> Vec<Origin> {
    app.commands.waiting.iter().map(|(o, _)| *o).collect()
}

#[test]
fn link_commands_run_with_the_voice_bar_closed() {
    let (mut app, _) = active_app();
    assert!(!app.voice.is_open());
    app.queue_actions(plot_t2m(), Origin::Link);
    app.run_commands(&egui::Context::default());
    assert!(app.layers.base.data.matrix.is_some(), "t2m is drawn");
    assert!(app.commands.waiting.is_empty());
}

#[test]
fn a_closed_voice_bar_drops_only_voice_commands() {
    let (mut app, _) = active_app();
    // Waits behind the dataset it opens.
    app.run_utterance("open procedural 4d, then plot");
    app.queue_actions(plot_t2m(), Origin::Link);
    assert!(origins(&app).contains(&Origin::Voice));
    app.run_commands(&egui::Context::default());
    assert_eq!(origins(&app), vec![Origin::Link, Origin::Link]);
}

#[test]
fn a_new_link_replaces_the_link_commands_waiting() {
    let (mut app, _) = active_app();
    app.run_utterance("open procedural 4d");
    app.queue_actions(plot_t2m(), Origin::Link);
    app.queue_actions(vec![AppAction::Play], Origin::Link);
    assert_eq!(app.commands.waiting.len(), 1);
}

#[test]
fn a_failing_link_command_is_a_toast_and_drops_the_rest() {
    let (mut app, _) = active_app();
    let mut actions = vec![AppAction::SelectVariable {
        path: "nope".into(),
    }];
    actions.extend(plot_t2m());
    app.queue_actions(actions, Origin::Link);
    app.run_commands(&egui::Context::default());
    assert!(!app.toasts.is_empty());
    assert!(app.commands.waiting.is_empty());
    assert!(app.layers.base.data.matrix.is_none(), "nothing plotted");
}

#[test]
fn a_link_replacing_one_whose_dataset_fails_still_runs() {
    let (mut app, _) = crate::app::test_support::memory_app();
    let ctx = egui::Context::default();
    let failing = AppAction::OpenSource {
        source: "/no/such/raster.tif".into(),
    };
    app.queue_actions(vec![failing, AppAction::Play], Origin::Link);
    app.run_commands(&ctx);
    assert!(app.metadata_rx.is_some(), "the first link's dataset loads");
    let open = AppAction::OpenSource {
        source: "procedural://volume4d".into(),
    };
    app.queue_actions(vec![open], Origin::Link);
    let deadline = web_time::Instant::now() + std::time::Duration::from_secs(10);
    while app.selected.store_target != "procedural://volume4d" {
        assert!(
            web_time::Instant::now() < deadline,
            "the second link was dropped"
        );
        app.poll_inspection(&ctx);
        app.run_commands(&ctx);
        std::thread::yield_now();
    }
}

#[test]
fn a_plot_last_in_the_queue_leaves_nothing_waiting_on_it() {
    let (mut app, _) = active_app();
    app.queue_actions(plot_t2m(), Origin::Link);
    app.run_commands(&egui::Context::default());
    assert!(
        !app.commands.plotting,
        "a plot started by hand later must not hold questions"
    );
}
