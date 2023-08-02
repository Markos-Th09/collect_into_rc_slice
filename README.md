# collect_into_rc_slice
A crate that let's you collect an `Iterator<Item=T>` into an `Rc<T>` without needing to make 2 heap allocations.

## The Problem
You just learnned about how cool using `Rc<[T]>` can be and you have an `Iterator<Item=char>` and you want to collect it to `Rc<str>`

One could naively do it as:
```rust
// Some iterator
let iter = ['a','b','c'].iter();
let rc: Rc<str>  = iter.collect::<String>>().into();
```

Which makes 2 seperate heap allocations, one for `String` and another one for `Rc`

## Solution
It is very possible to do this with only 1 heap allocation, however it requires the usage of unsafe code, and good knowledge of the internal data structure of `Rc` called `RcBox`

With this crate you can avoid another heap allocation:
```rust
use collect_into_rc_slice::*;

// Some iterator
let iter = ['a','b','c'].iter();
let rc: Rc<str>  = iter.collect_into_rc_str();
```

## Safety
This crate utilizes unsafe code but it is properly tested, and uses miri to identify possible undefined behavior
