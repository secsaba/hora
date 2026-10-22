//! Routes and pages; every page is served in English under `/en/` and in Hungarian under `/hu/`.

use askama::Template;
use axum::Router;
use axum::http::{HeaderMap, StatusCode, Uri, header};
use axum::response::{Html, IntoResponse, Redirect};
use axum::routing::get;
use fluent_templates::{LanguageIdentifier, Loader, langid, static_loader};

static_loader! {
    static LOCALES = {
        locales: "./locales",
        fallback_language: "en",
        // No Unicode isolation marks around placeables, so rendered text is plain.
        customise: |bundle| bundle.set_use_isolating(false),
    };
}

pub fn app() -> Router {
    Router::new()
        .route("/", get(root))
        .route("/en/", get(|uri| home(Lang::En, uri)))
        .route("/hu/", get(|uri| home(Lang::Hu, uri)))
        .route("/en", get(|| async { Redirect::permanent("/en/") }))
        .route("/hu", get(|| async { Redirect::permanent("/hu/") }))
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Lang {
    En,
    Hu,
}

impl Lang {
    fn code(self) -> &'static str {
        match self {
            Lang::En => "en",
            Lang::Hu => "hu",
        }
    }

    fn other(self) -> Lang {
        match self {
            Lang::En => Lang::Hu,
            Lang::Hu => Lang::En,
        }
    }

    fn id(self) -> LanguageIdentifier {
        match self {
            Lang::En => langid!("en"),
            Lang::Hu => langid!("hu"),
        }
    }

    /// The `Accept-Language` choice with the highest q (the first on a tie); English when neither is acceptable.
    fn from_accept_language(value: &str) -> Lang {
        let mut best = (Lang::En, 0.0);
        for item in value.split(',') {
            let mut parts = item.split(';').map(str::trim);
            let tag = parts.next().unwrap_or_default();
            let lang = match tag.split('-').next().unwrap_or_default() {
                t if t.eq_ignore_ascii_case("en") => Lang::En,
                t if t.eq_ignore_ascii_case("hu") => Lang::Hu,
                _ => continue,
            };
            // Parameter names are case-insensitive (RFC 9110 §5.6.6).
            let q = match parts.find_map(|p| {
                p.get(..2)
                    .filter(|k| k.eq_ignore_ascii_case("q="))
                    .map(|_| &p[2..])
            }) {
                None => 1.0,
                Some(q) => match q.parse::<f32>() {
                    Ok(q) if (0.0..=1.0).contains(&q) => q,
                    _ => continue,
                },
            };
            if q > best.1 {
                best = (lang, q);
            }
        }
        best.0
    }
}

async fn root(headers: HeaderMap) -> impl IntoResponse {
    let accept = headers
        .get(header::ACCEPT_LANGUAGE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();
    let lang = Lang::from_accept_language(accept);
    (
        [(header::VARY, "accept-language")],
        Redirect::to(&format!("/{}/", lang.code())),
    )
}

#[derive(Template)]
#[template(path = "home.html")]
struct Home {
    lang: Lang,
    switch_href: String,
}

impl Home {
    fn t(&self, id: &str) -> String {
        LOCALES.lookup(&self.lang.id(), id)
    }

    /// AGPL §13: every page links to the source.
    fn repository(&self) -> &'static str {
        env!("CARGO_PKG_REPOSITORY")
    }
}

async fn home(lang: Lang, uri: Uri) -> Result<Html<String>, StatusCode> {
    let page = Home {
        lang,
        switch_href: switch_href(lang, &uri),
    };
    page.render().map(Html).map_err(|e| {
        tracing::error!("rendering {uri}: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

/// The same path and query string in the other language; `uri` is under `/{lang}/`, as routed.
fn switch_href(lang: Lang, uri: &Uri) -> String {
    let rest = &uri.path()[1 + lang.code().len()..];
    match uri.query() {
        Some(query) => format!("/{}{rest}?{query}", lang.other().code()),
        None => format!("/{}{rest}", lang.other().code()),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode, header};
    use tower::ServiceExt;

    use super::*;

    async fn get(uri: &str, accept_language: Option<&str>) -> axum::response::Response {
        let mut request = Request::get(uri);
        if let Some(value) = accept_language {
            request = request.header(header::ACCEPT_LANGUAGE, value);
        }
        app()
            .oneshot(request.body(Body::empty()).unwrap())
            .await
            .unwrap()
    }

    async fn root_redirect(accept_language: Option<&str>) -> String {
        let response = get("/", accept_language).await;
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(response.headers()[header::VARY], "accept-language");
        response.headers()[header::LOCATION]
            .to_str()
            .unwrap()
            .to_owned()
    }

    #[tokio::test]
    async fn root_redirects_by_accept_language() {
        assert_eq!(root_redirect(Some("hu")).await, "/hu/");
        assert_eq!(root_redirect(None).await, "/en/");
        assert_eq!(root_redirect(Some("hu-HU,hu;q=0.9,en;q=0.8")).await, "/hu/");
        assert_eq!(root_redirect(Some("en-GB,hu;q=0.5")).await, "/en/");
        assert_eq!(root_redirect(Some("de, HU;q=0.3")).await, "/hu/");
        assert_eq!(root_redirect(Some("hu;q=0, en;q=0.1")).await, "/en/");
        assert_eq!(root_redirect(Some("en;q=0.2, hu;q=0.8")).await, "/hu/");
        assert_eq!(root_redirect(Some("hungarian, ;;q=x")).await, "/en/");
        assert_eq!(root_redirect(Some("hu;Q=0.1, en")).await, "/en/");
    }

    async fn body(uri: &str) -> String {
        let response = get(uri, None).await;
        assert_eq!(response.status(), StatusCode::OK);
        let page = String::from_utf8(
            to_bytes(response.into_body(), 1 << 20)
                .await
                .unwrap()
                .to_vec(),
        )
        .unwrap();
        // fluent-templates renders an id missing from every .ftl file as this text.
        assert!(!page.contains("Unknown localization key"), "{page}");
        page
    }

    #[tokio::test]
    async fn each_language_serves_the_home_page_in_it() {
        let en = body("/en/").await;
        assert!(en.contains(r#"<html lang="en">"#), "{en}");
        assert!(en.contains("Source code"), "{en}");
        let hu = body("/hu/").await;
        assert!(hu.contains(r#"<html lang="hu">"#), "{hu}");
        assert!(hu.contains("Forráskód"), "{hu}");
        assert!(
            hu.contains(r#"href="https://github.com/secsaba/hora""#),
            "{hu}"
        );
    }

    #[tokio::test]
    async fn the_language_link_keeps_the_path_and_query_string() {
        assert!(
            body("/en/?a=1&b=%C3%A9")
                .await
                .contains(r#"href="/hu/?a=1&#38;b=%C3%A9""#)
        );
        assert!(body("/hu/").await.contains(r#"href="/en/""#));
    }

    #[tokio::test]
    async fn a_language_without_its_slash_redirects_to_it() {
        for lang in ["en", "hu"] {
            let response = get(&format!("/{lang}"), None).await;
            assert_eq!(response.status(), StatusCode::PERMANENT_REDIRECT);
            assert_eq!(response.headers()[header::LOCATION], format!("/{lang}/"));
        }
    }

    #[tokio::test]
    async fn other_languages_are_not_found() {
        assert_eq!(get("/de/", None).await.status(), StatusCode::NOT_FOUND);
    }

    /// The Fluent message (and term) ids: lines that start an entry at column 0.
    fn keys(ftl: &str) -> BTreeSet<&str> {
        ftl.lines()
            .filter(|line| !line.starts_with([' ', '#', '.']))
            .filter_map(|line| line.split_once('=').map(|(id, _)| id.trim()))
            .collect()
    }

    #[test]
    fn both_languages_have_the_same_messages() {
        let en = keys(include_str!("../locales/en/main.ftl"));
        let hu = keys(include_str!("../locales/hu/main.ftl"));
        assert!(!en.is_empty());
        assert_eq!(
            en.difference(&hu).collect::<Vec<_>>(),
            [&""; 0],
            "missing in hu"
        );
        assert_eq!(
            hu.difference(&en).collect::<Vec<_>>(),
            [&""; 0],
            "missing in en"
        );
    }
}
