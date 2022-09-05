use captcha::filters::Noise;
use captcha::Captcha;
use std::path::Path;

async fn create_captcha() {
    Captcha::new()
        .add_chars(5)
        .apply_filter(Noise::new(0.1))
        .view(220, 180)
        .as_png()
        .unwrap()
}
