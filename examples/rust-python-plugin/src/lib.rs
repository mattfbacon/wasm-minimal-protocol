use std::{rc::Rc, cell::RefCell};

use rustpython::vm::{scope::Scope, builtins::PyDict, PyRef};

wasm_minimal_protocol::initiate_protocol!();

static mut buffer : String = String::new();

#[wasm_minimal_protocol::wasm_func]
pub fn exec(code: &str) -> String {
    let s = code.to_owned();
    // let s = Rc::new(RefCell::new(vec![]));
    let res = rustpython::run(move |vm| {
        let scope = vm.new_scope_with_builtins();
        let res = vm.run_code_string(scope, &s, "".into());
        let res = res.unwrap().str(&vm).unwrap().to_string();
        unsafe{ buffer = res }
    });
    // s.borrow_mut().pop().unwrap()
    unsafe{ buffer.clone()}
}
