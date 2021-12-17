// Shamefule theft from Lily Mara/Code Tech as a baseline - https://www.youtube.com/watch?v=Iapc-qGTEBQ
use zaps::{
    iso8583_use,
    iso8583::Iso8583Engine,
};
use zaps_sim::{
    iso8583_spec,
    serve,
};

iso8583_use!();

#[tokio::main]
async fn main() {
    let spec = iso8583_spec();
    // try sending "iso8583:020081003ABC0123456789abcde" or similar
    let engine = Iso8583Engine::new(spec);
    serve(engine).await;
}
