#[derive(Debug, Default)]
pub struct IndexService;

impl IndexService {
    pub fn sync_embeddings(&self) -> &'static str {
        "index service stub"
    }
}
