//! Compact namespace wire format from the retained structural experiment.
//! Serial inode keys are meaningful only within the namespace's allocation scope.
use super::directory::codec::{
    decode_node_value, exact_value, finish_node, node_count, node_header, node_subtree_bytes,
    node_subtree_count, ordered, put_bytes, take, take_bytes, validate_node_header,
};
use super::inode::{InodeKind, InodeRecordV1};
use crate::{encode_bytes_object, CanonicalName, CoreError, CoreResult, ObjectId};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct InodeSerial(u64);
impl InodeSerial {
    pub fn new(value: u64) -> CoreResult<Self> {
        if value == 0 || value > i64::MAX as u64 {
            return Err(CoreError::InvalidRecord("compact inode serial"));
        }
        Ok(Self(value))
    }
    pub const fn get(self) -> u64 {
        self.0
    }
    /// Logical tree keys are local to a namespace. The namespace root carries
    /// the full allocation scope; comparison across scopes must include it.
    pub fn inode_key(self) -> super::inode::InodeId {
        let mut bytes = [0; 32];
        bytes[24..].copy_from_slice(&self.0.to_be_bytes());
        super::inode::InodeId(bytes)
    }
    pub fn from_inode_key(key: super::inode::InodeId) -> CoreResult<Self> {
        if key.0[..24] != [0; 24] {
            return Err(CoreError::InvalidRecord("compact inode key"));
        }
        Self::read(&key.0[24..])
    }
    fn read(bytes: &[u8]) -> CoreResult<Self> {
        let bytes = bytes.try_into().map_err(|_| CoreError::UnexpectedEof)?;
        Self::new(u64::from_be_bytes(bytes))
    }
}

/// Point lookup follows a strictly decreasing level and authenticates every
/// selected node. No separately stored inode-record object is required.
pub fn inode_lookup<S: crate::object::access::ObjectRead>(
    store: &S,
    root: ObjectId,
    key: InodeSerial,
    counters: &mut super::inode::InodeTableCounters,
) -> CoreResult<Option<InodeRecordV1>> {
    counters.nodes_read = counters
        .nodes_read
        .checked_add(1)
        .ok_or(CoreError::LengthOverflow)?;
    let node = store.with_authenticated_canonical(root, decode_inode)?;
    inode_lookup_from_node(store, node, key, counters)
}

pub(crate) fn is_inode_table<S: crate::object::access::ObjectRead>(
    store: &S,
    root: ObjectId,
) -> CoreResult<bool> {
    store.with_authenticated_canonical(root, |bytes| {
        Ok(crate::decode_bytes_object(bytes)?.starts_with(b"LFS6INT\0"))
    })
}

enum InodeWalk {
    Node {
        id: ObjectId,
        expected: Option<(u8, InodeSerial)>,
    },
    End {
        before: u64,
        count: u64,
    },
}

/// Ordered streaming values with bounded depth/fanout and full subtree-count
/// checks when consumed. The cursor owns no borrowed Store or decoded history.
pub(crate) struct InodeCursor {
    stack: Vec<InodeWalk>,
    leaf: std::vec::IntoIter<(InodeSerial, InodeRecordV1)>,
    count: u64,
    previous: Option<InodeSerial>,
}
impl InodeCursor {
    pub(crate) fn new(root: ObjectId) -> Self {
        Self {
            stack: vec![InodeWalk::Node {
                id: root,
                expected: None,
            }],
            leaf: Vec::new().into_iter(),
            count: 0,
            previous: None,
        }
    }
    pub(crate) fn next<S: crate::object::access::ObjectRead>(
        &mut self,
        store: &S,
        counters: &mut super::inode::InodeTableCounters,
    ) -> CoreResult<Option<(InodeSerial, InodeRecordV1)>> {
        loop {
            if let Some(row) = self.leaf.next() {
                if self.previous.is_some_and(|previous| previous >= row.0) {
                    return Err(CoreError::NonCanonicalOrdering);
                }
                self.previous = Some(row.0);
                self.count = self.count.checked_add(1).ok_or(CoreError::LengthOverflow)?;
                return Ok(Some(row));
            }
            let Some(item) = self.stack.pop() else {
                return Ok(None);
            };
            let (id, expected) = match item {
                InodeWalk::Node { id, expected } => (id, expected),
                InodeWalk::End { before, count } => {
                    if self.count.checked_sub(before) != Some(count) {
                        return Err(CoreError::InvalidRecord("compact inode subtree count"));
                    }
                    continue;
                }
            };
            counters.nodes_read = counters
                .nodes_read
                .checked_add(1)
                .ok_or(CoreError::LengthOverflow)?;
            let node = store.with_authenticated_canonical(id, decode_inode)?;
            let (level, maximum, count) = match &node {
                InodeNode::Leaf(rows) => (
                    0,
                    rows.last().ok_or(CoreError::NonCanonicalPagePartition)?.0,
                    rows.len(),
                ),
                InodeNode::Branch {
                    level, children, ..
                } => (
                    *level,
                    children
                        .last()
                        .ok_or(CoreError::NonCanonicalPagePartition)?
                        .0,
                    children.len(),
                ),
            };
            if expected.is_some_and(|value| value != (level, maximum)) {
                return Err(CoreError::InvalidRecord("compact inode child summary"));
            }
            if expected.is_some() && count < if level == 0 { 50 } else { 64 }
                || level > 0 && count < 2
            {
                return Err(CoreError::NonCanonicalPagePartition);
            }
            match node {
                InodeNode::Leaf(rows) => self.leaf = rows.into_iter(),
                InodeNode::Branch {
                    level,
                    subtree_count,
                    children,
                } => {
                    if self.stack.len() + children.len() + 1 > 32 * 128 {
                        return Err(CoreError::ObjectLimitExceeded);
                    }
                    self.stack.push(InodeWalk::End {
                        before: self.count,
                        count: subtree_count,
                    });
                    self.stack
                        .extend(
                            children
                                .into_iter()
                                .rev()
                                .map(|(maximum, id)| InodeWalk::Node {
                                    id,
                                    expected: Some((level - 1, maximum)),
                                }),
                        );
                }
            }
        }
    }
}

pub(crate) fn inode_lookup_from_node<S: crate::object::access::ObjectRead>(
    store: &S,
    mut node: InodeNode,
    key: InodeSerial,
    counters: &mut super::inode::InodeTableCounters,
) -> CoreResult<Option<InodeRecordV1>> {
    let mut expected = None;
    loop {
        let (level, maximum, count) = match &node {
            InodeNode::Leaf(rows) => (
                0,
                rows.last().ok_or(CoreError::NonCanonicalPagePartition)?.0,
                rows.len(),
            ),
            InodeNode::Branch {
                level, children, ..
            } => (
                *level,
                children
                    .last()
                    .ok_or(CoreError::NonCanonicalPagePartition)?
                    .0,
                children.len(),
            ),
        };
        if let Some((expected_level, expected_maximum)) = expected {
            if level != expected_level || maximum != expected_maximum {
                return Err(CoreError::InvalidRecord("compact inode child summary"));
            }
            if count < if level == 0 { 50 } else { 64 } {
                return Err(CoreError::NonCanonicalPagePartition);
            }
        } else if level > 0 && count < 2 {
            return Err(CoreError::NonCanonicalPagePartition);
        }
        if key > maximum {
            return Ok(None);
        }
        match node {
            InodeNode::Leaf(rows) => {
                return Ok(rows
                    .binary_search_by_key(&key, |row| row.0)
                    .ok()
                    .map(|index| rows[index].1))
            }
            InodeNode::Branch {
                level, children, ..
            } => {
                let index = children.partition_point(|row| row.0 < key);
                let (maximum, child) = children[index];
                expected = Some((
                    level
                        .checked_sub(1)
                        .ok_or(CoreError::MappingDepthExceeded)?,
                    maximum,
                ));
                counters.nodes_read = counters
                    .nodes_read
                    .checked_add(1)
                    .ok_or(CoreError::LengthOverflow)?;
                node = store.with_authenticated_canonical(child, decode_inode)?;
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NamespaceRoot {
    pub profile_id: ObjectId,
    pub scope: ObjectId,
    pub root_inode: InodeSerial,
    pub inode_table: ObjectId,
}

pub fn profile_id() -> ObjectId {
    // A product profile distinct from the retained diagnostic profile. Wire
    // identities include the full scope; 50/100 inode leaves and 64/127 branches
    // use the existing 8-KiB, depth-31 bounded B+ tree mutation engine.
    static PROFILE: std::sync::OnceLock<ObjectId> = std::sync::OnceLock::new();
    *PROFILE.get_or_init(|| ObjectId::for_bytes(b"layerfs/namespace-profile/scoped-inline/v1\0scope32;serial8;inode81;leaf50-100;branch64-127;page8192;depth31;directory-fill2/5"))
}

pub fn scope_for_seed(seed: [u8; 32]) -> ObjectId {
    let mut bytes = b"layerfs/inode-scope/v1\0".to_vec();
    bytes.extend_from_slice(&seed);
    ObjectId::for_bytes(&bytes)
}

pub fn encode_root(root: NamespaceRoot) -> CoreResult<Vec<u8>> {
    let mut value = b"LFS6FSR\0\0\x01\x06\0".to_vec();
    value.extend_from_slice(root.profile_id.as_bytes());
    value.extend_from_slice(root.scope.as_bytes());
    value.extend_from_slice(&root.root_inode.0.to_be_bytes());
    value.extend_from_slice(root.inode_table.as_bytes());
    encode_bytes_object(&value)
}

pub fn decode_root(canonical: &[u8]) -> CoreResult<NamespaceRoot> {
    let value = exact_value(canonical, b"LFS6FSR\0", 116, 6)?;
    Ok(NamespaceRoot {
        profile_id: ObjectId::from_bytes(&value[12..44])?,
        scope: ObjectId::from_bytes(&value[44..76])?,
        root_inode: InodeSerial::read(&value[76..84])?,
        inode_table: ObjectId::from_bytes(&value[84..116])?,
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InodeNode {
    Leaf(Vec<(InodeSerial, InodeRecordV1)>),
    Branch {
        level: u8,
        subtree_count: u64,
        children: Vec<(InodeSerial, ObjectId)>,
    },
}

pub fn encode_inode_value(record: InodeRecordV1) -> [u8; 73] {
    let mut bytes = [0; 73];
    bytes[0] = record.kind as u8;
    bytes[1..9].copy_from_slice(&record.namespace_ref_count.to_be_bytes());
    bytes[9..41].copy_from_slice(record.content_root.as_bytes());
    bytes[41..].copy_from_slice(record.metadata_root.as_bytes());
    bytes
}

pub fn decode_inode_value(bytes: &[u8]) -> CoreResult<InodeRecordV1> {
    if bytes.len() != 73 {
        return Err(CoreError::InvalidRecord("inline inode length"));
    }
    Ok(InodeRecordV1 {
        kind: InodeKind::try_from(bytes[0])?,
        namespace_ref_count: u64::from_be_bytes(bytes[1..9].try_into().unwrap()),
        content_root: ObjectId::from_bytes(&bytes[9..41])?,
        metadata_root: ObjectId::from_bytes(&bytes[41..])?,
    })
}

pub fn encode_inode(node: &InodeNode) -> CoreResult<Vec<u8>> {
    let (role, level, count, total) = match node {
        InodeNode::Leaf(rows) => (7, 0, rows.len(), rows.len() as u64),
        InodeNode::Branch {
            level,
            subtree_count,
            children,
        } => (8, *level, children.len(), *subtree_count),
    };
    validate_node_header(level, count)?;
    if count == 0
        || count > if role == 7 { 100 } else { 127 }
        || (role == 8 && (level == 0 || total < count as u64))
    {
        return Err(CoreError::NonCanonicalPagePartition);
    }
    let mut value = node_header(
        b"LFS6INT\0",
        role,
        level,
        count,
        total,
        total.checked_mul(81).ok_or(CoreError::LengthOverflow)?,
    )?;
    match node {
        InodeNode::Leaf(rows) => {
            if rows.windows(2).any(|pair| pair[0].0 >= pair[1].0) {
                return Err(CoreError::InvalidRecord("compact inode key order"));
            }
            for (key, record) in rows {
                value.extend_from_slice(&key.0.to_be_bytes());
                value.extend_from_slice(&encode_inode_value(*record));
            }
        }
        InodeNode::Branch { children, .. } => {
            if children.windows(2).any(|pair| pair[0].0 >= pair[1].0) {
                return Err(CoreError::InvalidRecord("compact inode key order"));
            }
            for (key, child) in children {
                value.extend_from_slice(&key.0.to_be_bytes());
                value.extend_from_slice(child.as_bytes());
            }
        }
    }
    finish_node(value)
}

pub fn decode_inode(canonical: &[u8]) -> CoreResult<InodeNode> {
    let value = decode_node_value(canonical, b"LFS6INT\0")?;
    let count = node_count(value);
    let total = node_subtree_count(value);
    let level = value[11];
    let node = match value[10] {
        7 if level == 0 && total == count as u64 => {
            if count == 0 || count > 100 || value.len() != 31 + count * 81 {
                return Err(CoreError::InvalidRecord("compact inode leaf length"));
            }
            let mut rows = Vec::with_capacity(count);
            for row in value[31..].chunks_exact(81) {
                rows.push((
                    InodeSerial::read(&row[..8])?,
                    decode_inode_value(&row[8..])?,
                ));
            }
            InodeNode::Leaf(rows)
        }
        8 if level > 0 => {
            if count == 0 || count > 127 || value.len() != 31 + count * 40 {
                return Err(CoreError::InvalidRecord("compact inode branch length"));
            }
            let mut children = Vec::with_capacity(count);
            for row in value[31..].chunks_exact(40) {
                children.push((
                    InodeSerial::read(&row[..8])?,
                    ObjectId::from_bytes(&row[8..])?,
                ));
            }
            InodeNode::Branch {
                level,
                subtree_count: total,
                children,
            }
        }
        _ => return Err(CoreError::InvalidRecord("compact inode role/level")),
    };
    if node_subtree_bytes(value) != total.checked_mul(81).ok_or(CoreError::LengthOverflow)?
        || encode_inode(&node)? != canonical
    {
        return Err(CoreError::InvalidRecord("compact inode canonical form"));
    }
    Ok(node)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DirectoryNode {
    Leaf(Vec<(CanonicalName, InodeSerial)>),
    Branch {
        level: u8,
        subtree_count: u64,
        subtree_bytes: u64,
        children: Vec<(CanonicalName, ObjectId)>,
    },
}

pub fn encode_directory(node: &DirectoryNode) -> CoreResult<Vec<u8>> {
    let (role, level, count, total, bytes) = match node {
        DirectoryNode::Leaf(rows) => (
            1,
            0,
            rows.len(),
            rows.len() as u64,
            rows.iter()
                .map(|(name, _)| 10 + name.as_bytes().len() as u64)
                .sum(),
        ),
        DirectoryNode::Branch {
            level,
            subtree_count,
            subtree_bytes,
            children,
        } => (2, *level, children.len(), *subtree_count, *subtree_bytes),
    };
    validate_node_header(level, count)?;
    if role == 2
        && (level == 0
            || count == 0
            || total < count as u64
            || bytes < total.checked_mul(11).ok_or(CoreError::LengthOverflow)?)
    {
        return Err(CoreError::InvalidRecord("compact directory summary"));
    }
    let mut value = node_header(b"LFS6NSP\0", role, level, count, total, bytes)?;
    match node {
        DirectoryNode::Leaf(rows) => {
            ordered(rows.iter().map(|(name, _)| name.as_bytes()))?;
            for (name, serial) in rows {
                put_bytes(&mut value, name.as_bytes())?;
                value.extend_from_slice(&serial.0.to_be_bytes());
            }
        }
        DirectoryNode::Branch { children, .. } => {
            ordered(children.iter().map(|(name, _)| name.as_bytes()))?;
            for (name, id) in children {
                put_bytes(&mut value, name.as_bytes())?;
                value.extend_from_slice(id.as_bytes());
            }
        }
    }
    finish_node(value)
}

pub fn decode_directory(canonical: &[u8]) -> CoreResult<DirectoryNode> {
    let value = decode_node_value(canonical, b"LFS6NSP\0")?;
    let count = node_count(value);
    let mut cursor = 31;
    // Bound allocations by the minimum encoded entry size before reserving.
    if count > (value.len() - 31) / 11 {
        return Err(CoreError::UnexpectedEof);
    }
    let node = match (value[10], value[11]) {
        (1, 0) => {
            let mut rows = Vec::with_capacity(count);
            for _ in 0..count {
                let name = CanonicalName::from_bytes(take_bytes(value, &mut cursor, 255)?)?;
                rows.push((name, InodeSerial::read(take(value, &mut cursor, 8)?)?));
            }
            DirectoryNode::Leaf(rows)
        }
        (2, level) if level > 0 => {
            let mut children = Vec::with_capacity(count);
            for _ in 0..count {
                let name = CanonicalName::from_bytes(take_bytes(value, &mut cursor, 255)?)?;
                children.push((name, ObjectId::from_bytes(take(value, &mut cursor, 32)?)?));
            }
            DirectoryNode::Branch {
                level,
                subtree_count: node_subtree_count(value),
                subtree_bytes: node_subtree_bytes(value),
                children,
            }
        }
        _ => return Err(CoreError::InvalidRecord("compact directory role/level")),
    };
    if cursor != value.len() || encode_directory(&node)? != canonical {
        return Err(CoreError::InvalidRecord("compact directory canonical form"));
    }
    Ok(node)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn retained_offline_metadata_is_byte_identical() {
        let root = include_bytes!("../../tests/fixtures/compact/root.bin");
        assert_eq!(encode_root(decode_root(root).unwrap()).unwrap(), root);
        assert_eq!(
            crate::object::references::referenced_objects(root).unwrap(),
            vec![decode_root(root).unwrap().inode_table]
        );
        for bytes in [
            include_bytes!("../../tests/fixtures/compact/inode-leaf.bin").as_slice(),
            include_bytes!("../../tests/fixtures/compact/inode-branch.bin").as_slice(),
        ] {
            let node = decode_inode(bytes).unwrap();
            assert_eq!(encode_inode(&node).unwrap(), bytes);
            let expected = match node {
                InodeNode::Leaf(rows) => rows.len() * 2,
                InodeNode::Branch { children, .. } => children.len(),
            };
            assert_eq!(
                crate::object::references::referenced_objects(bytes)
                    .unwrap()
                    .len(),
                expected
            );
        }
        for bytes in [
            include_bytes!("../../tests/fixtures/compact/directory-leaf.bin").as_slice(),
            include_bytes!("../../tests/fixtures/compact/directory-branch.bin").as_slice(),
        ] {
            let node = decode_directory(bytes).unwrap();
            assert_eq!(encode_directory(&node).unwrap(), bytes);
            let expected = match node {
                DirectoryNode::Leaf(_) => 0,
                DirectoryNode::Branch { children, .. } => children.len(),
            };
            assert_eq!(
                crate::object::references::referenced_objects(bytes)
                    .unwrap()
                    .len(),
                expected
            );
        }
    }
    #[test]
    fn compact_namespace_roundtrip_and_invalid_inputs() {
        let id = ObjectId::for_bytes(b"content");
        let serial = InodeSerial::new(1).unwrap();
        let root = NamespaceRoot {
            profile_id: id,
            scope: id,
            root_inode: serial,
            inode_table: id,
        };
        assert_eq!(encode_root(root).unwrap().len(), 129);
        assert_eq!(decode_root(&encode_root(root).unwrap()).unwrap(), root);
        let record = InodeRecordV1 {
            kind: InodeKind::RegularFile,
            namespace_ref_count: 2,
            content_root: id,
            metadata_root: id,
        };
        let leaf = InodeNode::Leaf(
            (1..=100)
                .map(|n| (InodeSerial::new(n).unwrap(), record))
                .collect(),
        );
        let bytes = encode_inode(&leaf).unwrap();
        assert_eq!(bytes.len(), 8144);
        assert_eq!(decode_inode(&bytes).unwrap(), leaf);
        let branch = InodeNode::Branch {
            level: 1,
            subtree_count: 100,
            children: vec![(InodeSerial::new(100).unwrap(), id)],
        };
        assert_eq!(
            decode_inode(&encode_inode(&branch).unwrap()).unwrap(),
            branch
        );
        let names = DirectoryNode::Leaf(vec![
            (CanonicalName::from_bytes(b"a").unwrap(), serial),
            (CanonicalName::from_bytes(b"b").unwrap(), serial),
        ]);
        assert_eq!(
            decode_directory(&encode_directory(&names).unwrap()).unwrap(),
            names
        );
        assert!(InodeSerial::new(0).is_err());
        assert!(InodeSerial::new(u64::MAX).is_err());
        assert!(encode_inode(&InodeNode::Leaf(vec![(serial, record), (serial, record)])).is_err());
        for n in 0..bytes.len() {
            assert!(decode_inode(&bytes[..n]).is_err());
        }
        let mut invalid = bytes.clone();
        invalid.push(0);
        assert!(decode_inode(&invalid).is_err());
        let mut invalid = bytes;
        invalid[13 + 23] ^= 1;
        assert!(decode_inode(&invalid).is_err());
    }
}
