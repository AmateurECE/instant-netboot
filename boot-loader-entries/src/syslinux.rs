use core::fmt;
use std::path::PathBuf;

use crate::{uapi, BootFile};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ConfigurationConversionError;

/// A KERNEL-LIKE Directive, specifying the image to boot
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Kernel {
    Kernel(PathBuf),
}

impl fmt::Display for Kernel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Kernel::Kernel(image) => write!(f, "KERNEL {}", image.display()),
        }
    }
}

impl BootFile for Kernel {
    fn boot_file(&self) -> Option<&std::path::Path> {
        match self {
            Kernel::Kernel(image) => Some(image),
        }
    }
}

/// Directives that configure a boot label
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum LabelDirective {
    /// An initial ramdisk
    Initrd(PathBuf),
    /// A device tree blob
    Fdt(PathBuf),
    // TODO: The Append option is actually a "dual-purpose" directive, not a "label directive"
    /// Kernel configuration options
    Append(Vec<String>),
}

impl BootFile for LabelDirective {
    fn boot_file(&self) -> Option<&std::path::Path> {
        match self {
            LabelDirective::Initrd(initrd) => Some(initrd),
            LabelDirective::Fdt(fdt) => Some(fdt),
            LabelDirective::Append(_) => None,
        }
    }
}

impl TryFrom<uapi::EntryKey> for LabelDirective {
    type Error = ConfigurationConversionError;
    fn try_from(value: uapi::EntryKey) -> Result<Self, Self::Error> {
        match value {
            uapi::EntryKey::Title(_) => Err(ConfigurationConversionError),
            uapi::EntryKey::Linux(_) => Err(ConfigurationConversionError),
            uapi::EntryKey::Devicetree(fdt) => Ok(LabelDirective::Fdt(fdt)),
            uapi::EntryKey::Options(options) => Ok(LabelDirective::Append(options)),
        }
    }
}

impl fmt::Display for LabelDirective {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LabelDirective::Initrd(initrd) => write!(f, "INITRD {}", initrd.display()),
            LabelDirective::Fdt(fdt) => write!(f, "FDT {}", fdt.display()),
            LabelDirective::Append(options) => write!(f, "APPEND {}", options.join(" ")),
        }
    }
}

/// A label clause
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Label {
    pub name: String,
    pub kernel: Kernel,
    pub directives: Vec<LabelDirective>,
}

impl fmt::Display for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LABEL {}\n", self.name)?;
        self.kernel.fmt(f)?;
        write!(f, "\n")?;
        for directive in &self.directives {
            directive.fmt(f)?;
            write!(f, "\n")?;
        }
        Ok(())
    }
}

/// A Syslinux configuration
pub struct Configuration {
    pub labels: Vec<Label>,
}

// TODO: We probably care more about morphing Configurations than individual BootEntry/Label(s).
impl TryFrom<uapi::BootEntry> for Label {
    type Error = ConfigurationConversionError;
    fn try_from(value: uapi::BootEntry) -> Result<Self, Self::Error> {
        let mut name: Option<String> = None;
        let mut kernel: Option<PathBuf> = None;
        let directives = value
            .keys
            .into_iter()
            // TODO: The use of filter_map in TryFrom<BootEntry> will discard all invalid entries.
            // Is that really what we want?
            .filter_map(|key| match key {
                uapi::EntryKey::Title(title) => {
                    name = Some(title);
                    None
                }
                uapi::EntryKey::Linux(linux) => {
                    kernel = Some(linux);
                    None
                }
                key => key.try_into().ok(),
            })
            .collect::<Vec<LabelDirective>>();

        let name = name.ok_or(ConfigurationConversionError)?;
        let kernel = Kernel::Kernel(kernel.ok_or(ConfigurationConversionError)?);
        Ok(Label {
            name,
            kernel,
            directives,
        })
    }
}

#[cfg(test)]
mod test {
    use super::{Kernel, Label};
    use crate::uapi;

    #[test]
    fn valid_syslinux_from_uapi() {
        let configuration = uapi::BootEntry {
            keys: vec![
                uapi::EntryKey::Title("Fedora 19 (Rawhide)".to_string()),
                uapi::EntryKey::Linux("/Image".into()),
            ],
        };

        let result: Label = configuration.try_into().unwrap();
        assert_eq!(
            result,
            Label {
                name: "Fedora 19 (Rawhide)".to_string(),
                kernel: Kernel::Kernel("/Image".into()),
                directives: vec![],
            }
        );
    }
}
