#![deny(clippy::all)]
#![deny(unreachable_pub)]
#![deny(unused_allocation)]
#![deny(unused_extern_crates)]
#![deny(unused_assignments)]
#![deny(unused_comparisons)]
#![allow(clippy::module_name_repetitions)]

pub mod prelude;
mod runtime;

pub use compiler::{
    function, state, value, vm::Vm, Context, Expression, Function, Program, Target, Value,
    VrlRuntime,
};
pub use diagnostic;
pub use runtime::{Runtime, RuntimeResult, Terminate};

/// Compile a given source into the final [`Program`].
pub fn compile(source: &str, fns: &[Box<dyn Function>]) -> compiler::Result {
    let mut state = state::Compiler::new();

    compile_with_state(source, fns, &mut state)
}

pub fn compile_with_state(
    source: &str,
    fns: &[Box<dyn Function>],
    state: &mut state::Compiler,
) -> compiler::Result {
    let ast = parser::parse(source).map_err(|err| vec![Box::new(err) as _])?;

    compiler::compile_with_state(ast, fns, state)
}
