use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use zip::ZipArchive;

#[derive(Serialize, Deserialize)]
pub struct Chapter {
    pub label: String,
    pub href: String,
}

#[derive(Serialize, Deserialize)]
pub struct EpubMetadata {
    pub title: String,
    pub author: Option<String>,
    pub publisher: Option<String>,
    pub language: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct EpubBook {
    pub metadata: EpubMetadata,
    pub chapters: Vec<Chapter>,
    pub spine: Vec<String>,
}

#[wasm_bindgen]
pub struct EpubParser {
    book: Option<EpubBook>,
}

#[wasm_bindgen]
impl EpubParser {
    #[wasm_bindgen(constructor)]
    pub fn new() -> EpubParser {
        EpubParser { book: None }
    }

    /// Parse ePub from bytes (blazing fast using Rust!)
    #[wasm_bindgen]
    pub fn parse(&mut self, data: &[u8]) -> Result<(), JsValue> {
        let cursor = Cursor::new(data);
        let mut archive = ZipArchive::new(cursor)
            .map_err(|e| JsValue::from_str(&format!("Failed to open ZIP: {}", e)))?;

        let mut title = String::from("Unknown Title");
        let mut author: Option<String> = None;
        let mut publisher: Option<String> = None;
        let mut language: Option<String> = None;
        let mut chapters: Vec<Chapter> = Vec::new();
        let mut spine: Vec<String> = Vec::new();

        // Find content.opf
        let container_xml = self.read_file(&mut archive, "META-INF/container.xml")?;
        let content_opf_path = self.extract_content_opf_path(&container_xml)?;

        // Parse content.opf
        let content_opf = self.read_file(&mut archive, &content_opf_path)?;
        self.parse_content_opf(&content_opf, &mut title, &mut author, &mut publisher, &mut language, &mut spine)?;

        // Build chapters
        for item in &spine {
            chapters.push(Chapter {
                label: format!("Chapter {}", chapters.len() + 1),
                href: item.clone(),
            });
        }

        self.book = Some(EpubBook {
            metadata: EpubMetadata {
                title,
                author,
                publisher,
                language,
            },
            chapters,
            spine,
        });

        Ok(())
    }

    #[wasm_bindgen]
    pub fn get_metadata(&self) -> Result<JsValue, JsValue> {
        match &self.book {
            Some(book) => {
                serde_wasm_bindgen::to_value(&book.metadata)
                    .map_err(|e| JsValue::from_str(&format!("Error: {}", e)))
            }
            None => Err(JsValue::from_str("Not parsed")),
        }
    }

    #[wasm_bindgen]
    pub fn get_toc(&self) -> Result<JsValue, JsValue> {
        match &self.book {
            Some(book) => {
                serde_wasm_bindgen::to_value(&book.chapters)
                    .map_err(|e| JsValue::from_str(&format!("Error: {}", e)))
            }
            None => Err(JsValue::from_str("Not parsed")),
        }
    }

    #[wasm_bindgen]
    pub fn get_chapter(&self, index: usize, data: &[u8]) -> Result<String, JsValue> {
        match &self.book {
            Some(book) => {
                if index >= book.chapters.len() {
                    return Err(JsValue::from_str("Out of bounds"));
                }

                let cursor = Cursor::new(data);
                let mut archive = ZipArchive::new(cursor)
                    .map_err(|e| JsValue::from_str(&format!("ZIP error: {}", e)))?;

                let href = &book.chapters[index].href;
                self.read_file(&mut archive, href)
            }
            None => Err(JsValue::from_str("Not parsed")),
        }
    }

    fn read_file(&self, archive: &mut ZipArchive<Cursor<&[u8]>>, path: &str) -> Result<String, JsValue> {
        use std::io::Read;

        let mut file = archive
            .by_name(path)
            .map_err(|e| JsValue::from_str(&format!("File not found: {}: {}", path, e)))?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| JsValue::from_str(&format!("Read error: {}", e)))?;

        Ok(contents)
    }

    fn extract_content_opf_path(&self, container_xml: &str) -> Result<String, JsValue> {
        if let Some(start) = container_xml.find("full-path=\"") {
            if let Some(end) = container_xml[start + 11..].find("\"") {
                let path = &container_xml[start + 11..start + 11 + end];
                return Ok(path.to_string());
            }
        }
        Ok("OEBPS/content.opf".to_string())
    }

    fn parse_content_opf(
        &self,
        content: &str,
        title: &mut String,
        author: &mut Option<String>,
        publisher: &mut Option<String>,
        language: &mut Option<String>,
        spine: &mut Vec<String>,
    ) -> Result<(), JsValue> {
        if let Some(start) = content.find("<dc:title>") {
            if let Some(end) = content[start..].find("</dc:title>") {
                *title = content[start + 10..start + end].to_string();
            }
        }

        if let Some(start) = content.find("<dc:creator>") {
            if let Some(end) = content[start..].find("</dc:creator>") {
                *author = Some(content[start + 12..start + end].to_string());
            }
        }

        if let Some(start) = content.find("<dc:publisher>") {
            if let Some(end) = content[start..].find("</dc:publisher>") {
                *publisher = Some(content[start + 14..start + end].to_string());
            }
        }

        if let Some(start) = content.find("<dc:language>") {
            if let Some(end) = content[start..].find("</dc:language>") {
                *language = Some(content[start + 13..start + end].to_string());
            }
        }

        let mut in_spine = false;
        for line in content.lines() {
            if line.contains("<spine") {
                in_spine = true;
                continue;
            }
            if line.contains("</spine>") {
                break;
            }
            if in_spine && line.contains("<itemref") {
                if let Some(start) = line.find("idref=\"") {
                    if let Some(end) = line[start + 7..].find("\"") {
                        let idref = &line[start + 7..start + 7 + end];
                        spine.push(format!("{}.xhtml", idref));
                    }
                }
            }
        }

        Ok(())
    }
}
