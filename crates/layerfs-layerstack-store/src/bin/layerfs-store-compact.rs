//! Offline maintenance entry point for the same public product compaction API.
use layerfs_layerstack_store::{CompactionOptions, LayerStackStore};

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args_os().skip(1).collect::<Vec<_>>();
    if args.len() == 1 && args[0] == "--help" {
        println!("Usage: layerfs-store-compact SOURCE DESTINATION [TEMPORARY_BYTE_LIMIT]\nCreate a verified, self-contained compacted Store at a new destination. SOURCE is preserved.");
        return Ok(());
    }
    if !(2..=3).contains(&args.len()) {
        return Err("expected SOURCE DESTINATION [TEMPORARY_BYTE_LIMIT]".into());
    }
    let mut options = CompactionOptions::default();
    if args.len() == 3 {
        options.temporary_byte_limit = args[2]
            .to_str()
            .ok_or("temporary limit must be decimal bytes")?
            .parse()?;
    }
    let store = LayerStackStore::connect(&args[0])?;
    let receipt = store.compact_into(&args[1], options)?;
    println!("{receipt:#?}");
    if !receipt.published || !receipt.cleanup_complete || !receipt.directory_synced {
        return Err(
            "compaction published with an incomplete cleanup or directory sync; see receipt".into(),
        );
    }
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
