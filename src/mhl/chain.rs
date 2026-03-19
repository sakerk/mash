use crate::hashing::algorithms::c4_hash_bytes;
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Writer;
use std::fs;
use std::path::Path;

const CHAIN_FILENAME: &str = "ascmhl_chain.xml";

/// A record of an MHL file in the chain.
#[derive(Debug, Clone)]
pub struct ChainEntry {
    pub filename: String,
    pub c4_hash: String,
}

/// Read the existing chain file, if any.
pub fn read_chain(ascmhl_dir: &Path) -> Vec<ChainEntry> {
    let chain_path = ascmhl_dir.join(CHAIN_FILENAME);
    if !chain_path.exists() {
        return Vec::new();
    }

    let content = match fs::read_to_string(&chain_path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let mut entries = Vec::new();
    let mut reader = quick_xml::Reader::from_str(&content);
    let mut in_hashentry = false;
    let mut current_filename = String::new();
    let mut current_c4 = String::new();
    let mut current_tag = String::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if tag == "hashentry" {
                    in_hashentry = true;
                    current_filename.clear();
                    current_c4.clear();
                }
                current_tag = tag;
            }
            Ok(Event::Text(ref e)) => {
                if in_hashentry {
                    let text = e.unescape().unwrap_or_default().to_string();
                    match current_tag.as_str() {
                        "filename" => current_filename = text,
                        "c4" => current_c4 = text,
                        _ => {}
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if tag == "hashentry" && in_hashentry {
                    entries.push(ChainEntry {
                        filename: current_filename.clone(),
                        c4_hash: current_c4.clone(),
                    });
                    in_hashentry = false;
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
    }

    entries
}

/// Write the chain file with all entries.
pub fn write_chain(ascmhl_dir: &Path, entries: &[ChainEntry]) -> Result<(), String> {
    let chain_path = ascmhl_dir.join(CHAIN_FILENAME);
    let file = fs::File::create(&chain_path)
        .map_err(|e| format!("Failed to create chain file: {}", e))?;

    let mut writer = Writer::new_with_indent(file, b' ', 2);

    writer
        .write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))
        .map_err(|e| format!("XML write error: {}", e))?;

    let mut root = BytesStart::new("ascmhlchain");
    root.push_attribute(("xmlns", "urn:ASC:MHL:v2.0"));
    writer
        .write_event(Event::Start(root))
        .map_err(|e| format!("XML write error: {}", e))?;

    for entry in entries {
        writer
            .write_event(Event::Start(BytesStart::new("hashentry")))
            .map_err(|e| format!("XML write error: {}", e))?;

        // <filename>
        writer
            .write_event(Event::Start(BytesStart::new("filename")))
            .map_err(|e| format!("XML write error: {}", e))?;
        writer
            .write_event(Event::Text(BytesText::new(&entry.filename)))
            .map_err(|e| format!("XML write error: {}", e))?;
        writer
            .write_event(Event::End(BytesEnd::new("filename")))
            .map_err(|e| format!("XML write error: {}", e))?;

        // <c4>
        writer
            .write_event(Event::Start(BytesStart::new("c4")))
            .map_err(|e| format!("XML write error: {}", e))?;
        writer
            .write_event(Event::Text(BytesText::new(&entry.c4_hash)))
            .map_err(|e| format!("XML write error: {}", e))?;
        writer
            .write_event(Event::End(BytesEnd::new("c4")))
            .map_err(|e| format!("XML write error: {}", e))?;

        writer
            .write_event(Event::End(BytesEnd::new("hashentry")))
            .map_err(|e| format!("XML write error: {}", e))?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("ascmhlchain")))
        .map_err(|e| format!("XML write error: {}", e))?;

    Ok(())
}

/// Compute the C4 hash of an MHL file.
pub fn compute_mhl_c4_hash(mhl_path: &Path) -> Result<String, String> {
    let data = fs::read(mhl_path).map_err(|e| format!("Failed to read MHL file: {}", e))?;
    Ok(c4_hash_bytes(&data))
}

/// Add a new MHL file to the chain.
pub fn add_to_chain(ascmhl_dir: &Path, mhl_filename: &str, mhl_path: &Path) -> Result<(), String> {
    let mut entries = read_chain(ascmhl_dir);
    let c4_hash = compute_mhl_c4_hash(mhl_path)?;

    entries.push(ChainEntry {
        filename: mhl_filename.to_string(),
        c4_hash,
    });

    write_chain(ascmhl_dir, &entries)
}
