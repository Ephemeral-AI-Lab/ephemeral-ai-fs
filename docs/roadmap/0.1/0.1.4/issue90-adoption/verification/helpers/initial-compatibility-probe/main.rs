fn main() {
    let root = std::path::PathBuf::from(std::env::args_os().nth(1).expect("fresh output directory"));
    std::fs::create_dir(&root).unwrap();
    let native_path = root.join("native.sqlite");
    let store = candidate::LayerStackStore::create(&native_path).unwrap();
    store.initialize_layerstack(candidate::EntityName::new("native").unwrap(), candidate::LayerStackInitialization::Empty).unwrap();
    drop(store);
    let before = std::fs::read(&native_path).unwrap();
    assert!(matches!(legacy::LayerStackStore::connect(&native_path), Err(legacy::StoreError::WrongStoreSchema)));
    assert_eq!(std::fs::read(&native_path).unwrap(), before);
    let store = candidate::LayerStackStore::connect(&native_path).unwrap();
    assert_eq!(store.store_counts().unwrap().layer_stacks, 1);
    drop(store);
    println!("PASS actual older schema6 code rejects native7 open before mutation; candidate reopens intact");

    let legacy_path = root.join("legacy.sqlite");
    let store = legacy::LayerStackStore::create(&legacy_path).unwrap();
    store.initialize_layerstack(legacy::EntityName::new("old").unwrap(), legacy::LayerStackInitialization::Empty).unwrap();
    drop(store);
    let store = candidate::LayerStackStore::connect(&legacy_path).unwrap();
    assert_eq!(store.store_counts().unwrap().layer_stacks, 1);
    store.initialize_layerstack(candidate::EntityName::new("new").unwrap(), candidate::LayerStackInitialization::Empty).unwrap();
    drop(store);
    let store = legacy::LayerStackStore::connect(&legacy_path).unwrap();
    assert_eq!(store.store_counts().unwrap().layer_stacks, 2);
    store.initialize_layerstack(legacy::EntityName::new("old-again").unwrap(), legacy::LayerStackInitialization::Empty).unwrap();
    drop(store);
    let store = candidate::LayerStackStore::connect(&legacy_path).unwrap();
    assert_eq!(store.store_counts().unwrap().layer_stacks, 3);
    drop(store);
    println!("PASS actual older/newer alternating legacy6 reads and writes; no migration");
    let mut names = std::fs::read_dir(&root).unwrap().map(|p|p.unwrap().file_name()).collect::<Vec<_>>(); names.sort();
    assert_eq!(names, ["legacy.sqlite", "native.sqlite"].map(std::ffi::OsString::from));
    println!("PASS Stores closed; no journal/WAL/SHM residue");
}
