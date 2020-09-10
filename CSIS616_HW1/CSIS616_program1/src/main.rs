/*
 *
 * Author: Joseph O'Neill
 * <oneillj1@g.cofc.edu>
 * CSIS 616 Project 1
 * 9/10/2020
 *
 */

use std::io::{self, Write};

fn main() {
    // Takes in argument as a string
    let string = std::env::args().nth(1).expect("no args given");

    // Turn argument and Strings to Vec<char> values for comparison
    let mut node_names: Vec<char> = string.chars().collect();
    let comma: Vec<char> = ",".chars().collect();

    // While loop's needed items initialized
    let mut n = node_names.len();
    let mut f = false;

    // While loop to remove the parsed commas
    while f == false {
        for x in 0..n {
            if node_names[x] == comma[0] {
                node_names.remove(x);
                n = n - 1;
                break;
            } else if x + 1 == n {
                f = true;
            }
        }
    }

    // Create digraph string format
    let mut digraph_txt: String =
        "digraph {\n\n\tnode [shape=point]; start;\n\tnode [shape=doublecircle]; ".to_owned();
    digraph_txt.push(node_names[node_names.len() - 1]);
    digraph_txt.push_str(";\n\tnode [shape=circle];\n\n");

    for x in 0..node_names.len() {
        digraph_txt.push_str("\t");

        if x == 0 {
            digraph_txt.push_str("start -> ");
            digraph_txt.push(node_names[x]);
            digraph_txt.push_str("\n");
        } else if x == node_names.len() - 1 {
            digraph_txt.push(node_names[x - 1]);
            digraph_txt.push_str(" -> ");
            digraph_txt.push(node_names[x]);
            digraph_txt.push_str(";");
            digraph_txt.push_str("\n");
            digraph_txt.push_str("\n}");
        } else {
            digraph_txt.push(node_names[x - 1]);
            digraph_txt.push_str(" -> ");
            digraph_txt.push(node_names[x]);
            digraph_txt.push_str(";");
            digraph_txt.push_str("\n");
        }
    }

    // Writes digraph_txt to stdout
    io::stdout().write_all(digraph_txt.as_bytes()).unwrap();
}

// Test functions
#[test]
fn test_string_mutability() {
    let mut test_string: String = "This is a ".to_owned();
    let true_string: String = "This is a test".to_owned();
    test_string.push_str("test");
    assert_eq!(test_string, true_string);
}
