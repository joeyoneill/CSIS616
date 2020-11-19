use std::fs;
use std::io;
use std::io::BufRead;
use std::io::Write;
use std::process;

// ***********************************************************************
///
#[derive(Debug)]
struct NFA {
    /// The set of characters comprising the alphabet
    alphabet: Vec<char>,

    /// State number (1 relative) for the start state
    start: usize,

    /// Set of accept states (1 relative)
    accept: Vec<usize>,

    /// Matrix of transitions, rows are states, columns characters in the alphabet
    transitions: Vec<Vec<usize>>,
    /// Matrix of transition's symbols
    transition_symbols: Vec<Vec<char>>,

    // All states
    states: Vec<usize>,
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
    /// NFA for the state graph
    nfa: NFA,

    /// Vector of state objects
    states: Vec<State>,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut input = String::new();

    if args.len() == 1 {
        // Prompt
        println!("User input required: ");
        io::stdin()
            .read_line(&mut input)
            .ok()
            .expect("Couldn't read line");
    } else {
        // file
        // Get and validate the filename on the command line
        let filename = get_filename(std::env::args());

        // open the file
        input = fs::read_to_string(filename).expect("Something went wrong reading the file");
    }

    // Splits regEx into vector of chars
    let reg_ex: Vec<char> = input.trim_end().chars().collect();

    // Makes sure the RegEx will not be rejected
    check_reg_ex_chars(&reg_ex);

    // Get alphabet
    let alphabet = get_alphabet(&reg_ex);

    // First Parse of original regex
    let mut expressions: Vec<Vec<char>> = parse_original(&reg_ex);

    // Parse the expressions
    expressions = simplify_expressions(&expressions);

    // Get number of states
    let states: Vec<usize> = get_states(&expressions);

    // Start state alwasy 1
    let start: usize = 1;

    // Get trainsitions
    let transitions: Vec<Vec<usize>> = get_transitions(&expressions);

    // Get accept states
    let accept_states: Vec<usize> = get_accept_states(&expressions);

    // Get transition symbols
    let transition_symbols: Vec<Vec<char>> = get_transition_symbols(&expressions);

    // Initialize the NFA
    let nfa: NFA = NFA {
        alphabet: alphabet,
        start: start,
        accept: accept_states,
        transitions: transitions,
        transition_symbols: transition_symbols,
        states: states,
    };

    // Initialize states for StateGraph
    let mut state_graph_states: Vec<State> = Vec::new();
    for state in &nfa.states {
        let mut state_transitions: Vec<usize> = Vec::new();
        let mut state_accept_state: bool = false;

        // gets all states current state transitions to
        for transition in &nfa.transitions {
            if state == &transition[0] {
                state_transitions.push(transition[1]);
            } else if state == &transition[1] {
                state_transitions.push(transition[0]);
            }
        }

        // Finds if state is an accept state
        for num in &nfa.accept {
            if state == num {
                state_accept_state = true;
                break;
            }
        }
        let curr_state: State = State {
            accept_state: state_accept_state,
            transitions: state_transitions,
        };

        state_graph_states.push(curr_state);
    }

    // Initialize the StateGraph
    let state_graph: StateGraph = StateGraph {
        nfa: nfa,
        states: state_graph_states,
    };

    // Write graphviz
    state_graph.write_graphviz();

    // Get input string
    println!("Please enter a string:");
    let stdin = io::stdin();
    let str_input = stdin.lock().lines().next().unwrap().unwrap();
    println!();

    // Make sure string only contains alphabet characters
    state_graph.check_input_alphabet(&str_input);

    // Make transition vec to compare to
    let accept = state_graph.check_string(&str_input);
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
/// Checks input regular expression for errors
fn check_reg_ex_chars(reg_ex: &Vec<char>) {
    // Vectors for comparison
    let first_reject_chars: Vec<char> = ")|* ".chars().collect();
    let accepted_chars: Vec<char> = "abcdefghijklmnopqrstuvwxyz0123456789*|() "
        .chars()
        .collect();
    // test chars against these
    let front_or_reject: Vec<char> = "*|)".chars().collect();
    let star_reject: Vec<char> = "*".chars().collect();

    // test chars
    let front_char: Vec<char> = "(".chars().collect();
    let star_char: Vec<char> = "*".chars().collect();
    let or_char: Vec<char> = "|".chars().collect();

    // checks the first character is allowed
    for character in first_reject_chars {
        if reg_ex[0] == character {
            println!("Error: RegEx not accepted");
            std::process::exit(1);
        }
    }

    // checks all characters in RegEx are allowed
    for character in reg_ex {
        if accepted_chars.iter().any(|&i| i == *character) {
        } else {
            println!("Error: {} is not an accepted character.", character);
            std::process::exit(1);
        }
    }

    // check (, | following symbol
    let reg_ex_len = reg_ex.len();
    for i in 0..reg_ex_len - 1 {
        if reg_ex[i] == front_char[0] || reg_ex[i] == or_char[0] {
            for character in &front_or_reject {
                if character == &reg_ex[i + 1] {
                    println!(
                        "Error: '{}' cannot be immediately follwed by '{}'.",
                        reg_ex[i],
                        reg_ex[i + 1]
                    );
                    std::process::exit(1);
                }
            }
        }
    }

    // check * following symbol
    for i in 0..reg_ex_len - 1 {
        if reg_ex[i] == star_char[0] {
            if reg_ex[i + 1] == star_reject[0] {
                println!(
                    "Error: '{}' cannot be immediately follwed by '{}'.",
                    reg_ex[i],
                    reg_ex[i + 1]
                );
                std::process::exit(1);
            }
        }
    }

    // make sure the regex does not end in (, |
    let end_reject: Vec<char> = "(|".chars().collect();
    for character in end_reject {
        if reg_ex[reg_ex_len - 1] == character {
            println!(
                "Error: Regular Expression cannot end on '{}'.",
                reg_ex[reg_ex_len - 1]
            );
            std::process::exit(1);
        }
    }

    // the valid parentheses problem
    let parentheses: Vec<char> = "()".chars().collect();
    let stack_bottom: Vec<char> = "$".chars().collect();
    let mut p_stack: Vec<char> = "$".chars().collect();
    for character in reg_ex {
        if character == &parentheses[0] {
            p_stack.push(*character);
        } else {
            if character == &parentheses[1] {
                p_stack.pop();
            }
        }
    }
    if p_stack.is_empty() {
        println!("Error: Parentheses are not valid.");
        std::process::exit(1);
    }
    if p_stack[0] != stack_bottom[0] {
        println!("Error: Parentheses are not valid.");
        std::process::exit(1);
    }
    if p_stack.len() > 1 {
        println!("Error: Parentheses are not valid.");
        std::process::exit(1);
    }
}

// *********************************************************************
/// Parses the regular expression for its alphabet symbols
fn get_alphabet(reg_ex: &Vec<char>) -> Vec<char> {
    // instantiate alphabet vector
    let mut alphabet: Vec<char> = "".chars().collect();

    // instantiate non-alphabet symbols vector for comparison
    let non_alphabet_chars: Vec<char> = "())|* ".chars().collect();

    // Get one of each character and append to alphabet
    for character in reg_ex {
        if non_alphabet_chars.iter().any(|&i| i == *character) {
        } else if alphabet.iter().any(|&i| i == *character) {
        } else {
            alphabet.push(*character);
        }
    }

    return alphabet;
}

// *********************************************************************
/// First parse to the regular expression to seperate into expressions
fn parse_original(reg_ex: &Vec<char>) -> Vec<Vec<char>> {
    // return value
    let mut expressions: Vec<Vec<char>> = Vec::new();

    // symbols: '(' -> [0], ')' -> [1], '|' -> [2]
    let symbols: Vec<char> = "()|".chars().collect();

    // Temportary Vec for current expression being parsed
    let mut curr_expression: Vec<char> = Vec::new();

    // stack to keep track of parentheses
    let mut p_stack: Vec<char> = Vec::new();

    // goes through the regular expression and parses into expressions
    for character in reg_ex {
        if character == &symbols[0] {
            // '(' -> [0]
            p_stack.push(*character);
            curr_expression.push(*character);
        } else if character == &symbols[1] {
            // ')' -> [1]
            p_stack.pop();
            curr_expression.push(*character);
        } else if character == &symbols[2] && p_stack.is_empty() {
            // '|' -> [2]
            expressions.push(curr_expression);
            curr_expression = Vec::new();
        } else {
            curr_expression.push(*character);
        }
    }

    // push curr_expression at end to add last expression and return
    expressions.push(curr_expression);
    return expressions;
}

// *********************************************************************
/// Further recursive parsing of regular expression into expressions
fn simplify_expressions(expressions: &Vec<Vec<char>>) -> Vec<Vec<char>> {
    // return value
    let mut expressions: Vec<Vec<char>> = expressions.to_vec();

    // add new expressions to append later
    let mut new_expressions: Vec<Vec<char>> = Vec::new();

    // symbols: '(' -> [0], ')' -> [1], '|' -> [2], '*' -> [3]
    let symbols: Vec<char> = "()|*".chars().collect();

    // Vec to know which to retain
    let mut bool_retain: Vec<bool> = Vec::new();

    for expression in &expressions {
        if expression[0] == symbols[0] && expression[expression.len() - 1] == symbols[1] {
            // if first character == '(' and last character == ')'
            bool_retain.push(false);

            let simplified_expressions: Vec<Vec<char>> =
                simplify_parentheses_end_parentheses(&expression);
            for item in simplified_expressions {
                new_expressions.push(item);
            }
        } else if expression[0] == symbols[0]
            && expression[expression.len() - 1] == symbols[3]
            && expression[expression.len() - 2] == symbols[1]
        {
            // i.e (...)*
            bool_retain.push(false);

            let simplified_expressions: Vec<Vec<char>> = simplify_star_parentheses(&expression);
            for item in simplified_expressions {
                new_expressions.push(item);
            }
        } else {
            bool_retain.push(true);
        }
    }
    let mut i = 0;
    expressions.retain(|_| (bool_retain[i], i += 1).0);

    expressions.append(&mut new_expressions);

    return expressions;
}

// *********************************************************************
/// Simplify expresions like in this format: (...)
fn simplify_parentheses_end_parentheses(expression: &Vec<char>) -> Vec<Vec<char>> {
    // symbols: '(' -> [0], ')' -> [1], '|' -> [2]
    let symbols: Vec<char> = "()|*".chars().collect();

    // stack to keep track of parentheses
    let mut p_stack: Vec<char> = Vec::new();

    // Vec to know which characters to retain
    let mut bool_retain: Vec<bool> = Vec::new();
    // saves the original expression for mutation purposes
    let mut start_expression: Vec<char> = expression.to_vec();

    // Removing the outside parentheses
    for character in &start_expression {
        if character == &symbols[0] {
            if p_stack.is_empty() {
                bool_retain.push(false);
            } else {
                bool_retain.push(true);
            }
            p_stack.push(*character);
        } else if character == &symbols[1] {
            if p_stack.len() == 1 {
                bool_retain.push(false);
            } else {
                bool_retain.push(true);
            }
            p_stack.pop();
        } else {
            bool_retain.push(true);
        }
    }
    let mut i = 0;
    start_expression.retain(|_| (bool_retain[i], i += 1).0);

    // parse as if it was original
    let mut expressions: Vec<Vec<char>> = parse_original(&start_expression);

    // Simplify
    expressions = simplify_expressions(&expressions);

    return expressions;
}

// *********************************************************************
/// Simplify expresions like in this format: (...)*
fn simplify_star_parentheses(expression: &Vec<char>) -> Vec<Vec<char>> {
    // return value
    let mut expressions: Vec<Vec<char>> = Vec::new();

    // symbols: '(' -> [0], ')' -> [1], '|' -> [2], '*' -> [3]
    let symbols: Vec<char> = "()|*".chars().collect();

    // Initialize stack for parentheses
    let mut p_stack: Vec<char> = Vec::new();

    //
    let mut curr_expression: Vec<char> = Vec::new();

    for character in expression {
        if character == &symbols[0] {
            // if character = '('
            if !p_stack.is_empty() {
                curr_expression.push(*character);
            }
            p_stack.push(*character);
        } else if character == &symbols[1] {
            // else if character = ')'
            if p_stack.len() == 1 {
                break;
            }
            p_stack.pop();
            curr_expression.push(*character);
        } else if character == &symbols[2] && p_stack.len() == 1 {
            // else if character = '|'
            expressions.push(curr_expression);
            curr_expression = Vec::new();
        } else {
            curr_expression.push(*character);
        }
    }

    expressions.push(curr_expression);

    // simplify
    expressions = simplify_expressions(&expressions);

    // wrap all expressions in (expression)*
    let mut wrapped_expressions: Vec<Vec<char>> = Vec::new();

    for item in expressions {
        curr_expression = "(".chars().collect();
        for character in item {
            curr_expression.push(character)
        }
        curr_expression.push(symbols[1]);
        curr_expression.push(symbols[3]);
        wrapped_expressions.push(curr_expression);
    }

    return wrapped_expressions;
}

// *********************************************************************
/// Gets states from expressions for graph
fn get_states(expressions: &Vec<Vec<char>>) -> Vec<usize> {
    // Return value
    let mut states: Vec<usize> = Vec::new();

    // symbols: '(' -> [0], ')' -> [1], '|' -> [2], '*' -> [3]
    let symbols: Vec<char> = "()|* ".chars().collect();

    // Initialize counter and push state 1 (start state)
    let mut n: usize = 1;
    states.push(n);

    // Get States
    for expression in expressions {
        for character in expression {
            if symbols.iter().any(|&i| i == *character) {
                // do nothing / skip
            } else {
                n = n + 1;
                states.push(n);
            }
        }
    }

    return states;
}

// *********************************************************************
/// Gets all transitions of states
fn get_transitions(expressions: &Vec<Vec<char>>) -> Vec<Vec<usize>> {
    // Return Value
    let mut transitions: Vec<Vec<usize>> = Vec::new();
    // symbols: '(' -> [0], ')' -> [1], '|' -> [2], '*' -> [3]
    let symbols: Vec<char> = "()|* ".chars().collect();

    // Initialize holder for current transition
    let mut curr_transition: Vec<usize> = Vec::new();

    // Initialize counter and push state 1 (start state)
    let mut n: usize = 2;

    // Get Transitions
    for expression in expressions {
        if expression[0] == symbols[0]
            && expression[expression.len() - 1] == symbols[3]
            && expression[expression.len() - 2] == symbols[1]
        {
            // if (...)*
            let mut p_stack: Vec<char> = Vec::new();

            // First transition connected to first state
            p_stack.push(expression[0]);
            curr_transition.push(1);
            curr_transition.push(n);
            transitions.push(curr_transition);
            curr_transition = Vec::new();
            n = n + 1;

            for i in 2..expression.len() {
                if p_stack.is_empty() && expression[i] == symbols[3] {
                    // end of expression
                    curr_transition.push(n - 1);
                    curr_transition.push(1);
                    transitions.push(curr_transition);
                    curr_transition = Vec::new();
                } else if expression[i] == symbols[0] {
                    p_stack.push(expression[i]);
                } else if expression[i] == symbols[1] {
                    p_stack.pop();
                } else if !p_stack.is_empty() && expression[i] == symbols[3] {
                    curr_transition.push(n - 1);
                    curr_transition.push(n - 1);
                    transitions.push(curr_transition);
                    curr_transition = Vec::new();
                } else {
                    curr_transition.push(n - 1);
                    curr_transition.push(n);
                    transitions.push(curr_transition);
                    curr_transition = Vec::new();
                    n = n + 1;
                }
            }
        } else {
            // First transition connected to first state
            curr_transition.push(1);
            curr_transition.push(n);
            transitions.push(curr_transition);
            curr_transition = Vec::new();
            n = n + 1;

            for i in 1..expression.len() {
                if expression[i] == symbols[3] {
                    // character == '*'
                    curr_transition.push(n - 1);
                    curr_transition.push(n - 1);
                    transitions.push(curr_transition);
                    curr_transition = Vec::new();
                } else {
                    curr_transition.push(n - 1);
                    curr_transition.push(n);
                    transitions.push(curr_transition);
                    curr_transition = Vec::new();
                    n = n + 1;
                }
            }
        }
    }

    return transitions;
}

// *********************************************************************
/// Gets symbols for transitions
fn get_transition_symbols(expressions: &Vec<Vec<char>>) -> Vec<Vec<char>> {
    // Return Value
    let mut transition_symbols: Vec<Vec<char>> = Vec::new();
    // symbols: '(' -> [0], ')' -> [1], '|' -> [2], '*' -> [3]
    let symbols: Vec<char> = "()|* ".chars().collect();

    // Transition alphabet
    let mut curr_transition: Vec<char> = Vec::new();

    let mut begin_chars: Vec<char> = Vec::new();

    // Get the first symbol of each expression
    for expression in expressions {
        if expression[0] == symbols[0]
            && expression[expression.len() - 1] == symbols[3]
            && expression[expression.len() - 2] == symbols[1]
        {
            // if (...)*
            begin_chars.push(expression[1])
        } else {
            begin_chars.push(expression[0]);
        }
    }

    // Get Transitions
    for expression in expressions {
        if expression[0] == symbols[0]
            && expression[expression.len() - 1] == symbols[3]
            && expression[expression.len() - 2] == symbols[1]
        {
            // if (...)*
            let mut p_stack: Vec<char> = Vec::new();

            // First transition connected to first state
            curr_transition.push(expression[1]);
            transition_symbols.push(curr_transition);
            curr_transition = Vec::new();

            for i in 2..expression.len() {
                if p_stack.is_empty() && expression[i] == symbols[3] {
                    // end of expression
                    // !FIX THIS!
                    for character in &begin_chars {
                        curr_transition.push(*character);
                    }
                    transition_symbols.push(curr_transition);
                    curr_transition = Vec::new();
                } else if expression[i] == symbols[0] {
                    p_stack.push(expression[i]);
                } else if expression[i] == symbols[1] {
                    p_stack.pop();
                } else if !p_stack.is_empty() && expression[i] == symbols[3] {
                    curr_transition.push(expression[i - 1]);
                    transition_symbols.push(curr_transition);
                    curr_transition = Vec::new();
                } else {
                    curr_transition.push(expression[i]);
                    transition_symbols.push(curr_transition);
                    curr_transition = Vec::new();
                }
            }
        } else {
            // First transition connected to first state
            curr_transition.push(expression[0]);
            transition_symbols.push(curr_transition);
            curr_transition = Vec::new();

            for i in 1..expression.len() {
                if expression[i] == symbols[3] {
                    // character == '*'
                    curr_transition.push(expression[i - 1]);
                    transition_symbols.push(curr_transition);
                    curr_transition = Vec::new();
                } else {
                    curr_transition.push(expression[i]);
                    transition_symbols.push(curr_transition);
                    curr_transition = Vec::new();
                }
            }
        }
    }

    return transition_symbols;
}

// *********************************************************************
/// Gets all accept states from parsing expressions
fn get_accept_states(expressions: &Vec<Vec<char>>) -> Vec<usize> {
    // Return Value
    let mut accept_states: Vec<usize> = Vec::new();
    // Symbols: '(' -> [0], ')' -> [1], '|' -> [2], '*' -> [3]
    let symbols: Vec<char> = "()|* ".chars().collect();

    // Initialize state counter
    let mut n: usize = 2;

    // Iterate through expressions and find accept states
    for expression in expressions {
        if expression[0] == symbols[0]
            && expression[expression.len() - 1] == symbols[3]
            && expression[expression.len() - 2] == symbols[1]
        {
            // if (...)*
            accept_states.push(1);

            for character in expression {
                if symbols.iter().any(|&i| i == *character) {
                    // do nothing / skip
                } else {
                    n = n + 1;
                }
            }

            accept_states.push(n - 1);
        } else {
            for character in expression {
                if symbols.iter().any(|&i| i == *character) {
                    // do nothing / skip
                } else {
                    n = n + 1;
                }
            }
            accept_states.push(n - 1);
        }
    }

    // sort and make unique
    accept_states.sort();
    accept_states.dedup();

    return accept_states;
}

// *********************************************************************
/// Implement the methods of the NFA structure
impl NFA {}

// *********************************************************************
/// Implement the methods of the State Graph structure
impl StateGraph {
    /// Write the graph to stdout
    fn write_graphviz(&self) {
        println!("digraph {{");
        println!("\trankdir=LR;");
        println!("\tnode [shape=point]; start;");

        // Accept states
        for accept_state in &self.nfa.accept {
            println!("\tnode [shape = doublecircle]; q{};", accept_state);
        }

        println!("\tnode [shape=circle];");

        // Start State
        println!("\tstart -> q{}", self.nfa.start);

        // Transitions
        let mut i: usize = 0;
        for transition in &self.nfa.transitions {
            for n in 0..self.nfa.transition_symbols[i].len() {
                println!(
                    "\tq{} -> q{} [label=\"{}\"]",
                    transition[0], transition[1], self.nfa.transition_symbols[i][n]
                );
            }
            i = i + 1;
        }

        println!("}}");
    }

    // checks that the input string only contains symbols from the alphabet
    fn check_input_alphabet(&self, s: &String) {
        for letter in s.chars() {
            let mut contains = false;
            for symbol in &self.nfa.alphabet {
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

    //
    fn check_string(&self, input: &String) -> bool {
        let input_as_chars: Vec<char> = input.chars().collect();
        let mut transition_vec: Vec<Vec<char>> = Vec::new();

        let mut i_ts = 0;
        for transition in &self.nfa.transitions {
            for symbol in &self.nfa.transition_symbols[i_ts] {
                let v = format!("{}{}{}", transition[0], transition[1], symbol)
                    .chars()
                    .collect();
                transition_vec.push(v);
            }
            i_ts = i_ts + 1;
        }

        // prints transitions of str_input by symbol in string
        let mut curr_state: Vec<char> = "1".chars().collect();
        let one_char: Vec<char> = "1".chars().collect();

        let mut transition_count: Vec<String> = Vec::new();
        for letter in &input_as_chars {
            for i in 0..transition_vec.len() {
                // if curr_state goes back to q1 -> it is not read
                if curr_state[0] == transition_vec[i][0] && transition_vec[i][1] == one_char[0] {
                    curr_state[0] = transition_vec[i][1];
                }
                if curr_state[0] == transition_vec[i][0] && letter == &transition_vec[i][2] {
                    transition_count.push(format!(
                        "d(q{}, {}) -> q{}",
                        transition_vec[i][0], transition_vec[i][2], transition_vec[i][1]
                    ));
                    curr_state[0] = transition_vec[i][1];
                    break;
                }
            }
        }

        // is it in accept state?
        let mut string_in_accept: bool = false;
        for accept_state in &self.nfa.accept {
            let accept_as_char: Vec<char> = format!("{}", accept_state).chars().collect();
            if curr_state[0] == accept_as_char[0] {
                string_in_accept = true;
                break;
            }
        }

        if string_in_accept == true && transition_count.len() == input_as_chars.len() {
            println!("Transition steps:");
            for t in transition_count {
                println!("{}", t);
            }
            return true;
        } else {
            return false;
        }
    }
}
