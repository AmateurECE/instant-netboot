# Boot Loader Specification

The `boot-loader-entries` crate in this repository provides types and parsing
logic to work with boot loader entries of two varieties:

1. Entries in the UAPI Group's [boot loader specification][1] format.
2. Entries in the [Syslinux][2] format.

This crate is used to parse the boot loader entry, provided by the user, that
configures PXE clients.

[1]: https://uapi-group.org/specifications/specs/boot_loader_specification/
[2]: https://wiki.syslinux.org/wiki/index.php?title=Config#LABEL
