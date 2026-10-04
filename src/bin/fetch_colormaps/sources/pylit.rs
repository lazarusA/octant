//! A small parser for the Python literals colormap packages use to store data:
//! numbers, strings, tuples/lists, dicts, identifiers (`gfunc[7]`, `_flag_red`)
//! and calls (`dict(deep=[...])`, `np.transpose((...))`, `cubehelix()`).

/// Positional and keyword arguments of a call.
type Items = (Vec<Value>, Vec<(String, Value)>);

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
    let mut parser = Parser {
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

struct Parser<'a> {
    s: &'a [u8],
    i: usize,
}

impl Parser<'_> {
    fn skip(&mut self) {
        while let Some(&c) = self.s.get(self.i) {
            if c == b'#' {
                while self.s.get(self.i).is_some_and(|&c| c != b'\n') {
                    self.i += 1;
                }
            } else if c.is_ascii_whitespace() || c == b'\\' {
                self.i += 1;
            } else {
                break;
            }
        }
    }

    fn peek(&mut self) -> Option<u8> {
        self.skip();
        self.s.get(self.i).copied()
    }

    fn value(&mut self) -> Result<Value, String> {
        match self.peek().ok_or("unexpected end")? {
            b'(' | b'[' => self.seq(),
            b'{' => self.dict(),
            b'\'' | b'"' => self.string().map(Value::Str),
            c if c.is_ascii_digit() || c == b'-' || c == b'+' || c == b'.' => self.number(),
            c if c.is_ascii_alphabetic() || c == b'_' => self.ident_or_call(),
            c => Err(format!("unexpected `{}` at {}", c as char, self.i)),
        }
    }

    /// Items separated by commas until `close`, with optional `key=value` kwargs.
    fn items(&mut self, close: u8) -> Result<Items, String> {
        let (mut args, mut kwargs) = (Vec::new(), Vec::new());
        loop {
            if self.peek() == Some(close) {
                self.i += 1;
                return Ok((args, kwargs));
            }
            let save = self.i;
            if let Some(key) = self.kwarg_key() {
                kwargs.push((key, self.value()?));
            } else {
                self.i = save;
                args.push(self.value()?);
            }
            match self.peek() {
                Some(b',') => self.i += 1,
                Some(c) if c == close => {}
                other => return Err(format!("expected `,` at {} got {other:?}", self.i)),
            }
        }
    }

    fn kwarg_key(&mut self) -> Option<String> {
        let start = self.i;
        while self
            .s
            .get(self.i)
            .is_some_and(|c| c.is_ascii_alphanumeric() || *c == b'_')
        {
            self.i += 1;
        }
        let key = std::str::from_utf8(&self.s[start..self.i])
            .ok()?
            .to_string();
        let is_kw =
            !key.is_empty() && self.peek() == Some(b'=') && self.s.get(self.i + 1) != Some(&b'=');
        is_kw.then(|| {
            self.i += 1;
            key
        })
    }

    fn seq(&mut self) -> Result<Value, String> {
        let close = if self.s[self.i] == b'(' { b')' } else { b']' };
        self.i += 1;
        Ok(Value::Seq(self.items(close)?.0))
    }

    fn dict(&mut self) -> Result<Value, String> {
        self.i += 1;
        let mut items = Vec::new();
        loop {
            if self.peek() == Some(b'}') {
                self.i += 1;
                return Ok(Value::Dict(items));
            }
            let key = self.value()?;
            if self.peek() != Some(b':') {
                return Err(format!("expected `:` at {}", self.i));
            }
            self.i += 1;
            items.push((key, self.value()?));
            if self.peek() == Some(b',') {
                self.i += 1;
            }
        }
    }

    fn string(&mut self) -> Result<String, String> {
        let quote = self.s[self.i];
        let start = self.i + 1;
        let len = self.s[start..]
            .iter()
            .position(|&c| c == quote)
            .ok_or("unterminated string")?;
        self.i = start + len + 1;
        Ok(String::from_utf8_lossy(&self.s[start..start + len]).into_owned())
    }

    /// A number, optionally followed by `* n` / `/ n` (e.g. cmyt's `80 / 256.0`).
    fn number(&mut self) -> Result<Value, String> {
        let mut value = self.literal()?;
        loop {
            match self.peek() {
                Some(b'*') => {
                    self.i += 1;
                    self.skip();
                    value *= self.literal()?;
                }
                Some(b'/') => {
                    self.i += 1;
                    self.skip();
                    value /= self.literal()?;
                }
                _ => return Ok(Value::Num(value)),
            }
        }
    }

    fn literal(&mut self) -> Result<f64, String> {
        let start = self.i;
        while self.s.get(self.i).is_some_and(|&c| {
            c.is_ascii_digit() || matches!(c, b'.' | b'e' | b'E' | b'-' | b'+' | b'_')
        }) {
            self.i += 1;
        }
        let raw = String::from_utf8_lossy(&self.s[start..self.i]).replace('_', "");
        raw.parse().map_err(|_| format!("bad number `{raw}`"))
    }

    fn ident_or_call(&mut self) -> Result<Value, String> {
        let start = self.i;
        while self
            .s
            .get(self.i)
            .is_some_and(|&c| c.is_ascii_alphanumeric() || c == b'_' || c == b'.')
        {
            self.i += 1;
        }
        let mut name = String::from_utf8_lossy(&self.s[start..self.i]).into_owned();
        match self.s.get(self.i) {
            Some(b'(') => {
                self.i += 1;
                let (args, kwargs) = self.items(b')')?;
                Ok(Value::Call { name, args, kwargs })
            }
            Some(b'[') => {
                let close = self.s[self.i..]
                    .iter()
                    .position(|&c| c == b']')
                    .ok_or("unclosed index")?;
                name.push_str(&String::from_utf8_lossy(
                    &self.s[self.i..self.i + close + 1],
                ));
                self.i += close + 1;
                Ok(Value::Ident(name))
            }
            _ => Ok(Value::Ident(name)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_tables_dicts_calls_and_comments() {
        let py = "x = 1\n_a_data = {'red': ((0., 0, 0), (1.0, 1, 1)),  # c\n 'green': gfunc[7], 'blue': _f}\n\
                  P = dict(\n    deep=[\"#4C72B0\", \"#DD8452\"],\n)\nluts = np.transpose(([0.1, 0.2], [0.3, 0.4]))\n";
        let a = assignment(py, "_a_data").unwrap_or(Value::Num(0.0));
        assert_eq!(a.get("green"), Some(&Value::Ident("gfunc[7]".into())));
        assert_eq!(
            a.get("red").and_then(Value::seq).map(<[Value]>::len),
            Some(2)
        );
        let Ok(Value::Call { kwargs, .. }) = assignment(py, "P") else {
            panic!("dict call")
        };
        assert_eq!(kwargs[0].0, "deep");
        let Ok(Value::Call { name, args, .. }) = assignment(py, "luts") else {
            panic!("call")
        };
        assert_eq!(name, "np.transpose");
        assert_eq!(args.len(), 1);
        assert_eq!(assignment(py, "x"), Ok(Value::Num(1.0)));
        assert_eq!(
            assignment(
                "fire: list[list[float]] = [  # cmap_def\n[0, 1, 0]]\n",
                "fire"
            ),
            Ok(Value::Seq(vec![Value::Seq(vec![
                Value::Num(0.0),
                Value::Num(1.0),
                Value::Num(0.0)
            ])]))
        );
        assert_eq!(
            assignment("y = (80 / 256.0, 2 * 3)\n", "y"),
            Ok(Value::Seq(vec![Value::Num(0.3125), Value::Num(6.0)]))
        );
    }
}
