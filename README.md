# collect_into_rc_slice
A crate that let's you collect an `Iterator<Item=T>` into an `Rc<[T]>` or `Arc<[T]>`(coming soon) without needing to make 2 heap allocations.

## Important Note
Please **DO NOT** use this if you already have a `Vec<T>`, `sString` or `&[T]` that contains the exact block memory you are trying convert to `Rc<[T]>`.

It wouldn't do anything better than the `std` implementation. It always better to use `.into()` in this case.

For example
```rust
let v = vec![1,2,3];
let rc: Rc<[i32]> = v.into(); // Just use .into()
```

## The Problem
You just learned about how cool using `Rc<[T]>` can be and you have an `Iterator<Item=char>` and you want to collect it to `Rc<str>`

One could naively do it as:
```rust
let iter = /*Some iterator*/;
let rc: Rc<str>  = iter.collect::<String>>().into();
```

Which makes 2 seperate heap allocations, one for `String` and another one for `Rc`

## Solution
It is very possible to do this with only 1 heap allocation, however it requires the usage of unsafe code, and good knowledge of the internal data structure of `Rc` called `RcBox`

With this crate you can avoid another heap allocation:
```rust
use collect_into_rc_slice::*;

let iter = /*Some iterator*/;
let rc: Rc<str>  = iter.collect_into_rc_str();
```

## Safety
This crate utilizes unsafe code to create a safe abstraction. To ensure that it is safe, it is tested, and uses miri to identify possible undefined behavior
