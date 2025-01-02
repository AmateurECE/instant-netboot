//! Read only filesystem implementation using tar files

use std::io;
use std::{collections::HashMap, path::Path};

use async_std::stream::StreamExt;
use futures::AsyncRead;

use crate::fs;

/// Utility function. Produces the index used by the filesystem.
fn make_index<Reader, FileId>(archive: async_tar::Archive<Reader>) -> HashMap<FileId, fs::File>
where
    Reader: async_std::io::Read + Unpin,
{
    todo!()
}

/// Utility higher-order function. Returns a closure that returns Some(e) if the entry e matches
/// the provided path. Logs using tracing::debug if an error is encountered.
fn entry_matches_path<'a, R>(
    requested_path: &'a async_std::path::Path,
) -> impl FnMut(Result<async_tar::Entry<R>, io::Error>) -> Option<async_tar::Entry<R>> + 'a
where
    R: async_std::io::Read + Unpin,
{
    move |e| {
        let Ok(entry) = e else {
            tracing::debug!("Error while reading entry: {:?}", e);
            return None;
        };
        let Ok(path) = entry.path() else {
            tracing::debug!("Error while reading path from entry header: {:?}", entry);
            return None;
        };
        if path == requested_path {
            Some(entry)
        } else {
            None
        }
    }
}

pub struct ReadOnlyFilesystem<Reader>
where
    Reader: AsyncRead + Unpin,
{
    index: HashMap<fs::FileId, fs::File>,
    archive: async_tar::Archive<Reader>,
}

impl<Reader> ReadOnlyFilesystem<Reader>
where
    Reader: AsyncRead + Unpin,
{
    // TODO: Put this in the trait as a default impl and put the actual number in a FileIdGenerator
    // or something.
    pub fn root_id(&self) -> fs::FileId {
        1u64
    }

    // TODO: How do we get file IDs into here?
    pub fn new(archive: async_tar::Archive<Reader>) -> Self {
        let index = make_index(archive.clone());
        Self { index, archive }
    }

    pub fn getattr(&self, id: &fs::FileId) -> Result<&fs::Metadata, fs::FileError> {
        self.index
            .get(&id)
            .map(|f| &f.attributes)
            .ok_or(fs::FileError::NotFound)
    }

    pub async fn read(&self, id: &fs::FileId) -> Result<impl AsyncRead, fs::FileError> {
        // TODO: Is this performant enough?
        let requested_path: &async_std::path::Path = self
            .index
            .get(id)
            .ok_or(fs::FileError::NotFound)?
            .path
            .as_path()
            .into();

        // FIXME: Archive is just an Arc<Mutex<_>>. Cloning it satisfies the borrow checker, but it
        // probably doesn't have the desired effect--it may still consume the archive. We may need
        // to get more creative.
        let archive = self.archive.clone();
        let entry = archive
            .entries()
            .map_err(fs::FileError::Io)?
            .find_map(entry_matches_path(requested_path.into()))
            .await
            .ok_or(fs::FileError::NotFound)?;
        Ok(entry)
    }

    pub fn readdir<'a>(&'a self, id: &'a fs::FileId) -> impl Iterator<Item = &'a fs::File> + 'a {
        // TODO: Right now, this will return an empty iterator if id doesn't exist, or if it's not
        // a directory. If we implement a trait for attributes, we can be smarter here.
        self.index.values().filter(|f| f.parent == *id)
    }

    pub fn readlink(&self, id: &fs::FileId) -> Result<Option<&Path>, fs::FileError> {
        let file = self.index.get(&id).ok_or(fs::FileError::NotFound)?;
        let link = file.link_name.as_ref().map(AsRef::<Path>::as_ref);
        Ok(link)
    }
}
