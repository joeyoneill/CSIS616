use serde::Deserialize;
use std::io::{self, Write};

// Deterministic Finite Automata Structure
#[derive(Debug, Deserialize)]
struct DFA {
    alphabet: Vec<char>,
    start: u32,
    accept: Vec<u32>,
    transitions: Vec<Vec<u32>>,

    #[serde(default)]
    n_states: usize,
    #[serde(default)]
    states: Vec<u32>,
}

// new graph struct
#[derive(Debug, Deserialize)]
struct Graph {
    dfa: Box<DFA>,
    distructure: String,
    nodes: Vec<String>,
}

fn main() {
    let filename = get_filename(std::env::args());

    // Load the yaml file getting a Box pointing to a DFA
    // instance on the heap
    let mut d = DFA::new_from_file(&filename);
    d.compute_states();
    d.save_states();

    // 2a. makes sure symbols and states are defined
    d.check_defined();

    // 2b. all states referenced in transitions are valid
    d.check_tranistion_validity();

    // 2c. the start and accept states are valid
    d.check_start_accept_validity();

    // initialize new graph data structure
    let mut g = Graph {
        dfa: d,
        distructure: "".to_owned(),
        nodes: [].to_vec(),
    };

    // loads nodes
    g.load_node();

    // 4. Write to print
    println!("g by nodes:");
    g.print_graph();

    // creates graphviz diagraph structure
    g.distructure = g.create_digraph();

    // 5. writes graph structure as graphviz digraph to stdout
    println!();
    g.write_structure();
}

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

impl DFA {
    /// Load the .yaml file specified into a DFA structure
    fn new_from_file(filename: &str) -> Box<DFA> {
        let f = std::fs::File::open(filename).expect("Unable to open input");
        Box::new(serde_yaml::from_reader(f).expect("Unable to parse yaml"))
    }

    /// Compute the number of states
    fn compute_states(&mut self) {
        self.n_states = self.transitions.len();
    }

    // Adds all states to a vector
    fn save_states(&mut self) -> bool {
        let temp_start: u32 = self.start;
        self.states.push(temp_start);

        for state in &self.accept {
            let temp_state = state;
            self.states.push(*temp_state);
        }

        assert_eq!(self.states.len(), self.n_states);
        true
    }

    // 2a. Transitions are defined for symbols and states
    fn check_defined(&self) -> bool {
        assert_eq!(self.alphabet.is_empty(), false);
        assert_eq!(self.transitions.is_empty(), false);
        assert_eq!(self.accept.is_empty(), false);
        true
    }

    // 2b. All states referenced in the transition table are valid
    fn check_tranistion_validity(&self) -> bool {
        let mut b: bool = true;

        for transition in &self.transitions {
            if !self.states.contains(&transition[0]) {
                b = false;
                break;
            } else if !self.states.contains(&transition[1]) {
                b = false;
                break;
            }
        }
        assert_eq!(b, true);
        true
    }

    // 2c. the start and accept states are valid
    fn check_start_accept_validity(&self) -> bool {
        let mut b: bool = true;

        // checks start state is valid
        if !self.states.contains(&self.start) {
            b = false;
        }
        assert_eq!(b, true);

        // checks accept states are valid
        for state in &self.accept {
            if !self.states.contains(state) {
                b = false;
                break;
            }
        }
        assert_eq!(b, true);
        true
    }
}

impl Graph {
    // writes structure to graphviz
    fn write_structure(&self) {
        io::stdout().write_all(self.distructure.as_bytes()).unwrap();
    }

    // creates graph in digraph form for structure
    fn create_digraph(&self) -> String {
        // Create digraph string format
        let mut digraph_txt: String =
            "digraph {\n\n\tnode [shape=point]; start;\n\tnode [shape=doublecircle]; ".to_owned();
        //accept states
        for i in 0..self.dfa.accept.len() {
            if i != (self.dfa.accept.len() - 1) {
                let s: String = format!("{}, ", self.dfa.accept[i]).to_owned();
                let s_slice: &str = &*s;
                digraph_txt.push_str(s_slice);
            } else {
                let s: String = format!("{}", self.dfa.accept[i]).to_owned();
                let s_slice: &str = &*s;
                digraph_txt.push_str(s_slice);
            }
        }
        digraph_txt.push_str(";\n\tnode [shape=circle];\n\n");

        // Create the start state
        let s: String = format!("\tstart -> {};\n", self.dfa.start).to_owned();
        let s_slice: &str = &*s;
        digraph_txt.push_str(s_slice);
        //digraph_txt.push_str(format!("\tstart -> {};\n", self.start));

        // transitions
        for i in 0..self.dfa.transitions.len() {
            let s_trans: String = format!(
                "\t{} -> {};\n",
                self.dfa.transitions[i][0], self.dfa.transitions[i][1]
            );
            let s_trans_slice: &str = &*s_trans;
            digraph_txt.push_str(s_trans_slice);
        }

        digraph_txt.push_str("\n\n}");

        digraph_txt
    }

    // this function loads the nodes
    // 3d. () around a number means it is an accept state
    fn load_node(&mut self) {
        // 3b. start node
        self.nodes.push(format!(" -> {}", self.dfa.start));

        //
        for transition in &self.dfa.transitions {
            if self.dfa.accept.contains(&transition[0]) && self.dfa.accept.contains(&transition[1])
            {
                self.nodes
                    .push(format!("({}) -> ({})", transition[0], transition[1]));
            } else if self.dfa.accept.contains(&transition[0]) {
                self.nodes
                    .push(format!("({}) -> {}", transition[0], transition[1]));
            } else if self.dfa.accept.contains(&transition[1]) {
                self.nodes
                    .push(format!("{} -> ({})", transition[0], transition[1]));
            } else {
                self.nodes
                    .push(format!("{} -> {}", transition[0], transition[1]));
            }
        }
    }

    // 4. write a function to print
    fn print_graph(&self) {
        for node in &self.nodes {
            println!("{}", node);
        }
    }
}

// Test Functions
#[test]
fn test_validity_funcs() {
    let d = DFA {
        start : 1,
        transitions : [[1,1].to_vec()].to_vec(),
        accept : [1].to_vec(),
        n_states : 1,
        alphabet : ['a','b'].to_vec(),
        states : [1].to_vec(),
    };

    let b_transition = d.check_tranistion_validity();
    let b_start_accept = d.check_start_accept_validity();

    assert_eq!(b_transition, true);
    assert_eq!(b_start_accept, true);
}

