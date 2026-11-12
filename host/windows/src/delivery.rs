//! Signed delivery in the window (LLP 1030 D4, LLP 1026 D11): the app's
//! adapter (`exact_linux::Composed::updates`) opens the store before the boot
//! it selects, and this loop does what the Linux display loop does: it checks
//! the stream's head two seconds after the first present, off the launch
//! path, and drains a finished check, whose write posted the loop's wake.
use exact_linux::{app::Config, OpenStore, Presenter};
use exact_runner::DataSource;
use std::time::{Duration, Instant};

/// The store's opener when the bake has a store, `Some(None)` when it has
/// none, and `None` (said) when the bake has a store the entry cannot open.
pub(crate) fn opener(compat: &str, updates: Option<OpenStore>) -> Option<Option<OpenStore>> {
    if exact_runner::Delivery::default().with_compat(compat).store == '0' {
        return Some(None);
    }
    if updates.is_none() {
        eprintln!(
            "exact-windows: the bake has an update store and the entry no delivery adapter; regenerate the app entry"
        );
    }
    updates.map(Some)
}

/// Open the store before the boot it selects. A store that fails to open is
/// said and the baked plan boots, as on Linux.
pub(crate) fn attach(config: &mut Config, open: Option<OpenStore>, compat: &str, plan: &[u8]) {
    let Some(open) = open else { return };
    match open(compat, plan, &config.assets) {
        Ok(updates) => config.use_updates(updates, plan),
        Err(error) => eprintln!("exact update: {error}"),
    }
}

/// How soon a staged update or dev selection is looked at again while one
/// waits for its safe point, as the Linux loop polls it.
const PENDING: Duration = Duration::from_millis(50);

#[derive(Default)]
pub(crate) struct Check {
    armed: bool,
    due: Option<Instant>,
}

impl Check {
    /// A frame reached the screen: the first one arms the check.
    pub(crate) fn presented(&mut self) {
        if !self.armed {
            self.armed = true;
            self.due = Some(Instant::now() + Duration::from_secs(2));
        }
    }

    /// Drain a finished check, start a due one, and name the next deadline
    /// the loop must wake for.
    pub(crate) fn service<D: DataSource>(&mut self, p: &mut Presenter<D>) -> Option<Instant> {
        p.poll_update();
        if self.due.is_some_and(|due| Instant::now() >= due) {
            self.due = None;
            p.check_update();
        }
        let pending = p.delivery_pending().then(|| Instant::now() + PENDING);
        match (self.due, pending) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (a, b) => a.or(b),
        }
    }
}
