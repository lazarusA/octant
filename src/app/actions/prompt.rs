//! The names given to the speech model as its prompt, so it writes them as
//! the dataset spells them (`t2m`, not "T 2 M"): the active dataset's
//! variables and dimensions, each once.

use crate::data::DatasetMetadata;

/// The longest prompt built; the model keeps fewer tokens anyway.
const MAX_CHARS: usize = 600;

/// The active dataset's variable and dimension names, comma separated.
pub(crate) fn speech_prompt(meta: Option<&DatasetMetadata>) -> String {
    let Some(meta) = meta else {
        return String::new();
    };
    let mut seen = std::collections::HashSet::new();
    let variables = meta.variables.iter().filter(|v| !v.is_coarser_level());
    let names = variables
        .map(|v| v.display_name())
        .chain(
            meta.variables
                .iter()
                .flat_map(|v| v.dimension_names.iter().map(String::as_str)),
        )
        .filter(|name| seen.insert(*name));
    let mut prompt = String::new();
    for name in names {
        if prompt.len() + name.len() + 2 > MAX_CHARS {
            break;
        }
        if !prompt.is_empty() {
            prompt.push_str(", ");
        }
        prompt.push_str(name);
    }
    prompt
}

#[cfg(test)]
mod tests {
    use super::speech_prompt;
    use crate::app::test_support::memory_app;

    #[test]
    fn the_prompt_names_variables_then_dimensions_once() {
        let (_, meta) = memory_app();
        let prompt = speech_prompt(Some(&meta));
        for name in ["t2m", "sst", "elev", "rgb", "time", "lat", "lon", "band"] {
            assert_eq!(prompt.matches(name).count(), 1, "{name} in '{prompt}'");
        }
        assert_eq!(speech_prompt(None), "");
    }
}
