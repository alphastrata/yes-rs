# Noble

# PLEASE PLEASE PLEASE DON'T FUCKING USE THIS.

A Rust procedural macro crate that provides the `#[unsafe_wrap]` attribute macro for wrapping various code constructs in `unsafe` blocks.

## Features

- **Functions**: Wraps function bodies in `unsafe` blocks
- **Structs**: Generates unsafe constructors for structs
- **Impl blocks**: Wraps all method bodies in `unsafe` blocks  
- **Enums**: Provides unsafe variant constructors
- **Traits**: Converts traits to unsafe traits with unsafe methods

## Installation

Don't.

## Usage

### Functions

The macro wraps the entire function body in an `unsafe` block:

```rust
use unsafe_wrapper::unsafe_wrap;

#[unsafe_wrap]
fn dangerous_operation() -> i32 {
    // This entire block will be wrapped in unsafe
    std::ptr::read(0x1000 as *const i32)
}
```

Expands to:

```rust
fn dangerous_operation() -> i32 {
    unsafe {
        std::ptr::read(0x1000 as *const i32)
    }
}
```

### Structs

For structs, the macro generates unsafe constructor methods:

```rust
#[unsafe_wrap]
struct MyStruct {
    value: i32,
    name: String,
}

// Usage
let instance = unsafe { MyStruct::new_unsafe(42, "test".to_string()) };
```

Works with all struct types:
- Named fields: `new_unsafe(field1: Type1, field2: Type2)`
- Tuple structs: `new_unsafe(field_0: Type0, field_1: Type1)`  
- Unit structs: `new_unsafe()`

### Impl Blocks

All method bodies in impl blocks are wrapped in `unsafe`:

```rust
#[unsafe_wrap]  
impl MyStruct {
    fn get_value(&self) -> i32 {
        // This will be wrapped in unsafe
        self.value
    }
    
    fn dangerous_method(&mut self) {
        // This will also be wrapped in unsafe
        std::ptr::write(&mut self.value as *mut i32, 100);
    }
}
```

### Enums

For enums, the macro generates unsafe constructor methods for each variant:

```rust
#[unsafe_wrap]
enum MyEnum {
    Unit,
    Tuple(i32, String),
    Struct { field: i32 },
}

// Usage
let unit = unsafe { MyEnum::new_unit_unsafe() };
let tuple = unsafe { MyEnum::new_tuple_unsafe(42, "test".to_string()) };
let struct_variant = unsafe { MyEnum::new_struct_unsafe(100) };
```

### Traits

Traits are converted to unsafe traits with unsafe methods:

```rust
#[unsafe_wrap]
trait DangerousTrait {
    fn risky_method(&self) -> i32;
    
    fn default_risky(&self) -> String {
        "default".to_string()
    }
}
```

Expands to:

```rust
unsafe trait DangerousTrait {
    unsafe fn risky_method(&self) -> i32;
    
    unsafe fn default_risky(&self) -> String {
        unsafe {
            "default".to_string()
        }
    }
}
```
