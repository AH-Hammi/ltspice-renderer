use std::collections::HashMap;

const OLE2_MAGIC: [u8; 8] = [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];
const FREESECT: u32 = 0xFFFFFFFF;
const ENDOFCHAIN: u32 = 0xFFFFFFFE;

pub enum DownloadError {
    Http(String),
    Parse(String),
    Ole2(String),
    Zip(String),
}

impl std::fmt::Display for DownloadError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DownloadError::Http(s) => write!(f, "HTTP error: {s}"),
            DownloadError::Parse(s) => write!(f, "Parse error: {s}"),
            DownloadError::Ole2(s) => write!(f, "OLE2 error: {s}"),
            DownloadError::Zip(s) => write!(f, "ZIP error: {s}"),
        }
    }
}

impl std::fmt::Debug for DownloadError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        <Self as std::fmt::Display>::fmt(self, f)
    }
}

fn utf16le_to_string(bytes: &[u8]) -> String {
    let u16s: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .collect();
    String::from_utf16_lossy(&u16s)
}

struct Ole2Reader {
    data: Vec<u8>,
    sector_size: usize,
    fat: Vec<u32>,
    dir_entries: Vec<Ole2DirEntry>,
}

#[derive(Debug, Clone)]
struct Ole2DirEntry {
    name: String,
    object_type: u8,
    start_sector: u32,
    stream_size: u64,
}

impl Ole2Reader {
    fn new(data: Vec<u8>) -> Result<Self, DownloadError> {
        if data.len() < 512 {
            return Err(DownloadError::Ole2("data too short for OLE2 header".into()));
        }
        if data[0..8] != OLE2_MAGIC {
            return Err(DownloadError::Ole2("bad OLE2 magic".into()));
        }
        let sector_size_power = u16::from_le_bytes([data[14], data[15]]);
        let sector_size = 1 << sector_size_power as usize;
        let first_dir_sector = u32::from_le_bytes([data[28], data[29], data[30], data[31]]);

        let mut difat: Vec<u32> = Vec::new();
        for i in 0..109 {
            let offset = 52 + i * 4;
            difat.push(u32::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]));
        }

        let fat = Self::build_fat(&data, sector_size, &difat)?;
        let dir_entries = Self::read_directory(&data, sector_size, &fat, first_dir_sector)?;

        Ok(Ole2Reader { data, sector_size, fat, dir_entries })
    }

    fn build_fat(data: &[u8], sector_size: usize, difat: &[u32]) -> Result<Vec<u32>, DownloadError> {
        let entries_per_sector = sector_size / 4;
        let mut fat: Vec<u32> = Vec::new();
        for &sector in difat {
            if sector == FREESECT {
                break;
            }
            let offset = 512 + sector as usize * sector_size;
            if offset + sector_size > data.len() {
                return Err(DownloadError::Ole2("FAT sector out of bounds".into()));
            }
            for i in 0..entries_per_sector {
                let entry_offset = offset + i * 4;
                if entry_offset + 4 > data.len() {
                    break;
                }
                fat.push(u32::from_le_bytes([
                    data[entry_offset],
                    data[entry_offset + 1],
                    data[entry_offset + 2],
                    data[entry_offset + 3],
                ]));
            }
        }
        Ok(fat)
    }

    fn read_directory(
        data: &[u8],
        sector_size: usize,
        fat: &[u32],
        first_dir_sector: u32,
    ) -> Result<Vec<Ole2DirEntry>, DownloadError> {
        let chain = Self::follow_chain(fat, first_dir_sector);
        let mut raw = Vec::new();
        for &s in &chain {
            let offset = 512 + s as usize * sector_size;
            if offset + sector_size > data.len() {
                return Err(DownloadError::Ole2("dir sector out of bounds".into()));
            }
            raw.extend_from_slice(&data[offset..offset + sector_size]);
        }
        let mut entries = Vec::new();
        for i in 0..raw.len() / 128 {
            let off = i * 128;
            let name_size = u16::from_le_bytes([raw[off + 64], raw[off + 65]]) as usize;
            let name = if name_size > 2 && name_size <= 64 {
                utf16le_to_string(&raw[off..off + name_size - 2])
            } else {
                String::new()
            };
            let object_type = raw[off + 66];
            if object_type == 0 {
                continue;
            }
            let start_sector = u32::from_le_bytes([
                raw[off + 116], raw[off + 117], raw[off + 118], raw[off + 119],
            ]);
            let stream_size = u64::from_le_bytes([
                raw[off + 120], raw[off + 121], raw[off + 122], raw[off + 123],
                raw[off + 124], raw[off + 125], raw[off + 126], raw[off + 127],
            ]);
            entries.push(Ole2DirEntry {
                name,
                object_type,
                start_sector,
                stream_size,
            });
        }
        Ok(entries)
    }

    fn follow_chain(fat: &[u32], start: u32) -> Vec<u32> {
        let mut chain = Vec::new();
        let mut current = start;
        loop {
            if current == ENDOFCHAIN || current == FREESECT {
                break;
            }
            chain.push(current);
            current = fat[current as usize];
        }
        chain
    }

    fn read_stream_raw(&self, start_sector: u32, stream_size: u64) -> Vec<u8> {
        if stream_size == 0 {
            return Vec::new();
        }
        let chain = Self::follow_chain(&self.fat, start_sector);
        let mut result = Vec::with_capacity(stream_size as usize);
        for &s in &chain {
            let offset = 512 + s as usize * self.sector_size;
            let end = std::cmp::min(offset + self.sector_size, self.data.len());
            result.extend_from_slice(&self.data[offset..end]);
        }
        result.truncate(stream_size as usize);
        result
    }

    fn find_entry(&self, name: &str) -> Option<&Ole2DirEntry> {
        let lower = name.to_lowercase();
        self.dir_entries.iter().find(|e| e.name.to_lowercase() == lower)
    }
}

fn parse_msi_url(text: &str) -> Result<String, DownloadError> {
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(url) = trimmed.strip_prefix("URL = ") {
            return Ok(url.trim().to_string());
        }
    }
    Err(DownloadError::Parse("URL = line not found in updates.txt".into()))
}

fn extract_lib_zip(msi_data: &[u8]) -> Result<Vec<u8>, DownloadError> {
    let ole = Ole2Reader::new(msi_data.to_vec())?;
    let entry = ole.find_entry("lib.zip")
        .ok_or_else(|| DownloadError::Ole2("lib.zip not found in MSI".into()))?;
    Ok(ole.read_stream_raw(entry.start_sector, entry.stream_size))
}

fn extract_zip(zip_data: &[u8]) -> Result<HashMap<String, Vec<u8>>, DownloadError> {
    let mut reader = std::io::Cursor::new(zip_data);
    let mut archive = zip::ZipArchive::new(&mut reader)
        .map_err(|e| DownloadError::Zip(e.to_string()))?;
    let mut files = HashMap::new();
    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| DownloadError::Zip(e.to_string()))?;
        let name = file.name().to_string();
        let mut data = Vec::new();
        std::io::Read::read_to_end(&mut file, &mut data)
            .map_err(|e| DownloadError::Zip(e.to_string()))?;
        files.insert(name, data);
    }
    Ok(files)
}

pub async fn download_library() -> Result<HashMap<String, Vec<u8>>, DownloadError> {
    let client = reqwest::Client::new();

    let updates_text = client
        .get("https://ltspice.analog.com/download/updates.txt")
        .send()
        .await
        .map_err(|e| DownloadError::Http(e.to_string()))?
        .text()
        .await
        .map_err(|e| DownloadError::Http(e.to_string()))?;

    let msi_url = parse_msi_url(&updates_text)?;

    let msi_data = client
        .get(&msi_url)
        .send()
        .await
        .map_err(|e| DownloadError::Http(e.to_string()))?
        .bytes()
        .await
        .map_err(|e| DownloadError::Http(e.to_string()))?
        .to_vec();

    let lib_zip = extract_lib_zip(&msi_data)?;

    extract_zip(&lib_zip)
}