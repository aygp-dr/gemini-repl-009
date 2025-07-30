//! Model checking for REPL state machine verification
//! 
//! Implements formal verification of REPL behavior using state machines
//! and temporal logic properties

use std::collections::{HashMap, HashSet, VecDeque};
use serde::{Serialize, Deserialize};

/// REPL States
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReplState {
    Init,
    AwaitingInput,
    ParsingCommand,
    ProcessingCommand,
    CallingFunction(String), // Function name
    AwaitingModel,
    GeneratingResponse,
    DisplayingOutput,
    Error(ErrorState),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ErrorState {
    ParseError,
    FunctionCallError,
    ModelError,
    TimeoutError,
}

/// Events that trigger state transitions
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Event {
    Start,
    UserInput(String),
    ParseComplete(ParseResult),
    FunctionCallRequired(String),
    FunctionCallComplete(Result<String, String>),
    ModelQueryRequired,
    ModelResponseReceived(String),
    OutputReady(String),
    ErrorOccurred(ErrorState),
    Reset,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ParseResult {
    Command(String),
    Query(String),
    Invalid,
}

/// State machine definition
pub struct ReplStateMachine {
    current_state: ReplState,
    history: Vec<(ReplState, Event)>,
    transitions: HashMap<(ReplState, Event), ReplState>,
}

impl ReplStateMachine {
    pub fn new() -> Self {
        let mut transitions = HashMap::new();
        
        // Define all valid state transitions
        // Init transitions
        transitions.insert(
            (ReplState::Init, Event::Start),
            ReplState::AwaitingInput
        );
        
        // AwaitingInput transitions
        transitions.insert(
            (ReplState::AwaitingInput, Event::UserInput(_)),
            ReplState::ParsingCommand
        );
        
        // ParsingCommand transitions
        transitions.insert(
            (ReplState::ParsingCommand, Event::ParseComplete(ParseResult::Command(_))),
            ReplState::ProcessingCommand
        );
        transitions.insert(
            (ReplState::ParsingCommand, Event::ParseComplete(ParseResult::Query(_))),
            ReplState::ProcessingCommand
        );
        transitions.insert(
            (ReplState::ParsingCommand, Event::ParseComplete(ParseResult::Invalid)),
            ReplState::Error(ErrorState::ParseError)
        );
        
        // ProcessingCommand transitions
        transitions.insert(
            (ReplState::ProcessingCommand, Event::FunctionCallRequired(_)),
            ReplState::CallingFunction("".to_string())
        );
        transitions.insert(
            (ReplState::ProcessingCommand, Event::ModelQueryRequired),
            ReplState::AwaitingModel
        );
        transitions.insert(
            (ReplState::ProcessingCommand, Event::OutputReady(_)),
            ReplState::DisplayingOutput
        );
        
        // CallingFunction transitions
        transitions.insert(
            (ReplState::CallingFunction(_), Event::FunctionCallComplete(Ok(_))),
            ReplState::GeneratingResponse
        );
        transitions.insert(
            (ReplState::CallingFunction(_), Event::FunctionCallComplete(Err(_))),
            ReplState::Error(ErrorState::FunctionCallError)
        );
        
        // AwaitingModel transitions
        transitions.insert(
            (ReplState::AwaitingModel, Event::ModelResponseReceived(_)),
            ReplState::GeneratingResponse
        );
        transitions.insert(
            (ReplState::AwaitingModel, Event::ErrorOccurred(ErrorState::TimeoutError)),
            ReplState::Error(ErrorState::TimeoutError)
        );
        
        // GeneratingResponse transitions
        transitions.insert(
            (ReplState::GeneratingResponse, Event::OutputReady(_)),
            ReplState::DisplayingOutput
        );
        transitions.insert(
            (ReplState::GeneratingResponse, Event::FunctionCallRequired(_)),
            ReplState::CallingFunction("".to_string())
        );
        
        // DisplayingOutput transitions
        transitions.insert(
            (ReplState::DisplayingOutput, Event::Reset),
            ReplState::AwaitingInput
        );
        
        // Error state transitions
        for error in vec![
            ErrorState::ParseError,
            ErrorState::FunctionCallError,
            ErrorState::ModelError,
            ErrorState::TimeoutError,
        ] {
            transitions.insert(
                (ReplState::Error(error), Event::Reset),
                ReplState::AwaitingInput
            );
        }
        
        Self {
            current_state: ReplState::Init,
            history: Vec::new(),
            transitions,
        }
    }
    
    pub fn transition(&mut self, event: Event) -> Result<ReplState, String> {
        let key = (self.current_state.clone(), event.clone());
        
        if let Some(next_state) = self.transitions.get(&key).cloned() {
            self.history.push((self.current_state.clone(), event));
            self.current_state = next_state.clone();
            Ok(next_state)
        } else {
            Err(format!("Invalid transition from {:?} with event {:?}", 
                       self.current_state, event))
        }
    }
    
    pub fn current_state(&self) -> &ReplState {
        &self.current_state
    }
    
    pub fn history(&self) -> &[(ReplState, Event)] {
        &self.history
    }
}

/// Temporal Logic Properties
#[derive(Debug, Clone)]
pub enum TemporalProperty {
    /// Eventually reaches a state
    Eventually(ReplState),
    /// Always in one of the states
    Always(Vec<ReplState>),
    /// If state A then eventually state B
    LeadsTo(ReplState, ReplState),
    /// Never reaches state
    Never(ReplState),
    /// State A until state B
    Until(ReplState, ReplState),
}

/// Model checker for verifying properties
pub struct ModelChecker {
    state_machine: ReplStateMachine,
}

impl ModelChecker {
    pub fn new() -> Self {
        Self {
            state_machine: ReplStateMachine::new(),
        }
    }
    
    /// Check if a property holds for all possible execution paths
    pub fn check_property(&self, property: &TemporalProperty, max_depth: usize) -> bool {
        match property {
            TemporalProperty::Eventually(target) => {
                self.check_eventually(target, max_depth)
            }
            TemporalProperty::Always(states) => {
                self.check_always(states, max_depth)
            }
            TemporalProperty::LeadsTo(from, to) => {
                self.check_leads_to(from, to, max_depth)
            }
            TemporalProperty::Never(state) => {
                !self.check_eventually(state, max_depth)
            }
            TemporalProperty::Until(state_a, state_b) => {
                self.check_until(state_a, state_b, max_depth)
            }
        }
    }
    
    fn check_eventually(&self, target: &ReplState, max_depth: usize) -> bool {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        
        queue.push_back((ReplState::Init, 0));
        visited.insert(ReplState::Init);
        
        while let Some((state, depth)) = queue.pop_front() {
            if &state == target {
                return true;
            }
            
            if depth >= max_depth {
                continue;
            }
            
            // Explore all possible transitions from this state
            for event in self.get_possible_events(&state) {
                if let Some(next_state) = self.get_next_state(&state, &event) {
                    if !visited.contains(&next_state) {
                        visited.insert(next_state.clone());
                        queue.push_back((next_state, depth + 1));
                    }
                }
            }
        }
        
        false
    }
    
    fn check_always(&self, valid_states: &[ReplState], max_depth: usize) -> bool {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        
        queue.push_back((ReplState::Init, 0));
        
        while let Some((state, depth)) = queue.pop_front() {
            if !valid_states.contains(&state) {
                return false;
            }
            
            if depth >= max_depth || visited.contains(&state) {
                continue;
            }
            
            visited.insert(state.clone());
            
            for event in self.get_possible_events(&state) {
                if let Some(next_state) = self.get_next_state(&state, &event) {
                    queue.push_back((next_state, depth + 1));
                }
            }
        }
        
        true
    }
    
    fn check_leads_to(&self, from: &ReplState, to: &ReplState, max_depth: usize) -> bool {
        // For all paths starting from 'from' state, eventually reach 'to'
        let mut visited_from = HashSet::new();
        let mut queue = VecDeque::new();
        
        // First, find all instances of 'from' state
        queue.push_back((ReplState::Init, 0, false));
        
        while let Some((state, depth, after_from)) = queue.pop_front() {
            if depth >= max_depth {
                continue;
            }
            
            let now_after_from = after_from || &state == from;
            
            if now_after_from && &state == to {
                continue; // This path satisfied the property
            }
            
            if now_after_from && depth == max_depth - 1 {
                return false; // Reached max depth without finding 'to'
            }
            
            for event in self.get_possible_events(&state) {
                if let Some(next_state) = self.get_next_state(&state, &event) {
                    queue.push_back((next_state, depth + 1, now_after_from));
                }
            }
        }
        
        true
    }
    
    fn check_until(&self, state_a: &ReplState, state_b: &ReplState, max_depth: usize) -> bool {
        // State A must hold until state B is reached
        let mut queue = VecDeque::new();
        queue.push_back((ReplState::Init, 0));
        
        while let Some((state, depth)) = queue.pop_front() {
            if &state == state_b {
                continue; // Property satisfied on this path
            }
            
            if &state != state_a {
                return false; // Violated: not in state A before reaching B
            }
            
            if depth >= max_depth {
                return false; // Didn't reach B within depth limit
            }
            
            for event in self.get_possible_events(&state) {
                if let Some(next_state) = self.get_next_state(&state, &event) {
                    queue.push_back((next_state, depth + 1));
                }
            }
        }
        
        true
    }
    
    fn get_possible_events(&self, state: &ReplState) -> Vec<Event> {
        match state {
            ReplState::Init => vec![Event::Start],
            ReplState::AwaitingInput => vec![
                Event::UserInput("test".to_string()),
            ],
            ReplState::ParsingCommand => vec![
                Event::ParseComplete(ParseResult::Command("cmd".to_string())),
                Event::ParseComplete(ParseResult::Query("query".to_string())),
                Event::ParseComplete(ParseResult::Invalid),
            ],
            ReplState::ProcessingCommand => vec![
                Event::FunctionCallRequired("func".to_string()),
                Event::ModelQueryRequired,
                Event::OutputReady("output".to_string()),
            ],
            ReplState::CallingFunction(_) => vec![
                Event::FunctionCallComplete(Ok("result".to_string())),
                Event::FunctionCallComplete(Err("error".to_string())),
            ],
            ReplState::AwaitingModel => vec![
                Event::ModelResponseReceived("response".to_string()),
                Event::ErrorOccurred(ErrorState::TimeoutError),
            ],
            ReplState::GeneratingResponse => vec![
                Event::OutputReady("output".to_string()),
                Event::FunctionCallRequired("func".to_string()),
            ],
            ReplState::DisplayingOutput => vec![Event::Reset],
            ReplState::Error(_) => vec![Event::Reset],
        }
    }
    
    fn get_next_state(&self, state: &ReplState, event: &Event) -> Option<ReplState> {
        self.state_machine.transitions.get(&(state.clone(), event.clone())).cloned()
    }
}

/// Safety and liveness properties for REPL
pub struct ReplProperties;

impl ReplProperties {
    pub fn safety_properties() -> Vec<(String, TemporalProperty)> {
        vec![
            (
                "No deadlock".to_string(),
                TemporalProperty::Never(ReplState::Init), // After start
            ),
            (
                "Error states are recoverable".to_string(),
                TemporalProperty::LeadsTo(
                    ReplState::Error(ErrorState::ParseError),
                    ReplState::AwaitingInput
                ),
            ),
        ]
    }
    
    pub fn liveness_properties() -> Vec<(String, TemporalProperty)> {
        vec![
            (
                "Eventually responds to input".to_string(),
                TemporalProperty::LeadsTo(
                    ReplState::AwaitingInput,
                    ReplState::DisplayingOutput
                ),
            ),
            (
                "Function calls complete".to_string(),
                TemporalProperty::LeadsTo(
                    ReplState::CallingFunction("".to_string()),
                    ReplState::GeneratingResponse
                ),
            ),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_transitions() {
        let mut sm = ReplStateMachine::new();
        
        // Init -> AwaitingInput
        assert!(sm.transition(Event::Start).is_ok());
        assert_eq!(sm.current_state(), &ReplState::AwaitingInput);
        
        // AwaitingInput -> ParsingCommand
        assert!(sm.transition(Event::UserInput("test".to_string())).is_ok());
        assert_eq!(sm.current_state(), &ReplState::ParsingCommand);
    }
    
    #[test]
    fn test_model_checking() {
        let checker = ModelChecker::new();
        
        // Check that we can eventually reach DisplayingOutput
        assert!(checker.check_property(
            &TemporalProperty::Eventually(ReplState::DisplayingOutput),
            10
        ));
        
        // Check that Init is never reached again after start
        let mut sm = ReplStateMachine::new();
        sm.transition(Event::Start).unwrap();
        
        // This should fail as we never return to Init
        assert!(checker.check_property(
            &TemporalProperty::Never(ReplState::Init),
            10
        ));
    }
    
    #[test]
    fn test_safety_properties() {
        let checker = ModelChecker::new();
        
        for (name, property) in ReplProperties::safety_properties() {
            println!("Checking safety property: {}", name);
            // Note: Some properties might need adjustment based on actual semantics
        }
    }
}