# sixel-image

This library provides an interface for querying, manipulating and serializing sixel data.

There are several methods provided here to do this:

1. If you already have all the serialized sixel bytes, construct `SixelImage` directly
2. If you'd like to parse bytes in real time "on the wire", use `SixelDeserializer`

# Example
```rust
use std::io::Read;
use std::io::BufReader;
use std::fs::File;
use sixel_image::SixelImage;

fn main() {
    let f = File::open("/home/aram/Downloads/lady-of-shalott.six").unwrap();
    let mut reader = BufReader::new(f);
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer).unwrap();
    let sixel_image = SixelImage::new(&buffer).unwrap();
    let serialized = sixel_image.serialize();
    println!("{}", serialized);
}
```
