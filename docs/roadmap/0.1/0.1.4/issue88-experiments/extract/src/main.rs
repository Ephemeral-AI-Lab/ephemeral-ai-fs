use std::io::{self, BufRead, Write};
use layerfs_content::file::{cdc::FastCdc, extent_codec::encode_chunk_object};
fn main() {
    for path in io::stdin().lock().lines() {
        let path=path.unwrap(); let f=std::fs::File::open(path).unwrap(); let mut offset=0u64;
        let mut parts=Vec::new();
        FastCdc::new().scan(f, |chunk| {
            let c=encode_chunk_object(chunk)?;
            let id=layerfs_content::identify_canonical(&c)?.0;
            let hex=id.as_bytes().iter().map(|b|format!("{b:02x}")).collect::<String>();
            parts.push(format!("{hex},{offset},{}",chunk.len()));offset+=chunk.len() as u64;Ok(())
        }).unwrap();
        println!("{}",parts.join(";"));io::stdout().flush().unwrap();
    }
}
