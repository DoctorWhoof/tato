# TatoArena - Fixed size Arena Allocator

A no-allocation, fixed-size arena allocator optimized for severely constrained environments.
Each module has a single responsibility:
- **`Arena`**: Core bump allocator with alignment handling
- **`TempID<T>`**: Type-safe handles with compile-time guarantees
- **`Slice<T>`**: Fixed-size collections allocated at runtime

## Usage

### Basic Example

```rust
use tato_arena::Arena;

let mut arena: Arena<1024> = Arena::new();

// Allocate different types
let int_id = arena.alloc(42u32).unwrap();
let float_id = arena.alloc(3.14f32).unwrap();
let array_id = arena.alloc([1, 2, 3, 4]).unwrap();

// Access with compile-time type safety
assert_eq!(*arena.get(&int_id), 42);
assert_eq!(*arena.get(&float_id), 3.14);

// Modify values
*arena.get_mut(&int_id) = 100;
assert_eq!(*arena.get(&int_id), 100);
```

### Runtime-Sized Collections with Slice

You can allocate collections with runtime-determined sizes:

```rust
use tato_arena::Arena;

let mut arena: Arena<1024> = Arena::new();

// Allocate a collection of 10 sprites (size determined at runtime)
let sprites = arena.alloc_slice::<Sprite>(10).unwrap();

// Access as slices
let sprite_slice = arena.get_slice_mut(&sprites);
sprite_slice[0] = Sprite { x: 32, y: 64, .. };
sprite_slice[1] = Sprite { x: 48, y: 80, .. };

// Read back
let sprite_slice = arena.get_slice(&sprites);
assert_eq!(sprite_slice.len(), 10);

// Or initialize with a closure for more control
let numbers = arena.alloc_slice_from_fn(5, |i| i as u32 * 10).unwrap();
let slice = arena.get_slice(&numbers);
assert_eq!(slice, &[0, 10, 20, 30, 40]);
```
