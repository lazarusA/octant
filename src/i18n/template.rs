//! Translations with values: `{}` placeholders filled in order into a stack
//! buffer, so each language puts the values where its grammar needs them.

use std::fmt::{self, Display};

use crate::utils::stack_str;

/// Placeholder marking where a value goes in a translated template.
pub const PLACEHOLDER: &str = "{}";

/// Writes `template` into `buf` with each `{}` replaced by the next of `args`
/// (an empty string once they run out). Truncates at the buffer's end.
pub fn fill<'a>(buf: &'a mut [u8], template: &str, args: &[&dyn Display]) -> &'a str {
    stack_str(buf, format_args!("{}", Filled { template, args }))
}

/// `template` with its `{}` filled by `args`, as an owned `String`: for text
/// that is kept rather than drawn each frame, such as toast messages.
pub fn filled(template: &str, args: &[&dyn Display]) -> String {
    Filled { template, args }.to_string()
}

/// `template` with its `{}` filled by `args`, written straight into a
/// formatter (for `Display` impls) without an intermediate `String`.
pub fn display<'t>(template: &'t str, args: &'t [&'t dyn Display]) -> impl Display + 't {
    Filled { template, args }
}

struct Filled<'t> {
    template: &'t str,
    args: &'t [&'t dyn Display],
}

impl Display for Filled<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut args = self.args.iter();
        let mut pieces = self.template.split(PLACEHOLDER);
        if let Some(first) = pieces.next() {
            f.write_str(first)?;
        }
        for piece in pieces {
            if let Some(arg) = args.next() {
                arg.fmt(f)?;
            }
            f.write_str(piece)?;
        }
        Ok(())
    }
}
