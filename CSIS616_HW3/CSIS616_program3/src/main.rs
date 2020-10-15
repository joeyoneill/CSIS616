//! CSIS-616 program #3
//! Joseph O'Neill
//!
//! Original:
//! CSIS-616 - Program #2
//! Ralph W. Crosby PhD.
//!
//! # Usage
//!
//! cargo run filename
//!
//! where: `filename` is a yaml file containing the DFA definition
//!
//! # Input
//!
//! String to be evaluated by the graph
//!
//! # Output
//!
//! To `stderr`: Debug display of the internal graph structure
//!
//! To `stdout`: Graphviz definitions of the graph structure
//!
//! To println : Transition steps, acceptance of the string by the graph

use serde::Deserialize;
use std::io;
use std::io::BufRead;
use std::io::Write;
use std::process;

// *********************************************************************
/// # Deterministic Finite Automata Structure
///
/// Create a structure that the YAML files will be deserialized into.
/// Note the use of the `Deserialize` trait
///
#[derive(Debug, Deserialize)]
struct DFA {
    /// The set of characters comprising the alphabet
    alphabet: Vec<char>,

    /// State number (1 relative) for the start state
    start: usize,

    /// Set of accept states (1 relative)
    accept: Vec<usize>,

    /// Matrix of transitions, rows are states, columns characters in the alphabet
    transitions: Vec<Vec<usize>>,
}

// *********************************************************************
/// # Definition of a single state
#[derive(Debug)]
struct State {
    /// Is this an accept state
    accept_state: bool,

    /// Set of transitions (0 relative)
    transitions: Vec<usize>,
}

// *********************************************************************
/// # State based representation of the DFA
#[derive(Debug)]
struct StateGraph {
    /// The set of characters comprising the alphabet
    alphabet: Vec<char>,

    /// State number (0 relative) for the start state
    start_state: usize,

    /// Vector of state objects
    states: Vec<Box<State>>,
}

// *********************************************************************
fn main() {
    // Get and validat the filename on the command line
    let filename = get_filename(std::env::args());

    // Load the yaml file getting a Box pointing to a DFA
    // instance on the heap
    let dfa = DFA::new_from_file(&filename);

    // Validate the DFA
    dfa.validate().expect("Validation Failure:");

    // Get a state structure for the DFA
    let state_graph = StateGraph::new_from_dfa(&dfa);

    eprintln!("{:?}", state_graph);

    state_graph.write_graphviz();
    println!();

    // Get string
    println!("Please enter a string:");
    let stdin = io::stdin();
    let str_input = stdin.lock().lines().next().unwrap().unwrap();
    println!();

    // Make sure string only contains alphabet characters
    state_graph.check_input_alphabet(&str_input);

    // Make transition vec to compare to
    let transition_vec: Vec<Vec<String>> = Vec::new();
    let accept = state_graph.check_string(transition_vec, str_input);
    println!();

    // Gives output on the acceptance of the string by the graph
    if accept == true {
        println!("The string is accepted by the graph.");
    } else if accept == false {
        println!("The string is not accepted by the graph.");
    }
    println!();
}

// *********************************************************************
/// Return the filename passed as the first parameter
fn get_filename(args: std::env::Args) -> String {
    // Get the arguments as a vector
    let args: Vec<String> = args.collect();

    // Make sure only one argument was passed
    if args.len() != 2 {
        writeln!(std::io::stderr(), "Usage: hw1 dfafile").unwrap();
        std::process::exit(1);
    }
    args[1].to_string()
}

// *********************************************************************
/// Implement the methods of the DFA structure
impl DFA {
    /// Create and return a DFA on the heap
    ///
    /// Load the .yaml file specified into a DFA structure
    /// on the heap and return a point to it via a Box.

    fn new_from_file(filename: &str) -> Box<DFA> {
        let f = std::fs::File::open(filename).expect("Unable to open input");

        // Deserialize into the heap and return the pointer
        Box::new(serde_yaml::from_reader(f).expect("Unable to parse yaml"))
    }

    /// Validate the correctness of the DFA
    fn validate(&self) -> Result<(), String> {
        // The number of characters in the alphabet should match the number
        // of columns in each state row

        for (rnum, row) in self.transitions.iter().enumerate() {
            if row.len() != self.alphabet.len() {
                return Err(format!(
                    "Wrong number of columns({}) in row {}, should be {}",
                    row.len(),
                    rnum + 1,
                    self.alphabet.len()
                ));
            }
        }

        // Validate that all states in the transition table are valid
        for (rnum, row) in self.transitions.iter().enumerate() {
            for (cnum, state) in row.iter().enumerate() {
                if *state as usize > self.transitions.len() {
                    return Err(format!(
                        "Invalid transition state({}) in row {}, column {}",
                        state,
                        rnum + 1,
                        cnum + 1
                    ));
                }
            }
        }

        // The start and accept states must be valid
        if self.start as usize > self.transitions.len() {
            return Err(format!("Start state({}), is not valid", self.start));
        }

        for acc_state in self.accept.iter() {
            if *acc_state as usize > self.transitions.len() {
                return Err(format!("Accept state({}), is not valid", acc_state));
            }
        }

        Ok(())
    }
}

// *********************************************************************
/// Implement the methods of the State Graph structure
impl StateGraph {
    /// Create a state graph from a DFA structure
    fn new_from_dfa(dfa: &DFA) -> Box<StateGraph> {
        // Create an empty graph object
        let mut graph = Box::new(StateGraph {
            alphabet: dfa.alphabet.clone(),
            start_state: dfa.start - 1,
            states: vec![],
        });

        // Look through the transition table building state objects
        for row in dfa.transitions.iter() {
            let mut v = Box::new(State {
                accept_state: false,
                transitions: vec![],
            });
            for col in row {
                v.transitions.push(col - 1);
            }
            graph.states.push(v);
        }

        // Set the accept states
        for astate in dfa.accept.iter() {
            graph.states[*astate - 1].accept_state = true;
        }

        graph
    }

    /// Write the graph to stdout
    fn write_graphviz(&self) {
        println!("digraph {{");
        println!("\trankdir=LR;");
        println!("\tnode [shape=point]; start;");
        for (n, state) in self.states.iter().enumerate() {
            if state.accept_state {
                println!("\tnode [shape=doublecircle]; q{};", n + 1);
            }
        }
        println!("\tnode [shape=circle];");
        println!("\tstart -> q{}", self.start_state + 1);

        for (n, state) in self.states.iter().enumerate() {
            for (i, ch) in self.alphabet.iter().enumerate() {
                println!(
                    "\tq{} -> q{} [label=\"{}\"];",
                    n + 1,
                    state.transitions[i] + 1,
                    ch
                );
            }
        }

        println!("}}");
    }

    // checks that the input string only contains symbols from the alphabet
    fn check_input_alphabet(&self, s: &String) {
        for letter in s.chars() {
            let mut contains = false;
            for symbol in &self.alphabet {
                if letter == *symbol {
                    contains = true;
                    break;
                }
            }
            if contains == false {
                println!("Error: Character not in alphabet.");
                process::exit(1);
            }
        }
    }

    // fills all possible tranistions
    // checks string input against those transitions
    // prints out transitions for the string
    // returns true or false based on accepted or rejected
    fn check_string(&self, mut transition_vec: Vec<Vec<String>>, s: String) -> bool {
        // reads all transitions into a vector
        for (n, state) in self.states.iter().enumerate() {
            for (i, ch) in self.alphabet.iter().enumerate() {
                let v = vec![
                    format!("{}", n + 1),
                    format!("{}", ch),
                    format!("{}", state.transitions[i] + 1),
                ];
                transition_vec.push(v);
            }
        }

        // prints transitions of str_input by symbol in string
        let mut curr_state = format!("{}", 1);
        println!("Transition steps:");
        for letter in s.chars() {
            for (_pos, val) in transition_vec.iter().enumerate() {
                if val[0] == curr_state && val[1] == format!("{}", letter) {
                    println!("d(q{}, {}) -> q{}", val[0], val[1], val[2]);
                    curr_state = format!("{}", val[2]);
                    break;
                }
            }
        }

        // gets accept states
        let mut accept_states_s = vec![];
        for (n, state) in self.states.iter().enumerate() {
            if state.accept_state {
                accept_states_s.push(format!("{}", n + 1));
            }
        }

        // checks if the current state is an accept state
        let mut accept = false;
        for q in accept_states_s {
            if curr_state == q {
                accept = true;
                break;
            }
        }
        return accept;
    }
}

// Test Functions
#[test]
fn test_input_alphabet_function() {
    let alphabet = ['A', 'B'];
    let s = "ABBA";

    for letter in s.chars() {
        let mut contains = false;
        for symbol in &alphabet {
            if letter == *symbol {
                contains = true;
                break;
            }
        }
        assert_eq!(contains, true);
    }
}