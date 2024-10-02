use anyhow::{anyhow, Result};
use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    Router,
};
use image::{DynamicImage, ImageReader};
use log::info;
use serde::Deserialize;
use std::{env, io::Cursor};
use tokio::{io::AsyncWriteExt, net::TcpStream};

#[derive(Clone)]
struct AppState {
    printer_address: String,
}

struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::BAD_REQUEST,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

#[derive(Deserialize, Debug)]
struct PrintJobSettings {
    no_pages: Option<u8>,
    media: brother_ql::media::Media,
    high_dpi: Option<bool>,
    compressed: Option<bool>,
    quality_priority: Option<bool>,
    cut_behavior: Option<brother_ql::printjob::CutBehavior>,
}

async fn handler(State(state): State<AppState>, mut multipart: Multipart) -> Result<(), AppError> {
    let mut image: Option<DynamicImage> = None;
    let mut settings: Option<PrintJobSettings> = None;

    while let Ok(Some(field)) = multipart.next_field().await {
        match field.name() {
            Some("image") => {
                let bytes = field.bytes().await?;
                image = Some(
                    ImageReader::new(Cursor::new(bytes))
                        .with_guessed_format()?
                        .decode()?,
                );
            }
            Some("settings") => settings = Some(serde_json::from_str(&field.text().await?)?),
            _ => {}
        };
    }

    if let (Some(image), Some(settings)) = (image, settings) {
        let print_job = brother_ql::printjob::PrintJob {
            no_pages: settings.no_pages.unwrap_or(1),
            image,
            media: settings.media,
            high_dpi: settings.high_dpi.unwrap_or(false),
            compressed: settings.compressed.unwrap_or(false),
            quality_priority: settings.quality_priority.unwrap_or(true),
            cut_behaviour: settings
                .cut_behavior
                .unwrap_or(brother_ql::printjob::CutBehavior::CutAtEnd),
        }
        .compile()?;

        info!("Submitting print job with size {}B and {:?}", print_job.len(), settings);
        let mut stream = TcpStream::connect(state.printer_address).await?;
        stream.write_all(&print_job).await?;

        Ok(())
    } else {
        Err(anyhow!("Missing multipart fields 'image' and/or 'settings'").into())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().without_time().init();

    let address = env::var("BQL_PRINT_ADDRESS")?;
    let port = env::var("BQL_PRINT_PORT")?;
    let printer_address = env::var("BQL_PRINT_PRINTER_ADDRESS")?;
    let printer_port = env::var("BQL_PRINT_PRINTER_PORT")?;

    let app = Router::new()
        .route("/print", post(handler))
        .with_state(AppState {
            printer_address: format!("{}:{}", printer_address, printer_port),
        });

    let addr = format!("{}:{}", address, port);
    info!("Listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
