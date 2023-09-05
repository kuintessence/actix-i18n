use std::future::Future;
use std::pin::Pin;
use std::str::FromStr;

use actix_web::{dev::Payload, http::header, FromRequest, HttpRequest};
use smallvec::SmallVec;
use unic_langid::LanguageIdentifier;

use super::{I18NArgs, I18NBundle, I18NError, I18NResources};

type LanguageArray = SmallVec<[LanguageIdentifier; 8]>;

/// An extractor that parses the `Accept-Language` header and negotiates
/// language bundles.
///
/// # Example
///
/// Please refer to *examples/server.rs* in source code.
pub struct Locale {
    bundle: I18NBundle,
}

impl Locale {
    /// Gets the text with arguments.
    ///
    /// See also: [`I18NBundle::text_with_args`](I18NBundle::text_with_args)
    pub fn text_with_args<'a>(
        &self,
        id: impl AsRef<str>,
        args: impl Into<I18NArgs<'a>>,
    ) -> Result<String, I18NError> {
        self.bundle.text_with_args(id, args)
    }

    /// Gets the text.
    ///
    /// See also: [`I18NBundle::text`](I18NBundle::text)
    pub fn text(&self, id: impl AsRef<str>) -> Result<String, I18NError> {
        self.bundle.text(id)
    }
}

impl FromRequest for Locale {
    type Error = I18NError;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let resources = req
            .app_data::<I18NResources>()
            .expect("To use the `Locale` extractor, the `I18NResources` data is required.");

        let accept_languages = req
            .headers()
            .get(header::ACCEPT_LANGUAGE)
            .and_then(|value| value.to_str().ok())
            .map(parse_accept_languages)
            .unwrap_or_default();

        let bundle = resources.negotiate_languages(&accept_languages);
        Box::pin(async move { Ok(Self { bundle }) })
    }
}

fn parse_accept_languages(value: &str) -> LanguageArray {
    let mut languages = SmallVec::<[_; 8]>::new();

    for s in value.split(',').map(str::trim) {
        if let Some(res) = parse_language(s) {
            languages.push(res);
        }
    }

    languages.sort_by(|(_, a), (_, b)| b.cmp(a));
    languages
        .into_iter()
        .map(|(language, _)| language)
        .collect()
}

fn parse_language(value: &str) -> Option<(LanguageIdentifier, u16)> {
    let mut parts = value.split(';');
    let name = parts.next()?.trim();
    let quality = match parts.next() {
        Some(quality) => parse_quality(quality).unwrap_or_default(),
        None => 1000,
    };
    let language = LanguageIdentifier::from_str(name).ok()?;
    Some((language, quality))
}

fn parse_quality(value: &str) -> Option<u16> {
    let mut parts = value.split('=');
    let name = parts.next()?.trim();
    if name != "q" {
        return None;
    }
    let q = parts.next()?.trim().parse::<f32>().ok()?;
    Some((q.clamp(0.0, 1.0) * 1000.0) as u16)
}

#[cfg(test)]
mod tests {
    use unic_langid::langids;

    use super::*;

    #[test]
    fn test_parse_accept_languages() {
        assert_eq!(
            parse_accept_languages("zh-CN;q=0.5,en-US;q=0.7,fr;q=0.3").into_vec(),
            langids!("en-US", "zh-CN", "fr")
        );

        assert_eq!(
            parse_accept_languages("zh-CN ; q=0.5,en-US;q = 0.7,   fr;q=0.3").into_vec(),
            langids!("en-US", "zh-CN", "fr")
        );

        assert_eq!(
            parse_accept_languages("en-US;q=0.7,zh-CN,fr;q=0.3").into_vec(),
            langids!("zh-CN", "en-US", "fr")
        );
    }
}
