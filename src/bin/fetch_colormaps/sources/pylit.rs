//! A small parser for the Python literals colormap packages use to store data:
//! numbers, strings, tuples/lists, dicts, identifiers (`gfunc[7]`, `_flag_red`)
//! and calls (`dict(deep=[...])`, `np.transpose((...))`, `cubehelix()`).

/// Positional and keyword arguments of a call.
pub(super) type Items = (Vec<Value>, Vec<(String, Value)>);

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Num(f64),
    Str(String),
    Seq(Vec<Value>),
    Dict(Vec<(Value, Value)>),
    Ident(String),
    Call {
        name: String,
        args: Vec<Value>,
        kwargs: Vec<(String, Value)>,
    },
}

impl Value {
    pub fn get(&self, key: &str) -> Option<&Value> {
        let Value::Dict(items) = self else {
            return None;
        };
        items
            .iter()
            .find(|(k, _)| matches!(k, Value::Str(s) if s == key))
            .map(|(_, v)| v)
    }

    pub fn num(&self) -> Option<f64> {
        if let Value::Num(n) = self {
            Some(*n)
        } else {
            None
        }
    }

    pub fn seq(&self) -> Option<&[Value]> {
        if let Value::Seq(items) = self {
            Some(items)
        } else {
            None
        }
    }

    pub fn str(&self) -> Option<&str> {
        if let Value::Str(s) = self {
            Some(s)
        } else {
            None
        }
    }
}

/// Parses the value assigned to `var` at the start of a line (`var = ...`,
/// optionally with a type annotation: `var: list[float] = ...`).
pub fn assignment(text: &str, var: &str) -> Result<Value, String> {
    let value_start = text
        .match_indices(var)
        .map(|(i, _)| i)
        .filter(|&i| i == 0 || text.as_bytes()[i - 1] == b'\n')
        .find_map(|i| assigned_value_offset(&text[i + var.len()..]).map(|off| i + var.len() + off))
        .ok_or_else(|| format!("`{var}` not found"))?;
    let mut parser = super::pylit_parser::Parser {
        s: &text.as_bytes()[value_start..],
        i: 0,
    };
    parser.value().map_err(|e| format!("`{var}`: {e}"))
}

/// Offset just past `=` when `rest` (what follows a name) is `= ...` or `: T = ...`.
fn assigned_value_offset(rest: &str) -> Option<usize> {
    let line = rest.split('\n').next()?;
    let trimmed = line.trim_start();
    if !(trimmed.starts_with('=') || trimmed.starts_with(':')) {
        return None;
    }
    let eq = line.find('=')?;
    (!line[eq..].starts_with("==")).then_some(eq + 1)
}

#[cfg(test)]
#[path = "pylit_tests.rs"]
mod tests;
