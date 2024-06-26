//! The simplest linter

use std::{env, path::Path};

use oxc_allocator::Allocator;
use oxc_ast::AstKind;
use oxc_diagnostics::{LabeledSpan, OxcDiagnostic};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::{SourceType, Span};

// Instruction:
// create a `test.js`,
// run `cargo run -p oxc_linter --example linter`
// or `cargo watch -x "run -p oxc_linter --example linter"`

fn main() -> std::io::Result<()> {
    let name = env::args().nth(1).unwrap_or_else(|| "test.js".to_string());
    let path = Path::new(&name);
    let source_text = std::fs::read_to_string(path)?;
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(path).unwrap();
    let ret = Parser::new(&allocator, &source_text, source_type).parse();

    // Handle parser errors
    if !ret.errors.is_empty() {
        print_errors(&source_text, ret.errors);
        return Ok(());
    }

    let program = allocator.alloc(ret.program);
    let semantic_ret =
        SemanticBuilder::new(&source_text, source_type).with_trivias(ret.trivias).build(program);

    let mut errors: Vec<OxcDiagnostic> = vec![];

    for node in semantic_ret.semantic.nodes().iter() {
        match node.kind() {
            AstKind::DebuggerStatement(stmt) => {
                errors.push(no_debugger(stmt.span));
            }
            AstKind::ArrayPattern(array) if array.elements.is_empty() => {
                errors.push(no_empty_pattern("array", array.span));
            }
            AstKind::ObjectPattern(object) if object.properties.is_empty() => {
                errors.push(no_empty_pattern("object", object.span));
            }
            _ => {}
        }
    }

    if errors.is_empty() {
        println!("Success!");
    } else {
        print_errors(&source_text, errors);
    }

    Ok(())
}

fn print_errors(source_text: &str, errors: Vec<OxcDiagnostic>) {
    for error in errors {
        let error = error.with_source_code(source_text.to_string());
        println!("{error:?}");
    }
}

// This prints:
//
//   ⚠ `debugger` statement is not allowed
//   ╭────
// 1 │ debugger;
//   · ─────────
//   ╰────
fn no_debugger(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::new("`debugger` statement is not allowed").with_labels([span0.into()])
}

// This prints:
//
//   ⚠ empty destructuring pattern is not allowed
//   ╭────
// 1 │ let {} = {};
//   ·     ─┬
//   ·      ╰── Empty object binding pattern
//   ╰────
fn no_empty_pattern(s0: &str, span1: Span) -> OxcDiagnostic {
    OxcDiagnostic::new("empty destructuring pattern is not allowed").with_labels([
        LabeledSpan::new_with_span(Some(format!("Empty {s0} binding pattern")), span1),
    ])
}
