use std::thread::current;

use tree_sitter::{
    Language, Node, Parser, Query, QueryCursor, QueryMatch, Range, Tree,
    TreeCursor,
};

extern "C" {
    fn tree_sitter_python() -> Language;
}

const SUB_CAPTURE: &str = "sub"; // think @sub
const PREFIX: &str = "at_";
const SUB_KIND: &str = "identifier";
const SOURCE_CODE: &str = "x = 1 + 0";
const QUERY: &str = "(binary_operator (integer) @a (integer) @b) @sub";
const SUBST: &str = "7 + at_a + 1 - at_b + 7";

pub struct Engine {
    pub language: Language,
    pub parser: Parser,
    pub prefix: String,
    pub hole_kind: String,
}

impl Engine {
    pub fn new_python() -> Engine {
        let mut parser = Parser::new();
        let language = unsafe { tree_sitter_python() };
        parser
            .set_language(language)
            .expect("Could not set the language to python");

        Engine {
            language,
            parser,
            prefix: "at_".into(),
            hole_kind: "identifier".into(),
        }
    }

    fn build_query(&self, find: &str) -> (Query, u32) {
        let query =
            Query::new(self.language, find).expect("Could not built query");
        let sub_index = query
            .capture_index_for_name(SUB_CAPTURE)
            .expect("You must use an sub pattern to indicate which part of the AST to replace!");
        (query, sub_index)
    }

    fn parse(&mut self, source: &str) -> Tree {
        self.parser
            .parse(source, None)
            .expect("Could not parse source file")
    }

    fn new_sub(&mut self, find: String, replace_source: String) -> Sub {
        let (find, sub_index) = self.build_query(&find);
        let replace = self.parse(&replace_source);

        Sub {
            find,
            replace,
            replace_source,
            sub_index,
        }
    }
}

pub struct Sub {
    find: Query,
    replace: Tree,
    replace_source: String,
    sub_index: u32,
}

impl Sub {
    fn node_contents<'a>(node: &Node, source: &'a str) -> (Range, &'a str) {
        let range = node.range();
        let iden_name = &source[range.start_byte..range.end_byte];
        (range, iden_name)
    }

    fn expand_match(
        &self,
        engine: &Engine,
        given_match: QueryMatch,
        source: &str,
        new_source: &mut String,
    ) {
        // Traverse the tree. would use recursion but lifetimes haha
        let mut cursor = self.replace.walk();
        let mut ascending = false;
        let mut last_modified = 0;

        loop {
            if ascending {
                if cursor.goto_next_sibling() {
                    ascending = false;
                } else if !cursor.goto_parent() {
                    break;
                }
            } else {
                // vvv what we do is here

                let current_node = cursor.node();
                let kind = current_node.kind();
                if kind == engine.hole_kind {
                    let (range, iden_name) = Self::node_contents(
                        &current_node,
                        &self.replace_source,
                    );

                    // we substitute the matching ast from the query
                    if iden_name.starts_with(&engine.prefix) {
                        new_source.push_str(
                            &self.replace_source
                                [last_modified..range.start_byte],
                        );

                        let capture_name = &iden_name[engine.prefix.len()..];
                        let capture_index = self
                            .find
                            .capture_index_for_name(capture_name)
                            .expect("unknown capture group");

                        let capture_sub = given_match
                            .nodes_for_capture_index(capture_index)
                            .next()
                            .unwrap();

                        let (_, source_fragment) =
                            Self::node_contents(&capture_sub, source);

                        new_source.push_str(&source_fragment);
                        last_modified = range.end_byte;
                    }
                }

                // ^^^ too far!
                if !cursor.goto_first_child() {
                    ascending = true;
                }
            }
        }

        new_source.push_str(&self.replace_source[last_modified..]);
    }

    fn expand_first_match(&self, engine: &mut Engine, source: &str) -> String {
        // parse the source file we are given
        let source_tree = engine.parse(source);

        // apply the query to the parsed source file, and grab the first result
        let mut query_cursor = QueryCursor::new();
        let mut query_matches = query_cursor.matches(
            &self.find,
            source_tree.root_node(),
            source.as_bytes(),
        );
        let first_match = query_matches.next().unwrap();

        // find the branch of the AST we are substituting
        let branch = first_match
            .nodes_for_capture_index(self.sub_index)
            .next()
            .unwrap();

        // incrementally build up a new source file
        // push all the code up to the first match
        // walk the replace tree, and substitute items from the query
        // push the rest of the source code
        let mut new_source = String::new();
        let branch_range = branch.range();
        new_source.push_str(&source[0..branch_range.start_byte]);
        self.expand_match(engine, first_match, source, &mut new_source);
        new_source.push_str(&source[branch_range.end_byte..]);

        // all done!
        return new_source;
    }
}

pub fn print_thought(message: &str, item: &impl std::fmt::Display) {
    println!("{}:\n{}\n", message, item);
}

fn main() {
    let mut engine = Engine::new_python();
    let sub = engine.new_sub(QUERY.into(), SUBST.into());
    let new_source = sub.expand_first_match(&mut engine, SOURCE_CODE);
    println!("{}", new_source);

    // print_thought("given the following source code", source_code)    ;
    // print_thought("I searched for the following query", source_query)    ;
    // print_thought("This returned the following branch of the AST", location_contents) ;
    // print_thought("Using the following replacement template", self.replace_source)    ;
    // print_thought("I spliced in the captured patterns from the AST",     source_code)  ;
    // print_thought("Applying this replacement, the new file is", source_code)    ;
    // print_thought("Applying this replacement, the new source code is", new_source)   ;
}
