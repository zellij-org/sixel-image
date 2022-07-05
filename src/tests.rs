use crate::SixelImage;

fn remove_whitespace(s: &str) -> String {
    let mut s = s.to_string();
    s.retain(|c| !c.is_whitespace());
    s
}

#[test]
fn basic_serialization() {
    let sample = "
        \u{1b}Pq
        #0;2;0;0;0#1;2;100;100;0#2;2;0;100;0
        #1~~@@vv@@~~@@~~$
        #2??}}GG}}??}}??-
        #1!14@
        \u{1b}\\
    ";
    let sixel_image = SixelImage::new(sample.as_bytes());
    let serialized_image = sixel_image.unwrap().serialize();
    assert_eq!(serialized_image, remove_whitespace(&sample));
}

#[test]
fn pad_image_with_raster_attribute() {
    let sample = "
        \u{1b}Pq
        \"1;1;10;10
        \u{1b}\\
    ";
    // we don't serialize raster attributes because their behaviour is not very consistent across
    // terminal emulators, instead we explicitly emit the empty pixels
    let expected = "\u{1b}Pq#0!10~-#0!10N\u{1b}\\";
    let sixel_image = SixelImage::new(sample.as_bytes());
    let image = sixel_image.unwrap();
    let serialized_image = image.serialize();
    assert_eq!(image.pixel_size(), (10, 10));
    assert_eq!(serialized_image, expected);
}

#[test]
fn dont_pad_image_with_transparent_background() {
    let sample = "
        \u{1b}P0;1q
        \"1;1;10;10
        \u{1b}\\
    ";
    let expected = "\u{1b}Pq\u{1b}\\";
    let sixel_image = SixelImage::new(sample.as_bytes());
    let image = sixel_image.unwrap();
    let serialized_image = image.serialize();
    assert_eq!(image.pixel_size(), (1, 0));
    assert_eq!(serialized_image, expected);
}

#[test]
fn full_256_colors() {
    let mut sample = String::from("\u{1b}Pq");
    for i in 0..256 {
        sample.push_str(&format!("#{};1;50;50;50", i));
    }
    sample.push_str(&"\u{1b}\\");
    let sixel_image = SixelImage::new(sample.as_bytes());
    let serialized_image = sixel_image.unwrap().serialize();
    assert_eq!(serialized_image, remove_whitespace(&sample));
}

#[test]
fn color_definition_at_the_end() {
    let sample = "
        \u{1b}Pq
        #1~~@@vv@@~~@@~~$
        #2??}}GG}}??}}??-
        #1!14@
        #0;2;0;0;0#1;2;100;100;0#2;2;0;100;0
        \u{1b}\\
    ";
    let expected = "
        \u{1b}Pq
        #0;2;0;0;0#1;2;100;100;0#2;2;0;100;0
        #1~~@@vv@@~~@@~~$
        #2??}}GG}}??}}??-
        #1!14@
        \u{1b}\\
    ";
    let sixel_image = SixelImage::new(sample.as_bytes());
    let serialized_image = sixel_image.unwrap().serialize();
    assert_eq!(serialized_image, remove_whitespace(&expected));
}

#[test]
fn multiple_occurrences_into_repeat_characters() {
    let sample = "
        \u{1b}Pq
        #0;2;0;0;0#1;2;100;100;0#2;2;0;100;0
        #1~~~~????@@nnfffnn$
        #2????GG}GG????
        \u{1b}\\
    ";
    let expected = "
        \u{1b}Pq
        #0;2;0;0;0#1;2;100;100;0#2;2;0;100;0
        #1!4~!4?@@nn!3fnn$
        #2!4?GG}GG!8?
        \u{1b}\\
    ";
    let sixel_image = SixelImage::new(sample.as_bytes());
    let serialized_image = sixel_image.unwrap().serialize();
    assert_eq!(serialized_image, remove_whitespace(&expected));
}

#[test]
fn dcs_event() {
    let sample = "
        \u{1b}Pq
        \u{1b}\\
    ";
    let sixel_image = SixelImage::new(sample.as_bytes());
    let serialized_image = sixel_image.unwrap().serialize();
    assert_eq!(serialized_image, remove_whitespace(&sample));
}

#[test]
fn color_introducer_event() {
    let sample = "
        \u{1b}Pq
        #1;2;100;50;100
        \u{1b}\\
    ";
    let sixel_image = SixelImage::new(sample.as_bytes());
    let serialized_image = sixel_image.unwrap().serialize();
    assert_eq!(serialized_image, remove_whitespace(&sample));
}

#[test]
fn corrupted_sixel_string() {
    let sample = "
        \u{1b}Pq
        lsdkjfsldkfjslekdj23l4kj1l2k3`j13lk12j3l1k2j34123
        \u{1b}\\
    ";
    let sixel_image_result = SixelImage::new(sample.as_bytes());
    assert!(sixel_image_result.is_err());
}

#[test]
fn get_image_size_in_pixels() {
    let sample = "
        \u{1b}Pq
        #0;2;0;0;0#1;2;100;100;0#2;2;0;100;0
        #1~~@@vv@@~~@@~~$
        #2??}}GG}}??}}??-
        #1!14@
        \u{1b}\\
    ";
    let sixel_image = SixelImage::new(sample.as_bytes());
    let serialized_image = sixel_image.unwrap();
    assert_eq!(serialized_image.pixel_size(), (12, 14));
}

#[test]
fn crop_image_width_and_height() {
    let sample = "
        \u{1b}Pq
        #0;2;0;0;0#1;2;100;100;0#2;2;0;100;0
        #1~~@@vv@@~~@@~~$
        #2??}}GG}}??}}??-
        #1!14@
        \u{1b}\\
    ";
    let expected = "
        \u{1b}Pq
        #0;2;0;0;0#1;2;100;100;0#2;2;0;100;0
        #1@
        \u{1b}\\
    "; // only the upper left pixel
    let sixel_image = SixelImage::new(sample.as_bytes());
    let serialized_image = sixel_image.unwrap().serialize_range(0, 0, 1, 1); // x, y, width, height
    assert_eq!(serialized_image, remove_whitespace(&expected));
}

#[test]
fn crop_image_width_and_height_mid_image() {
    let sample = "
        \u{1b}Pq
        #0;2;0;0;0#1;2;100;100;0#2;2;0;100;0
        #1~~@@vv@@~~@@~~$
        #2??}}GG}}??}}??-
        #1!14@
        \u{1b}\\
    ";
    let expected = "
        \u{1b}Pq
        #0;2;0;0;0#1;2;100;100;0#2;2;0;100;0
        #1BAABB$#2?@@??
        \u{1b}\\
    "; // 5x5 pixels starting from x==5 y==5
    let sixel_image = SixelImage::new(sample.as_bytes());
    let serialized_image = sixel_image.unwrap().serialize_range(5, 5, 5, 5); // x, y, width, height
    assert_eq!(serialized_image, remove_whitespace(&expected));
}

#[test]
fn cut_out_from_image() {
    let sample = "
        \u{1b}Pq
        #0;2;0;0;0#1;2;100;100;0#2;2;0;100;0
        #1~~@@vv@@~~@@~~$
        #2??}}GG}}??}}??-
        #1!14@
        \u{1b}\\
    ";
    let expected = "
        \u{1b}Pq
        #0;2;0;0;0#1;2;100;100;0#2;2;0;100;0
        #1~!7@~~@@~~$
        #2!6?}}??}}??-
        #1!14@
        \u{1b}\\
    ";
    let mut sixel_image = SixelImage::new(sample.as_bytes()).unwrap();
    sixel_image.cut_out(1, 1, 5, 5); // cut out a rect starting from x/y 1/1 with a width and height of 5 and 5 respectively
    let serialized_image = sixel_image.serialize();
    assert_eq!(serialized_image, remove_whitespace(&expected));
}

#[test]
fn image_max_height() {
    let sample = "
        \u{1b}Pq
        #0;2;0;0;0#1;2;100;100;0#2;2;0;100;0
        #1~~@@vv@@~~@@~~$
        #2??}}GG}}??}}??-
        #1!14@
        \u{1b}\\
    ";
    let expected = "
        \u{1b}Pq
        #0;2;0;0;0
        #1;2;100;100;0
        #2;2;0;100;0
        #1~~@@vv@@~~@@~~$
        #2??}}GG}}??}}??
        \u{1b}\\
    ";
    let sixel_image = SixelImage::new_with_max_height(sample.as_bytes(), 7).unwrap();
    let serialized_image = sixel_image.serialize();
    assert_eq!(serialized_image, remove_whitespace(&expected));
}

#[test]
fn corrupted_image() {
    // notice this sample does not start with a DCS event
    let sample = "
        #0;2;0;0;0#1;2;100;100;0#2;2;0;100;0
        #1~~@@vv@@~~@@~~$
        #2??}}GG}}??}}??-
        #1!14@
        \u{1b}\\
    ";
    assert!(SixelImage::new(sample.as_bytes()).is_err());
}
