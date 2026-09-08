use super::*;

#[test]
fn file_provenance_is_scoped_and_restored_before_metadata_and_after_failure() {
    let mut delivered = Vec::new();
    let cancelled = std::sync::atomic::AtomicBool::new(false);
    run_finalized_output(
        1,
        1,
        [()].into_iter(),
        &cancelled,
        |_| Ok(()),
        |_, _, _, output| {
            build_checked_file(output, b"regular-file".as_slice(), 12)?;
            assert!(!output.file_payload_context);
            assert!(build_checked_file(output, b"short".as_slice(), 6).is_err());
            assert!(!output.file_payload_context);
            // Generic ropes carry metadata values, even if their user bytes look
            // like structural/file magic. Only the producer context is authority.
            layerfs_content::file::rope::build_bytes(output, b"LFS4CHK\0metadata")?;
            Ok(())
        },
        |_| Ok(()),
        |page| {
            delivered.extend(page);
            Ok(())
        },
    )
    .unwrap();
    let files: Vec<_> = delivered.iter().filter(|o| o.is_file_payload()).collect();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].1.first_span, Some((0, 12)));
    let canonical = layerfs_content::decode_bytes_object(&files[0].bytes).unwrap();
    assert_eq!(
        layerfs_content::file::extent_codec::decode_chunk_payload(canonical).unwrap(),
        b"regular-file"
    );
    let metadata = delivered
        .iter()
        .find(|o| {
            layerfs_content::decode_bytes_object(&o.bytes)
                .ok()
                .and_then(|b| layerfs_content::file::extent_codec::decode_chunk_payload(b).ok())
                == Some(b"LFS4CHK\0metadata".as_slice())
        })
        .unwrap();
    assert!(!metadata.is_file_payload());
    assert_eq!(metadata.1.first_span, Some((0, 16)));
}
