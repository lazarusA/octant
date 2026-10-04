//! Tests for the Python literal parser.

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
