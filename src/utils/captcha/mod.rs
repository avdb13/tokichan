use anyhow::Result;
use ripemd::{Digest, Ripemd160};
use text_to_png::TextRenderer;
use tokio::{fs::File, io::AsyncWriteExt};

// pub async fn new_text(text: &str) -> Result<&str> {
//     let r = TextRenderer::try_new_with_ttf_font_data(include_bytes!("../../fonts/opensans.ttf"))?;
//     let png = r.render_text_to_png_data(text, 64, 0x444444)?;

//     let mut hasher = Sha1::new();
//     hasher.update(text);

//     let result = hasher.finalize();
//     let hash = std::str::from_utf8(&result).unwrap();
//     let mut file = File::create(format!("/tmp/{}-text.png", hash)).await?;
//     file.write_all(&png.data).await?;

//     Ok(hash)
// }
