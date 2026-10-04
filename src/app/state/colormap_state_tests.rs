use super::*;
use eframe::Storage as _;
use std::collections::HashMap;

#[derive(Default)]
struct MemoryStorage(HashMap<String, String>);

impl eframe::Storage for MemoryStorage {
    fn get_string(&self, key: &str) -> Option<String> {
        self.0.get(key).cloned()
    }
    fn set_string(&mut self, key: &str, value: String) {
        self.0.insert(key.to_string(), value);
    }
    fn remove_string(&mut self, key: &str) {
        self.0.remove(key);
    }
    fn flush(&mut self) {}
}

fn spec(name: &str, colors: &str) -> CustomColormapSpec {
    CustomColormapSpec {
        name: name.into(),
        colors: colors.into(),
        ..Default::default()
    }
}

fn stored(raw: &str) -> MemoryStorage {
    let mut storage = MemoryStorage::default();
    storage.set_string(STORAGE_KEY, raw.into());
    storage
}

fn has_custom(app: &OctantApp, name: &str) -> bool {
    app.colormaps.custom.iter().any(|s| s.name == name)
}

#[test]
fn only_custom_colormaps_survive_a_restart() {
    let _registry = registry::test_lock();
    let mut app = OctantApp::default();
    let Ok(custom_id) = app.add_custom_colormap(spec("persist_test_map", "black, white")) else {
        panic!("valid spec rejected");
    };
    app.active_colormap = custom_id;
    app.colormaps.reversed = true;

    let mut storage = MemoryStorage::default();
    app.save_colormap_prefs(&mut storage);

    let mut restarted = OctantApp::default();
    restarted.load_colormap_prefs(Some(&storage));
    assert_eq!(restarted.active_colormap, registry::default_id());
    assert!(!restarted.colormaps.reversed);
    assert!(has_custom(&restarted, "persist_test_map"));
    registry::remove_custom("custom:persist_test_map");
}

#[test]
fn older_prefs_with_a_saved_selection_are_ignored() {
    let _registry = registry::test_lock();
    let storage = stored(
        "(active: \"cmocean:thermal\", reversed: true, custom: [(name: \"legacy_test_map\", \
         colors: \"red, blue\", interpolation: Linear, blend: Oklab, classes: 0)])",
    );
    let mut app = OctantApp::default();
    app.load_colormap_prefs(Some(&storage));
    assert_eq!(app.active_colormap, registry::default_id());
    assert!(!app.colormaps.reversed);
    assert!(has_custom(&app, "legacy_test_map"));
    registry::remove_custom("custom:legacy_test_map");
}

#[test]
fn specs_missing_fields_load_with_defaults() {
    let _registry = registry::test_lock();
    let storage = stored("(custom: [(name: \"partial_test_map\", colors: \"red, blue\")])");
    let mut app = OctantApp::default();
    app.load_colormap_prefs(Some(&storage));
    assert!(has_custom(&app, "partial_test_map"));
    registry::remove_custom("custom:partial_test_map");
}

#[test]
fn specs_that_no_longer_build_are_kept_on_save() {
    let _registry = registry::test_lock();
    let mut storage = stored(
        "(custom: [(name: \"broken_test_map\", colors: \"definitely-not-a-color\"), \
         (name: \"fine_test_map\", colors: \"red, blue\")])",
    );
    let mut app = OctantApp::default();
    app.load_colormap_prefs(Some(&storage));
    assert!(!has_custom(&app, "broken_test_map"));
    assert!(has_custom(&app, "fine_test_map"));
    assert_eq!(registry::find("custom:broken_test_map"), None);

    app.save_colormap_prefs(&mut storage);
    let saved = storage.get_string(STORAGE_KEY).unwrap_or_default();
    assert!(saved.contains("broken_test_map"), "saved: {saved}");
    assert!(saved.contains("fine_test_map"), "saved: {saved}");
    registry::remove_custom("custom:fine_test_map");
}

#[test]
fn unreadable_prefs_are_backed_up_before_being_replaced() {
    let _registry = registry::test_lock();
    let raw = "not ron at all {";
    let mut storage = stored(raw);
    let mut app = OctantApp::default();
    app.load_colormap_prefs(Some(&storage));
    app.save_colormap_prefs(&mut storage);
    assert_eq!(storage.get_string(UNREADABLE_KEY).as_deref(), Some(raw));
}

#[test]
fn renaming_while_editing_replaces_the_original() {
    let _registry = registry::test_lock();
    let mut app = OctantApp::default();
    let original = spec("rename_from_test_map", "red, blue");
    let old_key = original.key();
    assert!(app.add_custom_colormap(original).is_ok());
    let Ok(id) = app.save_custom_colormap(spec("rename_to_test_map", "red, blue"), Some(&old_key))
    else {
        panic!("rename rejected");
    };
    assert_eq!(registry::find(&old_key), None);
    assert_eq!(registry::find("custom:rename_to_test_map"), Some(id));
    assert!(!has_custom(&app, "rename_from_test_map"));
    assert!(has_custom(&app, "rename_to_test_map"));
    registry::remove_custom("custom:rename_to_test_map");
}

#[test]
fn removing_a_map_forgets_the_hovered_row() {
    let _registry = registry::test_lock();
    let mut app = OctantApp::default();
    let Ok(id) = app.add_custom_colormap(spec("hover_test_map", "red, blue")) else {
        panic!("valid spec rejected");
    };
    app.colormaps.picker.last_hovered = Some(id);
    app.remove_custom_colormap("custom:hover_test_map");
    assert_eq!(app.colormaps.picker.last_hovered, None);
}

#[test]
fn smooth_toggle_switches_to_the_twin_only_when_one_exists() {
    let _registry = registry::test_lock();
    let mut app = OctantApp::default();
    let set1 = registry::find("colorbrewer:Set1").unwrap_or(0);
    app.active_colormap = set1;
    assert_eq!(app.effective_colormap(), set1);
    app.colormaps.smooth = true;
    assert_eq!(
        Some(app.effective_colormap()),
        registry::smooth_variant(set1)
    );
    app.active_colormap = registry::default_id();
    assert_eq!(
        app.effective_colormap(),
        registry::default_id(),
        "continuous maps are unaffected"
    );
    assert!(!OctantApp::default().colormaps.smooth, "smooth starts off");
}

#[test]
fn hidden_smooth_toggle_does_not_change_the_preview() {
    let _registry = registry::test_lock();
    let mut app = OctantApp::default();
    let set1 = registry::find("colorbrewer:Set1").unwrap_or(0);
    app.active_colormap = registry::default_id();
    app.colormaps.smooth = true;
    app.preview_colormap = Some(set1);
    assert_eq!(app.effective_colormap(), set1);
}
