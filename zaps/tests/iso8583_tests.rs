use zaps::{
    core::Parser,
    iso8583::{
        Iso8583Engine,
        spec::{
            DataType,
            Field,
            FieldType,
        },
    },
    iso8583_spec_build,
};

macro_rules! assert_spec_has_field {
    ($mti_spec:ident: $($fnum:literal: $ftype:ident $fsize:literal $fdata:ident,)*) => {
        $(
            let field = $mti_spec.get(&$fnum).unwrap();
            assert_eq!(Field::new(FieldType::$ftype, $fsize, DataType::$fdata), *field);
        )*
    };
}

#[test]
fn it_works() {
    let spec = iso8583_spec_build!{
        "0200":
            0: Bitmap, 64;
            1: LLLVar, Alphanum;
            2: LLVar, Alphanum;
            3: LVar, Alphanum;
            4: Fixed, 15, Alphanum;
            5: Bitmap, 64;
            64: Fixed, 16, Hex;
        "0210":
            0: Bitmap, 64;
            1: LLLVar, Numeric;
            2: LLVar, Packed;
            3: LVar, Hex;
            4: Fixed, 15, Alpha;
            5: AsciiBitmap, 64;
    };

    let spec_0200 = spec.get_mti_spec(&"0200").unwrap();
    assert_spec_has_field!(spec_0200:
        1: LLLVar 0 Alphanum,
        2: LLVar 0 Alphanum,
        3: LVar 0 Alphanum,
        4: Fixed 15 Alphanum,
        5: Bitmap 64 Binary,
        64: Fixed 16 Hex,
    );

    let spec_0210 = spec.get_mti_spec(&"0210").unwrap();
    assert_spec_has_field!(spec_0210:
        1: LLLVar 0 Numeric,
        2: LLVar 0 Packed,
        3: LVar 0 Hex,
        4: Fixed 15 Alpha,
        5: AsciiBitmap 64 Packed,
    );

    let engine = Iso8583Engine::new(spec);

    let tokens = engine.tokenise(&[
        "0200".as_bytes(),
        &[0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01],
        "0100123456789".as_bytes(),
        "1234567890abcdef".as_bytes(),
    ].concat()[..])
        .unwrap();

    println!("{:?}", tokens);
}