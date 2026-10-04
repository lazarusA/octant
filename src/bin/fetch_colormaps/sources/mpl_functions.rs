//! Matplotlib's colormap generator functions (`_cm.py`): the `gfunc` table,
//! flag/prism, gist_heat/gist_yarg and cubehelix, by the names `_cm.py` uses.
//! Each port carries the upstream source lines it was written from, so a
//! changed definition upstream is an error instead of a silently wrong map.

use std::f64::consts::PI;

/// A hand-ported generator function and the `_cm.py` lines defining it.
pub struct Ported {
    pub eval: fn(f64) -> f64,
    pub source: &'static [&'static str],
}

/// The line that builds `gfunc` from `_g0` … `_g36`.
const GFUNC: &str = "gfunc = {i: globals()[f\"_g{i}\"] for i in range(37)}";

/// `cubehelix()` with its defaults and `_ch_helper`, as ported in `matplotlib.rs`.
pub const CUBEHELIX_SOURCE: &[&str] = &[
    "def cubehelix(gamma=1.0, s=0.5, r=-1.5, h=1.0):",
    "xg = x ** gamma",
    "a = h * xg * (1 - xg) / 2",
    "phi = 2 * np.pi * (s / 3 + r * x)",
    "return xg + a * (p0 * np.cos(phi) + p1 * np.sin(phi))",
    "return {'red': partial(_ch_helper, gamma, s, r, h, -0.14861, 1.78277),",
    "'green': partial(_ch_helper, gamma, s, r, h, -0.29227, -0.90649),",
    "'blue': partial(_ch_helper, gamma, s, r, h, 1.97294, 0.0)}",
];

macro_rules! g {
    ($f:expr, $($line:literal),+) => {
        Ported { eval: $f, source: &[$($line,)+ GFUNC] }
    };
}

macro_rules! named {
    ($name:literal, $f:expr, $line:literal) => {
        (
            $name,
            Ported {
                eval: $f,
                source: &[$line],
            },
        )
    };
}

/// Errors unless every line of `lines` appears (trimmed) in `text`.
pub fn require_lines(text: &str, what: &str, lines: &[&str]) -> Result<(), String> {
    match lines
        .iter()
        .find(|line| !text.lines().any(|l| l.trim() == **line))
    {
        Some(missing) => Err(format!(
            "`{what}` changed upstream (missing `{missing}`); update its port"
        )),
        None => Ok(()),
    }
}

/// Generator functions referenced by name in `_cm.py`.
pub fn function(name: &str) -> Option<&'static Ported> {
    if let Some(n) = name
        .strip_prefix("gfunc[")
        .and_then(|s| s.strip_suffix(']'))
    {
        return n.parse::<usize>().ok().and_then(|n| GFUNC_TABLE.get(n));
    }
    NAMED.iter().find(|(n, _)| *n == name).map(|(_, f)| f)
}

static NAMED: [(&str, Ported); 10] = [
    named!(
        "_flag_red",
        |x| 0.75 * ((x * 31.5 + 0.25) * PI).sin() + 0.5,
        "def _flag_red(x): return 0.75 * np.sin((x * 31.5 + 0.25) * np.pi) + 0.5"
    ),
    named!(
        "_flag_green",
        |x| (x * 31.5 * PI).sin(),
        "def _flag_green(x): return np.sin(x * 31.5 * np.pi)"
    ),
    named!(
        "_flag_blue",
        |x| 0.75 * ((x * 31.5 - 0.25) * PI).sin() + 0.5,
        "def _flag_blue(x): return 0.75 * np.sin((x * 31.5 - 0.25) * np.pi) + 0.5"
    ),
    named!(
        "_prism_red",
        |x| 0.75 * ((x * 20.9 + 0.25) * PI).sin() + 0.67,
        "def _prism_red(x): return 0.75 * np.sin((x * 20.9 + 0.25) * np.pi) + 0.67"
    ),
    named!(
        "_prism_green",
        |x| 0.75 * ((x * 20.9 - 0.25) * PI).sin() + 0.33,
        "def _prism_green(x): return 0.75 * np.sin((x * 20.9 - 0.25) * np.pi) + 0.33"
    ),
    named!(
        "_prism_blue",
        |x| -1.1 * ((x * 20.9) * PI).sin(),
        "def _prism_blue(x): return -1.1 * np.sin((x * 20.9) * np.pi)"
    ),
    named!(
        "_gist_heat_red",
        |x| 1.5 * x,
        "def _gist_heat_red(x): return 1.5 * x"
    ),
    named!(
        "_gist_heat_green",
        |x| 2.0 * x - 1.0,
        "def _gist_heat_green(x): return 2 * x - 1"
    ),
    named!(
        "_gist_heat_blue",
        |x| 4.0 * x - 3.0,
        "def _gist_heat_blue(x): return 4 * x - 3"
    ),
    named!("_gist_yarg", |x| 1.0 - x, "def _gist_yarg(x): return 1 - x"),
];

/// Matplotlib's `gfunc` table (`_g0` … `_g36`).
static GFUNC_TABLE: [Ported; 37] = [
    g!(|_| 0.0, "def _g0(x): return 0"),
    g!(|_| 0.5, "def _g1(x): return 0.5"),
    g!(|_| 1.0, "def _g2(x): return 1"),
    g!(|x| x, "def _g3(x): return x"),
    g!(|x| x.powi(2), "def _g4(x): return x ** 2"),
    g!(|x| x.powi(3), "def _g5(x): return x ** 3"),
    g!(|x| x.powi(4), "def _g6(x): return x ** 4"),
    g!(|x| x.sqrt(), "def _g7(x): return np.sqrt(x)"),
    g!(
        |x| x.sqrt().sqrt(),
        "def _g8(x): return np.sqrt(np.sqrt(x))"
    ),
    g!(
        |x| (x * PI / 2.0).sin(),
        "def _g9(x): return np.sin(x * np.pi / 2)"
    ),
    g!(
        |x| (x * PI / 2.0).cos(),
        "def _g10(x): return np.cos(x * np.pi / 2)"
    ),
    g!(|x| (x - 0.5).abs(), "def _g11(x): return np.abs(x - 0.5)"),
    g!(
        |x| (2.0 * x - 1.0).powi(2),
        "def _g12(x): return (2 * x - 1) ** 2"
    ),
    g!(|x| (x * PI).sin(), "def _g13(x): return np.sin(x * np.pi)"),
    g!(
        |x| (x * PI).cos().abs(),
        "def _g14(x): return np.abs(np.cos(x * np.pi))"
    ),
    g!(
        |x| (x * 2.0 * PI).sin(),
        "def _g15(x): return np.sin(x * 2 * np.pi)"
    ),
    g!(
        |x| (x * 2.0 * PI).cos(),
        "def _g16(x): return np.cos(x * 2 * np.pi)"
    ),
    g!(
        |x| (x * 2.0 * PI).sin().abs(),
        "def _g17(x): return np.abs(np.sin(x * 2 * np.pi))"
    ),
    g!(
        |x| (x * 2.0 * PI).cos().abs(),
        "def _g18(x): return np.abs(np.cos(x * 2 * np.pi))"
    ),
    g!(
        |x| (x * 4.0 * PI).sin().abs(),
        "def _g19(x): return np.abs(np.sin(x * 4 * np.pi))"
    ),
    g!(
        |x| (x * 4.0 * PI).cos().abs(),
        "def _g20(x): return np.abs(np.cos(x * 4 * np.pi))"
    ),
    g!(|x| 3.0 * x, "def _g21(x): return 3 * x"),
    g!(|x| 3.0 * x - 1.0, "def _g22(x): return 3 * x - 1"),
    g!(|x| 3.0 * x - 2.0, "def _g23(x): return 3 * x - 2"),
    g!(
        |x| (3.0 * x - 1.0).abs(),
        "def _g24(x): return np.abs(3 * x - 1)"
    ),
    g!(
        |x| (3.0 * x - 2.0).abs(),
        "def _g25(x): return np.abs(3 * x - 2)"
    ),
    g!(
        |x| (3.0 * x - 1.0) / 2.0,
        "def _g26(x): return (3 * x - 1) / 2"
    ),
    g!(
        |x| (3.0 * x - 2.0) / 2.0,
        "def _g27(x): return (3 * x - 2) / 2"
    ),
    g!(
        |x| ((3.0 * x - 1.0) / 2.0).abs(),
        "def _g28(x): return np.abs((3 * x - 1) / 2)"
    ),
    g!(
        |x| ((3.0 * x - 2.0) / 2.0).abs(),
        "def _g29(x): return np.abs((3 * x - 2) / 2)"
    ),
    g!(
        |x| x / 0.32 - 0.78125,
        "def _g30(x): return x / 0.32 - 0.78125"
    ),
    g!(|x| 2.0 * x - 0.84, "def _g31(x): return 2 * x - 0.84"),
    g!(
        |x| match x {
            x if x < 0.25 => 4.0 * x,
            x if x < 0.92 => -2.0 * x + 1.84,
            x => x / 0.08 - 11.5,
        },
        "def _g32(x):",
        "m = (x < 0.25)",
        "ret[m] = 4 * x[m]",
        "m = (x >= 0.25) & (x < 0.92)",
        "ret[m] = -2 * x[m] + 1.84",
        "m = (x >= 0.92)",
        "ret[m] = x[m] / 0.08 - 11.5"
    ),
    g!(
        |x| (2.0 * x - 0.5).abs(),
        "def _g33(x): return np.abs(2 * x - 0.5)"
    ),
    g!(|x| 2.0 * x, "def _g34(x): return 2 * x"),
    g!(|x| 2.0 * x - 0.5, "def _g35(x): return 2 * x - 0.5"),
    g!(|x| 2.0 * x - 1.0, "def _g36(x): return 2 * x - 1"),
];
