use crate::{fs, Client};
use anyhow::{Context, Result as AnyResult};
use futures::{Stream, StreamExt};
use mangadex_api_types_rust::{ChapterSortOrder, Language, OrderDirection};
use std::path::PathBuf;
use uuid::Uuid;

/// A manga chapter.
#[derive(Debug)]
pub struct Chapter {
    /// Title.
    title: String,
    /// Chapter ID
    id: Uuid,
    /// Volume "number".
    volume_id: String,
    /// Chapter "number" in the serie.
    number: String,
    /// Number of pages.
    page_count: u16,
}

impl Chapter {
    pub async fn list(
        client: &Client,
        lang: Language,
        manga_id: Uuid,
        mut offset: u16,
    ) -> AnyResult<Vec<Self>> {
        let mut chapters = Vec::new();
        let start_offset = u32::from(offset);
        loop {
            let response = client
                .get()
                .await
                .chapter()
                .list()
                .manga_id(manga_id)
                .translated_languages(vec![lang])
                .order(ChapterSortOrder::Chapter(OrderDirection::Ascending))
                .offset(offset)
                .build()
                .context("ListChapter request")?
                .send()
                .await
                .context("ListChapter")?;
            let total = usize::try_from(response.total - start_offset)
                .expect("too many chapter");

            let old_len = chapters.len();

            chapters.extend(response.data.into_iter().map(|object| Self {
                title: object.attributes.title,
                id: object.id,
                volume_id:
                    object.attributes.volume.unwrap_or_else(|| "".to_owned()),
                number: format!(
                    "{:0>3}",
                    object.attributes.chapter.expect("missing chapter number")
                ),
                page_count:
                    object.attributes.pages.try_into().expect("too many pages"),
            }));
            chapters.truncate(total);

            // If we've got everything or there is nothing more to get.
            // Can happen when we filter: we'll never reach total.
            if chapters.len() == total || chapters.len() == old_len {
                break;
            }

            offset += u16::try_from(response.limit).expect("limit too large");
        }

        Ok(chapters)
    }

    pub async fn download_pages(
        &self,
        client: Client,
        destination: PathBuf,
    ) -> AnyResult<impl Stream<Item = AnyResult<()>>> {
        let at_home = client
            .get()
            .await
            .at_home()
            .server()
            .chapter_id(self.id)
            .build()
            .context("GetChapter request")?
            .send()
            .await
            .context("GetChapter")?;
        let number = self.number.clone();

        Ok(
            // XXX: we can use enumerate because the pages are sorted.
            futures::stream::iter(at_home.chapter.data.into_iter().enumerate())
                .then(move |(i, filename)| {
                    let url = at_home
                        .base_url
                        .join(&format!(
                            "/data/{}/{filename}",
                            at_home.chapter.hash
                        ))
                        .expect("valid URL");
                    let client = client.clone();
                    let ext = fs::extname_from_url(&url);
                    let filename = format!("{number}-{i:03}.{ext}");
                    let path = [&destination, &PathBuf::from(&filename)]
                        .iter()
                        .collect::<PathBuf>();

                    async move {
                        if path.is_file() {
                            return Ok(());
                        }

                        let res = client
                            .http_client()
                            .await
                            .get(url.clone())
                            .send()
                            .await
                            .context("get page")?;

                        let bytes =
                            res.bytes().await.context("download page")?;

                        fs::atomic_write(&path, &bytes)
                            .with_context(|| format!("save page {filename}"))?;

                        Ok(())
                    }
                }),
        )
    }

    pub fn page_count(&self) -> u16 {
        self.page_count
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn volume(&self) -> &str {
        &self.volume_id
    }
}
