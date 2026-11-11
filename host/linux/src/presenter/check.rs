//! `check` on the painted presenter (LLP 1116.003 D8): what the presented
//! screen shows, as observations the driver judges (scripts/agent-check.mjs),
//! read from the boxes this presenter last painted — each view's box in the
//! viewport, a paragraph's text, ink and type and whether its lines fit its
//! box, a control and who sized it, what the hit test finds at its middle, a
//! filled box's fill and edge, and how far a view runs past the viewport's
//! sides. Linux, Windows and Android's reader paint through it, so each
//! answers the same observation. The fill a text stands on is the nearest
//! ancestor's background where the styles say it (LLP 1116.006 D5); a
//! picture, a material or a gradient leaves it to the capture's pixels.
use super::*;
use crate::paint::Rect4;
use exact_kernel::{ControlKind, StyleId, StyleMask};
use serde_json::{json, Map, Value};

/// Rows an author sizes a control by (agent-check's `sized: "author"`).
const SIZE: [StyleId; 11] = [
    StyleId::Width,
    StyleId::Height,
    StyleId::MinWidth,
    StyleId::MinHeight,
    StyleId::MaxWidth,
    StyleId::MaxHeight,
    StyleId::PaddingTop,
    StyleId::PaddingBottom,
    StyleId::PaddingLeft,
    StyleId::PaddingRight,
    StyleId::FlexBasis,
];

fn r2(v: f32) -> f64 {
    (f64::from(v) * 100.).round() / 100.
}

fn rgba(c: exact_kernel::Color, opacity: f32) -> Option<[f64; 4]> {
    let a = f64::from(c.a()) / 255. * f64::from(opacity);
    (a > 0.).then(|| {
        [
            f64::from(c.r()),
            f64::from(c.g()),
            f64::from(c.b()),
            (a * 1000.).round() / 1000.,
        ]
    })
}

/// Whether `node` draws dark: its own `color-scheme` (an artwork's band), else the app's.
fn dark_at(node: &exact_kernel::NodeRef<'_>, app: bool) -> bool {
    match node.computed_row(StyleId::ColorScheme, |s| s.color_scheme) {
        exact_kernel::ColorScheme::Dark => true,
        exact_kernel::ColorScheme::Light => false,
        exact_kernel::ColorScheme::Normal => app,
    }
}

/// A colour as the painter draws it at `node`: in its scheme, the accent roles reading its `accent-color`.
fn painted(
    node: &exact_kernel::NodeRef<'_>,
    c: exact_kernel::ColorValue,
    app: bool,
) -> exact_kernel::Color {
    let _accent = crate::paint::accent_scope(node);
    c.resolve(dark_at(node, app))
}

fn mix(fg: [f64; 4], bg: [f64; 3]) -> [f64; 3] {
    [0, 1, 2].map(|i| fg[i] * fg[3] + bg[i] * (1. - fg[3]))
}

impl<D: DataSource> Presenter<D> {
    /// The agent's `check` observation of the painted screen.
    pub(crate) fn check_json(&mut self) -> Value {
        let (vw, vh) = self.viewport;
        let dark = self.scheme.1;
        let boxes: Vec<PaintedBox> = self.boxes().to_vec();
        let shown: BTreeMap<ViewId, PaintedBox> = boxes
            .iter()
            .filter(|b| b.rect.2 > 0. && b.rect.3 > 0. && !self.placement_hidden(b.id))
            .map(|b| (b.id, *b))
            .collect();
        let mut items = Vec::new();
        let mut out: Vec<ViewId> = Vec::new();
        let mut covers = Vec::new();
        for (&id, b) in &shown {
            let kernel = self.host.kernel();
            let Some(node) = kernel.node(id) else {
                continue;
            };
            let (x, y, w, h) = b.rect;
            let mut item = Map::new();
            item.insert("id".into(), json!(id));
            for (k, v) in [("x", x), ("y", y), ("w", w), ("h", h)] {
                item.insert(k.into(), json!(r2(v)));
            }
            let on_screen = x + w > 0. && y + h > 0. && x < vw && y < vh;
            if !on_screen {
                continue;
            }
            let s = node.computed_style(StyleMask::EMPTY);
            let opacity = self.opacity(id);
            // Past the viewport's sides, nothing clipping it: the outermost such view only.
            let past = (x + w - vw).max(-x);
            let ancestors = self.ancestors(id);
            if past > 1. && !self.held_x(&ancestors) && !ancestors.iter().any(|a| out.contains(a)) {
                item.insert("outX".into(), json!(r2(past)));
                out.push(id);
            }
            let control = self.control_kind(&node);
            let text = if node.node_type == NodeType::Text && self.paragraph(id).is_some() {
                self.text_of(id)
            } else {
                String::new()
            };
            if !text.is_empty() {
                let shown_text: String = if text.chars().count() > 40 {
                    text.chars().take(39).chain(std::iter::once('…')).collect()
                } else {
                    text.clone()
                };
                item.insert("text".into(), json!(shown_text));
                let fg = node.computed_row(StyleId::TextColor, |s| s.text_color);
                if let Some(fg) = rgba(painted(&node, fg, dark), opacity) {
                    item.insert("fg".into(), json!(fg));
                }
                item.insert("size".into(), json!(r2(s.font_size)));
                item.insert("weight".into(), json!(s.font_weight));
                if let Some(of) = ancestors.iter().take(3).copied().find(|a| {
                    kernel
                        .node(*a)
                        .is_some_and(|n| self.control_kind(&n).is_some())
                }) {
                    item.insert("of".into(), json!(of));
                }
                let (pl, pt, pr, pb) = kernel.resolved_padding(node.key).unwrap_or_default();
                let (bl, bt, br, bb) = kernel.resolved_border(node.key).unwrap_or_default();
                let (room_w, room_h) = (w - pl - pr - bl - br, h - pt - pb - bt - bb);
                if let Some(p) = self.paragraph(id) {
                    let widest = p.lines().lines.iter().map(|l| l.w).fold(0., f32::max);
                    let tall = p.line_bottoms().last().copied().unwrap_or(0.);
                    let (dx, dy) = ((widest - room_w).max(0.), (tall - room_h).max(0.));
                    let clips =
                        s.overflow_x != Overflow::Visible || s.overflow_y != Overflow::Visible;
                    if dx > 1. || dy > 1. {
                        item.insert("over".into(), json!([r2(dx), r2(dy)]));
                        if clips {
                            item.insert("clips".into(), json!(true));
                        }
                    }
                }
                if let Some(cut) = self.cut(b.rect, &ancestors, &shown) {
                    item.insert("cut".into(), cut);
                }
                if s.line_clamp > 0 || s.text_overflow == exact_kernel::TextOverflow::Ellipsis {
                    item.insert("clamp".into(), json!(true));
                }
                if s.text_overflow == exact_kernel::TextOverflow::Ellipsis {
                    item.insert("ellipsis".into(), json!(true));
                }
                if let Some((bg, by, solid)) = self.backing(id, &ancestors, dark) {
                    item.insert("bg".into(), json!(bg.map(f64::round)));
                    let on = match by {
                        Some(by) if by == id => json!("own"),
                        Some(by) => json!(by),
                        None => json!("page"),
                    };
                    item.insert("on".into(), on);
                    if solid {
                        item.insert("solid".into(), json!(true));
                    }
                }
            }
            if let Some(kind) = control {
                let authored = SIZE.iter().any(|row| node.says(*row));
                let platform =
                    !authored && matches!(node.node_type, NodeType::Control | NodeType::TextInput);
                item.insert("control".into(), json!(kind));
                item.insert(
                    "sized".into(),
                    json!(if platform { "platform" } else { "author" }),
                );
                if node.props.get(PropId::Disabled).is_some() {
                    item.insert("disabled".into(), json!(true));
                }
            }
            if control.is_some() || !text.is_empty() {
                covers.push((id, b.center()));
            }
            // A card: a filled box, not text or a control, that holds one, with rounded corners or inset from the sides.
            let fill = (text.is_empty() && control.is_none())
                .then_some(s.background_color)
                .flatten()
                .and_then(|c| rgba(painted(&node, c, dark), opacity));
            if let Some(fill) =
                fill.filter(|_| w >= 16. && h >= 16. && s.background_image.layers().is_empty())
            {
                let env = kernel.env();
                let radius = match s.border_radius_top_left.resolve(&env) {
                    exact_kernel::Dimension::Points(r) => r,
                    _ => 0.,
                };
                let inset = x >= 2. && x + w <= vw - 2.;
                if (radius > 0. || inset) && self.holds(id, &shown) {
                    item.insert("fill".into(), json!(fill));
                    item.insert("radius".into(), json!(r2(radius)));
                    let (bl, bt, br, bb) = kernel.resolved_border(node.key).unwrap_or_default();
                    if bl + bt + br + bb > 0. || !s.box_shadow.shadows().is_empty() {
                        item.insert("edge".into(), json!(true));
                    }
                }
            }
            if let Some(clip) = b.clip {
                let (l, t) = (x.max(clip.0), y.max(clip.1));
                let (r, btm) = ((x + w).min(clip.0 + clip.2), (y + h).min(clip.1 + clip.3));
                let v = (l, t, (r - l).max(0.), (btm - t).max(0.));
                if (v.0 - x).abs() > 0.5
                    || (v.1 - y).abs() > 0.5
                    || (v.2 - w).abs() > 0.5
                    || (v.3 - h).abs() > 0.5
                {
                    item.insert(
                        "vis".into(),
                        json!({"x": r2(v.0), "y": r2(v.1), "w": r2(v.2), "h": r2(v.3)}),
                    );
                }
            }
            let keep = ["text", "control", "fill", "outX"]
                .iter()
                .any(|k| item.contains_key(*k));
            if keep {
                items.push(item);
            }
        }
        // What a pointer at each middle reaches: covered where another view's subtree is hit there.
        for (id, (cx, cy)) in covers {
            if cy < 0. || cy >= vh {
                continue;
            }
            let Some(hit) = self.hit(cx.clamp(1., (vw - 1.).max(1.)), cy) else {
                continue;
            };
            if hit == id || self.ancestors(hit).contains(&id) || self.ancestors(id).contains(&hit) {
                continue;
            }
            if let Some(item) = items.iter_mut().find(|i| i["id"] == json!(id)) {
                let name = self
                    .host
                    .kernel()
                    .node(hit)
                    .map(|n| format!("{:?}", n.node_type).to_lowercase())
                    .unwrap_or_default();
                item.insert(
                    "cover".into(),
                    json!({"by": hit, "name": name, "kind": "node"}),
                );
            }
        }
        let coarse =
            self.host.runner().viewport().preferences.pointer == exact_runner::Pointer::Coarse;
        json!({
            "viewport": {"w": r2(vw), "h": r2(vh), "scrollW": r2(vw)},
            "scheme": if dark { "dark" } else { "light" },
            "target": if coarse { 48 } else { 24 },
            "items": items,
        })
    }

    /// The kind name agent-check gives a control, or `None` for anything else.
    fn control_kind(&self, node: &exact_kernel::NodeRef<'_>) -> Option<&'static str> {
        match node.node_type {
            NodeType::TextInput => Some("text"),
            NodeType::Pressable if node.props.str(PropId::Href).is_some() => Some("link"),
            NodeType::Pressable => Some("press"),
            NodeType::Control => Some(match ControlKind::of(node.node_type, node.props)? {
                ControlKind::Checkbox => "checkbox",
                ControlKind::Switch => "switch",
                ControlKind::Radio => "radio",
                ControlKind::File => "file",
                ControlKind::Select => "select",
                ControlKind::Range => "range",
                ControlKind::Date => "date",
                ControlKind::Time => "time",
                ControlKind::DateTimeLocal => "datetime-local",
                ControlKind::Button => "button",
                ControlKind::Progress | ControlKind::ProgressBar => return None,
            }),
            _ => None,
        }
    }

    /// Whether an ancestor keeps a view's sideways overflow its own: one that clips on x, or scrolls sideways
    /// because its author said so (a vertical scroller's x computes `auto` only beside its `y`, CSS Overflow 3 §3.1).
    fn held_x(&self, ancestors: &[ViewId]) -> bool {
        let kernel = self.host.kernel();
        ancestors.iter().filter_map(|a| kernel.node(*a)).any(|n| {
            let x = n.computed_row(StyleId::OverflowX, |s| s.overflow_x);
            matches!(x, Overflow::Hidden | Overflow::Clip)
                || (x != Overflow::Visible && n.says(StyleId::OverflowX))
        })
    }

    /// How much of a text's box an ancestor that clips (never one that scrolls: a person scrolls to it) cuts away,
    /// judged on the part inside every scroller between them; a clip that hides it all is a collapsed disclosure.
    fn cut(
        &self,
        text: Rect4,
        ancestors: &[ViewId],
        shown: &BTreeMap<ViewId, PaintedBox>,
    ) -> Option<Value> {
        let kernel = self.host.kernel();
        let (mut l, mut t, mut r, mut btm) = (text.0, text.1, text.0 + text.2, text.1 + text.3);
        for a in ancestors {
            let (Some(n), Some(b)) = (kernel.node(*a), shown.get(a)) else {
                continue;
            };
            let (ox, oy) = n.computed_row(StyleId::OverflowX, |s| (s.overflow_x, s.overflow_y));
            let (bl, bt, br, bb) = kernel.resolved_border(n.key).unwrap_or_default();
            let (al, at, ar, ab) = (
                b.rect.0 + bl,
                b.rect.1 + bt,
                b.rect.0 + b.rect.2 - br,
                b.rect.1 + b.rect.3 - bb,
            );
            let scrolls = |o| matches!(o, Overflow::Auto | Overflow::Scroll);
            let hides = |o| matches!(o, Overflow::Hidden | Overflow::Clip);
            if scrolls(ox) || scrolls(oy) {
                if scrolls(ox) {
                    l = l.max(al);
                    r = r.min(ar);
                }
                if scrolls(oy) {
                    t = t.max(at);
                    btm = btm.min(ab);
                }
                if r <= l || btm <= t {
                    return None;
                }
            }
            if !hides(ox) && !hides(oy) {
                continue;
            }
            if r <= al || l >= ar || btm <= at || t >= ab {
                return None;
            }
            let dx = if hides(ox) {
                (al - l).max(0.) + (r - ar).max(0.)
            } else {
                0.
            };
            let dy = if hides(oy) {
                (at - t).max(0.) + (btm - ab).max(0.)
            } else {
                0.
            };
            if dx >= 2. || dy >= 2. {
                return Some(json!({"by": a, "dx": r2(dx), "dy": r2(dy)}));
            }
        }
        None
    }

    /// The logical ancestors of `id`, nearest first.
    fn ancestors(&self, id: ViewId) -> Vec<ViewId> {
        let kernel = self.host.kernel();
        let mut out = Vec::new();
        let mut at = kernel.node(id).and_then(|n| n.parent);
        while let Some(a) = at {
            out.push(a);
            at = kernel.node(a).and_then(|n| n.parent);
        }
        out
    }

    /// `id`'s opacity times its ancestors'.
    fn opacity(&self, id: ViewId) -> f32 {
        let kernel = self.host.kernel();
        std::iter::once(id)
            .chain(self.ancestors(id))
            .filter_map(|a| {
                kernel
                    .node(a)
                    .map(|n| n.computed_row(StyleId::Opacity, |s| s.opacity))
            })
            .product()
    }

    /// A paragraph's text: its own, else its text children's, in order.
    fn text_of(&self, id: ViewId) -> String {
        let kernel = self.host.kernel();
        let rows = kernel.rows(Some(id)).unwrap_or_default();
        let words: Vec<&str> = rows
            .iter()
            .filter_map(|r| kernel.node(r.id).and_then(|n| n.props.str(PropId::Text)))
            .collect();
        words
            .join("")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Whether a painted descendant of `id` is a text or a control.
    fn holds(&self, id: ViewId, shown: &BTreeMap<ViewId, PaintedBox>) -> bool {
        let kernel = self.host.kernel();
        kernel
            .rows(Some(id))
            .unwrap_or_default()
            .iter()
            .skip(1)
            .any(|r| {
                shown.contains_key(&r.id)
                    && kernel.node(r.id).is_some_and(|n| {
                        self.control_kind(&n).is_some()
                            || (n.node_type == NodeType::Text
                                && n.props.str(PropId::Text).is_some())
                    })
            })
    }

    /// The fill a text stands on: its own box's, else its nearest ancestors' that paint one, composited over the
    /// page's canvas where translucent; whose it is (`None`: the page's), and whether that fill is opaque. `None`
    /// where a picture, a gradient or a material paints there, which the capture answers.
    fn backing(
        &self,
        id: ViewId,
        ancestors: &[ViewId],
        dark: bool,
    ) -> Option<([f64; 3], Option<ViewId>, bool)> {
        let kernel = self.host.kernel();
        let mut layers = Vec::new();
        let mut by = None;
        for a in std::iter::once(id).chain(ancestors.iter().copied()) {
            let node = kernel.node(a)?;
            if matches!(
                node.node_type,
                NodeType::Image
                    | NodeType::Video
                    | NodeType::Canvas
                    | NodeType::WebView
                    | NodeType::NativeView
            ) || node.props.str(PropId::BackgroundMaterial).is_some()
            {
                return None;
            }
            let (color, image) = node.computed_row(StyleId::BackgroundColor, |s| {
                (s.background_color, s.background_image.layers().is_empty())
            });
            if !image {
                return None;
            }
            let Some(c) = color.and_then(|c| rgba(painted(&node, c, dark), self.opacity(a))) else {
                continue;
            };
            if layers.is_empty() {
                by = Some(a);
            }
            layers.push(c);
            if c[3] >= 0.999 {
                break;
            }
        }
        let solid = by.is_some() && layers.first().is_some_and(|c| c[3] >= 0.999);
        let canvas = if layers.last().is_some_and(|c| c[3] >= 0.999) {
            None
        } else {
            let page = exact_kernel::ColorValue::Role(exact_kernel::style::roles::role("Canvas")?);
            let page_dark = kernel
                .node(*ancestors.last().unwrap_or(&id))
                .map_or(dark, |n| dark_at(&n, dark));
            Some(rgba(page.resolve(page_dark), 1.)?)
        };
        let mut bg = match canvas {
            Some(c) => [c[0], c[1], c[2]],
            None => {
                let top = layers.pop()?;
                [top[0], top[1], top[2]]
            }
        };
        while let Some(layer) = layers.pop() {
            bg = mix(layer, bg);
        }
        Some((bg, by, solid))
    }
}
