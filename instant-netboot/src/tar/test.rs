use super::ReadOnlyFilesystem;
use crate::fs;
use async_std::stream::StreamExt;
use std::path::{Path, PathBuf};

async fn make_files_1(root_id: fs::FileId) -> Vec<fs::File> {
    vec![fs::File {
        parent: Some(root_id),
        attributes: fs::Metadata {
            file_type: fs::FileType::Regular,
        },
        link_name: None,
        path: PathBuf::from("foo.txt"),
    }]
}

async fn make_test_archive_1() -> anyhow::Result<async_tar::Archive<async_std::io::Cursor<Vec<u8>>>>
{
    let mut builder = async_tar::Builder::new(Vec::new());

    let foo_contents = "Hello, world!\n";
    let mut foo_header = async_tar::Header::new_ustar();
    foo_header.set_path("foo.txt")?;
    foo_header.set_size(foo_contents.len().try_into().unwrap());
    foo_header.set_cksum();
    builder.append(&foo_header, foo_contents.as_bytes()).await?;

    Ok(async_tar::Archive::new(async_std::io::Cursor::new(
        builder.into_inner().await?,
    )))
}

#[async_std::test]
async fn readdir_root_listing() {
    let filesystem = ReadOnlyFilesystem::new(make_test_archive_1().await.unwrap())
        .await
        .unwrap();
    let root_id = filesystem.root_id();
    let expected = make_files_1(root_id).await;
    let contents = filesystem.readdir(&root_id);
    assert_eq!(expected, contents);
}

async fn make_files_2_root(root_id: fs::FileId) -> Vec<fs::File> {
    vec![
        fs::File {
            parent: Some(root_id),
            attributes: fs::Metadata {
                file_type: fs::FileType::Directory,
            },
            link_name: None,
            path: PathBuf::from("usr"),
        },
        fs::File {
            parent: Some(root_id),
            attributes: fs::Metadata {
                file_type: fs::FileType::Link,
            },
            link_name: Some(PathBuf::from("usr/bin")),
            path: PathBuf::from("bin"),
        },
    ]
}

async fn make_files_2_usr(root_id: fs::FileId) -> Vec<fs::File> {
    vec![fs::File {
        parent: Some(root_id + 2),
        attributes: fs::Metadata {
            file_type: fs::FileType::Directory,
        },
        link_name: None,
        path: PathBuf::from("usr/bin"),
    }]
}

async fn make_test_archive_2() -> anyhow::Result<async_tar::Archive<async_std::io::Cursor<Vec<u8>>>>
{
    let mut builder = async_tar::Builder::new(Vec::new());

    let mut header = async_tar::Header::new_ustar();
    header.set_path("bin")?;
    header.set_entry_type(async_tar::EntryType::Link);
    header.set_link_name(Path::new("usr/bin"))?;
    header.set_size(0);
    header.set_cksum();
    builder.append(&header, [].as_slice()).await?;

    let mut header = async_tar::Header::new_ustar();
    header.set_path("usr")?;
    header.set_entry_type(async_tar::EntryType::Directory);
    header.set_size(0);
    header.set_cksum();
    builder.append(&header, [].as_slice()).await?;

    let mut header = async_tar::Header::new_ustar();
    header.set_path("usr/bin")?;
    header.set_entry_type(async_tar::EntryType::Directory);
    header.set_size(0);
    header.set_cksum();
    builder.append(&header, [].as_slice()).await?;

    Ok(async_tar::Archive::new(async_std::io::Cursor::new(
        builder.into_inner().await?,
    )))
}

#[async_std::test]
async fn multiple_root_entries() {
    let filesystem = ReadOnlyFilesystem::new(make_test_archive_2().await.unwrap())
        .await
        .unwrap();
    let root_id = filesystem.root_id();
    let expected = make_files_2_root(root_id).await;
    let contents = filesystem.readdir(&root_id);
    assert_eq!(expected, contents);
}
