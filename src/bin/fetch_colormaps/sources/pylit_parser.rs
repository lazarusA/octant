//! Recursive-descent parser behind [`super::pylit`]: numbers (with `*` and `/`),
//! strings, sequences, dicts, identifiers and calls, skipping comments.

use super::pylit::{Items, Value};

pub struct Parser<'a> {
    pub s: &'a [u8],
    pub i: usize,
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

    pub fn value(&mut self) -> Result<Value, String> {
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
