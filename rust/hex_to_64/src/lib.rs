extern crate itertools;
use itertools::Itertools;
use std::iter::FromIterator;
use std::vec::IntoIter;

pub fn hex_to_base64<I, B>(bytes: I) -> B
    where
        I: Iterator<Item=u8>,
        B: FromIterator<u8> {
    bytes
        .batching(|it| it.next().map_or(None, |x| Some((x, it.next(), it.next()))))
        .flat_map(|group_of_3| translate(group_of_3))
        .map(|byte| lookup(byte))
        .collect()
}

fn translate(bytes_in: (u8, Option<u8>, Option<u8>)) -> IntoIter<Option<u8>> {
    vec![
        Some(bytes_in.0 >> 2),
        Some((bytes_in.0 << 6 >> 2) + bytes_in.1.map_or(0, |byte| byte >> 4)),
        bytes_in.1.map_or(None,
            |byte| Some((byte << 4 >> 2) + bytes_in.2.map_or(0, |byte| byte >> 6))),
        bytes_in.2.map_or(None, |byte| Some(byte << 2 >> 2))
    ].into_iter()
}

fn lookup(value: Option<u8>) -> u8 {
    value.map_or('=' as u8, |value| match value {
        n if n < 26 => 'A' as u8 + n,
        n if n > 25 && n < 52 => 'a' as u8 + n - 26,
        n if n > 51 && n < 62 => '0' as u8 + n - 52,
        62 => '+' as u8,
        63 => '/' as u8,
        _ => panic!("invalid value")
    })
}

// Tests from here on
#[cfg(test)]
pub mod hex_to_base64_tests {
    pub use hex_to_base64;
    use translate;
    use itertools::Itertools;

    fn char_to_hex(c: u8) -> u8 {
        match c {
            n if n >= '0' as u8 && n <= '9' as u8 => n - '0' as u8,
            a if a >= 'a' as u8 && a <= 'f' as u8 => a + 10 - 'a' as u8,
            a if a >= 'A' as u8 && a <= 'F' as u8 => a + 10 - 'A' as u8,
            v => panic!("unexpected value {}", v),
        }
    }

    fn batch_by_2<I>(it: &mut I) -> Option<(I::Item, Option<I::Item>)> where I: Iterator {
        match it.next() {
            Some(x) => Some((x, it.next())),
            None => None,
        }
    }

    pub fn hex_str_to_u8_iter<'a>(s: &'a str) -> Box<Iterator<Item=u8> + 'a> {
        Box::new(s.bytes()
                 .map(|c| char_to_hex(c))
                 .batching(batch_by_2)
                 .map(|(a, b)| (a << 4) + b.unwrap_or(0)))
    }
    
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
        let binary_base64 = hex_to_base64(hex_u8);
        let base64 = String::from_utf8(binary_base64).unwrap();
        assert_eq!(base64, expected_base64_str);
    }
    
    #[test]
    fn test_translate_3_values() {
        let bytes_in = (77, Some(97), Some(110));
        println!("{:?}", bytes_in);
        let mut iter = translate(bytes_in);
        assert_eq!(iter.next().expect("There should be more items").expect("Should be 19"), 19);
        assert_eq!(iter.next().expect("There should be more items").expect("Should be 22"), 22);
        assert_eq!(iter.next().expect("There should be more items").expect("Should be 5"), 5);
        assert_eq!(iter.next().expect("There should be more items").expect("Should be 46"), 46);
        assert_eq!(iter.next().is_none(), true);
    }
    
    #[test]
    fn test_translate_2_values() {
        let bytes_in = (77, Some(97), None);
        println!("{:?}", bytes_in);
        let mut iter = translate(bytes_in);
        assert_eq!(iter.next().expect("There should be more items").expect("Should be 19"), 19);
        assert_eq!(iter.next().expect("There should be more items").expect("Should be 22"), 22);
        assert_eq!(iter.next().expect("There should be more items").expect("Should be 4"), 4);
        assert_eq!(iter.next().expect("There should be more items").is_none(), true);
        assert_eq!(iter.next().is_none(), true);
    }
    
    #[test]
    fn test_translate_1_value() {
        let bytes_in = (77, None, None);
        println!("{:?}", bytes_in);
        let mut iter = translate(bytes_in);
        assert_eq!(iter.next().expect("There should be more items").expect("Should be 19"), 19);
        assert_eq!(iter.next().expect("There should be more items").expect("Should be 16"), 16);
        assert_eq!(iter.next().expect("There should be more items").is_none(), true);
        assert_eq!(iter.next().expect("There should be more items").is_none(), true);
        assert_eq!(iter.next().is_none(), true);
    }
}
