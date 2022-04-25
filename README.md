# sixel-image
This is a (pretty fast!) sixel serializer/deserializer with cropping support.

It accepts a sixel serialized string byte-by-byte, deserializes it into an internal representation, and is thenable to answer questions about it (eg. size in pixels), crop it to x/y/width/height or serialize it back.

This is especially useful for terminal emulators and terminal multiplexers who want to be able to represent the image on screen in various dispositions.

In the future, if the need arises, it shouldn't be too difficult to add a filter method to the serializer so that one could for example create a thumbnail out of an image.

** This is still a pre-release, the API is still being hammered out **

## Example
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

```
$ cargo build --release
$ time target/release/converter > /tmp/new-lady.six
________________________________________________________
Executed in  280.87 millis    fish           external
   usr time  276.78 millis  603.00 micros  276.18 millis
   sys time    3.32 millis    0.00 micros    3.32 millis
```

## Example with crop
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

    let serialized = sixel_image.serialize_range(500, 500, 100, 200);
    println!("{}", serialized);
}
```
