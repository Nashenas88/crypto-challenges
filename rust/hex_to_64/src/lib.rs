#![feature(conservative_impl_trait)]
extern crate itertools;
use itertools::Itertools;

pub fn hex_to_base64<I: Iterator<Item=u8>>(bytes: I) -> impl Iterator<Item=Option<u8>> {
    bytes
        .batching(|it| it.next().map(|x| (x, it.next(), it.next())))
        .flat_map(translate)
}

struct QuadIterator<T> {
    array: [T;4],
    idx: usize,
}

impl<T> QuadIterator<T> {
    fn new (array: [T;4]) -> QuadIterator<T> {
        QuadIterator { array: array, idx: 0 }
    }
}

impl<T> Iterator for QuadIterator<T> where T: Copy {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        if self.idx < 4 {
            let ret = self.array[self.idx];
            self.idx += 1;
            Some(ret)
        } else {
            None
        }
    }
}

fn translate(bytes_in: (u8, Option<u8>, Option<u8>)) -> impl Iterator<Item=Option<u8>> {
    let mask = 0b0011_1111;

    QuadIterator::new([
        Some(bytes_in.0 >> 2),
        Some(((bytes_in.0 << 4) & mask) + bytes_in.1.map_or(0, |byte| byte >> 4)),
        bytes_in.1.map(|byte| ((byte << 2) & mask) + bytes_in.2.map_or(0, |byte| byte >> 6)),
        bytes_in.2.map(|byte| byte & mask)
    ])
}

// utility functions
pub fn ascii_encode_base64(value: Option<u8>) -> u8 {
    value.map_or('=' as u8, |value| match value {
        n if n < 26 => 'A' as u8 + n,
        n if n > 25 && n < 52 => 'a' as u8 + n - 26,
        n if n > 51 && n < 62 => '0' as u8 + n - 52,
        62 => '+' as u8,
        63 => '/' as u8,
        _ => panic!("invalid value")
    })
}

fn char_to_hex(c: u8) -> u8 {
    match c {
        n if n >= '0' as u8 && n <= '9' as u8 => n - '0' as u8,
        a if a >= 'a' as u8 && a <= 'f' as u8 => a + 10 - 'a' as u8,
        a if a >= 'A' as u8 && a <= 'F' as u8 => a + 10 - 'A' as u8,
        v => panic!("unexpected value {}", v),
    }
}

pub fn hex_str_to_u8_iter<'s>(s: &'s str) -> impl Iterator<Item=u8> + 's {
    s.bytes()
        .map(char_to_hex)
        .batching(|it| it.next().map(|x| (x, it.next())))
        .map(|(a, b)| (a << 4) + b.unwrap_or(0))
}

// Tests from here on
#[cfg(test)]
pub mod hex_to_base64_tests {
    use hex_to_base64;
    use ascii_encode_base64;
    use hex_str_to_u8_iter;
    use translate;
    
    #[test]
    fn when_full24_bits_does_not_output_equals() {
        hex_to_base_64(
            "49276d206b696c6c696e6720796f757220627261696e206c696b65206120706f69736f6e6f7573206d757368726f6f6d",
            "SSdtIGtpbGxpbmcgeW91ciBicmFpbiBsaWtlIGEgcG9pc29ub3VzIG11c2hyb29t");
    }
    
    #[test]
    fn when_20_bits_does_not_output_equals() {
        hex_to_base_64(
            "49276d206b696c6c696e6720796f757220627261696e206c696b65206120706f69736f6e6f7573206d757368726f6f6",
            "SSdtIGtpbGxpbmcgeW91ciBicmFpbiBsaWtlIGEgcG9pc29ub3VzIG11c2hyb29g");
    }
    
    #[test]
    fn when_16_bits_outputs_one_equal() {
        hex_to_base_64(
            "49276d206b696c6c696e6720796f757220627261696e206c696b65206120706f69736f6e6f7573206d757368726f6f",
            "SSdtIGtpbGxpbmcgeW91ciBicmFpbiBsaWtlIGEgcG9pc29ub3VzIG11c2hyb28=");
    }

    #[test]
    fn when_12_bits_outputs_one_equal() {
        hex_to_base_64(
            "49276d206b696c6c696e6720796f757220627261696e206c696b65206120706f69736f6e6f7573206d757368726f6",
            "SSdtIGtpbGxpbmcgeW91ciBicmFpbiBsaWtlIGEgcG9pc29ub3VzIG11c2hyb2A=");
    }
    
    #[test]
    fn when_8_bits_outputs_two_equals() {
        hex_to_base_64(
            "49276d206b696c6c696e6720796f757220627261696e206c696b65206120706f69736f6e6f7573206d757368726f",
            "SSdtIGtpbGxpbmcgeW91ciBicmFpbiBsaWtlIGEgcG9pc29ub3VzIG11c2hybw==");
    }
    
    #[test]
    fn when_4_bits_outputs_two_equals() {
        hex_to_base_64(
            "49276d206b696c6c696e6720796f757220627261696e206c696b65206120706f69736f6e6f7573206d757368726",
            "SSdtIGtpbGxpbmcgeW91ciBicmFpbiBsaWtlIGEgcG9pc29ub3VzIG11c2hyYA==");
    }
    
    fn hex_to_base_64 (hex_str: &str, expected_base64_str: &str)
    {
        let hex_u8 = hex_str_to_u8_iter(hex_str);
        let base64_u8 = hex_to_base64(hex_u8)
            .map(|byte| ascii_encode_base64(byte))
            .collect();
        let base64 = String::from_utf8(base64_u8).unwrap();
        assert_eq!(base64, expected_base64_str);
    }
    
    #[test]
    fn translate_3_values_should_return_4_values() {
        let bytes_in = (77, Some(97), Some(110));
        let mut iter = translate(bytes_in);
        
        assert_eq!(iter.next().expect("There should be more items").expect("Should be 19"), 19);
        assert_eq!(iter.next().expect("There should be more items").expect("Should be 22"), 22);
        assert_eq!(iter.next().expect("There should be more items").expect("Should be 5"), 5);
        assert_eq!(iter.next().expect("There should be more items").expect("Should be 46"), 46);
        assert_eq!(iter.next().is_none(), true);
    }
    
    #[test]
    fn translate_2_values_should_return_3_values() {
        let bytes_in = (77, Some(97), None);
        let mut iter = translate(bytes_in);

        assert_eq!(iter.next().expect("There should be more items").expect("Should be 19"), 19);
        assert_eq!(iter.next().expect("There should be more items").expect("Should be 22"), 22);
        assert_eq!(iter.next().expect("There should be more items").expect("Should be 4"), 4);
        assert_eq!(iter.next().expect("There should be more items").is_none(), true);
        assert_eq!(iter.next().is_none(), true);
    }
    
    #[test]
    fn translate_1_value_should_return_2_values() {
        let bytes_in = (77, None, None);
        let mut iter = translate(bytes_in);

        assert_eq!(iter.next().expect("There should be more items").expect("Should be 19"), 19);
        assert_eq!(iter.next().expect("There should be more items").expect("Should be 16"), 16);
        assert_eq!(iter.next().expect("There should be more items").is_none(), true);
        assert_eq!(iter.next().expect("There should be more items").is_none(), true);
        assert_eq!(iter.next().is_none(), true);
    }
}
