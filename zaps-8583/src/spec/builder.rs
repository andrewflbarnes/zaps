#[macro_export]
macro_rules! iso8583_use {
    () => {
use zaps_8583::{
    iso8583_spec_build,
    iso8583_field_build,
    Field,
    FieldType,
    DataType,
    Spec,
};
    };
}

#[macro_export]
macro_rules! iso8583_field_build {
    ($field_type:ident $field_size:literal $field_data:ident) => {{
        iso8583_field_build!($field_type $field_size DataType::$field_data)
    }};
    ($field_type:ident $field_size:literal $field_data:path) => {{
        Field::new(FieldType::$field_type, $field_size, $field_data)
    }};
}

#[macro_export]
macro_rules! iso8583_spec_build {
    // fully defined field
    (@build $spec:ident $mti:literal => $mti_spec:ident $field_num:literal: $field_type:ident, $field_size:literal, $field_data:ident; $($rest:tt)*) => {{
        let field = iso8583_field_build!($field_type $field_size $field_data);
        $mti_spec.insert($field_num, field);
        iso8583_spec_build!(@build $spec $mti => $mti_spec $($rest)*);
    }};
    // variable length field (LLVar, etc.)
    (@build $spec:ident $mti:literal => $mti_spec:ident $field_num:literal: $field_type:ident, $field_data:ident; $($rest:tt)*) => {{
        let field = iso8583_field_build!($field_type 0 $field_data);
        $mti_spec.insert($field_num, field);
        iso8583_spec_build!(@build $spec $mti => $mti_spec $($rest)*);
    }};
    // data specific field (LLVar, etc.)
    (@build $spec:ident $mti:literal => $mti_spec:ident $field_num:literal: $field_type:ident, $field_size:literal; $($rest:tt)*) => {{
        let field_data = match FieldType::$field_type {
            FieldType::Bitmap => DataType::Binary,
            FieldType::AsciiBitmap => DataType::Packed,
            _ => DataType::Alphanum,
        };
        let field = Field::new(FieldType::$field_type, $field_size, field_data);
        $mti_spec.insert($field_num, field);
        iso8583_spec_build!(@build $spec $mti => $mti_spec $($rest)*);
    }};
    // new MTI block
    (@build $spec:ident $mti:literal => $mti_spec:ident $next_mti:literal: $($rest:tt)*) => {{
        $spec.add_spec($mti.to_string(), $mti_spec);
        let mut mti_spec: std::collections::HashMap<u16, Field> = std::collections::HashMap::new();
        iso8583_spec_build!(@build $spec $next_mti => mti_spec $($rest)*);
    }};
    // exitpoint
    (@build $spec:ident $mti:literal => $mti_spec:ident) => {{
        $spec.add_spec($mti.to_string(), $mti_spec);
    }};
    // entrypoint
    ($first_mti:literal: $($rest:tt)*) => {{
        let mut spec = Spec::new();
        let mut mti_spec: std::collections::HashMap<u16, Field> = std::collections::HashMap::new();
        iso8583_spec_build!(@build spec $first_mti => mti_spec $($rest)*);
        spec
    }};
}

#[cfg(test)]
mod test {
    // can't use is08583_use! directly here so just copy
    use crate::{
        iso8583_spec_build,
        iso8583_field_build,
        Field,
        FieldType,
        DataType,
        Spec,
    };

    #[test]
    fn test() {
        let spec2 = iso8583_spec_build!{
            "0200":
                1: LLLVar, Alphanum;
                2: LLVar, Alphanum;
                3: LVar, Alphanum;
                4: Fixed, 15, Alphanum;
                5: Bitmap, 64;
            "0210":
                1: LLLVar, Numeric;
                2: LLVar, Packed;
                3: LVar, Hex;
                4: Fixed, 15, Alpha;
                5: AsciiBitmap, 64;
        };
        println!("{:#?}", spec2);
    }
}