//! `check`'s observation on the painted presenter (LLP 1116.003 D8): what
//! scripts/agent-check.mjs judges, read from the painted boxes.
use super::*;
use serde_json::Value;

fn observe(app: &str) -> (Presenter<()>, Value) {
    let (mut p, error) = Presenter::boot_with(
        &contract::compile(app)
            .unwrap_or_else(|e| panic!("{e}"))
            .encode(),
        (),
        (400., 400.),
        1.,
        PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../apps/caltrain")),
        PainterChoice::Cpu,
    )
    .unwrap();
    assert!(error.is_none(), "{error:?}");
    let obs = p.check_json();
    (p, obs)
}

fn item<'a>(p: &Presenter<()>, obs: &'a Value, test_id: &str) -> &'a Value {
    let k = p.host().kernel();
    let id = k.node_by_key(k.find_by_test_id(test_id)[0]).unwrap().id;
    obs["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["id"] == serde_json::json!(id))
        .unwrap_or_else(|| panic!("no item for {test_id}: {obs}"))
}

#[test]
fn a_text_reports_its_ink_and_the_page_it_stands_on() {
    let (p, obs) = observe(
        r##"component App
  view
    column
      text "Faint" color="#bbbbbb" testId="faint"
"##,
    );
    assert_eq!(obs["scheme"], "light");
    assert_eq!(obs["target"], 24, "a desktop pointer: WCAG 2.5.8's minimum");
    let faint = item(&p, &obs, "faint");
    assert_eq!(faint["text"], "Faint");
    assert_eq!(faint["fg"], serde_json::json!([187., 187., 187., 1.]));
    assert_eq!(faint["on"], "page");
    assert!(faint["bg"].is_array(), "{faint}");
}

#[test]
fn a_clip_cuts_a_text_and_a_wide_column_runs_past_the_viewport() {
    let (p, obs) = observe(
        r##"component App
  view
    scroll height=400
      row width=80 overflow="hidden" testId="narrow"
        text "a sentence far too long to fit this narrow box" white-space="nowrap" testId="clipped"
      column width=600 testId="wide"
        text "wide"
"##,
    );
    let k = p.host().kernel();
    let narrow = k.node_by_key(k.find_by_test_id("narrow")[0]).unwrap().id;
    let cut = &item(&p, &obs, "clipped")["cut"];
    assert_eq!(cut["by"], serde_json::json!(narrow), "{cut}");
    assert!(cut["dx"].as_f64().unwrap() > 100., "{cut}");
    assert_eq!(
        item(&p, &obs, "wide")["outX"],
        200.,
        "a vertical scroller holds no sideways overflow"
    );
}

#[test]
fn a_control_says_who_sized_it_and_a_card_its_fill() {
    let (p, obs) = observe(
        r##"component App
  state on = false
  action go
    on = not on
  view
    column padding=16
      button press=go width=14 height=14 testId="tiny"
        text "x"
      input type="checkbox" testId="box"
      column background-color="#eeeeee" border-radius=8 padding=8 testId="card"
        text "inside"
"##,
    );
    let tiny = item(&p, &obs, "tiny");
    assert_eq!(
        (&tiny["control"], &tiny["sized"]),
        (&"button".into(), &"author".into())
    );
    let checkbox = item(&p, &obs, "box");
    assert_eq!(
        (&checkbox["control"], &checkbox["sized"]),
        (&"checkbox".into(), &"platform".into())
    );
    let card = item(&p, &obs, "card");
    assert_eq!(card["fill"], serde_json::json!([238., 238., 238., 1.]));
    assert_eq!(card["radius"], 8.);
    assert!(card.get("edge").is_none(), "{card}");
}
