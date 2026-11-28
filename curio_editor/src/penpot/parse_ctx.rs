use serde_json::Value;

/// What the shape walk reads from the file, and what it collects on the way.
pub struct ParseCtx<'a> {
    /// The page's flat `objects` map, shape id -> shape.
    pub objects: &'a Value,
    /// The file's `components` map, component id -> component.
    pub components: Option<&'a Value>,
    pub file_id: &'a str,
    /// `(shape id, shape name, media object id, mime type)` per visible image to download.
    pub images: Vec<(String, String, String, String)>,
    pub warnings: Vec<String>,
}
