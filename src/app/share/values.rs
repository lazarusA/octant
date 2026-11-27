//! Share-link values read into actions: `view`, `camera`,
//! `multiscale_level`, and `a..b` number ranges.

use crate::app::actions::AppAction;

/// `camera=pitch,yaw,distance[,tx,ty,tz]`: angles in degrees, the target
/// the point looked at; anything else (another count, `nan`) is ignored.
pub(super) fn camera_action(text: &str) -> Option<AppAction> {
    let numbers: Vec<f32> = text
        .split(',')
        .map(|n| n.trim().parse().ok().filter(|v: &f32| v.is_finite()))
        .collect::<Option<_>>()?;
    let (target, [pitch, yaw, distance]) = match numbers[..] {
        [p, y, d] => (None, [p, y, d]),
        [p, y, d, x, ty, z] => (Some([x, ty, z]), [p, y, d]),
        _ => return None,
    };
    Some(AppAction::SetCamera {
        pitch,
        yaw,
        distance,
        target,
    })
}

/// `multiscale_level`: `n` pins level n, `auto:n` is Auto from level n,
/// `auto` plain Auto; anything else is ignored.
pub(super) fn level_action(text: &str) -> Option<AppAction> {
    let (auto, number) = match text.strip_prefix("auto") {
        Some("") => {
            return Some(AppAction::SetLevel {
                level: None,
                auto: true,
            });
        }
        Some(rest) => (true, rest.strip_prefix(':')?),
        None => (false, text),
    };
    let level = Some(number.parse().ok()?);
    Some(AppAction::SetLevel { level, auto })
}

/// `x0..x1,y0..y1` as `SetView`.
pub(super) fn view_action(view: &str) -> Option<AppAction> {
    let (x, y) = view.split_once(',')?;
    let ((x_from, x_to), (y_from, y_to)) = (x.split_once("..")?, y.split_once("..")?);
    Some(AppAction::SetView {
        x_from: x_from.into(),
        x_to: x_to.into(),
        y_from: y_from.into(),
        y_to: y_to.into(),
    })
}

/// `a..b` as two finite numbers.
pub(super) fn numbers(range: &str) -> Option<(f32, f32)> {
    let (a, b) = range.split_once("..")?;
    let (a, b) = (a.trim().parse::<f32>().ok()?, b.trim().parse::<f32>().ok()?);
    (a.is_finite() && b.is_finite()).then_some((a, b))
}
