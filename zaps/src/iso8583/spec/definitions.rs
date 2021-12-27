use std::str::FromStr;
use std::fmt;

#[derive(Debug)]
pub enum FieldParseError {
    InvalidDataType(String),
    InvalidFormat(String),
    InvalidType(String),
    InvalidLength(String),
    JunkTrail(String),
}

#[derive(Debug, PartialEq)]
pub enum DataType {
    Alpha,
    Alphanum,
    Binary,
    Hex,
    Numeric,
    Packed,
}

impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl FromStr for DataType {
    type Err = FieldParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match &s.to_ascii_lowercase()[..] {
            "alpha" | "a" => Ok(DataType::Alpha),
            "alphanum" | "alphanumeric" | "an" => Ok(DataType::Alphanum),
            "bin" | "binary" | "bit" | "bitmap" => Ok(DataType::Binary),
            "hex" | "hexadecimal" | "h" => Ok(DataType::Hex),
            "numeric" | "n" => Ok(DataType::Numeric),
            "packed" | "asciihex" => Ok(DataType::Packed),
            _ => Err(FieldParseError::InvalidDataType(s.to_string())),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum FieldType {
    Fixed,
    LVar,
    LLVar,
    LLLVar,
    Bitmap,
    AsciiBitmap,
}

impl FieldType {
    pub fn var_size_len(&self) -> Option<usize> {
        match self {
            FieldType::LLLVar => Some(3),
            FieldType::LLVar => Some(2),
            FieldType::LVar => Some(1),
            _ => None,
        }
    }
}

impl fmt::Display for FieldType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl FromStr for FieldType {
    type Err = FieldParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match &s.to_ascii_lowercase()[..] {
            "fixed" => Ok(FieldType::Fixed),
            "lvar" => Ok(FieldType::LVar),
            "llvar" => Ok(FieldType::LLVar),
            "lllvar" => Ok(FieldType::LLLVar),
            "bitmap" => Ok(FieldType::Bitmap),
            "asciibitmap" => Ok(FieldType::AsciiBitmap),
            _ => Err(FieldParseError::InvalidType(s.to_string())),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Field {
    pub ftype: FieldType,
    /// The logical size of the field e.g. for a binary bitmap with 8 bits this would be 8.
    /// In general size should be preferred.
    pub raw_size: usize,
    /// The actual size of a the field e.g. for a binary bitmap with 8 bits this would be 1.
    pub size: usize,
    pub data_type: DataType,
}

impl Field {
    pub fn new(ftype: FieldType, raw_size: usize, data_type: DataType) -> Self {
        let size = match ftype {
            FieldType::AsciiBitmap => (raw_size + 3) / 4,
            FieldType::Bitmap => (raw_size + 7) / 8,
            _ => raw_size,
        };
        Field {
            ftype,
            raw_size,
            size,
            data_type,
        }
    }
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.ftype, self.size, self.data_type)
    }
}

impl FromStr for Field {
    type Err = FieldParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (s_ftype, rest) = s.split_once('(')
            .ok_or_else(|| FieldParseError::InvalidFormat(s.to_string()))?;

        let ftype = s_ftype.parse::<FieldType>()?;

        let (internal, junk_trail) = rest.split_once(')')
            .ok_or_else(|| FieldParseError::InvalidFormat(s.to_string()))?;

        let (s_size, s_data_type) = match ftype {
            FieldType::AsciiBitmap => {
                if internal.contains(':') {
                    Err(FieldParseError::InvalidFormat(s.to_string()))
                } else {
                    Ok((internal, "packed"))
                }?
            },
            FieldType::Bitmap => {
                if internal.contains(':') {
                    Err(FieldParseError::InvalidFormat(s.to_string()))
                } else {
                    Ok((internal, "binary"))
                }?
            },
            FieldType::LVar |
            FieldType::LLVar |
            FieldType::LLLVar => {
                if internal.contains(':') {
                    Err(FieldParseError::InvalidFormat(s.to_string()))
                } else {
                    Ok(("0", internal))
                }?
            },
            _ => {
                internal.split_once(':')
                    .ok_or_else(|| FieldParseError::InvalidFormat(s.to_string()))?
            }
        };

        let size = s_size.parse::<usize>()
            .map_err(|_e| FieldParseError::InvalidLength(s_size.to_string()))?;
        

        let data_type = s_data_type.parse::<DataType>()?;

        if !junk_trail.is_empty() {
            return Err(FieldParseError::JunkTrail(junk_trail.to_string()));
        }

        Ok(Field::new(
            ftype,
            size,
            data_type,
        ))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! parse_tests {
        ($($name:ident: $str:expr => $expect_type:expr, $expect_size:expr, $expect_data:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let result = $str.parse::<Field>().unwrap();
                    assert_eq!($expect_type, result.ftype);
                    assert_eq!($expect_size, result.size);
                    assert_eq!($expect_data, result.data_type);
                }
            )*
        };
    }

    parse_tests!(
        parse_fixed: "Fixed(16:a)" => FieldType::Fixed, 16, DataType::Alpha,
        parse_lllvar: "LLLVar(an)" => FieldType::LLLVar, 0, DataType::Alphanum,
        parse_llvar: "LLVar(n)" => FieldType::LLVar, 0, DataType::Numeric,
        parse_lvar: "LVar(h)" => FieldType::LVar, 0, DataType::Hex,
        parse_bitmap: "Bitmap(64)" => FieldType::Bitmap, 8, DataType::Binary,
        parse_bitmap_adjust_low: "Bitmap(65)" => FieldType::Bitmap, 9, DataType::Binary,
        parse_bitmap_adjust_high: "Bitmap(71)" => FieldType::Bitmap, 9, DataType::Binary,
        parse_ascii_bitmap: "AsciiBitmap(64)" => FieldType::AsciiBitmap, 16, DataType::Packed,
        parse_ascii_bitmap_adjust_low: "AsciiBitmap(65)" => FieldType::AsciiBitmap, 17, DataType::Packed,
        parse_ascii_bitmap_adjust_high: "AsciiBitmap(67)" => FieldType::AsciiBitmap, 17, DataType::Packed,
    );

    macro_rules! parse_error_tests {
        ($($name:ident: $str:expr => $expect_err:path,)*) => {
            $(
                #[test]
                fn $name() -> Result<(), String> {
                    let res = $str.parse::<Field>();
                    if let Err($expect_err(_)) = res {
                        Ok(())
                    } else {
                        Err(format!("Unexpected result: {:?}", res))
                    }
                }
            )*
        };
    }

    parse_error_tests! {
        error_format_open_bracket: "fixed19:an)" => FieldParseError::InvalidFormat,
        error_format_separator_bracket: "fixed(19an)" => FieldParseError::InvalidFormat,
        error_format_close_bracket: "fixed(19:an" => FieldParseError::InvalidFormat,
        error_junk_trail: "fixed(19:an)skflnasf" => FieldParseError::JunkTrail,
        error_length: "fixed(weg:an)" => FieldParseError::InvalidLength,
        error_data_type: "fixed(19:asgaeg)" => FieldParseError::InvalidDataType,
        error_type: "asg(19:an)" => FieldParseError::InvalidType,
        error_lvar_len: "lvar(19:an)" => FieldParseError::InvalidFormat,
        error_llvar_len: "llvar(19:an)" => FieldParseError::InvalidFormat,
        error_lllvar_len: "lllvar(19:an)" => FieldParseError::InvalidFormat,
    }
}