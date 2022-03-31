use std::thread::current;

use tree_sitter::{Language, Node, Parser, Query, QueryCursor, TreeCursor};

extern "C" {
    fn tree_sitter_python() -> Language;
}

struct Sub {
    pat: Query,
    exp: Query,
}

const SUB_CAPTURE: &str = "sub"; // think @sub
const PREFIX: &str = "at_";
const SUB_KIND: &str = "identifier";
const SOURCE_CODE: &str = "x = 1 + 0";
const QUERY: &str = "(binary_operator (integer) @a (integer) @b) @sub";
const SUBST: &str = "7 + at_a + 1 - at_b + 7";

fn main() {
    let mut parser = Parser::new();
    let language = unsafe { tree_sitter_python() };
    parser.set_language(language).unwrap();

    let source_code = SOURCE_CODE;
    let tree = parser.parse(source_code, None).unwrap();
    let node = tree.root_node();

    dbg!(tree.root_node().to_sexp());
    let source_query = QUERY;
    let query = Query::new(language, source_query).unwrap();
    let sub_index = query
        .capture_index_for_name(SUB_CAPTURE)
        .expect("you must use an sub pattern to indicate which part of the AST to replace!");

    let exp_source_code = SUBST;
    let exp_tree = parser.parse(exp_source_code, None).unwrap();

    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&query, node, source_code.as_bytes());
    let first = matches.next().unwrap();
    let captures = first.captures;
    dbg!(captures);

    // Traverse the tree. would use recursion but lifetimes haha
    let mut exp_cursor = exp_tree.walk();
    let mut ascending = false;
    let mut edited_source = String::new();
    let mut last_modified = 0;
    loop {
        if ascending {
            if exp_cursor.goto_next_sibling() {
                ascending = false;
            } else if !exp_cursor.goto_parent() {
                break;
            }
        } else {
            // vvv what we do is here

            let current_node = exp_cursor.node();
            let kind = dbg!(current_node.kind());
            if kind == SUB_KIND {
                let range = dbg!(current_node.range());
                let iden_name = dbg!(&exp_source_code[range.start_byte..range.end_byte]);
                // we substitute the matching ast from the query
                if iden_name.starts_with(PREFIX) {
                    edited_source.push_str(&exp_source_code[last_modified..range.start_byte]);
                    last_modified = range.end_byte;
                    dbg!(&edited_source);

                    let capture_name = &iden_name[PREFIX.len()..];
                    dbg!(capture_name);

                    let capture_index = query
                        .capture_index_for_name(capture_name)
                        .expect("unknown capture group");
                    dbg!(capture_index);

                    let mut nodes = first.nodes_for_capture_index(capture_index);
                    let capture_sub = nodes.next().unwrap();
                    dbg!(capture_sub);
                    let original_range = capture_sub.byte_range();
                    let source_fragment = &source_code[original_range.start..original_range.end];
                    dbg!(source_fragment);
                    edited_source.push_str(&source_fragment);
                }
            }

            // ^^^ too far!
            if !exp_cursor.goto_first_child() {
                ascending = true;
            }
        }
    }

    edited_source.push_str(&exp_source_code[last_modified..]);

    let location = first.nodes_for_capture_index(sub_index).next().unwrap();
    dbg!(location.to_sexp());
    let location_range = location.range();
    let location_contents = &source_code[location_range.start_byte..location_range.end_byte];
    let mut spliced_source = String::new();
    spliced_source.push_str(&source_code[0..location_range.start_byte]);
    spliced_source.push_str(&edited_source);
    spliced_source.push_str(&source_code[location_range.end_byte..]);

    println!("\nDONE!\n");
    println!("given the following source code:\n{}\n", source_code);
    println!("I matched the following query:\n{}\n", source_query);
    println!(
        "and then used the following substitution:\n{}\n",
        exp_source_code
    );
    println!(
        "to produce the following spliced fragment:\n{}\n",
        edited_source
    );
    println!(
        "which (via @{}) corresponds to the following location in the source code:\n{}\n",
        SUB_CAPTURE, location_contents
    );
    println!(
        "using this information, I was able to reconstruct the source with the edit applied:\n{}\n",
        spliced_source
    )
}
