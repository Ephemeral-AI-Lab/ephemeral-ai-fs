use crate::{Attr, Data, Error, FileData, LiveWorkspace, NodeId, Result};
use layerfs_content::tree::inode::InodeId;
use layerfs_content::ObjectId;

impl LiveWorkspace {
    pub fn validate_checkpoint_record(&self, id: NodeId, inode: InodeId, attr: Attr) -> Result<()> {
        let node = self
            .nodes
            .get(&id)
            .ok_or(Error::Integrity("checkpoint node"))?;
        if (node.paths.is_empty() && node.links == 0)
            || self.attr(id)? != attr
            || node.canonical.is_some_and(|old| old != inode)
            || self
                .canonical_nodes
                .get(&inode)
                .is_some_and(|old| *old != id)
        {
            return Err(Error::Integrity("checkpoint presentation"));
        }
        if matches!(&node.data, Data::File(FileData::Edited { .. }))
            && !self.edited_nodes.contains(&id)
        {
            return Err(Error::Integrity("spool descriptor"));
        }
        Ok(())
    }

    /// The caller validates the complete bounded checkpoint journal before the
    /// first install, and retains it for exact retry after any partial failure.
    pub fn install_checkpoint_record(
        &mut self,
        id: NodeId,
        inode: InodeId,
        content: ObjectId,
        attr: Attr,
    ) -> Result<()> {
        self.validate_checkpoint_record(id, inode, attr)?;
        let node = self
            .nodes
            .get_mut(&id)
            .ok_or(Error::Integrity("checkpoint node"))?;
        match &mut node.data {
            Data::File(data) => {
                *data = FileData::Base {
                    root: layerfs_content::file::content::FileContentRoot(content),
                    len: attr.size,
                }
            }
            Data::Directory(directory) => {
                directory.base = Some(layerfs_content::tree::directory::DirectoryStateRoot(
                    content,
                ));
                directory.changes.clear();
            }
            Data::Symlink(_) => {}
        }
        self.edited_nodes.remove(&id);
        node.canonical = Some(inode);
        self.canonical_nodes.insert(inode, id);
        Ok(())
    }

    pub fn finish_checkpoint(&mut self, root: ObjectId) -> Result<()> {
        self.spool_bytes = 0;
        self.inline_bytes = 0;
        self.piece_allocation_bytes = 0;
        for id in &self.dirty {
            if let Some(node) = self
                .nodes
                .get_mut(id)
                .filter(|node| node.paths.is_empty() && node.links == 0)
            {
                if let Some(inode) = node.canonical.take() {
                    self.canonical_nodes.remove(&inode);
                }
            }
        }
        for id in &self.edited_nodes {
            let Data::File(FileData::Edited {
                spool_high_water,
                pieces,
                ..
            }) = &self.nodes[id].data
            else {
                return Err(Error::Integrity("checkpoint retained spool"));
            };
            self.spool_bytes = self.spool_bytes.saturating_add(*spool_high_water);
            self.inline_bytes = self.inline_bytes.saturating_add(pieces.inline_len());
            self.piece_allocation_bytes = self
                .piece_allocation_bytes
                .saturating_add(pieces.logical_allocation_charge()?);
        }
        self.base_root = root;
        self.known_names.clear();
        self.spool_bytes_peak = self.spool_bytes;
        self.mutation_generation = 0;
        self.mutation_paths.clear();
        self.dirty.clear();
        Ok(())
    }
}
