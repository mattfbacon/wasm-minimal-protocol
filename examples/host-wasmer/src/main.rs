// this is an example of host
// you need to build the hello example first

use anyhow::Result;
use wasmer::{imports, Function, Instance, Module, Store, Value, TypedFunction};

fn main() -> Result<()> {
    let mut store = Store::default();
    let module = Module::new(
        &store,
        include_bytes!("../../../target/wasm32-unknown-unknown/debug/hello.wasm"),
    )?; // this is just compiled with the hello example
    let import_object = imports! {};
    let instance = Instance::new(&mut store, &module, &import_object)?;
    let mut plugin_instance = PluginInstance::new(&instance, &mut store);

    println!("{:?}", plugin_instance.call("hello", &[]));
    println!("{:?}", plugin_instance.call("double_it", &["double me!!"]));
    println!(
        "{:?}",
        plugin_instance.call("concatenate", &["val1", "value2"])
    );
    println!(
        "{:?}",
        plugin_instance.call("shuffle", &["value1", "value2", "value3"])
    );
    // can't make python work yet

    println!("start");
    let mut store = Store::default();
    let module = Module::from_binary(
        &store,
        // include_bytes!("../../../target/wasm32-unknown-unknown/release/python_plugin.wasm"),
        // include_bytes!("../../../target/wasm32-wasi/release/python_plugin.wasm"),
        include_bytes!("../../../target/wasm32-wasi/release/pyo3_plugin.wasm"),
    )?;
    println!("mod");

    // let get_time : TypedFunction<(i32, i32), i32> = Function::new_typed(&mut store, |x : i32,y : i64,z : i32| 0u32).typed(&store).unwrap();
    let import_object = imports! {
        // "env" => {
            // "wasi_snapshot_preview1" => Function::new_typed(&mut store, || println!()),
            // "clock_time_get" => Function::new_typed(&mut store, |x : i32,y : i64,z : i32| 0u32)
        // }
    };
    let instance = Instance::new(&mut store, &module, &import_object)?;
    println!("instance");
    let mut plugin_instance = PluginInstance::new(&instance, &mut store);
    println!("no call");
    println!("{:?}", plugin_instance.call("exec", &["2 + 2"]));
    println!("called");
    Ok(())
}

struct PluginInstance<'a> {
    instance: &'a Instance,
    store: &'a mut Store,
    allocate_storage: &'a wasmer::Function,
    get_storage_pointer: &'a wasmer::Function,
    get_storage_len: &'a wasmer::Function,
    memory: &'a wasmer::Memory,
}

impl<'a> PluginInstance<'a> {
    fn new(instance: &'a Instance, store: &'a mut Store) -> Self {
        // important functions that we will often use.
        let allocate_storage = instance
            .exports
            .get_function("wasm_minimal_protocol::allocate_storage")
            .unwrap();
        let get_storage_pointer = instance
            .exports
            .get_function("wasm_minimal_protocol::get_storage_pointer")
            .unwrap();
        let get_storage_len = instance
            .exports
            .get_function("wasm_minimal_protocol::get_storage_len")
            .unwrap();
        let memory = instance.exports.get_memory("memory").unwrap();
        Self {
            instance,
            store,
            allocate_storage,
            get_storage_pointer,
            get_storage_len,
            memory,
        }
    }

    /// Write arguments in `__RESULT`.
    fn write(&mut self, args: &[&str]) {
        let total_len = args.iter().map(|a| a.len()).sum::<usize>();
        self.allocate_storage
            .call(self.store, &[Value::I32(total_len as _)])
            .unwrap();
        let mut storage_pointer =
            self.get_storage_pointer.call(self.store, &[]).unwrap()[0].unwrap_i32() as u64;
        for arg in args {
            self.memory
                .view(self.store)
                .write(storage_pointer, arg.as_bytes())
                .unwrap();
            storage_pointer += arg.len() as u64;
        }
    }

    fn call(&mut self, function: &str, args: &[&str]) -> String {
        self.write(args);
        let args = args
            .iter()
            .map(|a| Value::I32(a.len() as _))
            .collect::<Vec<_>>();

        let function = self.instance.exports.get_function(function).unwrap();
        function.call(self.store, &args).unwrap();

        // Get the resulting string in `__RESULT`
        let storage_pointer =
            self.get_storage_pointer.call(self.store, &[]).unwrap()[0].unwrap_i32() as u64;
        let len = self.get_storage_len.call(self.store, &[]).unwrap()[0].unwrap_i32();

        let mut result = vec![0u8; len as usize];
        self.memory
            .view(self.store)
            .read(storage_pointer, &mut result)
            .unwrap();
        String::from_utf8(result).unwrap()
    }
}
