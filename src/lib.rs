// SPDX-License-Identifier: Apache-2.0
//! SigmaOS-power-manager — logind/wlogout-style power & session manager.
//!
//! Power actions (suspend/hibernate/reboot/shutdown/logout/lock/dpms),
//! battery/UPS state, idle detection, and a dispatch layer that records
//! the requested action for a session daemon to execute.

use std::collections::BTreeMap;
use std::fmt;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PowerAction {
    Suspend,
    Hibernate,
    HybridSleep,
    Reboot,
    Shutdown,
    Logout,
    Lock,
    DpmsOff,
    DpmsOn,
    CancelShutdown,
}

impl PowerAction {
    pub fn label(self) -> &'static str {
        match self {
            Self::Suspend => "suspend",
            Self::Hibernate => "hibernate",
            Self::HybridSleep => "hybrid-sleep",
            Self::Reboot => "reboot",
            Self::Shutdown => "shutdown",
            Self::Logout => "logout",
            Self::Lock => "lock",
            Self::DpmsOff => "dpms-off",
            Self::DpmsOn => "dpms-on",
            Self::CancelShutdown => "cancel-shutdown",
        }
    }
    /// Actions that end the session/power — need confirmation.
    pub fn destructive(self) -> bool {
        matches!(self, Self::Reboot | Self::Shutdown | Self::Hibernate)
    }
}

impl fmt::Display for PowerAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self.label()) }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatteryState {
    Charging,
    Discharging,
    Full,
    NotPresent,
    Unknown,
}

impl BatteryState {
    pub fn label(self) -> &'static str {
        match self {
            Self::Charging => "charging",
            Self::Discharging => "discharging",
            Self::Full => "full",
            Self::NotPresent => "none",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Battery {
    pub present: bool,
    pub percent: u8,          // 0-100
    pub state: BatteryState,
    pub time_to_empty_min: Option<u32>,
    pub time_to_full_min: Option<u32>,
}

impl Default for Battery {
    fn default() -> Self {
        Self { present: true, percent: 100, state: BatteryState::Full,
               time_to_empty_min: None, time_to_full_min: None }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PowerError {
    InhibitActive(String),
    UnknownAction,
    NoSession,
    LowBatteryBlocking,
}

impl fmt::Display for PowerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InhibitActive(w) => write!(f, "power action inhibited by {w}"),
            Self::UnknownAction => write!(f, "unknown power action"),
            Self::NoSession => write!(f, "no active session"),
            Self::LowBatteryBlocking => write!(f, "critical battery blocks sleep"),
        }
    }
}
impl std::error::Error for PowerError {}

/// A recorded action request awaiting execution by the session daemon.
#[derive(Debug, Clone)]
pub struct PendingAction {
    pub action: PowerAction,
    pub scheduled_at: Instant,
    pub confirmed: bool,
}

pub struct SigmaOSPower {
    pub battery: Battery,
    pub ac_online: bool,
    last_activity: Instant,
    idle_timeout: Duration,
    pending: Option<PendingAction>,
    inhibits: BTreeMap<String, String>,   // who -> why
    session_active: bool,
}

impl SigmaOSPower {
    pub fn new() -> Self {
        Self {
            battery: Battery::default(), ac_online: true,
            last_activity: Instant::now(),
            idle_timeout: Duration::from_secs(300),
            pending: None, inhibits: BTreeMap::new(), session_active: true,
        }
    }

    // ---- activity / idle ----
    pub fn note_activity(&mut self) { self.last_activity = Instant::now(); }
    pub fn idle_for(&self) -> Duration { self.last_activity.elapsed() }
    pub fn is_idle(&self) -> bool { self.idle_for() >= self.idle_timeout }
    pub fn set_idle_timeout(&mut self, d: Duration) { self.idle_timeout = d; }

    // ---- inhibit locks (like systemd-inhibit) ----
    pub fn inhibit(&mut self, who: impl Into<String>, why: impl Into<String>) {
        self.inhibits.insert(who.into(), why.into());
    }
    pub fn uninhibit(&mut self, who: &str) { self.inhibits.remove(who); }
    pub fn inhibitors(&self) -> impl Iterator<Item = (&String, &String)> { self.inhibits.iter() }

    // ---- session ----
    pub fn set_session_active(&mut self, on: bool) { self.session_active = on; }

    // ---- actions ----
    /// Request a power action. Destructive actions require `confirmed=true`
    /// or they return PendingAction unconfirmed for the UI to confirm.
    pub fn request(&mut self, action: PowerAction, confirmed: bool) -> Result<Option<PendingAction>, PowerError> {
        // Inhibitors block session/power-ending actions (lock/dpms always allowed).
        let power_affecting = matches!(action,
            PowerAction::Suspend | PowerAction::Hibernate | PowerAction::HybridSleep
            | PowerAction::Reboot | PowerAction::Shutdown);
        if power_affecting && !self.inhibits.is_empty() {
            let who = self.inhibits.keys().next().cloned().unwrap_or_default();
            return Err(PowerError::InhibitActive(who));
        }
        if action.destructive() && !confirmed {
            self.pending = Some(PendingAction { action, scheduled_at: Instant::now(), confirmed: false });
            return Ok(self.pending.clone());
        }
        // Critical battery blocks sleep when on battery.
        if matches!(action, PowerAction::Suspend | PowerAction::Hibernate | PowerAction::HybridSleep)
            && self.battery.percent <= 5 && self.battery.state == BatteryState::Discharging {
            return Err(PowerError::LowBatteryBlocking);
        }
        self.pending = Some(PendingAction { action, scheduled_at: Instant::now(), confirmed: true });
        Ok(self.pending.clone())
    }

    /// Execute the pending action (called by session daemon / test).
    pub fn execute_pending(&mut self) -> Option<PowerAction> {
        self.pending.take().map(|p| p.action)
    }
    pub fn pending(&self) -> Option<&PendingAction> { self.pending.as_ref() }
    pub fn cancel_pending(&mut self) { self.pending = None; }

    // ---- reporting ----
    pub fn battery_line(&self) -> String {
        if !self.battery.present { return "battery: none".into(); }
        let t = match self.battery.state {
            BatteryState::Discharging => self.battery.time_to_empty_min.map(|m| format!(" ~{m}min")).unwrap_or_default(),
            BatteryState::Charging => self.battery.time_to_full_min.map(|m| format!(" ~{m}min")).unwrap_or_default(),
            _ => String::new(),
        };
        format!("battery: {}% {}{}", self.battery.percent, self.battery.state.label(), t)
    }
    pub fn status_line(&self) -> String {
        let p = self.pending.as_ref().map(|p| p.action.label()).unwrap_or("none");
        format!("{} ac={} pending={} inhibitors={}", self.battery_line(), self.ac_online, p, self.inhibits.len())
    }
}

impl Default for SigmaOSPower {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_destructive_executes_immediately() {
        let mut m = SigmaOSPower::new();
        let r = m.request(PowerAction::Lock, false).unwrap();
        assert!(r.unwrap().confirmed);
        assert_eq!(m.execute_pending(), Some(PowerAction::Lock));
    }

    #[test]
    fn destructive_needs_confirm() {
        let mut m = SigmaOSPower::new();
        let p = m.request(PowerAction::Shutdown, false).unwrap().unwrap();
        assert!(!p.confirmed);
        assert_eq!(m.pending().unwrap().action, PowerAction::Shutdown);
        // now confirm
        let p2 = m.request(PowerAction::Shutdown, true).unwrap().unwrap();
        assert!(p2.confirmed);
        assert_eq!(m.execute_pending(), Some(PowerAction::Shutdown));
    }

    #[test]
    fn inhibit_blocks() {
        let mut m = SigmaOSPower::new();
        m.inhibit("upower", "firmware update");
        assert!(m.request(PowerAction::Suspend, false).is_err());
        m.uninhibit("upower");
        assert!(m.request(PowerAction::Suspend, false).is_ok());
    }

    #[test]
    fn low_battery_blocks_sleep() {
        let mut m = SigmaOSPower::new();
        m.battery.percent = 3;
        m.battery.state = BatteryState::Discharging;
        m.ac_online = false;
        assert!(m.request(PowerAction::Suspend, false).is_err());
        // but shutdown still allowed
        assert!(m.request(PowerAction::Shutdown, true).is_ok());
    }

    #[test]
    fn idle_detection() {
        let mut m = SigmaOSPower::new();
        assert!(!m.is_idle());
        m.set_idle_timeout(Duration::from_millis(1));
        std::thread::sleep(Duration::from_millis(5));
        assert!(m.is_idle());
        m.note_activity();
        assert!(!m.is_idle());
    }

    #[test]
    fn battery_line() {
        let mut m = SigmaOSPower::new();
        m.battery.percent = 64;
        m.battery.state = BatteryState::Discharging;
        m.battery.time_to_empty_min = Some(180);
        assert!(m.battery_line().contains("64%") && m.battery_line().contains("180min"));
        m.battery.present = false;
        assert_eq!(m.battery_line(), "battery: none");
    }

    #[test]
    fn cancel_and_labels() {
        let mut m = SigmaOSPower::new();
        m.request(PowerAction::Reboot, false).unwrap();
        m.cancel_pending();
        assert!(m.pending().is_none());
        assert_eq!(PowerAction::DpmsOff.label(), "dpms-off");
        assert!(PowerAction::Shutdown.destructive());
        assert!(!PowerAction::Lock.destructive());
    }
}
