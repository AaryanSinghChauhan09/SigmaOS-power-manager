//! SigmaOS-power-manager — Power efficiency and battery management for SigmaOS
//! OOPS Principles: State, Strategy, Observer patterns
#![forbid(unsafe_code)]
#![warn(missing_docs)]

use thiserror::Error;

/// Error types for this component.
#[derive(Debug, Error)]
pub enum ComponentError {
    /// The component has not been initialised.
    #[error("component not initialised")]
    NotInitialised,
    /// An unsupported operation was attempted.
    #[error("unsupported operation: {0}")]
    Unsupported(&'static str),
}

/// Convenience result type.
pub type ComponentResult<T> = Result<T, ComponentError>;

/// Power state - State Pattern
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum PowerState {
    #[default]
    Performance,
    Balanced,
    PowerSaver,
}

/// Power mode strategy - Strategy Pattern
pub trait PowerStrategy {
    fn get_state(&self) -> PowerState;
    fn apply(&self) -> ComponentResult<()>;
}

/// Performance strategy
pub struct PerformanceStrategy;

impl PowerStrategy for PerformanceStrategy {
    fn get_state(&self) -> PowerState {
        PowerState::Performance
    }
    
    fn apply(&self) -> ComponentResult<()> {
        println!("Applying performance mode");
        Ok(())
    }
}

/// Balanced strategy
pub struct BalancedStrategy;

impl PowerStrategy for BalancedStrategy {
    fn get_state(&self) -> PowerState {
        PowerState::Balanced
    }
    
    fn apply(&self) -> ComponentResult<()> {
        println!("Applying balanced mode");
        Ok(())
    }
}

/// Power saver strategy
pub struct PowerSaverStrategy;

impl PowerStrategy for PowerSaverStrategy {
    fn get_state(&self) -> PowerState {
        PowerState::PowerSaver
    }
    
    fn apply(&self) -> ComponentResult<()> {
        println!("Applying power saver mode");
        Ok(())
    }
}

/// Power observer - Observer Pattern
pub trait PowerObserver {
    fn on_state_change(&self, state: PowerState);
}

/// Logging observer
pub struct LoggingObserver;

impl PowerObserver for LoggingObserver {
    fn on_state_change(&self, state: PowerState) {
        println!("LOG: Power state changed to {:?}", state);
    }
}

/// Core component struct for power_manager.
pub struct Component {
    initialised: bool,
    name: String,
    current_state: PowerState,
    observers: Vec<Box<dyn PowerObserver>>,
}

impl std::fmt::Debug for Component {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Component")
            .field("initialised", &self.initialised)
            .field("name", &self.name)
            .field("current_state", &self.current_state)
            .field("observers_count", &self.observers.len())
            .finish()
    }
}

impl Default for Component {
    fn default() -> Self {
        Self::new()
    }
}

impl Component {
    /// Create a new, uninitialised component.
    pub fn new() -> Self {
        Self {
            initialised: false,
            name: String::from("power_manager"),
            current_state: PowerState::Balanced,
            observers: vec![],
        }
    }

    /// Initialise the component.
    pub fn init(&mut self) -> ComponentResult<()> {
        self.initialised = true;
        Ok(())
    }

    /// Returns true if the component has been initialised.
    pub fn is_ready(&self) -> bool {
        self.initialised
    }

    /// Execute the component's primary operation.
    pub fn run(&self) -> ComponentResult<()> {
        if !self.initialised {
            return Err(ComponentError::NotInitialised);
        }
        Ok(())
    }

    /// Return the component name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Reset the component to uninitialised state.
    pub fn reset(&mut self) {
        self.initialised = false;
    }
    
    /// Apply power strategy
    pub fn apply_strategy(&mut self, strategy: Box<dyn PowerStrategy>) -> ComponentResult<()> {
        if !self.initialised {
            return Err(ComponentError::NotInitialised);
        }
        let new_state = strategy.get_state();
        self.current_state = new_state.clone();
        strategy.apply()?;
        for observer in &self.observers {
            observer.on_state_change(new_state.clone());
        }
        Ok(())
    }
    
    /// Add observer
    pub fn add_observer(&mut self, observer: Box<dyn PowerObserver>) {
        self.observers.push(observer);
    }
    
    /// Get current power state
    pub fn get_state(&self) -> PowerState {
        self.current_state.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_not_ready_aaa() {
        // Arrange + Act
        let c = Component::new();
        // Assert
        assert!(!c.is_ready());
    }

    #[test]
    fn test_init_makes_ready_aaa() {
        // Arrange
        let mut c = Component::new();
        // Act
        c.init().unwrap();
        // Assert
        assert!(c.is_ready());
    }

    #[test]
    fn test_run_before_init_fails_aaa() {
        // Arrange
        let c = Component::new();
        // Act
        let result = c.run();
        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn test_run_after_init_succeeds_aaa() {
        // Arrange
        let mut c = Component::new();
        c.init().unwrap();
        // Act
        let result = c.run();
        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn test_reset_clears_init_aaa() {
        // Arrange
        let mut c = Component::new();
        c.init().unwrap();
        assert!(c.is_ready());
        // Act
        c.reset();
        // Assert
        assert!(!c.is_ready());
    }

    #[test]
    fn test_name_returns_component_name_aaa() {
        // Arrange
        let c = Component::new();
        // Act
        let n = c.name();
        // Assert
        assert!(!n.is_empty());
    }
    
    #[test]
    fn test_strategy_pattern_aaa() {
        // Arrange
        let mut c = Component::new();
        c.init().unwrap();
        let strategy = PerformanceStrategy;
        // Act
        let result = c.apply_strategy(Box::new(strategy));
        // Assert
        assert!(result.is_ok());
        assert_eq!(c.get_state(), PowerState::Performance);
    }
    
    #[test]
    fn test_observer_pattern_aaa() {
        // Arrange
        let mut c = Component::new();
        c.init().unwrap();
        c.add_observer(Box::new(LoggingObserver));
        let strategy = BalancedStrategy;
        // Act
        let result = c.apply_strategy(Box::new(strategy));
        // Assert
        assert!(result.is_ok());
    }
}
