use std::path::Path;

use nom::{
    bytes::complete::{tag_no_case, take_till1},
    character::{
        complete::{newline, space1},
        is_newline,
    },
    combinator::opt,
    multi::separated_list0,
    sequence::{separated_pair, terminated},
    IResult, Parser,
};

use crate::{BootEntry, EntryKey};

/// Adapts a Fn(u8) to a Fn(char)
fn with_byte(inner: impl Fn(u8) -> bool) -> impl Fn(char) -> bool {
    move |c| match u8::try_from(c) {
        Ok(b) => inner(b),
        Err(_) => false,
    }
}

/// This entry attribute is a single path
fn single_path_argument(input: &str) -> IResult<&str, &Path> {
    let (rest, path) = take_till1(with_byte(is_newline))(input)?;
    Ok((rest, Path::new(path)))
}

/// Parse a "linux" menu entry key and its associated value
fn linux(input: &str) -> IResult<&str, EntryKey> {
    let (input, (_, path)) =
        separated_pair(tag_no_case("linux"), space1, single_path_argument)(input)?;
    Ok((input, EntryKey::Linux(path.into())))
}

/// Parse a "devicetree" menu entry key and its associated value
fn devicetree(input: &str) -> IResult<&str, EntryKey> {
    let (input, (_, path)) =
        separated_pair(tag_no_case("devicetree"), space1, single_path_argument)(input)?;
    Ok((input, EntryKey::Devicetree(path.into())))
}

pub fn entry_key(input: &str) -> IResult<&str, EntryKey> {
    linux.or(devicetree).parse(input)
}

pub fn boot_entry(input: &str) -> IResult<&str, BootEntry> {
    let (input, keys) = terminated(separated_list0(newline, entry_key), opt(newline))(input)?;
    Ok((input, BootEntry { keys }))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn linux_entry() {
        let (_, entry) = entry_key("linux /Image").unwrap();
        assert_eq!(entry, EntryKey::Linux("/Image".into()));
    }

    #[test]
    fn devicetree_entry() {
        let (_, entry) = entry_key("devicetree /boot.dtb").unwrap();
        assert_eq!(entry, EntryKey::Devicetree("/boot.dtb".into()));
    }

    #[test]
    fn single_erroneous_entry() {
        let (input, entry) = boot_entry("foo /bar\n").unwrap();
        assert_eq!(input, "foo /bar\n");
        assert_eq!(entry, BootEntry { keys: Vec::new() });
    }

    #[test]
    fn single_line_entry_no_newline() {
        let (_, entry) = boot_entry("linux /Image").unwrap();
        assert_eq!(
            entry,
            BootEntry {
                keys: vec![EntryKey::Linux("/Image".into())]
            }
        );
    }

    #[test]
    fn single_line_entry_with_newline() {
        let (_, entry) = boot_entry("linux /Image\n").unwrap();
        assert_eq!(
            entry,
            BootEntry {
                keys: vec![EntryKey::Linux("/Image".into())]
            },
        );
    }

    #[test]
    fn two_line_entry() {
        let (_, entry) = boot_entry("linux /Image\ndevicetree /boot.dtb\n").unwrap();
        assert_eq!(
            entry,
            BootEntry {
                keys: vec![
                    EntryKey::Linux("/Image".into()),
                    EntryKey::Devicetree("/boot.dtb".into()),
                ],
            },
        );
    }

    #[test]
    fn two_line_entry_no_newline() {
        let (_, entry) = boot_entry("linux /Image\ndevicetree /boot.dtb").unwrap();
        assert_eq!(
            entry,
            BootEntry {
                keys: vec![
                    EntryKey::Linux("/Image".into()),
                    EntryKey::Devicetree("/boot.dtb".into()),
                ],
            },
        );
    }

    #[test]
    fn two_line_typo() {
        let (rest, entry) = boot_entry("linux /Image\ndevisetree /boot.dtb\n").unwrap();
        assert_eq!(rest, "devisetree /boot.dtb\n");
        assert_eq!(
            entry,
            BootEntry {
                keys: vec![EntryKey::Linux("/Image".into())]
            }
        );
    }
}
