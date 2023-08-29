
use std::thread::sleep;
use std::{sync::Arc, time::Duration};

use tokio::{sync::RwLock, task};

use captcha_a::{Captcha, CaptchaBuilder, Font};
use rand::{
    distributions::{Alphanumeric, DistString},
    seq::SliceRandom,
};

#[derive(Clone, Debug)]
pub struct MyCaptcha(pub String, pub Vec<u8>);

#[derive(Clone, Debug)]
pub struct CaptchaService(Arc<RwLock<Vec<MyCaptcha>>>);

impl From<Captcha> for MyCaptcha {
    fn from(c: Captcha) -> Self {
        MyCaptcha(c.phrase, c.raw_data)
    }
}

impl CaptchaService {
    pub async fn new(amount: usize) -> CaptchaService {
        let font: Font =
            Font::try_from_bytes(include_bytes!("../../windows_command_prompt.ttf")).unwrap();

        let v = Arc::new(RwLock::new(generate(amount, font.clone())));
        let w = v.clone();

        task::spawn_blocking(|| async move {
            loop {
                sleep(Duration::from_secs(1));
                let mut v = w.write().await;

                *v = generate(5, font.clone());
            }
        });

        CaptchaService(v)
    }
}

pub fn generate(i: usize, font: Font<'static>) -> Vec<MyCaptcha> {
    let source = Alphanumeric.sample_string(&mut rand::thread_rng(), 6);
    (0..i)
        .map(|_| {
            let builder = CaptchaBuilder {
                width: 120,
                height: 40,
                length: 4,
                source: source.clone(),
                fonts: &[font.clone()],
                ..Default::default()
            };
            builder.build().unwrap().into()
        })
        .collect()
}

impl CaptchaService {
    pub async fn recv(&self) -> MyCaptcha {
        let v = self.0.read().await;
        v.choose(&mut rand::thread_rng()).unwrap().to_owned()
    }
}
