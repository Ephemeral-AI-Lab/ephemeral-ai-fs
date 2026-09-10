use super::*;
use crate::CoreReader;
use layerfs_content::filesystem;

#[test]
fn directory_and_fallback_import_encode_only_regular_payloads_and_reopen() {
    use std::os::unix::fs::{symlink, MetadataExt, PermissionsExt};
    for hard_link in [false, true] {
        let root = std::env::temp_dir().join(format!(
            "layerfs-issue90-ingestion-{}-{}-{hard_link}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let source = root.join("input");
        std::fs::create_dir_all(source.join("nested")).unwrap();
        let data = b"LFS4INT\0arbitrary user content";
        std::fs::write(source.join("nested/file"), data).unwrap();
        std::fs::write(source.join("empty"), []).unwrap();
        std::fs::set_permissions(
            source.join("nested/file"),
            std::fs::Permissions::from_mode(0o640),
        )
        .unwrap();
        let metadata = std::fs::metadata(source.join("nested/file")).unwrap();
        if hard_link {
            symlink("nested/file", source.join("link")).unwrap();
            std::fs::hard_link(source.join("nested/file"), source.join("alias")).unwrap();
        }
        let path = root.join("store.sqlite");
        let store = LayerStackStore::create(&path).unwrap();
        // Prove this exact fixture reaches the intended native entrypoint;
        // dropping the unpublished handoff must clean its private admission.
        let prepared = direct_initialize_root_directories(&store.db, &source, [73; 32]).unwrap();
        assert_eq!(prepared.is_some(), !hard_link);
        drop(prepared);
        assert_eq!(store.store_counts().unwrap().objects, 0);
        let physical_before = store.physical_storage_receipt();
        let initialized = store
            .initialize_layerstack(
                EntityName::new("directory").unwrap(),
                LayerStackInitialization::Directory(source),
            )
            .unwrap();
        let receipts = store.take_layerstack_initialization_receipts();
        let receipt = receipts
            .iter()
            .find(|r| r.layer_stack_id == initialized.layer_stack_id)
            .unwrap();
        assert_eq!(receipt.source_passes, 1);
        let layer = store.layer(initialized.genesis_layer_id).unwrap().unwrap();
        let before = store.store_counts().unwrap();
        let native = store.physical_storage_receipt().since(physical_before);
        assert_eq!(native.native_admitted_full_count, 0,
            "this small regular payload uses SmallContent; metadata must not enter native admission");
        let reader = CoreReader(&store);
        let (stat, _) = filesystem::stat(
            &reader,
            layer.root_id,
            &layerfs_content::CanonicalPath::new("nested/file").unwrap(),
        )
        .unwrap();
        let canonical = crate::ObjectSource::read_object(&store, stat.content_root).unwrap();
        layerfs_content::authenticate_identity(&canonical, stat.content_root).unwrap();
        assert_eq!(
            layerfs_content::file::content::small_bytes(&canonical).unwrap(),
            Some(data.as_slice())
        );
        drop(store);
        let reopened = LayerStackStore::connect(&path).unwrap();
        let reader = CoreReader(&reopened);
        let file = layerfs_content::CanonicalPath::from_bytes(b"nested/file").unwrap();
        let mut contents = Vec::new();
        filesystem::stream(&reader, layer.root_id, &file, &mut contents).unwrap();
        assert_eq!(contents, data);
        let (stat, _) = filesystem::stat(&reader, layer.root_id, &file).unwrap();
        let mut expected_metadata = ObjectBuffer::empty().unwrap();
        let expected_metadata = filesystem::build_portable_metadata(
            &mut expected_metadata,
            layerfs_content::tree::inode::InodeKind::RegularFile,
            metadata.mode(),
            metadata.mtime(),
            metadata.mtime_nsec() as u32,
        )
        .unwrap();
        assert_eq!(stat.metadata_root, expected_metadata);
        if hard_link {
            assert_eq!(
                filesystem::readlink(
                    &reader,
                    layer.root_id,
                    &layerfs_content::CanonicalPath::from_bytes(b"link").unwrap()
                )
                .unwrap()
                .0,
                b"nested/file"
            );
        }
        assert_eq!(reopened.store_counts().unwrap(), before);
        if hard_link {
            let (stat, _) = filesystem::stat(&reader, layer.root_id, &file).unwrap();
            assert_eq!(stat.namespace_ref_count, 2);
        }
        drop(reopened);
        std::fs::remove_dir_all(root).unwrap();
    }
}
