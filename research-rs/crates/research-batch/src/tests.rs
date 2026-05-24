use crate::eval_set::load_eval_set;
use std::path::Path;

#[test]
fn broad_30_eval_set_loads() {
    let eval = load_eval_set(Path::new("../eval_sets/broad_30_probe.yaml"))
        .or_else(|_| load_eval_set(Path::new("../../eval_sets/broad_30_probe.yaml")))
        .or_else(|_| load_eval_set(Path::new("../../../eval_sets/broad_30_probe.yaml")))
        .expect("broad_30 eval set should load");
    assert_eq!(eval.tickers.len(), 30);
    assert_eq!(
        eval.expected_family.get("GOOGL").unwrap(),
        "Platform Internet / Mega-cap Tech"
    );
}
