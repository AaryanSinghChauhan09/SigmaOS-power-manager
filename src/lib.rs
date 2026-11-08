#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LidAction { Suspend, TurnOffDisplay, DoNothing }

/// Mechanism-neutral actions for the session manager or device backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerAction { Suspend, TurnOffDisplay }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PowerManagerConfig {
    pub lid_close_action: LidAction,
    pub idle_sleep_minutes: u16,
}

impl Default for PowerManagerConfig {
    fn default() -> Self {
        Self { lid_close_action: LidAction::Suspend, idle_sleep_minutes: 15 }
    }
}

impl PowerManagerConfig {
    /// Resolve lid-close policy without performing device or session I/O.
    pub fn action_on_lid_close(&self) -> Option<PowerAction> {
        match self.lid_close_action {
            LidAction::Suspend => Some(PowerAction::Suspend),
            LidAction::TurnOffDisplay => Some(PowerAction::TurnOffDisplay),
            LidAction::DoNothing => None,
        }
    }

    /// Resolve idle policy from elapsed whole minutes. A threshold of zero disables it.
    pub fn action_after_idle_minutes(&self, elapsed_minutes: u16) -> Option<PowerAction> {
        if self.idle_sleep_minutes != 0 && elapsed_minutes >= self.idle_sleep_minutes {
            Some(PowerAction::Suspend)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_power_manager_defaults() {
        let cfg = PowerManagerConfig::default();
        assert_eq!(cfg.lid_close_action, LidAction::Suspend);
        assert_eq!(cfg.idle_sleep_minutes, 15);
    }
}
