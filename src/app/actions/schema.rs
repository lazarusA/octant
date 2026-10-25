//! The actions as tools a language model calls: one tool per action, its
//! name the action's tag, its input the action's fields and its
//! description the action's doc comment, all derived from `AppAction`
//! itself (`schemars`), so tools and actions never drift apart. The tools
//! are fixed; what the dataset holds goes in as each name argument's
//! `enum` (variables, dimensions, layers, plot types) while few enough to
//! list, else the argument stays free text and `search`/`describe_dataset`
//! find names. A call becomes an action by the same deserialization any
//! JSON action takes, so it is checked as every other input is.

use serde_json::{Map, Value, json};

use super::AppAction;

/// Names listed as an argument's `enum` at most; past it the argument is
/// free text.
// shortcut: a fixed cut-off, not yet measured on the catalog's datasets;
// measure the tool list's tokens there when the assistant (phase 5) lands.
const MAX_ENUM: usize = 200;

/// One tool: a name, what it does and the JSON Schema of its input.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

/// The names a dataset offers, listed in the tools while few enough.
#[derive(Debug, Default)]
pub struct Names {
    pub variables: Vec<String>,
    pub dimensions: Vec<String>,
    pub layers: Vec<String>,
    pub plot_types: Vec<String>,
}

/// Which arguments of which tools take which names.
fn names_of<'n>(names: &'n Names, tool: &str, argument: &str) -> Option<&'n [String]> {
    Some(match (tool, argument) {
        ("select_variable" | "describe_variable", "path") => &names.variables,
        ("set_dim" | "set_range" | "set_role", "dim") => &names.dimensions,
        ("set_layer_visible" | "remove_overlay" | "set_colormap" | "set_color_range", "layer") => {
            &names.layers
        }
        ("plot", "plot_type") => &names.plot_types,
        _ => return None,
    })
}

/// Every action as a tool, with `names` as the enums of the arguments that
/// take them.
pub fn tools(names: &Names) -> Vec<Tool> {
    let schema = serde_json::to_value(schemars::schema_for!(AppAction)).unwrap_or_default();
    let defs = schema.get("$defs").cloned().unwrap_or_default();
    let variants = schema
        .get("oneOf")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    variants
        .into_iter()
        .filter_map(|variant| tool_of(variant, &defs, names))
        .collect()
}

/// The tool of one variant's schema: its `action` tag taken out as the
/// name, references to shared definitions inlined.
fn tool_of(mut variant: Value, defs: &Value, names: &Names) -> Option<Tool> {
    inline_refs(&mut variant, defs);
    let object = variant.as_object_mut()?;
    let description = object
        .remove("description")
        .and_then(|d| d.as_str().map(str::to_string));
    let properties = object.get_mut("properties")?.as_object_mut()?;
    let name = properties
        .remove("action")?
        .get("const")?
        .as_str()?
        .to_string();
    for (argument, schema) in properties.iter_mut() {
        let listed =
            names_of(names, &name, argument).filter(|l| !l.is_empty() && l.len() <= MAX_ENUM);
        if let Some(list) = listed {
            let mut options = json!(list);
            // An optional argument may still be left out as null.
            let nullable = schema["type"]
                .as_array()
                .is_some_and(|t| t.contains(&json!("null")));
            if nullable && let Some(options) = options.as_array_mut() {
                options.push(Value::Null);
            }
            schema["enum"] = options;
        }
    }
    if let Some(required) = object.get_mut("required").and_then(Value::as_array_mut) {
        required.retain(|r| r != "action");
    }
    Some(Tool {
        description: description.unwrap_or_else(|| name.replace('_', " ")),
        name,
        input_schema: variant,
    })
}

/// How deep definitions may refer to definitions: the actions' arguments
/// nest one level; a definition referring to itself stops here.
const MAX_REF_DEPTH: usize = 8;

/// Replaces every `{"$ref": "#/$defs/X"}` in `value` by `X`'s definition.
fn inline_refs(value: &mut Value, defs: &Value) {
    inline_refs_within(value, defs, MAX_REF_DEPTH);
}

fn inline_refs_within(value: &mut Value, defs: &Value, depth: usize) {
    match value {
        Value::Object(map) => {
            let target = map
                .get("$ref")
                .and_then(Value::as_str)
                .and_then(|r| r.strip_prefix("#/$defs/"));
            if let Some(def) = target.and_then(|t| defs.get(t))
                && depth > 0
            {
                *value = def.clone();
                return inline_refs_within(value, defs, depth - 1);
            }
            map.values_mut()
                .for_each(|v| inline_refs_within(v, defs, depth));
        }
        Value::Array(items) => items
            .iter_mut()
            .for_each(|v| inline_refs_within(v, defs, depth)),
        _ => {}
    }
}

/// The action a call of tool `name` with `input` asks for, checked as any
/// JSON action is; why not, when it is no valid call.
pub fn action_of(name: &str, input: &Value) -> Result<AppAction, String> {
    let mut call = match input {
        Value::Object(map) => map.clone(),
        Value::Null => Map::new(),
        other => return Err(format!("the input of {name} is no object: {other}")),
    };
    call.insert("action".to_string(), Value::String(name.to_string()));
    serde_json::from_value(Value::Object(call)).map_err(|e| format!("{name}: {e}"))
}

impl crate::app::OctantApp {
    /// The actions as tools, with the names the active dataset and the
    /// plotted layers offer.
    pub fn tools(&self) -> Vec<Tool> {
        let meta = self.selected.metadata.as_ref();
        let variables = meta
            .iter()
            .flat_map(|m| &m.variables)
            .filter(|v| !v.is_coarser_level());
        let dimensions = meta
            .into_iter()
            .flat_map(dimension_names)
            .map(str::to_string)
            .collect();
        let names = Names {
            variables: variables
                .map(|v| v.name.trim_matches('/').to_string())
                .collect(),
            dimensions,
            layers: self
                .layers
                .iter()
                .map(|l| super::layers::layer_name(self, l.id()))
                // The base layer before anything is plotted has no name.
                .filter(|name| !name.is_empty())
                .collect(),
            plot_types: super::candidates::plot_type_ids()
                .map(str::to_string)
                .collect(),
        };
        tools(&names)
    }

    /// Runs a call of tool `name` with `input`: the action it asks for, or,
    /// when it is no valid call, why, as text not understood.
    pub fn call_tool(&mut self, name: &str, input: &Value) -> super::Outcome {
        match action_of(name, input) {
            Ok(action) => self.dispatch(action),
            Err(why) => Err(super::Failure::NotUnderstood(why)),
        }
    }
}

/// Every dimension name of `meta`'s variables, each once, in order.
fn dimension_names(meta: &crate::data::DatasetMetadata) -> impl Iterator<Item = &str> {
    let mut seen = std::collections::HashSet::new();
    meta.variables
        .iter()
        .flat_map(|v| v.dimension_names.iter().map(String::as_str))
        .filter(move |name| seen.insert(*name))
}
