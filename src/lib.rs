#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LidAction { Suspend, TurnOffDisplay, DoNothing }

#[derive(Debug)]
pub struct PowerManagerConfig {
    pub lid_close_action: LidAction,
    pub idle_sleep_minutes: u16,
}

impl Default for PowerManagerConfig {
    fn default() -> Self {
        Self { lid_close_action: LidAction::Suspend, idle_sleep_minutes: 15 }
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
