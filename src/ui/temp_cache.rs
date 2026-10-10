//! Values derived once and kept in egui temp memory until their key changes,
//! for per-frame UI text that is costly to format (coordinate labels).

/// The value kept under `id` for `key`; `make` computes (and stores) it when
/// nothing is kept or it was kept for another key.
pub fn cached<K, T>(ctx: &egui::Context, id: egui::Id, key: K, make: impl FnOnce() -> T) -> T
where
    K: PartialEq + Clone + Send + Sync + 'static,
    T: Clone + Send + Sync + 'static,
{
    let kept = ctx.data(|d| d.get_temp::<(K, T)>(id));
    if let Some((kept_key, value)) = kept
        && kept_key == key
    {
        return value;
    }
    let value = make();
    ctx.data_mut(|d| d.insert_temp(id, (key, value.clone())));
    value
}

#[cfg(test)]
mod tests {
    use super::cached;

    #[test]
    fn values_are_made_once_per_key() {
        let ctx = egui::Context::default();
        let id = egui::Id::new("temp_cache_test");
        let mut made = 0;
        let mut get = |key: u32| {
            cached(&ctx, id, key, || {
                made += 1;
                key * 10
            })
        };
        assert_eq!((get(1), get(1), get(2)), (10, 10, 20));
        drop(get);
        assert_eq!(made, 2);
    }
}
