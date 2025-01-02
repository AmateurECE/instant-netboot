use super::ReadOnlyFilesystem;
use crate::fs;
use std::path::{Path, PathBuf};

async fn make_archive() -> anyhow::Result<async_tar::Archive<async_std::io::Cursor<Vec<u8>>>> {
    let foo_contents = "Hello, world!\n";
    let mut builder = async_tar::Builder::new(async_std::io::Cursor::new(Vec::new()));
    builder
        .append_data(
            &mut async_tar::Header::new_gnu(),
            Path::new("foo.txt"),
            async_std::io::Cursor::new(foo_contents),
        )
        .await?;
    builder.finish().await?;
    Ok(async_tar::Archive::new(builder.into_inner().await?))
}

#[async_std::test]
async fn root_listing() {
    let filesystem = ReadOnlyFilesystem::new(make_archive().await.unwrap());
    let root_id = filesystem.root_id();
    let expected = vec![fs::File {
        parent: root_id,
        attributes: fs::Metadata {
            file_type: fs::FileType::Regular,
        },
        link_name: None,
        path: PathBuf::from("foo.txt"),
    }];
    let contents = filesystem.readdir(&root_id);
    assert!(contents.eq(expected.iter()));
}

#[test]
fn no_duplicated_file_ids() {
    todo!()
}
