fn main() {
    let root = std::path::PathBuf::from(std::env::args_os().nth(1).expect("fresh output directory"));
    std::fs::create_dir(&root).unwrap();
    let input = root.join("input");
    std::fs::create_dir(&input).unwrap();
    let payload = (0..8192).map(|i| (i % 251) as u8).collect::<Vec<_>>();
    std::fs::write(input.join("file"), &payload).unwrap();
    let native_path = root.join("native.sqlite");
    let store = candidate::LayerStackStore::create(&native_path).unwrap();
    store.initialize_layerstack(candidate::EntityName::new("native").unwrap(), candidate::LayerStackInitialization::Directory(input.clone())).unwrap();
    verify_new(&store, "native", &payload);
    assert!(store.physical_storage_receipt().native_admitted_full_count > 0);
    drop(store);
    let before = std::fs::read(&native_path).unwrap();
    assert!(matches!(legacy::LayerStackStore::connect(&native_path), Err(legacy::StoreError::WrongStoreSchema)));
    assert_eq!(std::fs::read(&native_path).unwrap(), before);
    let store = candidate::LayerStackStore::connect(&native_path).unwrap();
    assert_eq!(store.store_counts().unwrap().layer_stacks, 1);
    verify_new(&store, "native", &payload);
    drop(store);
    println!("PASS actual older schema6 code rejects native7 open before mutation; candidate reopens intact");

    let legacy_path = root.join("legacy.sqlite");
    let store = legacy::LayerStackStore::create(&legacy_path).unwrap();
    store.initialize_layerstack(legacy::EntityName::new("old").unwrap(), legacy::LayerStackInitialization::Directory(input.clone())).unwrap();
    drop(store);
    let store = candidate::LayerStackStore::connect(&legacy_path).unwrap();
    assert_eq!(store.store_counts().unwrap().layer_stacks, 1);
    verify_new(&store, "old", &payload);
    store.initialize_layerstack(candidate::EntityName::new("new").unwrap(), candidate::LayerStackInitialization::Directory(input.clone())).unwrap();
    drop(store);
    let store = legacy::LayerStackStore::connect(&legacy_path).unwrap();
    assert_eq!(store.store_counts().unwrap().layer_stacks, 2);
    verify_old(&store, "old", &payload);
    verify_old(&store, "new", &payload);
    store.initialize_layerstack(legacy::EntityName::new("old-again").unwrap(), legacy::LayerStackInitialization::Directory(input.clone())).unwrap();
    drop(store);
    let store = candidate::LayerStackStore::connect(&legacy_path).unwrap();
    assert_eq!(store.store_counts().unwrap().layer_stacks, 3);
    for name in ["old", "new", "old-again"] { verify_new(&store, name, &payload); }
    drop(store);
    println!("PASS actual older/newer alternating legacy6 reads and writes; no migration");
    let mut names = std::fs::read_dir(&root).unwrap().map(|p|p.unwrap().file_name()).collect::<Vec<_>>(); names.sort();
    assert_eq!(names, ["input", "legacy.sqlite", "native.sqlite"].map(std::ffi::OsString::from));
    println!("PASS Stores closed; no journal/WAL/SHM residue");
}

fn verify_new(store: &candidate::LayerStackStore, name: &str, expected: &[u8]) {
    let stack = store.layer_stack_by_name(&candidate::EntityName::new(name).unwrap()).unwrap().unwrap();
    let root = store.layer(stack.head_layer_id).unwrap().unwrap().root_id;
    let mut bytes = Vec::new();
    candidate_content::filesystem::stream(&candidate::CoreReader(store), root, &candidate_content::CanonicalPath::new("file").unwrap(), &mut bytes).unwrap();
    assert_eq!(bytes, expected);
}
fn verify_old(store: &legacy::LayerStackStore, name: &str, expected: &[u8]) {
    let stack = store.layer_stack_by_name(&legacy::EntityName::new(name).unwrap()).unwrap().unwrap();
    let root = store.layer(stack.head_layer_id).unwrap().unwrap().root_id;
    let mut bytes = Vec::new();
    legacy_content::filesystem::stream(&legacy::CoreReader(store), root, &legacy_content::CanonicalPath::new("file").unwrap(), &mut bytes).unwrap();
    assert_eq!(bytes, expected);
}
