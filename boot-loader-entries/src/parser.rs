use nom::{
    bytes::complete::{tag_no_case, take_till1},
    character::complete::{line_ending, space1},
    combinator::opt,
    multi::{many1, separated_list0},
    sequence::{separated_pair, terminated},
    IResult, InputTakeAtPosition, Parser,
};

use crate::{BootEntry, EntryKey};

/// Matches a line ending
fn is_line_ending(byte: char) -> bool {
    byte == '\r' || byte == '\n'
}

/// Matches a sequence of non-space characters
fn non_space(input: &str) -> IResult<&str, &str> {
    input.split_at_position_complete(char::is_whitespace)
}

/// This entry attribute is a single path
fn single_string_argument(input: &str) -> IResult<&str, &str> {
    let (rest, argument) = take_till1(is_line_ending)(input)?;
    Ok((rest, argument))
}

/// This entry attribute is a space-separated list of tokens
fn space_separated_list(input: &str) -> IResult<&str, Vec<&str>> {
    separated_list0(space1, non_space)(input)
}

/// Parse a "linux" menu entry key and its associated value
fn linux(input: &str) -> IResult<&str, EntryKey> {
    let (input, (_, path)) =
        separated_pair(tag_no_case("linux"), space1, single_string_argument)(input)?;
    Ok((input, EntryKey::Linux(path.into())))
}

/// Parse a "devicetree" menu entry key and its associated value
fn devicetree(input: &str) -> IResult<&str, EntryKey> {
    let (input, (_, path)) =
        separated_pair(tag_no_case("devicetree"), space1, single_string_argument)(input)?;
    Ok((input, EntryKey::Devicetree(path.into())))
}

/// Parse an "options" menu entry key and its associated value
fn options(input: &str) -> IResult<&str, EntryKey> {
    let (input, (_, options)) =
        separated_pair(tag_no_case("options"), space1, space_separated_list)(input)?;
    Ok((
        input,
        EntryKey::Options(options.into_iter().map(|o| o.to_string()).collect()),
    ))
}

/// Parse a "title" meny entry key and its associated value
fn title(input: &str) -> IResult<&str, EntryKey> {
    let (input, (_, title)) =
        separated_pair(tag_no_case("title"), space1, single_string_argument)(input)?;
    Ok((input, EntryKey::Title(title.to_string())))
}

pub fn entry_key(input: &str) -> IResult<&str, EntryKey> {
    linux.or(devicetree).or(options).or(title).parse(input)
}

pub fn boot_entry(input: &str) -> IResult<&str, BootEntry> {
    let (input, keys) = terminated(
        separated_list0(many1(line_ending), entry_key),
        opt(line_ending),
    )(input)?;
    Ok((input, BootEntry { keys }))
}

#[cfg(test)]
mod test {
    use super::*;
    use std::cell::LazyCell;

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
    fn options_entry() {
        let (_, entry) =
            entry_key("options root=UUID=6d3376e4-fc93-4509-95ec-a21d68011da2 quiet").unwrap();
        assert_eq!(
            entry,
            EntryKey::Options(
                vec!["root=UUID=6d3376e4-fc93-4509-95ec-a21d68011da2", "quiet"]
                    .into_iter()
                    .map(|s| s.to_string())
                    .collect()
            )
        );
    }

    #[test]
    fn title_entry() {
        let (_, entry) = entry_key("title Fedora 19 (Rawhide)").unwrap();
        assert_eq!(entry, EntryKey::Title("Fedora 19 (Rawhide)".to_string()));
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
    fn multiple_newlines() {
        let (_, entry) = boot_entry("linux /Image\n\ndevicetree /boot.dtb\n").unwrap();
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
    fn multiple_crlf_newlines() {
        let (_, entry) = boot_entry("linux /Image\r\n\r\ndevicetree /boot.dtb\r\n").unwrap();
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

    const COMPLETE_TEST: LazyCell<Vec<String>> = LazyCell::new(|| {
        vec![
            "title Fedora 19 (Rawhide)".to_string(),
            "linux /Image".to_string(),
            "devicetree /boot.dtb".to_string(),
            "options root=UUID=6d3376e4-fc93-4509-95ec-a21d68011da2 quiet".to_string(),
        ]
    });

    const COMPLETE_RESULT: LazyCell<BootEntry> = LazyCell::new(|| BootEntry {
        keys: vec![
            EntryKey::Title("Fedora 19 (Rawhide)".into()),
            EntryKey::Linux("/Image".into()),
            EntryKey::Devicetree("/boot.dtb".into()),
            EntryKey::Options(
                vec!["root=UUID=6d3376e4-fc93-4509-95ec-a21d68011da2", "quiet"]
                    .into_iter()
                    .map(|s| s.to_string())
                    .collect(),
            ),
        ],
    });

    #[test]
    fn complete() {
        let (_, entry) = boot_entry(&COMPLETE_TEST.join("\n")).unwrap();
        assert_eq!(entry, *COMPLETE_RESULT);
    }

    #[test]
    fn complete_with_crlf() {
        let (_, entry) = boot_entry(&COMPLETE_TEST.join("\r\n")).unwrap();
        assert_eq!(entry, *COMPLETE_RESULT);
    }
}
