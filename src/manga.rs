use crate::{Chapter, Client};
use anyhow::{Context, Result as AnyResult};
use mangadex_api_types_rust::Language;
use uuid::Uuid;

/// A series.
pub struct Manga {
    /// Manga title.
    title: String,
    /// Chapter list.
    chapters: Vec<Chapter>,
}

impl Manga {
    pub async fn new(
        client: &Client,
        lang: Language,
        manga_id: Uuid,
        offset: u16,
    ) -> AnyResult<Self> {
        let manga = client
            .get()
            .await
            .manga()
            .get()
            .manga_id(manga_id)
            .build()
            .context("GetManga request")?
            .send()
            .await
            .context("GetManga")?;
        let title = manga
            .data
            .attributes
            .title
            .get(&lang)
            .or_else(|| manga.data.attributes.title.get(&Language::English))
            .context("missing manga title")?
            .clone();

        Ok(Self {
            title,
            chapters: Chapter::list(client, lang, manga_id, offset).await?,
        })
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn chapters(&self) -> &[Chapter] {
        self.chapters.as_slice()
    }

    pub fn chapter_count(&self) -> usize {
        self.chapters.len()
    }

    pub fn page_count(&self) -> u64 {
        self.chapters
            .iter()
            .map(|chapter| u64::from(chapter.page_count()))
            .sum()
    }
}
