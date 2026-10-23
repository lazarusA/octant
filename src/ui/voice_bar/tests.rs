use super::*;

#[test]
fn closed_until_toggled_and_closing_releases_the_microphone() {
    let mut voice = VoiceBar::default();
    assert!(!voice.is_open() && !voice.is_listening());
    // Opens even without a microphone (CI): the failure is a toast.
    voice.toggle();
    assert!(voice.is_open());
    voice.toggle();
    assert!(!voice.is_open() && !voice.is_listening());
}
