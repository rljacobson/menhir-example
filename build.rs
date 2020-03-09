extern crate menhir;
use menhir::MenhirOption;
use std::path::Path;

fn main() {
    let msg_path = Path::new("src/parseerrors.messages");
    let grammar_path = Path::new("src/parser.rsy");

    menhir::compile_errors(
        msg_path,
        grammar_path,
        &[MenhirOption::CompileErrors(msg_path)]
    );
    menhir::process_file(
        grammar_path,
        &[]
    );

    println!("cargo:rerun-if-changed=src/parser.rsy");
    println!("cargo:rerun-if-changed=src/parseerrors.messages");
    // ToDo: What does this line do?
    menhir::cargo_rustc_flags().unwrap();
}
