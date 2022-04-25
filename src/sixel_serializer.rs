use std::collections::{HashMap, BTreeMap};

use crate::{SixelColor, Pixel};

pub struct SixelSerializer <'a>{
    color_registers: &'a BTreeMap<u8, SixelColor>,
    pixels: &'a Vec<Vec<Pixel>>,
}

impl <'a>SixelSerializer <'a>{
    pub fn new(color_registers: &'a BTreeMap<u8, SixelColor>, pixels: &'a Vec<Vec<Pixel>>) -> Self {
        SixelSerializer {
            color_registers,
            pixels
        }
    }
    pub fn serialize(&self) -> String {
        let serialized_image = String::new();
        let serialized_image = self.serialize_empty_dcs(serialized_image);
        let serialized_image = self.serialize_color_registers(serialized_image);
        let serialized_image = self.serialize_pixels(serialized_image, None, None, None, None);
        let serialized_image = self.serialize_end_event(serialized_image);
        serialized_image
    }
    pub fn serialize_range(&self, start_x_index: usize, start_y_index: usize, width: usize, height: usize) -> String {
        let serialized_image = String::new();
        let serialized_image = self.serialize_empty_dcs(serialized_image);
        let serialized_image = self.serialize_color_registers(serialized_image);
        let serialized_image = self.serialize_pixels(serialized_image, Some(start_x_index), Some(start_y_index), Some(width), Some(height));
        let serialized_image = self.serialize_end_event(serialized_image);
        serialized_image
    }
    fn serialize_empty_dcs(&self, mut append_to: String) -> String {
        append_to.push_str("\u{1b}Pq");
        append_to
    }
    fn serialize_color_registers(&self, mut append_to: String) -> String {
        for (color_register, sixel_color_code) in &*self.color_registers {
            match sixel_color_code {
                SixelColor::Hsl(x, y, z) => append_to.push_str(&format!("#{};1;{};{};{}", color_register, x, y, z)),
                SixelColor::Rgb(x, y, z) => append_to.push_str(&format!("#{};2;{};{};{}", color_register, x, y, z)),
            }
        }
        append_to
    }
    fn serialize_pixels(&self, mut append_to: String, start_x_index: Option<usize>, start_y_index: Option<usize>, width: Option<usize>, height: Option<usize>) -> String {
        let start_y_index = start_y_index.unwrap_or(0);
        let start_x_index = start_x_index.unwrap_or(0);
        let max_x_index = width.map(|width| (start_x_index + width).saturating_sub(1));
        let max_y_index = height.map(|height| (start_y_index + height).saturating_sub(1));
        let mut current_line_index = start_y_index;
        let mut current_column_index = start_x_index;
        let mut color_index_to_sixel_data_string: BTreeMap<u8, String> = BTreeMap::new();
        let max_lines = std::cmp::min(
            height.unwrap_or(self.pixels.len()),
            self.pixels.len(),
        );
        loop {
            let relative_column_index = current_column_index - start_x_index;
            let relative_line_index = current_line_index - start_y_index;
            let continue_serializing = SixelColumn::new(
                current_line_index,
                current_column_index,
                max_x_index,
                max_y_index,
                &self.pixels
            ).map(|mut sixel_column| {
                sixel_column.serialize(&mut color_index_to_sixel_data_string, relative_column_index);
                current_column_index += 1;
            })
            .or_else(|| {
                // end of row
                SixelLine::new(
                    &mut append_to,
                    relative_line_index,
                    relative_column_index,
                    max_lines
                )
                .as_mut()
                .map(|sixel_line| {
                    sixel_line.serialize(&mut color_index_to_sixel_data_string);
                    current_line_index += 6;
                    current_column_index = start_x_index;
                })
            })
            .is_some();
            if !continue_serializing {
                break;
            }
        }
        append_to
    }
    fn serialize_end_event(&self, mut append_to: String) -> String {
        append_to.push_str("\u{1b}\\");
        append_to
    }
}

struct SixelColumn {
    color_index_to_byte: HashMap<u8, u8>,
}

impl SixelColumn {
    pub fn new(
        absolute_line_index: usize,
        absolute_column_index: usize,
        max_x_index: Option<usize>,
        max_y_index: Option<usize>,
        pixels: &Vec<Vec<Pixel>>
    ) -> Option<Self> {
        let mut empty_rows = 0;
        let mut color_index_to_byte = HashMap::new();
        if let Some(max_x_index) = max_x_index {
            if max_x_index < absolute_column_index {
                return None;
            }
        }
        if let Some(max_y_index) = max_y_index {
            if max_y_index < absolute_line_index {
                return None;
            }
        }
        let pixels_in_column = max_y_index
            .map(|max_y_index| std::cmp::min(
                max_y_index.saturating_sub(absolute_line_index) + 1,
                6
            ))
            .unwrap_or(6);
        for i in 0..pixels_in_column {
            let pixel_at_current_position = pixels
                .get(absolute_line_index + i)
                .map(|current_line| current_line.get(absolute_column_index));
            match pixel_at_current_position {
                Some(Some(pixel)) => {
                    if pixel.on {
                        let color_char = color_index_to_byte.entry(pixel.color).or_insert(0);
                        let mask = 1 << i;
                        *color_char += mask;
                    }
                }
                _ => empty_rows += 1,
            }
        }
        let row_ended = empty_rows == 6;
        if row_ended {
            None
        } else {
            Some(SixelColumn { color_index_to_byte })
        }
    }
    fn serialize(
        &mut self,
        color_index_to_character_string: &mut BTreeMap<u8, String>,
        current_index: usize,
    ) {
        for (color_index, char_representation) in self.color_index_to_byte.iter_mut() {
            let color_chars = color_index_to_character_string
                .entry(*color_index)
                .or_insert(String::new());
            for _ in color_chars.len()..current_index {
                color_chars.push('?');
            }
            color_chars.push(char::from(*char_representation + 0x3f));
        }
    }
}

struct SixelLine <'a>{
    append_to: &'a mut String,
    relative_line_index: usize, // line index inside cropped selection, or as part of total if not cropping
    line_length: usize,
}

impl <'a>SixelLine <'a>{
    pub fn new(append_to: &'a mut String, relative_line_index: usize, relative_column_index: usize, max_lines: usize) -> Option<Self> {
        if relative_line_index >= max_lines {
            None
        } else {
            Some(SixelLine {
                append_to,
                relative_line_index,
                line_length: relative_column_index,
            })
        }
    }
    pub fn serialize(&mut self, color_index_to_character_string: &'a mut BTreeMap<u8, String>) {
        let mut is_first = true;
        if self.relative_line_index != 0 {
            self.append_to.push('-');
        }
        for (color_index, sixel_chars) in color_index_to_character_string.iter_mut() {
            if !is_first {
                self.append_to.push('$');
            }
            is_first = false;
            self.pad_sixel_string(sixel_chars, self.line_length);
            self.serialize_color_introducer(color_index);
            self.group_identical_characters(sixel_chars);
        }
        color_index_to_character_string.clear();
    }
    fn serialize_one_or_more_sixel_characters (&mut self, character_occurrences: usize, character: char) {
        if character_occurrences > 2 {
            self.append_to.push_str(&format!("!{}{}", character_occurrences, character));
        } else {
            for _ in 0..character_occurrences {
                self.append_to.push(character);
            }
        }
    }
    fn group_identical_characters(&mut self, sixel_chars: &mut String) {
        let mut current_character = None;
        let mut current_character_occurrences = 0;
        for character in sixel_chars.drain(..) {
            if current_character.is_none() {
                current_character = Some(character);
                current_character_occurrences = 1;
            } else if current_character == Some(character) {
                current_character_occurrences += 1;
            } else {
                self.serialize_one_or_more_sixel_characters(
                    current_character_occurrences,
                    current_character.unwrap(),
                );
                current_character_occurrences = 1;
                current_character = Some(character);
            }
        }
        self.serialize_one_or_more_sixel_characters(
            current_character_occurrences,
            current_character.unwrap(),
        );
    }
    fn serialize_color_introducer(&mut self, color_index: &u8) {
        self.append_to.push_str(&format!("#{}", color_index));
    }
    fn pad_sixel_string(&self, sixel_chars: &mut String, desired_length: usize) {
        for _ in sixel_chars.len()..desired_length {
            sixel_chars.push('?');
        }
    }
}
