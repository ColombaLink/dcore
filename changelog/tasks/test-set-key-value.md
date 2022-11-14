# Test setting key value pairs on a resource 

assign to: @timo

Prob. there is a bug in document.rs:328 where the key value pair is not set correctly... 


The reload_update_test_resource_with_key_value function can be extended for more extensive testing.

## Bug 

The bug that caused the nondeterministic behavior was caused by the yjs merge function. 
Something goes wrong when merging multiple updates.

Let's try to replicate this with only the yjs library in a test case.
Then create a bug report for the yjs library on github.

# Rust Notes & Examples

### Ownership Example

    let x = 5;
    let y = x;

Both variables x and y exist y is a copy of the value in x.
Because Integers a simple values, fixed size, saved on the Stack.

    let s1 = String::from("Hello");
    let s2 = s1;    // invalidates s1 only s2 left

Because the String Type is complex, the content is saved on the Heap, 
only the pointer, the length and capacity are saved on the Stack.

Here only the pointer gets copied resulting in s1 & s2 pointing to the same
address/content.
**But** since this could lead to a "double free error", where both variables
would try to "drop" the content and free the memory when the variable is out of scope.

**In Rust after "let s2=s1;" s1 is no longer valid!**

**To Copy use Clone**
    
    let s1 = String::from("Hello");
    let s2 = s1.clone();

### Ownership and Functions

    fn main() {
        let s = String::from("hello");  // s comes into scope

        takes_ownership(s);             // s's value moves into the function...
                                    // ... and so is no longer valid here

        let x = 5;                      // x comes into scope

        makes_copy(x);                  // x would move into the function,
                                    // but i32 is Copy, so it's okay to still
                                    // use x afterward

    } // Here, x goes out of scope, then s. But because s's value was moved, nothing
    // special happens.

    fn takes_ownership(some_string: String) { // some_string comes into scope
        println!("{}", some_string);
    } // Here, some_string goes out of scope and `drop` is called. The backing
    // memory is freed.

    fn makes_copy(some_integer: i32) { // some_integer comes into scope
        println!("{}", some_integer);
    } // Here, some_integer goes out of scope. Nothing special happens.

### Use References for to give functions variables without droppping them

    fn main() {
        let s1 = String::from("hello");

        let len = calculate_length(&s1);

        println!("The length of '{}' is {}.", s1, len);
    }

    fn calculate_length(s: &String) -> usize {
        s.len()
    }

**But we cant change values we borrowed with references,
only if we make them mutable**

    fn main() {
        let s = String::from("hello");

        change(&s); // ! ERROR!
    }

    fn change(some_string: &String) {
        some_string.push_str(", world");
    }
**Solution: make the reference mutable**

    fn main() {
        let mut s = String::from("hello");

        change(&mut s);
    }

    fn change(some_string: &mut String) {
    some_string.push_str(", world");
    }
**If we have ha mutable reference we can NOT have another reference on the same value in the same scope
**
