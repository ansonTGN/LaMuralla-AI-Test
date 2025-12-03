use std::io::Read;
use lopdf::Document;
use xml::reader::{EventReader, XmlEvent};
use crate::domain::errors::AppError;

pub fn parse_text_from_bytes(filename: &str, bytes: &[u8]) -> Result<String, AppError> {
    let extension = std::path::Path::new(filename)
        .extension()
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or("")
        .to_lowercase();

    match extension.as_str() {
        "pdf" => extract_text_from_pdf(bytes),
        "docx" => extract_text_from_docx(bytes),
        "txt" | "md" | "json" | "csv" => {
            String::from_utf8(bytes.to_vec())
                .map_err(|e| AppError::ParseError(format!("Invalid UTF-8: {}", e)))
        },
        _ => Err(AppError::ValidationError(format!("Unsupported file format: .{}", extension))),
    }
}

fn extract_text_from_pdf(bytes: &[u8]) -> Result<String, AppError> {
    // Cargar PDF desde memoria
    let doc = Document::load_mem(bytes)
        .map_err(|e| AppError::ParseError(format!("Failed to load PDF: {}", e)))?;
    
    // Extraer texto página por página
    let mut text = String::new();
    for page_num in doc.get_pages().keys() {
        let content = doc.extract_text(&[*page_num])
            .unwrap_or_default();
        text.push_str(&content);
        text.push('\n');
    }
    
    if text.trim().is_empty() {
        return Err(AppError::ParseError("PDF appears to be empty or scanned images".to_string()));
    }
    
    Ok(text)
}

fn extract_text_from_docx(bytes: &[u8]) -> Result<String, AppError> {
    let cursor = std::io::Cursor::new(bytes);
    let mut zip = zip::ZipArchive::new(cursor)
        .map_err(|e| AppError::ParseError(format!("Failed to read DOCX zip: {}", e)))?;

    // El texto en DOCX está en word/document.xml
    let mut xml_file = zip.by_name("word/document.xml")
        .map_err(|_| AppError::ParseError("Invalid DOCX: missing document.xml".to_string()))?;

    let mut xml_content = String::new();
    xml_file.read_to_string(&mut xml_content)
        .map_err(|e| AppError::ParseError(format!("Failed to read XML: {}", e)))?;

    // Parsear XML simple para sacar el texto
    let parser = EventReader::from_str(&xml_content);
    let mut text = String::new();

    for e in parser {
        match e {
            Ok(XmlEvent::Characters(s)) => {
                text.push_str(&s);
                text.push(' ');
            },
            Err(e) => return Err(AppError::ParseError(format!("XML Error: {}", e))),
            _ => {}
        }
    }

    Ok(text)
}