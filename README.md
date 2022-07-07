# sixel-image

This library provides an interface for querying, manipulating and serializing sixel data.

There are several methods provided here to do this:

1. If you already have all the serialized sixel bytes, construct `SixelImage` directly
2. If you'd like to parse bytes in real time "on the wire", use `SixelDeserializer` (accompanied by the [`sixel-tokenizer`](https://github.com/zellij-org/sixel-tokenizer) sister crate).

# Example

## With all the serialized bytes ahead of time (option 1)
```rust
use sixel_image::SixelImage;

fn main() {
    let sample = "
        \u{1b}Pq
        \"2;1;100;200
        #0;2;0;0;0#1;2;100;100;0#2;2;0;100;0
        #1~~@@vv@@~~@@~~$
        #2??}}GG}}??}}??-
        #1!14@
        \u{1b}\\
    ";
    let bytes = sample.as_bytes();

    let sixel_image = SixelImage::new(&bytes).unwrap();
    let serialized = sixel_image.serialize();
    println!("{:?}", serialized);
}
```

## Parsing bytes "on the wire" (option 2)
```rust
use sixel_tokenizer::Parser;
use sixel_image::SixelDeserializer;

fn main() {
    let sample = "
        \u{1b}Pq
        \"2;1;100;200
        #0;2;0;0;0#1;2;100;100;0#2;2;0;100;0
        #1~~@@vv@@~~@@~~$
        #2??}}GG}}??}}??-
        #1!14@
        \u{1b}\\
    ";
    let bytes = sample.as_bytes();
    let mut parser = Parser::new();
    let mut sixel_deserializer = SixelDeserializer::new();
    for byte in bytes {
        parser.advance(&byte, |sixel_event| {
            let _ = sixel_deserializer.handle_event(sixel_event);
        });
    }
    let sixel_image = sixel_deserializer.create_image().unwrap();
    let serialized = sixel_image.serialize();
    println!("{:?}", serialized);
}
```

# License
MIT
