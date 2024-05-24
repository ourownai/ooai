/*
This module demonstrates the integration of the libloading crate for dynamically loading a shared library at runtime and invoking a specified function within it. The target library is "libmy_module.so", anticipated to reside within the "target/release" directory of a Rust project.

The process begins with an attempt to load the library using Library::new, which yields a Result<Library, libloading::Error>. This step is crucial as it handles the potential failure of finding or loading the specified library file. Upon successful loading, the library object is utilized to access symbols within the dynamic library.

A specific function, denoted as "call" within the dynamic library, is accessed by invoking Library::get, which requires specifying the function's signature. For this module, the function's signature is defined as unsafe extern "C" fn(*const c_char, *const c_char, *const c_char, *const c_char) -> *const c_char, representing a function that accepts four C-style string pointers as arguments and returns a C-style string pointer.

Arguments for the "call" function are prepared by converting Rust string literals into C-compatible strings (CString), ensuring that null-terminated strings are passed to the C function. The function is invoked with these arguments, and its return value, a pointer to a C-style string, is safely converted back into a Rust string. This conversion process meticulously handles the potential UTF-8 encoding errors.

The return value, a JSON-formatted string, is then parsed into a Rust data structure (specifically, serde_json::Value) to facilitate further manipulation or inspection of the data.

This module emphasizes robust error handling, showcasing graceful recovery from various potential failures, including library loading, symbol resolution, string conversion, and JSON deserialization. It illustrates a comprehensive approach to interfacing with dynamic libraries in Rust, ensuring safety and correctness throughout the operation. The dynamic library is unloaded when the Library object is dropped, following Rust's resource management conventions.
*/

use libloading::{Library, Symbol};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use serde_json::Value;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Attempt to load the dynamic library, handling errors gracefully
    let lib = unsafe { Library::new("target/release/libmy_module.so")? };

    // Define a Rust-friendly wrapper around the unsafe external function
    let call = |operator_id: &str, package: &str, data: &str, kwargs: &str| -> Result<Value, Box<dyn std::error::Error>> {
        // Convert Rust strings to C strings, handling potential NulError
        let operator_id_c = CString::new(operator_id)?;
        let package_c = CString::new(package)?;
        let data_c = CString::new(data)?;
        let kwargs_c = CString::new(kwargs)?;

        // Get a function pointer for the DataExchange call method, handling errors
        let call_func: Symbol<unsafe extern "C" fn(*const c_char, *const c_char, *const c_char, *const c_char) -> *const c_char> =
            unsafe { lib.get(b"call")? };

        // Safely call the function
        let result_ptr = unsafe { call_func(operator_id_c.as_ptr(), package_c.as_ptr(), data_c.as_ptr(), kwargs_c.as_ptr()) };

        // Convert the result back into a Rust string, then parse as JSON
        let result_cstr = unsafe { CStr::from_ptr(result_ptr) };
        let result_str = result_cstr.to_str()?;
        let result_json = serde_json::from_str(result_str)?;

        Ok(result_json)
    };

    // Example usage of the `call` wrapper function
    match call("my_operator_id", "my_package", "my_data", "{\"query\": \"my_query\"}") {
        Ok(result) => println!("Result: {:?}", result),
        Err(e) => eprintln!("Error calling function: {}", e),
    }

    // Library is automatically unloaded when `lib` goes out of scope
    Ok(())
}
