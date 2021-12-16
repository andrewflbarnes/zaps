#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        zaps_8583::iso8583_use!();
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
