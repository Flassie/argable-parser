# argable-parser

Simple parser using [nom](https://github.com/Geal/nom)

## Usage

```rust
use argable_parser::parse;

fn main() {
    let test = r#"\$ $datetime(format = "%Y") $text(bold, value = 'test \'value\'')"#;
    println!("{:?}", parse(test));
}
```
