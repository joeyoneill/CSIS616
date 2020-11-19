use serde::Deserialize;
use std::io::Write;

// ***********************************************************************
///
#[derive(Debug, Deserialize)]
struct PDA {
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
/// # State based representation of the PDA
#[derive(Debug)]
struct StateGraph {
    /// The set of characters comprising the alphabet
    alphabet: Vec<char>,

    /// State number (0 relative) for the start state
    start_state: usize,

    /// Vector of state objects
    states: Vec<Box<State>>,
}

fn main() {
    // Get and validat the filename on the command line
    let filename = get_filename(std::env::args());

    // Load the yaml file getting a Box pointing to a DFA
    // instance on the heap
    let pda = PDA::new_from_file(&filename);

    // Validate the DFA
    pda.validate().expect("Validation Failure:");

    println!("{:?}", pda);

    // Get a state structure for the DFA
    let state_graph = StateGraph::new_from_pda(&pda);

    // 4. graph printed to debug format
    println!();
    eprintln!("{:?}", state_graph);
    println!();

    // 5. stdout GraphViz definition
    state_graph.write_graphviz(&pda);
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
/// Implement the methods of the NFA structure
impl PDA {
    fn new_from_file(filename: &str) -> Box<PDA> {
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
    fn new_from_pda(pda: &PDA) -> Box<StateGraph> {
        // Create an empty graph object
        let mut graph = Box::new(StateGraph {
            alphabet: pda.alphabet.clone(),
            start_state: pda.start,
            states: vec![],
        });

        // Look through the transition table building state objects
        for row in pda.transitions.iter() {
            let mut v = Box::new(State {
                accept_state: false,
                transitions: vec![],
            });
            for col in row {
                v.transitions.push(*col);
            }
            graph.states.push(v);
        }

        // Set the accept states
        for astate in pda.accept.iter() {
            graph.states[*astate].accept_state = true;
        }

        graph
    }

    /// Write the graph to stdout
    fn write_graphviz(&self, pda: &PDA) {
        println!("digraph {{");
        println!("\trankdir=LR;");
        println!("\tnode [shape=point]; start;");
        for (n, state) in self.states.iter().enumerate() {
            if state.accept_state {
                println!("\tnode [shape=doublecircle]; q{};", n);
            }
        }
        println!("\tnode [shape=circle];");
        println!("\tstart -> q{}", self.start_state);

        for transition in &pda.transitions {
            if transition[0] == self.start_state {
                // first state
                println!(
                    "\tq{} -> q{} [label=\"{}, {} -> {}\"];",
                    self.start_state, transition[1], "e", "e", "$"
                );
            } else if pda.accept.iter().any(|&i| i == transition[1]) {
                // if it it entering accept state
                println!(
                    "\tq{} -> q{} [label=\"{}, {} -> {}\"];",
                    transition[0], transition[1], "e", "$", "e"
                );
            } else if transition[0] != transition[1] {
                // if it is transitioning
                println!(
                    "\tq{} -> q{} [label=\"{}, {} -> {}\"];",
                    transition[0], transition[1], "e", "e", "e"
                );
            } else if transition[0] == transition[1] && transition[0] == 2 {
                for letter in &pda.alphabet {
                    println!(
                        "\tq{} -> q{} [label=\"{}, {} -> {}\"];",
                        transition[0], transition[1], letter, "e", letter
                    );
                }
            } else {
                for letter in &pda.alphabet {
                    println!(
                        "\tq{} -> q{} [label=\"{}, {} -> {}\"];",
                        transition[0], transition[1], letter, letter, "e"
                    );
                }
            }
        }
        println!("}}");
    }
}

// *********************************************************************
// Test Functions
#[test]
fn test_alphabet_loads_properly() {
    let mut transitions: Vec<Vec<usize>> = Vec::new();
    transitions.push(vec![1, 2]);
    transitions.push(vec![2, 2]);
    transitions.push(vec![2, 3]);
    transitions.push(vec![3, 3]);
    transitions.push(vec![3, 4]);

    let pda: PDA = PDA {
        alphabet: "xy".chars().collect(),
        start: 1,
        accept: vec![4],
        transitions: transitions,
    };

    // Get a state structure for the DFA
    let state_graph = StateGraph::new_from_pda(&pda);

    eprintln!("{:?}", state_graph);

    assert_eq!(pda.alphabet, state_graph.alphabet);
}
