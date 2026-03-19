use crate::mhl::types::HashList;
use crate::util;
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Writer;
use std::io::Write;

const MHL_NAMESPACE: &str = "urn:ASC:MHL:v2.0";

/// Write a HashList to MHL XML format.
pub fn write_mhl<W: Write>(hashlist: &HashList, output: W) -> Result<(), String> {
    let mut writer = Writer::new_with_indent(output, b' ', 2);

    // XML declaration
    writer
        .write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))
        .map_err(|e| format!("XML write error: {}", e))?;

    // <hashlist>
    let mut hashlist_elem = BytesStart::new("hashlist");
    hashlist_elem.push_attribute(("xmlns", MHL_NAMESPACE));
    hashlist_elem.push_attribute(("version", "2.0"));
    writer
        .write_event(Event::Start(hashlist_elem))
        .map_err(|e| format!("XML write error: {}", e))?;

    // <creatorinfo>
    write_creator_info(&mut writer, &hashlist.creator_info)?;

    // <processinfo>
    write_process_info(&mut writer, &hashlist.process_info)?;

    // <hashes>
    writer
        .write_event(Event::Start(BytesStart::new("hashes")))
        .map_err(|e| format!("XML write error: {}", e))?;

    // File hashes
    for entry in &hashlist.hashes {
        writer
            .write_event(Event::Start(BytesStart::new("hash")))
            .map_err(|e| format!("XML write error: {}", e))?;

        // <path size="..." creationdate="..." lastmodificationdate="...">relative/path</path>
        let mut path_elem = BytesStart::new("path");
        path_elem.push_attribute(("size", entry.size.to_string().as_str()));
        if let Some(ref dt) = entry.creation_date {
            path_elem.push_attribute(("creationdate", util::format_datetime(dt).as_str()));
        }
        if let Some(ref dt) = entry.last_modification_date {
            path_elem
                .push_attribute(("lastmodificationdate", util::format_datetime(dt).as_str()));
        }
        writer
            .write_event(Event::Start(path_elem))
            .map_err(|e| format!("XML write error: {}", e))?;
        writer
            .write_event(Event::Text(BytesText::new(&entry.path)))
            .map_err(|e| format!("XML write error: {}", e))?;
        writer
            .write_event(Event::End(BytesEnd::new("path")))
            .map_err(|e| format!("XML write error: {}", e))?;

        // Hash values: <xxh64 action="original" hashdate="...">hex</xxh64>
        for hv in &entry.hashes {
            let tag = hv.algorithm.xml_tag();
            let mut hash_elem = BytesStart::new(tag);
            hash_elem.push_attribute(("action", hv.action.as_str()));
            hash_elem
                .push_attribute(("hashdate", util::format_datetime(&hv.hash_date).as_str()));
            writer
                .write_event(Event::Start(hash_elem))
                .map_err(|e| format!("XML write error: {}", e))?;
            writer
                .write_event(Event::Text(BytesText::new(&hv.value)))
                .map_err(|e| format!("XML write error: {}", e))?;
            writer
                .write_event(Event::End(BytesEnd::new(tag)))
                .map_err(|e| format!("XML write error: {}", e))?;
        }

        writer
            .write_event(Event::End(BytesEnd::new("hash")))
            .map_err(|e| format!("XML write error: {}", e))?;
    }

    // Directory hashes
    for dir_entry in &hashlist.directory_hashes {
        writer
            .write_event(Event::Start(BytesStart::new("directoryhash")))
            .map_err(|e| format!("XML write error: {}", e))?;

        // <path>
        let mut path_elem = BytesStart::new("path");
        if let Some(ref dt) = dir_entry.creation_date {
            path_elem.push_attribute(("creationdate", util::format_datetime(dt).as_str()));
        }
        if let Some(ref dt) = dir_entry.last_modification_date {
            path_elem
                .push_attribute(("lastmodificationdate", util::format_datetime(dt).as_str()));
        }
        writer
            .write_event(Event::Start(path_elem))
            .map_err(|e| format!("XML write error: {}", e))?;
        writer
            .write_event(Event::Text(BytesText::new(&dir_entry.path)))
            .map_err(|e| format!("XML write error: {}", e))?;
        writer
            .write_event(Event::End(BytesEnd::new("path")))
            .map_err(|e| format!("XML write error: {}", e))?;

        // Content hash
        let tag = dir_entry.content_hash.algorithm.xml_tag();
        let content_tag = format!("content_{}", tag);
        let mut ch_elem = BytesStart::new(&content_tag);
        ch_elem.push_attribute(("action", dir_entry.content_hash.action.as_str()));
        ch_elem.push_attribute((
            "hashdate",
            util::format_datetime(&dir_entry.content_hash.hash_date).as_str(),
        ));
        writer
            .write_event(Event::Start(ch_elem))
            .map_err(|e| format!("XML write error: {}", e))?;
        writer
            .write_event(Event::Text(BytesText::new(&dir_entry.content_hash.value)))
            .map_err(|e| format!("XML write error: {}", e))?;
        writer
            .write_event(Event::End(BytesEnd::new(&content_tag)))
            .map_err(|e| format!("XML write error: {}", e))?;

        // Structure hash
        let structure_tag = format!("structure_{}", tag);
        let mut sh_elem = BytesStart::new(&structure_tag);
        sh_elem.push_attribute(("action", dir_entry.structure_hash.action.as_str()));
        sh_elem.push_attribute((
            "hashdate",
            util::format_datetime(&dir_entry.structure_hash.hash_date).as_str(),
        ));
        writer
            .write_event(Event::Start(sh_elem))
            .map_err(|e| format!("XML write error: {}", e))?;
        writer
            .write_event(Event::Text(BytesText::new(
                &dir_entry.structure_hash.value,
            )))
            .map_err(|e| format!("XML write error: {}", e))?;
        writer
            .write_event(Event::End(BytesEnd::new(&structure_tag)))
            .map_err(|e| format!("XML write error: {}", e))?;

        writer
            .write_event(Event::End(BytesEnd::new("directoryhash")))
            .map_err(|e| format!("XML write error: {}", e))?;
    }

    // </hashes>
    writer
        .write_event(Event::End(BytesEnd::new("hashes")))
        .map_err(|e| format!("XML write error: {}", e))?;

    // </hashlist>
    writer
        .write_event(Event::End(BytesEnd::new("hashlist")))
        .map_err(|e| format!("XML write error: {}", e))?;

    Ok(())
}

fn write_creator_info<W: Write>(
    writer: &mut Writer<W>,
    info: &crate::mhl::types::CreatorInfo,
) -> Result<(), String> {
    writer
        .write_event(Event::Start(BytesStart::new("creatorinfo")))
        .map_err(|e| format!("XML write error: {}", e))?;

    write_text_element(writer, "creationdate", &util::format_datetime(&info.creation_date))?;
    write_text_element(writer, "hostname", &info.hostname)?;

    // <tool>
    writer
        .write_event(Event::Start(BytesStart::new("tool")))
        .map_err(|e| format!("XML write error: {}", e))?;
    write_text_element(writer, "name", &info.tool_name)?;
    write_text_element(writer, "version", &info.tool_version)?;
    writer
        .write_event(Event::End(BytesEnd::new("tool")))
        .map_err(|e| format!("XML write error: {}", e))?;

    // Author info
    if let Some(ref author) = info.author {
        writer
            .write_event(Event::Start(BytesStart::new("author")))
            .map_err(|e| format!("XML write error: {}", e))?;
        write_text_element(writer, "name", &author.name)?;
        if let Some(ref email) = author.email {
            write_text_element(writer, "email", email)?;
        }
        if let Some(ref phone) = author.phone {
            write_text_element(writer, "phone", phone)?;
        }
        if let Some(ref role) = author.role {
            write_text_element(writer, "role", role)?;
        }
        writer
            .write_event(Event::End(BytesEnd::new("author")))
            .map_err(|e| format!("XML write error: {}", e))?;
    }

    if let Some(ref location) = info.location {
        write_text_element(writer, "location", location)?;
    }
    if let Some(ref comment) = info.comment {
        write_text_element(writer, "comment", comment)?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("creatorinfo")))
        .map_err(|e| format!("XML write error: {}", e))?;

    Ok(())
}

fn write_process_info<W: Write>(
    writer: &mut Writer<W>,
    info: &crate::mhl::types::ProcessInfo,
) -> Result<(), String> {
    writer
        .write_event(Event::Start(BytesStart::new("processinfo")))
        .map_err(|e| format!("XML write error: {}", e))?;

    write_text_element(writer, "process", info.process.as_str())?;

    if let Some(ref root_hash) = info.root_hash {
        let tag = root_hash.algorithm.xml_tag();
        let elem = BytesStart::new("roothash");
        writer
            .write_event(Event::Start(elem))
            .map_err(|e| format!("XML write error: {}", e))?;
        write_text_element(writer, tag, &root_hash.value)?;
        writer
            .write_event(Event::End(BytesEnd::new("roothash")))
            .map_err(|e| format!("XML write error: {}", e))?;
    }

    if !info.ignore_patterns.is_empty() {
        writer
            .write_event(Event::Start(BytesStart::new("ignore")))
            .map_err(|e| format!("XML write error: {}", e))?;
        for pattern in &info.ignore_patterns {
            write_text_element(writer, "pattern", pattern)?;
        }
        writer
            .write_event(Event::End(BytesEnd::new("ignore")))
            .map_err(|e| format!("XML write error: {}", e))?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("processinfo")))
        .map_err(|e| format!("XML write error: {}", e))?;

    Ok(())
}

fn write_text_element<W: Write>(
    writer: &mut Writer<W>,
    tag: &str,
    text: &str,
) -> Result<(), String> {
    writer
        .write_event(Event::Start(BytesStart::new(tag)))
        .map_err(|e| format!("XML write error: {}", e))?;
    writer
        .write_event(Event::Text(BytesText::new(text)))
        .map_err(|e| format!("XML write error: {}", e))?;
    writer
        .write_event(Event::End(BytesEnd::new(tag)))
        .map_err(|e| format!("XML write error: {}", e))?;
    Ok(())
}
