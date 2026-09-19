//! A zip written here: stored entries, no compression, UTF-8 names. The
//! browser writes the accountant's download the same way (`ui/finances/src/
//! lib/zip.ts`), so the bundle a mail carries is byte-for-byte the shape the
//! download has. PDFs do not compress; a dependency would buy nothing.

/// One file inside the zip.
#[derive(Debug, Clone)]
pub struct Entry {
    /// The path inside the zip, `/`-separated.
    pub name: String,
    /// The bytes.
    pub bytes: Vec<u8>,
}

/// Local-header and central-directory flag: names are UTF-8.
const UTF8: u16 = 0x0800;
/// Version 2.0: stored entries, nothing newer.
const VERSION: u16 = 20;

/// The zip's bytes, entries in the order given.
///
/// # Errors
/// An empty or duplicate name; a name with a backslash or a leading slash;
/// more than 65,535 entries or a size the format cannot carry.
pub fn write(entries: &[Entry]) -> Result<Vec<u8>, String> {
    if entries.len() > usize::from(u16::MAX) {
        return Err("too many files for one zip".into());
    }
    let mut seen = std::collections::HashSet::new();
    for e in entries {
        if e.name.is_empty() || e.name.starts_with('/') || e.name.contains('\\') {
            return Err(format!("not a zip entry name: {:?}", e.name));
        }
        if !seen.insert(e.name.as_str()) {
            return Err(format!("the zip would hold {:?} twice", e.name));
        }
    }
    let (time, date) = dos_now();
    let mut out = Vec::new();
    let mut central = Vec::new();
    for e in entries {
        let name = e.name.as_bytes();
        let size = u32::try_from(e.bytes.len())
            .map_err(|_| format!("{} is too large for a zip", e.name))?;
        let name_len =
            u16::try_from(name.len()).map_err(|_| format!("{} is too long a name", e.name))?;
        let crc = crc32fast::hash(&e.bytes);
        let offset = u32::try_from(out.len()).map_err(|_| "the zip is too large".to_owned())?;

        out.extend_from_slice(&0x0403_4b50_u32.to_le_bytes());
        out.extend_from_slice(&VERSION.to_le_bytes());
        out.extend_from_slice(&UTF8.to_le_bytes());
        out.extend_from_slice(&0_u16.to_le_bytes()); // stored
        out.extend_from_slice(&time.to_le_bytes());
        out.extend_from_slice(&date.to_le_bytes());
        out.extend_from_slice(&crc.to_le_bytes());
        out.extend_from_slice(&size.to_le_bytes());
        out.extend_from_slice(&size.to_le_bytes());
        out.extend_from_slice(&name_len.to_le_bytes());
        out.extend_from_slice(&0_u16.to_le_bytes()); // extra
        out.extend_from_slice(name);
        out.extend_from_slice(&e.bytes);

        central.extend_from_slice(&0x0201_4b50_u32.to_le_bytes());
        central.extend_from_slice(&VERSION.to_le_bytes()); // made by
        central.extend_from_slice(&VERSION.to_le_bytes()); // needed
        central.extend_from_slice(&UTF8.to_le_bytes());
        central.extend_from_slice(&0_u16.to_le_bytes()); // stored
        central.extend_from_slice(&time.to_le_bytes());
        central.extend_from_slice(&date.to_le_bytes());
        central.extend_from_slice(&crc.to_le_bytes());
        central.extend_from_slice(&size.to_le_bytes());
        central.extend_from_slice(&size.to_le_bytes());
        central.extend_from_slice(&name_len.to_le_bytes());
        central.extend_from_slice(&0_u16.to_le_bytes()); // extra
        central.extend_from_slice(&0_u16.to_le_bytes()); // comment
        central.extend_from_slice(&0_u16.to_le_bytes()); // disk
        central.extend_from_slice(&0_u16.to_le_bytes()); // internal attributes
        central.extend_from_slice(&0_u32.to_le_bytes()); // external attributes
        central.extend_from_slice(&offset.to_le_bytes());
        central.extend_from_slice(name);
    }
    let central_offset = u32::try_from(out.len()).map_err(|_| "the zip is too large".to_owned())?;
    let central_size =
        u32::try_from(central.len()).map_err(|_| "the zip is too large".to_owned())?;
    #[allow(clippy::cast_possible_truncation)] // checked above
    let count = entries.len() as u16;
    out.extend_from_slice(&central);
    out.extend_from_slice(&0x0605_4b50_u32.to_le_bytes());
    out.extend_from_slice(&0_u16.to_le_bytes()); // this disk
    out.extend_from_slice(&0_u16.to_le_bytes()); // directory's disk
    out.extend_from_slice(&count.to_le_bytes());
    out.extend_from_slice(&count.to_le_bytes());
    out.extend_from_slice(&central_size.to_le_bytes());
    out.extend_from_slice(&central_offset.to_le_bytes());
    out.extend_from_slice(&0_u16.to_le_bytes()); // comment
    Ok(out)
}

/// MS-DOS time and date fields for now, as the format wants them.
fn dos_now() -> (u16, u16) {
    use chrono::{Datelike, Timelike};
    let now = chrono::Local::now();
    let year = u16::try_from((now.year() - 1980).max(0))
        .unwrap_or(0)
        .min(127);
    #[allow(clippy::cast_possible_truncation)] // all fields are small by construction
    let (month, day, hour, minute, second) = (
        now.month() as u16,
        now.day() as u16,
        now.hour() as u16,
        now.minute() as u16,
        now.second() as u16,
    );
    (
        (hour << 11) | (minute << 5) | (second >> 1),
        (year << 9) | (month << 5) | day,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Reads the zip back with nothing but the format: every local header,
    /// then the directory, and the counts agree.
    fn entries_of(zip: &[u8]) -> Vec<(String, Vec<u8>, u32)> {
        let end = zip.len() - 22;
        assert_eq!(&zip[end..end + 4], &0x0605_4b50_u32.to_le_bytes());
        let count = u16::from_le_bytes([zip[end + 8], zip[end + 9]]);
        let dir_size = u32::from_le_bytes(zip[end + 12..end + 16].try_into().unwrap()) as usize;
        let dir_at = u32::from_le_bytes(zip[end + 16..end + 20].try_into().unwrap()) as usize;
        assert_eq!(dir_at + dir_size, end);
        let mut out = Vec::new();
        let mut at = dir_at;
        for _ in 0..count {
            assert_eq!(&zip[at..at + 4], &0x0201_4b50_u32.to_le_bytes());
            let crc = u32::from_le_bytes(zip[at + 16..at + 20].try_into().unwrap());
            let size = u32::from_le_bytes(zip[at + 20..at + 24].try_into().unwrap()) as usize;
            let name_len = u16::from_le_bytes([zip[at + 28], zip[at + 29]]) as usize;
            let offset = u32::from_le_bytes(zip[at + 42..at + 46].try_into().unwrap()) as usize;
            let name = String::from_utf8(zip[at + 46..at + 46 + name_len].to_vec()).unwrap();
            assert_eq!(&zip[offset..offset + 4], &0x0403_4b50_u32.to_le_bytes());
            let data_at = offset + 30 + name_len;
            out.push((name, zip[data_at..data_at + size].to_vec(), crc));
            at += 46 + name_len;
        }
        out
    }

    #[test]
    fn a_zip_holds_its_entries_in_order_with_their_crcs() {
        let zip = write(&[
            Entry {
                name: "PROCITAJ.txt".into(),
                bytes: b"hello".to_vec(),
            },
            Entry {
                name: "racuni/2026-08-06_Hetzner_55_00_EUR.pdf".into(),
                bytes: b"%PDF-1.4".to_vec(),
            },
            Entry {
                name: "racuni/prazno.pdf".into(),
                bytes: vec![],
            },
        ])
        .unwrap();
        let got = entries_of(&zip);
        assert_eq!(got.len(), 3);
        assert_eq!(got[0].0, "PROCITAJ.txt");
        assert_eq!(got[0].1, b"hello");
        assert_eq!(got[0].2, 0x3610_a686, "the CRC-32 of \"hello\"");
        assert_eq!(got[1].0, "racuni/2026-08-06_Hetzner_55_00_EUR.pdf");
        assert_eq!(got[1].1, b"%PDF-1.4");
        assert_eq!(got[2].1, b"");
        assert_eq!(got[2].2, 0);
        assert!(
            write(&[]).unwrap().len() == 22,
            "an empty zip is the end record alone"
        );
    }

    #[test]
    fn a_bad_name_is_refused_before_anything_is_written() {
        let e = |name: &str| Entry {
            name: name.into(),
            bytes: vec![],
        };
        assert!(
            write(&[e("")])
                .unwrap_err()
                .contains("not a zip entry name")
        );
        assert!(
            write(&[e("/abs")])
                .unwrap_err()
                .contains("not a zip entry name")
        );
        assert!(
            write(&[e("a\\b")])
                .unwrap_err()
                .contains("not a zip entry name")
        );
        assert!(write(&[e("x"), e("x")]).unwrap_err().contains("twice"));
    }
}
