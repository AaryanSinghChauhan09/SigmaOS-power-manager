use sigmaos_component::{PowerManager, PowerProfileMode};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_power_manager_new_aaa() {
        // Arrange & Act
        let pm = PowerManager::new();
        // Assert
        assert_eq!(pm.current_mode, PowerProfileMode::Balanced);
        assert_eq!(pm.battery.percentage, 100);
    }

    #[test]
    fn test_set_profile_performance_aaa() {
        // Arrange
        let mut pm = PowerManager::new();
        // Act
        pm.set_profile(PowerProfileMode::Performance);
        // Assert
        assert_eq!(pm.current_mode, PowerProfileMode::Performance);
        assert_eq!(pm.cpu_freq_mhz, 3800);
    }

    #[test]
    fn test_set_profile_power_saver_aaa() {
        // Arrange
        let mut pm = PowerManager::new();
        // Act
        pm.set_profile(PowerProfileMode::PowerSaver);
        // Assert
        assert_eq!(pm.current_mode, PowerProfileMode::PowerSaver);
        assert_eq!(pm.cpu_freq_mhz, 1200);
    }
}
