use std::path::PathBuf;

fn main() {
    let dir: PathBuf = ["tree-sitter-python", "src"].iter().collect();

    cc::Build::new()
        .cpp(true)
        .include(&dir)
        .file(dir.join("scanner.cc"))
        .compile("tree_sitter_python_scanner");

    cc::Build::new()
        .cpp(false)
        .include(&dir)
        .file(dir.join("parser.c"))
        .compile("tree_sitter_python_parser");
}
