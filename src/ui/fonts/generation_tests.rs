//! A font rebuild reads as pending only in the context that made it, until
//! egui uses the new fonts the next pass.

use crate::i18n::Language;

#[test]
fn a_font_rebuild_is_pending_only_in_its_own_context() {
    let run = |ctx: &egui::Context| {
        let mut output = ctx.run_ui(egui::RawInput::default(), |_| {});
        output.textures_delta.clear();
    };
    let (busy, fresh) = (egui::Context::default(), egui::Context::default());
    (0..5).for_each(|_| run(&busy));
    super::FontEngine::on_language_changed(&busy, Language::English);
    assert_eq!(
        super::generation(&busy),
        u64::MAX,
        "pending until next pass"
    );
    assert_ne!(
        super::generation(&fresh),
        u64::MAX,
        "other contexts unaffected"
    );
    run(&busy);
    assert_ne!(
        super::generation(&busy),
        u64::MAX,
        "in use from the next pass"
    );
}
