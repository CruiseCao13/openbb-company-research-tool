# Training CLI Check

## research-rs --help
```text
error[E0382]: borrow of moved value: `issue_lines`
   --> crates/research-batch/src/training.rs:646:16
    |
563 |     let issue_lines = issue_counts
    |         ----------- move occurs because `issue_lines` has type `std::string::String`, which does not implement the `Copy` trait
...
593 |                 issue_lines
    |                 ----------- value moved here
...
646 |             if issue_lines.is_empty() {
    |                ^^^^^^^^^^^ value borrowed here after move
    |
help: consider cloning the value if the performance cost is acceptable
    |
593 |                 issue_lines.clone()
    |                            ++++++++

For more information about this error, try `rustc --explain E0382`.
error: could not compile `research-batch` (lib) due to 1 previous error
```
## research-rs train --help
```text
error[E0382]: borrow of moved value: `issue_lines`
   --> crates/research-batch/src/training.rs:646:16
    |
563 |     let issue_lines = issue_counts
    |         ----------- move occurs because `issue_lines` has type `std::string::String`, which does not implement the `Copy` trait
...
593 |                 issue_lines
    |                 ----------- value moved here
...
646 |             if issue_lines.is_empty() {
    |                ^^^^^^^^^^^ value borrowed here after move
    |
help: consider cloning the value if the performance cost is acceptable
    |
593 |                 issue_lines.clone()
    |                            ++++++++

For more information about this error, try `rustc --explain E0382`.
error: could not compile `research-batch` (lib) due to 1 previous error
```
## research-rs batch --help
```text
error[E0382]: borrow of moved value: `issue_lines`
   --> crates/research-batch/src/training.rs:646:16
    |
563 |     let issue_lines = issue_counts
    |         ----------- move occurs because `issue_lines` has type `std::string::String`, which does not implement the `Copy` trait
...
593 |                 issue_lines
    |                 ----------- value moved here
...
646 |             if issue_lines.is_empty() {
    |                ^^^^^^^^^^^ value borrowed here after move
    |
help: consider cloning the value if the performance cost is acceptable
    |
593 |                 issue_lines.clone()
    |                            ++++++++

For more information about this error, try `rustc --explain E0382`.
error: could not compile `research-batch` (lib) due to 1 previous error
```
