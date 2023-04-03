//! biomeat - Download manga from `MangaDex`

// Lints {{{

#![deny(
    nonstandard_style,
    rust_2018_idioms,
    future_incompatible,
    rustdoc::all,
    rustdoc::missing_crate_level_docs,
    missing_docs,
    unreachable_pub,
    unsafe_code,
    unused,
    unused_import_braces,
    unused_lifetimes,
    unused_qualifications,
    variant_size_differences,
    warnings,
    clippy::all,
    clippy::pedantic,
    clippy::clone_on_ref_ptr,
    clippy::exit,
    clippy::filetype_is_file,
    clippy::float_cmp_const,
    clippy::lossy_float_literal,
    clippy::mem_forget,
    clippy::panic,
    clippy::pattern_type_mismatch,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::unneeded_field_pattern,
    clippy::verbose_file_reads,
    clippy::dbg_macro,
    clippy::let_underscore_must_use,
    clippy::todo,
    clippy::unwrap_used,
    clippy::use_debug
)]
#![allow(
    // The 90’s called and wanted their charset back :p
    clippy::non_ascii_literal,
)]

// }}}

use anyhow::{Context, Result as AnyResult};
use biomeat::{fs, Chapter, Client, Manga};
use clap::Parser;
use futures::TryStreamExt;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use mangadex_api_types_rust::Language;
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[tokio::main]
async fn main() -> AnyResult<()> {
    let opts = Opts::parse();

    let client = Client::default();
    let manga = Manga::new(&client, opts.lang, &opts.manga, opts.start)
        .await
        .context("get manga")?;

    // Create manga directory, if necessary.
    let destination = [opts.output.clone(), fs::sanitize_name(manga.title())]
        .iter()
        .collect::<PathBuf>();
    fs::mkdir_p(&destination).context("create manga directory")?;

    download(&client, &destination, &manga)
        .await
        .with_context(|| format!("download manga {}", opts.manga))?;

    Ok(())
}

async fn download(
    client: &Client,
    destination: &Path,
    manga: &Manga,
) -> AnyResult<()> {
    // Setup the progress bars (for chapters and pages).
    println!("Downloading {}", manga.title());
    let progress_bars = MultiProgress::new();
    let chapter_pb =
        progress_bars.add(ProgressBar::new(manga.chapter_count() as u64));
    chapter_pb.set_style(
        ProgressStyle::default_bar()
            .template("{msg:10}    [{bar:40.cyan/blue}] {pos:>4}/{len:4}")
            .expect("chapter progress bar template")
            .progress_chars("##-"),
    );
    chapter_pb.set_message("chapter");
    let page_pb = progress_bars.add(ProgressBar::new(manga.page_count()));
    setup_page_progress_bar(&page_pb);

    for chapter in manga.chapters() {
        download_pages(client.clone(), chapter, destination, &page_pb)
            .await
            .with_context(|| format!("download {}", chapter.title()))?;
        chapter_pb.inc(1);
    }

    page_pb.finish();
    chapter_pb.finish();

    Ok(())
}

/// Downloads the specified chapter pages as CBZ.
async fn download_pages(
    client: Client,
    chapter: &Chapter,
    directory: &Path,
    progress_bar: &ProgressBar,
) -> AnyResult<()> {
    // Create volume directory, if necessary.
    let destination = [
        directory,
        &PathBuf::from(format!("Volume {:0>2}", chapter.volume())),
    ]
    .iter()
    .collect::<PathBuf>();
    fs::mkdir_p(&destination).context("create volume directory")?;

    chapter
        .download_pages(client.clone(), destination)
        .await?
        .try_for_each(|_| {
            async {
                progress_bar.inc(1);
                Ok(())
            }
        })
        .await?;

    Ok(())
}

/// Configures the progress bar for the pages.
fn setup_page_progress_bar(progress_bar: &ProgressBar) {
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("{msg:10}    [{bar:40.cyan/blue}] {pos:>4}/{len:4} ETA: {eta_precise}")
            .expect("page progress bar template")
            .progress_chars("##-"),
    );
    progress_bar.set_message("pages");
}

/// CLI options.
#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct Opts {
    /// Path to the output directory.
    #[clap(short, long, default_value = ".")]
    output: PathBuf,

    /// Manga ID.
    #[clap(short, long)]
    manga: Uuid,

    /// Lang.
    #[clap(short, long, default_value = "en")]
    lang: Language,

    /// Start downloading from the specified chapter number.
    #[clap(short, long, default_value_t = 0)]
    start: u16,
}
