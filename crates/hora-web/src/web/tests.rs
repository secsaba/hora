use std::collections::BTreeSet;

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use tower::ServiceExt;

use super::*;

pub(crate) async fn get(uri: &str, accept_language: Option<&str>) -> axum::response::Response {
    let mut request = Request::get(uri);
    if let Some(value) = accept_language {
        request = request.header(header::ACCEPT_LANGUAGE, value);
    }
    app(places(), None, jiff::Timestamp::now)
        .oneshot(request.body(Body::empty()).unwrap())
        .await
        .unwrap()
}

/// The dev shell's places file, loaded once for every test.
pub(crate) fn places() -> Arc<Places> {
    static PLACES: std::sync::LazyLock<Arc<Places>> = std::sync::LazyLock::new(|| {
        let var =
            |name| std::env::var_os(name).unwrap_or_else(|| panic!("{name}, set by the dev shell"));
        let tzdb = jiff::tz::TimeZoneDatabase::from_dir(var("TZDIR")).unwrap();
        Arc::new(Places::load(var("HORA_PLACES").as_ref(), &tzdb).unwrap())
    });
    PLACES.clone()
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

const VESZPREM: u32 = 3042929;

/// `uri` with `VESZPREM` as the default place at `now`.
async fn get_at(uri: &str, now: fn() -> jiff::Timestamp) -> axum::response::Response {
    app(places(), Some(VESZPREM), now)
        .oneshot(Request::get(uri).body(Body::empty()).unwrap())
        .await
        .unwrap()
}

#[tokio::test]
async fn with_a_default_place_an_empty_request_redirects_to_the_chart_of_now_there() {
    let now = || "2026-07-01T10:15:42Z".parse().unwrap();
    for lang in ["en", "hu"] {
        let location =
            format!("/{lang}/chart?date=2026-07-01&time=12:15&pick=3042929&offset-pick=%2B02:00");
        for uri in [format!("/{lang}/"), format!("/{lang}/chart")] {
            let response = get_at(&uri, now).await;
            assert_eq!(response.status(), StatusCode::SEE_OTHER, "{uri}");
            assert_eq!(response.headers()[header::LOCATION], location, "{uri}");
        }
        let chart = get_at(&location, now).await;
        assert_eq!(chart.status(), StatusCode::OK, "{location}");
        assert!(
            !page(chart).await.contains(r#"class="error""#),
            "{location}"
        );
    }
}

#[tokio::test]
async fn the_chart_of_now_states_the_offset_in_force_across_a_clock_change() {
    // Budapest's clocks went back at 01:00 UT: both instants are 02:30 there.
    let summer = || "2026-10-25T00:30:00Z".parse().unwrap();
    let winter = || "2026-10-25T01:30:00Z".parse().unwrap();
    for (now, offset) in [(summer as fn() -> _, "%2B02:00"), (winter, "%2B01:00")] {
        let response = get_at("/en/chart", now).await;
        let location = response.headers()[header::LOCATION].to_str().unwrap();
        assert!(
            location.ends_with(&format!("time=02:30&pick=3042929&offset-pick={offset}")),
            "{location}"
        );
        let chart = get_at(location, now).await;
        assert_eq!(chart.status(), StatusCode::OK, "{location}");
        let html = page(chart).await;
        assert!(!html.contains(r#"class="error""#), "{html}");
        assert!(html.contains("Veszprém"), "{html}");
        assert!(html.contains("Lagna"), "{html}");
    }
}

#[tokio::test]
async fn a_request_with_birth_data_is_never_redirected() {
    let now = || jiff::Timestamp::UNIX_EPOCH;
    assert_eq!(
        get_at(&format!("/en/chart?{CHART_1}"), now).await.status(),
        StatusCode::OK
    );
    assert_eq!(
        get_at("/hu/chart?date=x", now).await.status(),
        StatusCode::BAD_REQUEST
    );
    for uri in ["/en/?utm_source=x", "/hu/chart?utm_source=x"] {
        assert!(!get_at(uri, now).await.status().is_redirection(), "{uri}");
    }
}

#[tokio::test]
async fn without_a_default_place_an_empty_request_shows_the_form() {
    for uri in ["/en/", "/hu/chart"] {
        let response = get(uri, None).await;
        assert!(!response.status().is_redirection(), "{uri}");
        assert!(page(response).await.contains(r#"name="date""#), "{uri}");
    }
}

#[test]
fn a_default_place_must_be_in_the_places_file() {
    assert!(default_place(None, &places()).unwrap().is_none());
    assert_eq!(
        default_place(Some(VESZPREM), &places()).unwrap(),
        Some(VESZPREM)
    );
    let err = default_place(Some(1), &places()).unwrap_err();
    assert!(err.to_string().contains("HORA_DEFAULT_PLACE"), "{err}");
}

pub(crate) async fn page(response: axum::response::Response) -> String {
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

pub(crate) async fn body(uri: &str) -> String {
    let response = get(uri, None).await;
    assert_eq!(response.status(), StatusCode::OK);
    page(response).await
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
async fn no_field_offers_a_keypad_without_a_minus_sign() {
    // iOS numeric and decimal keypads have no "-", which dates and south/west coordinates need.
    let home = body("/en/").await;
    assert!(!home.contains("inputmode"), "{home}");
}

#[tokio::test]
async fn the_footer_links_to_the_privacy_page_in_its_language() {
    for (lang, heading) in [("en", "Privacy"), ("hu", "Adatvédelem")] {
        let home = body(&format!("/{lang}/")).await;
        assert!(
            home.contains(&format!(r#"href="/{lang}/privacy""#)),
            "{home}"
        );
        let privacy = body(&format!("/{lang}/privacy")).await;
        assert!(
            privacy.contains(&format!("<h1>{heading}</h1>")),
            "{privacy}"
        );
        assert!(privacy.contains("Cloudflare"), "{privacy}");
    }
}

#[tokio::test]
async fn the_footer_links_to_the_disclaimer_in_its_language() {
    for (lang, heading, text) in [
        ("en", "Disclaimer", "not medical, legal"),
        ("hu", "Felelősségkizárás", "nem orvosi, jogi"),
    ] {
        let home = body(&format!("/{lang}/")).await;
        assert!(
            home.contains(&format!(r#"href="/{lang}/disclaimer""#)),
            "{home}"
        );
        let disclaimer = body(&format!("/{lang}/disclaimer")).await;
        assert!(
            disclaimer.contains(&format!("<h1>{heading}</h1>")),
            "{disclaimer}"
        );
        assert!(disclaimer.contains(text), "{disclaimer}");
    }
}

#[tokio::test]
async fn the_footer_links_the_version_to_its_tagged_source() {
    let version = env!("CARGO_PKG_VERSION");
    for (lang, label) in [("en", "Version"), ("hu", "Verzió")] {
        let home = body(&format!("/{lang}/")).await;
        assert!(
            home.contains(&format!(
                r#"<a href="{}/tree/v{version}">{label} {version}</a>"#,
                env!("CARGO_PKG_REPOSITORY")
            )),
            "{home}"
        );
    }
}

#[tokio::test]
async fn every_page_names_the_icons_and_they_are_served() {
    let home = body("/en/").await;
    for link in [
        r#"<link rel="icon" href="/favicon.svg" type="image/svg+xml">"#,
        r#"<link rel="apple-touch-icon" href="/apple-touch-icon.png">"#,
        r#"<meta name="apple-mobile-web-app-title" content="hora">"#,
    ] {
        assert!(home.contains(link), "{home}");
    }
    for (uri, content_type) in [
        ("/favicon.svg", "image/svg+xml"),
        ("/apple-touch-icon.png", "image/png"),
    ] {
        let response = get(uri, None).await;
        assert_eq!(response.status(), StatusCode::OK, "{uri}");
        assert_eq!(response.headers()[header::CONTENT_TYPE], content_type);
        assert!(response.headers().contains_key(header::ETAG), "{uri}");
    }
}

#[tokio::test]
async fn robots_txt_keeps_crawlers_off_the_chart_pages() {
    let response = get("/robots.txt", None).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers()[header::CONTENT_TYPE],
        "text/plain; charset=utf-8"
    );
    let robots = body("/robots.txt").await;
    for lang in ["en", "hu"] {
        assert!(
            robots.contains(&format!("Disallow: /{lang}/chart\n")),
            "{robots}"
        );
    }
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
    for path in ["/en", "/hu", "/en/kb", "/hu/kb"] {
        let response = get(path, None).await;
        assert_eq!(response.status(), StatusCode::PERMANENT_REDIRECT);
        assert_eq!(response.headers()[header::LOCATION], format!("{path}/"));
    }
}

#[tokio::test]
async fn every_page_links_to_the_knowledge_base() {
    for (uri, link) in [
        ("/en/", r#"<a href="/en/kb/">Knowledge base</a>"#),
        ("/hu/", r#"<a href="/hu/kb/">Tudástár</a>"#),
        ("/hu/kb/rasi", r#"<a href="/hu/kb/">Tudástár</a>"#),
    ] {
        assert!(body(uri).await.contains(link), "{uri}");
    }
}

#[tokio::test]
async fn the_kb_index_lists_the_learning_path_with_book_references() {
    let en = body("/en/kb/").await;
    assert!(en.contains("<h1>Knowledge base</h1>"), "{en}");
    let first = r#"<li><a href="/en/kb/jyotish">Jyotish</a> (§ 1.1–1.2)</li>"#;
    let graha = r#"<li><a href="/en/kb/graha">Graha</a> (§ 1.3.1)</li>"#;
    let rasi = r#"<li><a href="/en/kb/rasi">Rasi</a> (§ 1.3.2)</li>"#;
    let bhava = r#"<li><a href="/en/kb/bhava">Bhava</a> (§ 1.3.3)</li>"#;
    let chart = r#"<li><a href="/en/kb/chart">Chart</a> (§ 1.3.4)</li>"#;
    assert!(
        en.contains(first) && en.find(first) < en.find(graha),
        "{en}"
    );
    assert!(en.find(graha) < en.find(rasi), "{en}");
    assert!(en.find(rasi) < en.find(bhava), "{en}");
    assert!(en.find(bhava) < en.find(chart), "{en}");
    let varga = r#"<li><a href="/en/kb/varga-chakra">Varga chakra</a> (§ 1.3.5)</li>"#;
    assert!(en.find(chart) < en.find(varga), "{en}");
    let nakshatra = r#"<li><a href="/en/kb/nakshatra">Nakshatra</a> (§ 1.3.6)</li>"#;
    assert!(en.find(varga) < en.find(nakshatra), "{en}");
    let tithi = r#"<li><a href="/en/kb/tithi">Tithi</a> (§ 1.3.8.1)</li>"#;
    assert!(en.find(nakshatra) < en.find(tithi), "{en}");
    let ayanamsa = r#"<li><a href="/en/kb/ayanamsa">Ayanamsa</a> (§ 1.3.13)</li>"#;
    let month = r#"<li><a href="/en/kb/lunar-month">Lunar month</a> (§ 1.3.8.2)</li>"#;
    assert!(en.find(tithi) < en.find(month), "{en}");
    let yoga = r#"<li><a href="/en/kb/sun-moon-yoga">Sun-Moon yoga</a> (§ 1.3.9)</li>"#;
    assert!(en.find(month) < en.find(yoga), "{en}");
    let karana = r#"<li><a href="/en/kb/karana">Karana</a> (§ 1.3.10)</li>"#;
    assert!(en.find(yoga) < en.find(karana), "{en}");
    let vaara_and_hora =
        r#"<li><a href="/en/kb/vaara-and-hora">Vaara and hora</a> (§ 1.3.11)</li>"#;
    assert!(en.find(karana) < en.find(vaara_and_hora), "{en}");
    let panchaanga = r#"<li><a href="/en/kb/panchaanga">Panchaanga</a> (§ 1.3.12)</li>"#;
    assert!(en.find(vaara_and_hora) < en.find(panchaanga), "{en}");
    assert!(en.find(panchaanga) < en.find(ayanamsa), "{en}");
    let dasa = r#"<li><a href="/en/kb/dasa">Dasa</a> (§ 1.4)</li>"#;
    assert!(en.find(ayanamsa) < en.find(dasa), "{en}");
    let characteristics =
        r#"<li><a href="/en/kb/rasi-characteristics">Rasi characteristics</a> (§ 2.2–2.3)</li>"#;
    assert!(en.find(dasa) < en.find(characteristics), "{en}");
    let [basic, rasis] = [
        r#"<details>
<summary><h2>Chapter 1: Basic concepts</h2></summary>
<ol id="chapter-1">"#,
        r#"<details>
<summary><h2>Chapter 2: Rasis</h2></summary>
<ol id="chapter-2">"#,
    ];
    assert!(en.find(basic) < en.find(first), "{en}");
    assert!(en.find(dasa) < en.find(rasis), "{en}");
    assert!(en.find(rasis) < en.find(characteristics), "{en}");
    let grahas = r#"<details>
<summary><h2>Chapter 3: Grahas</h2></summary>
<ol id="chapter-3">"#;
    let graha_characteristics =
        r#"<li><a href="/en/kb/graha-characteristics">Graha characteristics</a> (§ 3.1–3.2)</li>"#;
    assert!(en.find(characteristics) < en.find(grahas), "{en}");
    assert!(en.find(grahas) < en.find(graha_characteristics), "{en}");
    let special_lagna = r#"<li><a href="/en/kb/special-lagna">Special lagna</a> (§ 5.1–5.7)</li>"#;
    let divisional = r#"<details>
<summary><h2>Chapter 6: Divisional charts</h2></summary>
<ol id="chapter-6">"#;
    let vargas = r#"<li><a href="/en/kb/vargas">The twenty varga chakras</a> (§ 6.1–6.5)</li>"#;
    let amsabala = r#"<li><a href="/en/kb/amsabala">Amsabala</a> (§ 6.6)</li>"#;
    assert!(
        en.contains(special_lagna) && en.find(special_lagna) < en.find(divisional),
        "{en}"
    );
    assert!(en.find(divisional) < en.find(vargas), "{en}");
    assert!(en.find(vargas) < en.find(amsabala), "{en}");
    let houses = r#"<details>
<summary><h2>Chapter 7: Bhavas</h2></summary>
<ol id="chapter-7">"#;
    let significations =
        r#"<li><a href="/en/kb/bhava-significations">What the bhavas show</a> (§ 7.1–7.3)</li>"#;
    let categories = r#"<li><a href="/en/kb/bhava-categories">Kinds of bhavas</a> (§ 7.4)</li>"#;
    assert!(en.find(amsabala) < en.find(houses), "{en}");
    assert!(en.find(houses) < en.find(significations), "{en}");
    assert!(
        en.contains(categories) && en.find(significations) < en.find(categories),
        "{en}"
    );
    let karakas_chapter = r#"<details>
<summary><h2>Chapter 8: Karakas</h2></summary>
<ol id="chapter-8">"#;
    let karakas = r#"<li><a href="/en/kb/karakas">Karakas</a> (§ 8.1–8.4)</li>"#;
    assert!(en.find(categories) < en.find(karakas_chapter), "{en}");
    assert!(
        en.contains(karakas) && en.find(karakas_chapter) < en.find(karakas),
        "{en}"
    );
    let arudha_chapter = r#"<details>
<summary><h2>Chapter 9: Arudha padas</h2></summary>
<ol id="chapter-9">"#;
    let arudha_padas = r#"<li><a href="/en/kb/arudha-padas">Arudha padas</a> (§ 9.1–9.7)</li>"#;
    assert!(en.find(karakas) < en.find(arudha_chapter), "{en}");
    assert!(
        en.contains(arudha_padas) && en.find(arudha_chapter) < en.find(arudha_padas),
        "{en}"
    );
    let aspects_chapter = r#"<details>
<summary><h2>Chapter 10: Aspects and argalas</h2></summary>
<ol id="chapter-10">"#;
    let aspects =
        r#"<li><a href="/en/kb/aspects-and-argalas">Aspects and argalas</a> (§ 10.1–10.7)</li>"#;
    assert!(en.find(arudha_padas) < en.find(aspects_chapter), "{en}");
    assert!(
        en.contains(aspects) && en.find(aspects_chapter) < en.find(aspects),
        "{en}"
    );
    let yogas_chapter = r#"<details>
<summary><h2>Chapter 11: Yogas</h2></summary>
<ol id="chapter-11">"#;
    let yogas = r#"<li><a href="/en/kb/yogas">Yogas</a> (§ 11.1–11.11)</li>"#;
    assert!(en.find(aspects) < en.find(yogas_chapter), "{en}");
    assert!(
        en.contains(yogas) && en.find(yogas_chapter) < en.find(yogas),
        "{en}"
    );
    let hu = body("/hu/kb/").await;
    let first = r#"<li><a href="/hu/kb/jyotish">Jyotish</a> (§ 1.1–1.2)</li>"#;
    let graha = r#"<li><a href="/hu/kb/graha">Graha</a> (§ 1.3.1)</li>"#;
    let rasi = r#"<li><a href="/hu/kb/rasi">Rasi</a> (§ 1.3.2)</li>"#;
    let bhava = r#"<li><a href="/hu/kb/bhava">Bhava</a> (§ 1.3.3)</li>"#;
    let chart = r#"<li><a href="/hu/kb/chart">Chakra</a> (§ 1.3.4)</li>"#;
    assert!(
        hu.contains(first) && hu.find(first) < hu.find(graha),
        "{hu}"
    );
    assert!(hu.find(graha) < hu.find(rasi), "{hu}");
    assert!(hu.find(rasi) < hu.find(bhava), "{hu}");
    assert!(hu.find(bhava) < hu.find(chart), "{hu}");
    let varga = r#"<li><a href="/hu/kb/varga-chakra">Varga chakra</a> (§ 1.3.5)</li>"#;
    assert!(hu.find(chart) < hu.find(varga), "{hu}");
    let nakshatra = r#"<li><a href="/hu/kb/nakshatra">Nakshatra</a> (§ 1.3.6)</li>"#;
    assert!(hu.find(varga) < hu.find(nakshatra), "{hu}");
    let tithi = r#"<li><a href="/hu/kb/tithi">Tithi</a> (§ 1.3.8.1)</li>"#;
    assert!(hu.find(nakshatra) < hu.find(tithi), "{hu}");
    let ayanamsa = r#"<li><a href="/hu/kb/ayanamsa">Ayanamsa</a> (§ 1.3.13)</li>"#;
    let month = r#"<li><a href="/hu/kb/lunar-month">Holdhónap</a> (§ 1.3.8.2)</li>"#;
    assert!(hu.find(tithi) < hu.find(month), "{hu}");
    let yoga = r#"<li><a href="/hu/kb/sun-moon-yoga">Nap–Hold yoga</a> (§ 1.3.9)</li>"#;
    assert!(hu.find(month) < hu.find(yoga), "{hu}");
    let karana = r#"<li><a href="/hu/kb/karana">Karana</a> (§ 1.3.10)</li>"#;
    assert!(hu.find(yoga) < hu.find(karana), "{hu}");
    let vaara_and_hora = r#"<li><a href="/hu/kb/vaara-and-hora">Vaara és hora</a> (§ 1.3.11)</li>"#;
    assert!(hu.find(karana) < hu.find(vaara_and_hora), "{hu}");
    let panchaanga = r#"<li><a href="/hu/kb/panchaanga">Panchaanga</a> (§ 1.3.12)</li>"#;
    assert!(hu.find(vaara_and_hora) < hu.find(panchaanga), "{hu}");
    assert!(hu.find(panchaanga) < hu.find(ayanamsa), "{hu}");
    let dasa = r#"<li><a href="/hu/kb/dasa">Dasa</a> (§ 1.4)</li>"#;
    assert!(hu.find(ayanamsa) < hu.find(dasa), "{hu}");
    let rasis = r#"<details>
<summary><h2>2. fejezet: Rasik</h2></summary>
<ol id="chapter-2">"#;
    assert!(hu.find(dasa) < hu.find(rasis), "{hu}");
    let special_lagna =
        r#"<li><a href="/hu/kb/special-lagna">Különleges lagna</a> (§ 5.1–5.7)</li>"#;
    let divisional = r#"<details>
<summary><h2>6. fejezet: Osztott chakrák</h2></summary>
<ol id="chapter-6">"#;
    let vargas = r#"<li><a href="/hu/kb/vargas">A húsz varga chakra</a> (§ 6.1–6.5)</li>"#;
    let amsabala = r#"<li><a href="/hu/kb/amsabala">Amsabala</a> (§ 6.6)</li>"#;
    assert!(
        hu.contains(special_lagna) && hu.find(special_lagna) < hu.find(divisional),
        "{hu}"
    );
    assert!(hu.find(divisional) < hu.find(vargas), "{hu}");
    assert!(hu.find(vargas) < hu.find(amsabala), "{hu}");
    let houses = r#"<details>
<summary><h2>7. fejezet: Bhavák</h2></summary>
<ol id="chapter-7">"#;
    let significations =
        r#"<li><a href="/hu/kb/bhava-significations">Amit a bhavák mutatnak</a> (§ 7.1–7.3)</li>"#;
    let categories = r#"<li><a href="/hu/kb/bhava-categories">A bhavák fajtái</a> (§ 7.4)</li>"#;
    assert!(hu.find(amsabala) < hu.find(houses), "{hu}");
    assert!(hu.find(houses) < hu.find(significations), "{hu}");
    assert!(
        hu.contains(categories) && hu.find(significations) < hu.find(categories),
        "{hu}"
    );
    let karakas_chapter = r#"<details>
<summary><h2>8. fejezet: Karakák</h2></summary>
<ol id="chapter-8">"#;
    let karakas = r#"<li><a href="/hu/kb/karakas">Karakák</a> (§ 8.1–8.4)</li>"#;
    assert!(hu.find(categories) < hu.find(karakas_chapter), "{hu}");
    assert!(
        hu.contains(karakas) && hu.find(karakas_chapter) < hu.find(karakas),
        "{hu}"
    );
    let arudha_chapter = r#"<details>
<summary><h2>9. fejezet: Arudha padák</h2></summary>
<ol id="chapter-9">"#;
    let arudha_padas = r#"<li><a href="/hu/kb/arudha-padas">Arudha padák</a> (§ 9.1–9.7)</li>"#;
    assert!(hu.find(karakas) < hu.find(arudha_chapter), "{hu}");
    assert!(
        hu.contains(arudha_padas) && hu.find(arudha_chapter) < hu.find(arudha_padas),
        "{hu}"
    );
    let aspects_chapter = r#"<details>
<summary><h2>10. fejezet: Aspektusok és argalák</h2></summary>
<ol id="chapter-10">"#;
    let aspects =
        r#"<li><a href="/hu/kb/aspects-and-argalas">Aspektusok és argalák</a> (§ 10.1–10.7)</li>"#;
    assert!(hu.find(arudha_padas) < hu.find(aspects_chapter), "{hu}");
    assert!(
        hu.contains(aspects) && hu.find(aspects_chapter) < hu.find(aspects),
        "{hu}"
    );
    let yogas_chapter = r#"<details>
<summary><h2>11. fejezet: Yogák</h2></summary>
<ol id="chapter-11">"#;
    let yogas = r#"<li><a href="/hu/kb/yogas">Yogák</a> (§ 11.1–11.11)</li>"#;
    assert!(hu.find(aspects) < hu.find(yogas_chapter), "{hu}");
    assert!(
        hu.contains(yogas) && hu.find(yogas_chapter) < hu.find(yogas),
        "{hu}"
    );
}

#[tokio::test]
async fn the_appendix_lists_every_chart_of_the_book_in_both_languages() {
    for (lang, label) in [("en", "Chart #"), ("hu", "#. chakra")] {
        let html = body(&format!("/{lang}/kb/book-charts")).await;
        for n in 1..=96 {
            let label = label.replace('#', &n.to_string());
            if [30, 36, 45, 52].contains(&n) {
                let cell = format!("<td>{label}</td>");
                assert!(html.contains(&cell), "{label} is listed without a link");
            } else {
                let link = format!(">{label}</a></td>");
                assert!(html.contains(&link), "{label} links to its chart");
            }
        }
    }
}

/// The sky a golden test pins for a chart that has birth data of its own.
async fn golden_sky(key: &str) -> Sky {
    match key {
        "1" => crate::book::chart_1_wilmington_2000().await,
        "2" => crate::book::chart_2_wilmington_2000_at_utc_minus_5().await,
        "3" => crate::book::chart_3_gwalior_1926().await,
        "4" => crate::book::chart_4_pithapuram_1968().await,
        "5" => crate::book::chart_5_aspects_exercise_2000().await,
        "6" => crate::book::chart_6_p_v_narasimha_rao_1921().await,
        "7" => crate::book::chart_7_ronald_reagan_1911().await,
        "8" => crate::book::chart_8_guess_his_livelihood_1946().await,
        "9" => crate::book::chart_9_chatrapati_shivaji_1630().await,
        "10" => crate::book::chart_10_akbar_1542().await,
        "11" => crate::book::chart_11_p_v_narasimha_rao_1921().await,
        "12" => crate::book::chart_12_sav_exercise_1958().await,
        "13" => crate::book::chart_13_an_example_1894().await,
        "14" => crate::book::chart_14_rajiv_gandhi_1944().await,
        "15" => crate::book::chart_15_satyam_gaur_1970().await,
        "15-shivam" => crate::book::chart_15_shivam_gaur_1970().await,
        "17" => crate::book::chart_17_pandit_sanjay_rath_1963().await,
        "18" => crate::book::chart_18_a_lady_1972().await,
        "19" => crate::book::chart_19_sri_navin_patnaik_1946().await,
        "20" => crate::book::chart_20_an_engineer_in_usa_1954().await,
        "21" => crate::book::chart_21_john_f_kennedy_jr_1960().await,
        "22" => crate::book::chart_22_a_lady_1954().await,
        "23" => crate::book::chart_23_dr_b_v_raman_1912().await,
        "24" => crate::book::chart_24_bill_gates_1955().await,
        "25" => crate::book::chart_25_india_1947().await,
        "26" => crate::book::chart_26_narayana_dasa_exercise_1971().await,
        "27" => crate::book::chart_27_foreign_stay_example_1970().await,
        "28" => crate::book::chart_28_a_lady_1971().await,
        "29" => crate::book::chart_29_a_lady_1969().await,
        "31" => crate::book::chart_31_an_electrical_engineer_1947().await,
        "32" => crate::book::chart_32_a_gentleman_1961().await,
        "35" => crate::book::chart_35_jayalalita_1948().await,
        "37" => crate::book::chart_37_iskcon_devotee_1971().await,
        "38" => crate::book::chart_38_sri_aurobindo_ghose_1872().await,
        "40" => crate::book::chart_40_a_male_1927().await,
        "41" => crate::book::chart_41_a_gentleman_1902().await,
        "42" => crate::book::chart_42_niryaana_shoola_exercise_1889().await,
        "43" => crate::book::chart_43_shoola_dasa_exercise_1915().await,
        "44" => crate::book::chart_44_pitri_shoola_example_1971().await,
        "48" => crate::book::chart_48_a_gentleman_1933().await,
        "49" => crate::book::chart_49_an_iskcon_devotee_1971().await,
        "50" => crate::book::chart_50_divorced_lady_1968().await,
        "51" => crate::book::chart_51_an_actor_1937().await,
        "53" => crate::book::chart_53_a_lady_1973().await,
        "54" => crate::book::chart_54_an_engineer_1970().await,
        "54-transit" => crate::book::chart_54_wedding_transit_1999().await,
        "55" => crate::book::chart_55_a_gentleman_1961().await,
        "55-transit" => crate::book::chart_55_childbirth_transit_1992().await,
        "56" => crate::book::chart_56_john_f_kennedy_jr_1960().await,
        "56-transit" => crate::book::chart_56_death_transit_1999().await,
        "58" => crate::book::chart_58_vajpayee_1926().await,
        "59" => crate::book::chart_59_vajpayee_swearing_in_1998().await,
        "61" => crate::book::chart_61_indira_gandhi_1917().await,
        "61-transit" => crate::book::chart_61_indira_gandhi_assassination_1984().await,
        "62" => crate::book::chart_62_princess_diana_1961().await,
        "66" => crate::book::chart_66_annual_2000().await,
        "67" => crate::book::chart_67_annual_1993().await,
        "68" => crate::book::chart_68_annual_1991().await,
        "69" => crate::book::chart_69_annual_1987().await,
        "70" => crate::book::chart_70_annual_1998().await,
        "71" => crate::book::chart_71_annual_1995().await,
        "73-annual" => crate::book::chart_73_indira_gandhi_annual_1976().await,
        "74-annual" => crate::book::chart_74_a_lady_annual_1998().await,
        "75" => crate::book::chart_75_rectification_example_1966().await,
        "77" => crate::book::chart_77_a_gentleman_1939().await,
        "78" => crate::book::chart_78_bill_gates_wrong_time_1955().await,
        "79" => crate::book::chart_79_a_music_teacher_1973().await,
        "80-annual" => crate::book::chart_80_india_annual_1997().await,
        "81" => crate::book::chart_81_india_lunar_year_1992().await,
        "82" => crate::book::chart_82_india_solar_year_1998().await,
        "84" => crate::book::chart_84_vajpayee_swearing_in_1996().await,
        "86" => crate::book::chart_86_writing_this_book_1997().await,
        "88" => crate::book::chart_88_an_astrologer_1949().await,
        "89" => crate::book::chart_89_annual_1999().await,
        "90" => crate::book::chart_90_an_indian_in_usa_1968().await,
        "91" => crate::book::chart_91_annual_1998().await,
        "92" => crate::book::chart_92_monthly_may_1998().await,
        "92-june" => crate::book::chart_92_monthly_june_1998().await,
        "93" => crate::book::chart_93_monthly_march_2000().await,
        "94" => crate::book::chart_94_sixty_hours_2000().await,
        "95" => crate::book::chart_95_a_lady_1968().await,
        "96" => crate::book::chart_96_tajaka_1991().await,
        key => panic!("no golden fixture for chart {key}"),
    }
}

#[tokio::test]
async fn every_appendix_link_opens_the_chart_its_golden_test_pins() {
    for lang in ["en", "hu"] {
        let html = body(&format!("/{lang}/kb/book-charts")).await;
        let prefix = format!(r#"<a href="/{lang}/chart?"#);
        let links: Vec<_> = html
            .split(&prefix)
            .skip(1)
            .map(|link| {
                let query = link[..link.find('"').unwrap()].replace("&amp;", "&");
                format!("/{lang}/chart?{query}")
            })
            .collect();
        let linked = book_charts::CHARTS
            .iter()
            .filter(|chart| chart.request().is_some());
        assert_eq!(links.len(), linked.clone().count());
        for (chart, link) in linked.zip(links) {
            let response = get(&link, None).await;
            assert_eq!(response.status(), StatusCode::OK, "{}: {link}", chart.key);
            let uri: Uri = link.parse().unwrap();
            let Query(form) = Query::<HashMap<String, String>>::try_from_uri(&uri).unwrap();
            let Ok((request, ..)) = parse(&form, None) else {
                panic!("{}: {link}", chart.key);
            };
            let mut source = chart;
            while let book_charts::Data::Same(key) = source.data {
                source = book_charts::CHARTS.iter().find(|c| c.key == key).unwrap();
            }
            assert_eq!(
                hora_swe::calculate(request).await.unwrap(),
                golden_sky(source.key).await,
                "{}: {link}",
                chart.key
            );
        }
    }
}

#[tokio::test]
async fn the_appendix_says_which_chart_a_repeat_shows_again_and_why_one_has_no_link() {
    let en = body("/en/kb/book-charts").await;
    assert!(en.contains("Rasi chart; the same data as Chart 7"), "{en}");
    assert!(
        en.contains("Not linked: the book prints no birth data"),
        "{en}"
    );
    let hu = body("/hu/kb/book-charts").await;
    assert!(
        hu.contains("Rasi chakra; az adatok azonosak ezzel: 7. chakra"),
        "{hu}"
    );
    assert!(hu.contains("a könyv nem közöl születési adatokat"), "{hu}");
}

#[tokio::test]
async fn the_kb_index_lists_the_appendix_with_no_book_section() {
    for (lang, heading, title) in [
        ("en", "Appendix", "The book&#39;s charts"),
        ("hu", "Függelék", "A könyv chakrái"),
    ] {
        let index = body(&format!("/{lang}/kb/")).await;
        assert!(index.contains(&format!("<h2>{heading}</h2>")), "{index}");
        let link = format!(r#"<a href="/{lang}/kb/book-charts">{title}</a></li>"#);
        assert!(index.contains(&link), "{link}: {index}");
    }
}

#[tokio::test]
async fn a_kb_page_names_its_chapter_and_links_its_neighbours_on_the_path() {
    let en = body("/en/kb/rasi").await;
    for text in [
        r#"<a href="/en/kb/#chapter-1">Chapter 1: Basic concepts</a>"#,
        r#"<a href="/en/kb/graha" rel="prev"><span>Previous</span> Graha</a>"#,
        r#"<a href="/en/kb/bhava" rel="next"><span>Next</span> Bhava</a>"#,
    ] {
        assert!(en.contains(text), "{text}: {en}");
    }
    let hu = body("/hu/kb/rasi-characteristics").await;
    assert!(
        hu.contains(r#"<a href="/hu/kb/#chapter-2">2. fejezet: Rasik</a>"#),
        "{hu}"
    );
    assert!(
        hu.contains(r#"<a href="/hu/kb/dasa" rel="prev"><span>Előző</span> Dasa</a>"#),
        "{hu}"
    );
    assert!(
            hu.contains(r#"<a href="/hu/kb/graha-characteristics" rel="next"><span>Következő</span> A grahák jellemzői</a>"#),
            "{hu}"
        );
    // The path's ends link one way only.
    let last = kb::PAGES.last().unwrap().slug;
    assert!(
        !body(&format!("/hu/kb/{last}"))
            .await
            .contains(r#"rel="next""#),
        "{last}"
    );
    assert!(!body("/en/kb/jyotish").await.contains(r#"rel="prev""#));
}

#[tokio::test]
async fn a_kb_page_shows_its_table_book_reference_and_link_in_its_language() {
    let book = r#"<a href="https://www.vedicastrologer.org/articles/astro_books.htm">"#;
    let en = body("/en/kb/rasi").await;
    for text in [
        "<title>Rasi – hora</title>",
        "<td>Aries</td>",
        "Book reference:",
        "chapter 1, § 1.3.2",
        book,
    ] {
        assert!(en.contains(text), "{text}: {en}");
    }
    assert!(en.contains(r#"href="/hu/kb/rasi""#), "{en}");
    let hu = body("/hu/kb/rasi").await;
    for text in [
        "<td>Kos</td>",
        "Hivatkozás a könyvre:",
        "1. fejezet, § 1.3.2",
        book,
    ] {
        assert!(hu.contains(text), "{text}: {hu}");
    }
}

#[tokio::test]
async fn an_unknown_kb_page_is_not_found_in_its_language() {
    for (uri, text) in [
        (
            "/en/kb/rashi",
            "There is no knowledge base page at this address.",
        ),
        ("/hu/kb/rashi", "Ezen a címen nincs tudástár-oldal."),
        (
            "/en/kb/..%2Fmain",
            "There is no knowledge base page at this address.",
        ),
    ] {
        let response = get(uri, None).await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND, "{uri}");
        assert!(page(response).await.contains(text), "{uri}");
    }
}

#[tokio::test]
async fn other_languages_are_not_found() {
    assert_eq!(get("/de/", None).await.status(), StatusCode::NOT_FOUND);
}

/// Chart 1's birth data (ch. 9 *Arudha Padas*, p. 87) as the book prints it: 2000-04-09 17:55 at UTC−4:00, 71°W12′, 42°N30′.
pub(crate) const CHART_1: &str = "date=2000-04-09&time=17%3A55&calendar=gregorian&utc=stated&offset=-04%3A00\
        &latitude=42.5&longitude=-71.2&altitude=&ayanamsa=lahiri&node=mean&sunrise=upper-limb-apparent&reference=lagna&style=south&sound=";

/// The chart table, without the form that echoes the request.
pub(crate) fn table(page: &str) -> &str {
    let start = page.find("<table").expect("a table");
    &page[start..page.find("</table>").expect("a table end")]
}

/// The chart table's rows, without its header links that echo the request too.
pub(crate) fn tbody(page: &str) -> &str {
    let table = table(page);
    &table[table.find("<tbody>").expect("a tbody")..]
}

/// One `<tr>` holding all of `cells` in order, or `None`.
pub(crate) fn row<'a>(table: &'a str, cells: &[&str]) -> Option<&'a str> {
    table.split("<tr").find(|row| {
        let mut rest = *row;
        cells.iter().all(|cell| match rest.find(cell) {
            Some(i) => {
                rest = &rest[i + cell.len()..];
                true
            }
            None => false,
        })
    })
}

/// `golden.rs` owns the values' correctness; this checks the wiring: form → chart → table, in both languages.
#[tokio::test]
async fn the_chart_page_shows_chart_1_from_its_form_in_both_languages() {
    // The book prints whole arcminutes, and the page truncates, so its degrees and minutes are the book's.
    // Nakshatra, pada and lord follow from them (ch. 1 §1.3.6, Table 2); Venus, the closest, is 1′ into its pada.
    let books = [
        // Only the degree: the book's lagna (10°58′) is 6–10′ off ours for an unknown reason.
        (
            ("Lagna", "Virgo", "Moon"),
            ("Lagna", "Szűz", "Hold"),
            "10°",
            "Hasta (13)",
            "1",
        ),
        (
            ("Sun", "Pisces", "Mercury"),
            ("Nap", "Halak", "Merkúr"),
            "26°29′",
            "Revati (27)",
            "3",
        ),
        (
            ("Moon", "Gemini", "Mars"),
            ("Hold", "Ikrek", "Mars"),
            "4°45′",
            "Mrigasira (5)",
            "4",
        ),
        (
            ("Mars", "Aries", "Venus"),
            ("Mars", "Kos", "Vénusz"),
            "19°09′",
            "Bharani (2)",
            "2",
        ),
        (
            ("Mercury", "Pisces", "Jupiter"),
            ("Merkúr", "Halak", "Jupiter"),
            "1°36′",
            "Poorvaabhaadra (25)",
            "4",
        ),
        (
            ("Jupiter", "Aries", "Venus"),
            ("Jupiter", "Kos", "Vénusz"),
            "17°21′",
            "Bharani (2)",
            "2",
        ),
        (
            ("Venus", "Pisces", "Saturn"),
            ("Vénusz", "Halak", "Szaturnusz"),
            "10°01′",
            "Uttaraabhaadra (26)",
            "3",
        ),
        (
            ("Saturn", "Aries", "Venus"),
            ("Szaturnusz", "Kos", "Vénusz"),
            "22°41′",
            "Bharani (2)",
            "3",
        ),
        (
            ("Rahu", "Cancer", "Saturn"),
            ("Rahu", "Rák", "Szaturnusz"),
            "5°55′",
            "Pushyami (8)",
            "1",
        ),
        (
            ("Ketu", "Capricorn", "Sun"),
            ("Ketu", "Bak", "Nap"),
            "5°55′",
            "Uttaraashaadha (21)",
            "3",
        ),
    ];
    for (lang, heading) in [(Lang::En, "Vimsottari lord"), (Lang::Hu, "Vimsottari ura")] {
        let page = body(&format!("/{}/chart?{CHART_1}", lang.code())).await;
        let headings = ["Graha", "Rasi", "Bhava", "Nakshatra", "Pada", heading];
        let headings = headings.map(|h| format!(">{h}<"));
        let headings = headings.each_ref().map(String::as_str);
        assert!(row(table(&page), &headings).is_some(), "{page}");
        let view = view(lang, CHART_1).await;
        // The book prints no ayanamsa value; this is the library's Lahiri at chart 1, truncated like the grahas.
        assert_eq!(view.ayanamsa, "23°51′39″");
        // Of chart 1's grahas only the mean nodes move backward.
        let marked: Vec<_> = view.grahas.iter().filter(|r| r.retrograde).collect();
        let marked: Vec<_> = marked.iter().map(|r| r.name.as_str()).collect();
        assert_eq!(marked, ["graha-rahu", "graha-ketu"]);
        for (en_names, hu_names, position, nakshatra, pada) in books {
            let (graha, rasi, lord) = if lang == Lang::En { en_names } else { hu_names };
            let row = view.grahas.iter().find(|r| t(lang, &r.name) == graha);
            let row = row.expect(graha);
            let shown = format!("{} ({})", t(lang, row.nakshatra), row.nakshatra_number);
            assert_eq!(
                (&*t(lang, row.rasi), &*shown, &*row.pada.to_string()),
                (rasi, nakshatra, pada),
                "{graha}"
            );
            assert_eq!(t(lang, row.lord), lord, "{graha}");
            assert!(
                row.position.starts_with(position),
                "{graha}: {}",
                row.position
            );
        }
    }
}

#[tokio::test]
async fn every_graha_the_chart_page_names_links_to_its_kb_page() {
    for (lang, sun, lagna_nakshatra_lord) in [("en", "Sun", "Moon"), ("hu", "Nap", "Hold")] {
        let page = body(&format!("/{lang}/chart?{CHART_1}")).await;
        let link = |anchor, name| {
            format!(r#"<a href="/{lang}/kb/graha-characteristics#{anchor}">{name}</a>"#)
        };
        let table = table(&page);
        assert!(
            table.contains(&format!(r#"<th scope="row">{}</th>"#, link("sun", sun))),
            "{table}"
        );
        assert!(table.contains(r#"<th scope="row">Lagna</th>"#), "{table}");
        assert!(
            table.contains(&format!(
                // The Vimsottari lord, then the lagna's empty dignity.
                "<td>{}</td><td>—</td>",
                link("moon", lagna_nakshatra_lord)
            )),
            "{table}"
        );
        // Chart 1's tithi, vaara, nakshatra and hora each have a lord.
        let day = body(&format!("/{lang}/chart?{CHART_1}&topic=day")).await;
        let panchaanga = section(&day, "panchaanga");
        let href = format!(r#"<td><a href="/{lang}/kb/graha-characteristics#"#);
        assert_eq!(panchaanga.matches(&href).count(), 4, "{panchaanga}");
    }
}

#[tokio::test]
async fn every_rasi_the_chart_page_names_links_to_its_kb_page() {
    for (lang, virgo) in [("en", "Virgo"), ("hu", "Szűz")] {
        let page = body(&format!("/{lang}/chart?{CHART_1}")).await;
        let table = table(&page);
        let href = format!(r#"<td><a href="/{lang}/kb/rasi-characteristics#"#);
        // The lagna and the nine grahas.
        assert_eq!(table.matches(&href).count(), 10, "{table}");
        assert!(
            table.contains(&format!(r#"{href}virgo">{virgo}</a></td>"#)),
            "{table}"
        );
    }
}

#[tokio::test]
async fn every_nakshatra_the_chart_page_names_links_to_its_kb_page() {
    for lang in ["en", "hu"] {
        let page = body(&format!("/{lang}/chart?{CHART_1}")).await;
        let href = format!(r#"<td><a href="/{lang}/kb/nakshatra">"#);
        // The lagna and the nine grahas.
        let table = table(&page);
        assert_eq!(table.matches(&href).count(), 10, "{table}");
        assert!(
            table.contains(&format!("{href}Hasta (13)</a></td>")),
            "{table}"
        );
        // The Moon's nakshatra, the panchaanga's only one.
        let day = body(&format!("/{lang}/chart?{CHART_1}&topic=day")).await;
        let panchaanga = section(&day, "panchaanga");
        assert_eq!(panchaanga.matches(&href).count(), 1, "{panchaanga}");
    }
}

/// The ayanamsa page's worked examples: GREENWICH_2000 under each ayanamsa, and Lahiri a century either side.
#[tokio::test]
async fn the_ayanamsa_pages_examples_are_horas_charts() {
    const SOURCES: [(&str, &str); 2] = [
        ("en", include_str!("../../kb/en/ayanamsa.md")),
        ("hu", include_str!("../../kb/hu/ayanamsa.md")),
    ];
    /// The chart, once its ayanamsa is checked and both pages are found to link it and quote the value.
    async fn chart(year: &str, ayanamsa: &str, value: &str) -> ChartView {
        let query = GREENWICH_2000
            .replace("date=2000", &format!("date={year}"))
            .replace("ayanamsa=lahiri", &format!("ayanamsa={ayanamsa}"));
        let view = view(Lang::En, &query).await;
        assert_eq!(view.ayanamsa, value);
        for (lang, source) in SOURCES {
            for text in [&format!("(/{lang}/chart?{query})"), value] {
                assert!(source.contains(text), "{lang}: {text}");
            }
        }
        view
    }
    // The Sun's degree, pada and bhava, and the lagna's rasi.
    let suns = [
        ("lahiri", "23°51′25″", ["16°31′17″", "1", "9", "Aries"]),
        ("true-chitra", "23°50′28″", ["16°32′13″", "1", "9", "Aries"]),
        ("raman", "22°24′38″", ["17°58′04″", "2", "9", "Aries"]),
        ("true-pushya", "22°43′19″", ["17°39′23″", "2", "9", "Aries"]),
        ("true-mula", "24°35′08″", ["15°47′34″", "1", "10", "Pisces"]),
    ];
    for (ayanamsa, value, [sun, pada, bhava, lagna]) in suns {
        let view = chart("2000", ayanamsa, value).await;
        assert_eq!(
            row_named(&view.grahas, "Sun")[1..6],
            ["Sagittarius", bhava, sun, "Poorvaashaadha (20)", pada],
            "{ayanamsa}"
        );
        assert_eq!(row_named(&view.grahas, "Lagna")[1], lagna, "{ayanamsa}");
        for (lang, source) in SOURCES {
            assert!(source.contains(sun), "{lang}: {sun}");
        }
    }
    for (year, value) in [("1900", "22°27′37″"), ("2100", "25°15′15″")] {
        chart(year, "lahiri", value).await;
    }
    for (lang, source) in SOURCES {
        for text in ["256°31′17″", "280°22′42″", "257°58′04″"] {
            assert!(source.contains(text), "{lang}: {text}");
        }
    }
}

/// The KB's worked examples: noon UT, 1 January 2000, at Greenwich.
pub(crate) const GREENWICH_2000: &str = "date=2000-01-01&time=12%3A00&calendar=gregorian&utc=stated&offset=%2B00%3A00\
        &latitude=51.48&longitude=0&altitude=&ayanamsa=lahiri&node=mean&sunrise=upper-limb-apparent&reference=lagna&style=south";

/// The rasi page's worked example is hora's own Sun at noon UT, 1 January 2000, at Greenwich.
#[tokio::test]
async fn the_rasi_pages_example_is_horas_chart() {
    let query = GREENWICH_2000;
    let view = view(Lang::En, query).await;
    let sun = row_named(&view.grahas, "Sun");
    assert_eq!([&*sun[1], &*sun[3]], ["Sagittarius", "16°31′17″"]);
    assert_eq!(view.ayanamsa, "23°51′25″");
    for (lang, source) in [
        ("en", include_str!("../../kb/en/rasi.md")),
        ("hu", include_str!("../../kb/hu/rasi.md")),
    ] {
        for text in [
            &format!("(/{lang}/chart?{query})"),
            "16°31′17″",
            "256°31′17″",
            "16 Sg 31",
            "8s 16° 31′",
            "23°51′",
        ] {
            assert!(source.contains(text), "{lang}: {text}");
        }
    }
}

#[tokio::test]
async fn every_bhava_the_chart_page_names_links_to_its_kb_page() {
    for lang in ["en", "hu"] {
        let page = body(&format!("/{lang}/chart?{CHART_1}")).await;
        let table = table(&page);
        let href = format!(r#"<td><a href="/{lang}/kb/bhava">"#);
        // The lagna and the nine grahas.
        assert_eq!(table.matches(&href).count(), 10, "{table}");
        assert!(table.contains(&format!("{href}1</a></td>")), "{table}");
    }
}

/// The bhava page's example is hora's chart of the rasi page, counted from the lagna and from the Moon.
#[tokio::test]
async fn the_bhava_pages_example_is_horas_chart() {
    let query = "date=2000-01-01&time=12%3A00&calendar=gregorian&utc=stated&offset=%2B00%3A00\
            &latitude=51.48&longitude=0&altitude=&ayanamsa=lahiri&node=mean&sunrise=upper-limb-apparent";
    for (reference, bhavas) in [
        (
            "lagna",
            [
                ("Lagna", "Aries", 1),
                ("Sun", "Sagittarius", 9),
                ("Moon", "Libra", 7),
                ("Ketu", "Capricorn", 10),
            ],
        ),
        (
            "moon",
            [
                ("Lagna", "Aries", 7),
                ("Sun", "Sagittarius", 3),
                ("Moon", "Libra", 1),
                ("Ketu", "Capricorn", 4),
            ],
        ),
    ] {
        let view = view(
            Lang::En,
            &format!("{query}&reference={reference}&style=south"),
        )
        .await;
        for (graha, rasi, bhava) in bhavas {
            let row = row_named(&view.grahas, graha);
            assert_eq!(
                [&*row[1], &*row[2]],
                [rasi, &*bhava.to_string()],
                "{reference} {graha}"
            );
        }
    }
    for (lang, source) in [
        ("en", include_str!("../../kb/en/bhava.md")),
        ("hu", include_str!("../../kb/hu/bhava.md")),
    ] {
        for reference in ["lagna", "moon"] {
            let link = format!("(/{lang}/chart?{query}&reference={reference}&style=south)");
            assert!(source.contains(&link), "{lang}: {reference}");
        }
        // The page's sums: the Sun, Ketu and the lagna counted from the Moon.
        for sum in ["9 − 7 + 1 = 3", "10 − 7 + 1 = 4", "1 − 7 + 1 = −5"] {
            assert!(source.contains(sum), "{lang}: {sum}");
        }
    }
}

#[tokio::test]
async fn the_chart_heading_links_to_its_kb_page() {
    for (lang, name) in [("en", "Chart"), ("hu", "Chakra")] {
        let page = body(&format!("/{lang}/chart?{CHART_1}")).await;
        let heading = format!(r#"<h2><a href="/{lang}/kb/chart">{name}</a></h2>"#);
        assert!(page.contains(&heading), "{page}");
    }
}

/// hora's own example chart, the chart KB page's, without a style.
pub(crate) const HORAS_CHART: &str = "date=2000-01-01&time=06%3A00&calendar=gregorian&utc=stated&offset=%2B00%3A00\
        &latitude=51.48&longitude=0&altitude=&ayanamsa=lahiri&node=mean&sunrise=upper-limb-apparent&reference=lagna";

/// The chart KB page's example is hora's chart, drawn in each style.
#[tokio::test]
async fn the_chart_pages_example_is_horas_chart() {
    let page = body(&format!("/en/chart?{HORAS_CHART}&style=north")).await;
    let view = view(Lang::En, &format!("{HORAS_CHART}&style=north")).await;
    for (point, rasi, retrograde) in [
        ("Lagna", "Scorpio", false),
        ("Venus", "Scorpio", false),
        ("Sun", "Sagittarius", false),
        ("Mercury", "Sagittarius", false),
        ("Moon", "Libra", false),
        ("Mars", "Aquarius", false),
        ("Jupiter", "Aries", false),
        ("Saturn", "Aries", true),
        ("Rahu", "Cancer", true),
        ("Ketu", "Capricorn", true),
    ] {
        let row = view
            .grahas
            .iter()
            .find(|r| en(&r.name) == point)
            .expect(point);
        assert_eq!(
            (&*en(row.rasi), row.retrograde),
            (rasi, retrograde),
            "{point}"
        );
    }
    for text in [
        r#"aria-label="Lagna, 17 degrees 14 minutes in Scorpio""#,
        r#"aria-label="Venus, 7 degrees 25 minutes in Scorpio""#,
        r#"aria-label="Saturn, retrograde, 16 degrees 32 minutes in Aries""#,
    ] {
        assert!(page.contains(text), "{text}: {page}");
    }
    // The page walks the bhavas from the top diamond: Scorpio's 8, then on round to Libra's 7.
    let numbers: Vec<_> = drawn(&page)
        .split(r#"class="mark" aria-hidden="true">"#)
        .skip(1)
        .map(|rest| &rest[..rest.find('<').unwrap()])
        .collect();
    assert_eq!(
        numbers,
        [
            "8", "9", "10", "11", "12", "1", "2", "3", "4", "5", "6", "7"
        ]
    );
    for (lang, source) in [
        ("en", include_str!("../../kb/en/chart.md")),
        ("hu", include_str!("../../kb/hu/chart.md")),
    ] {
        for style in ["south", "east", "north"] {
            let link = format!("(/{lang}/chart?{HORAS_CHART}&style={style})");
            assert!(source.contains(&link), "{lang}: {style}");
        }
        for text in ["17°14′", "7°25′", "16°32′"] {
            assert!(source.contains(text), "{lang}: {text}");
        }
    }
}

/// Tromsø, 69°39′N 18°57′E, at the winter solstice: no Vedic day.
pub(crate) const TROMSO: [(&str, &str); 5] = [
    ("date", "2000-12-21"),
    ("time", "12%3A00"),
    ("offset", "%2B01%3A00"),
    ("latitude", "69.65"),
    ("longitude", "18.95"),
];

/// Chart 1's query with each `(field, value)` set (values URL-encoded).
pub(crate) fn with(fields: &[(&str, &str)]) -> String {
    for (field, _) in fields {
        // A field chart 1 lacks would leave the test on chart 1 itself.
        assert!(
            CHART_1
                .split('&')
                .any(|p| p.starts_with(&format!("{field}="))),
            "{field}"
        );
    }
    CHART_1
        .split('&')
        .map(|pair| {
            let name = pair.split_once('=').map_or(pair, |(name, _)| name);
            match fields.iter().find(|(field, _)| *field == name) {
                Some((field, value)) => format!("{field}={value}"),
                None => pair.to_owned(),
            }
        })
        .collect::<Vec<_>>()
        .join("&")
}

#[tokio::test]
async fn a_retrograde_graha_shows_r_in_its_row() {
    // Chart 4 (ch. 9 *Arudha Padas*, p. 98), whose Jupiter hora-chart's golden test marks retrograde.
    let chart_4 = "date=1968-03-19&time=18%3A30&utc=stated&offset=%2B05%3A30\
            &latitude=17.116666666666667&longitude=82.31666666666666";
    assert!(graha(&view(Lang::En, chart_4).await, "graha-jupiter").retrograde);
    // The table marks it.
    let page = body(&format!("/en/chart?{chart_4}")).await;
    let table = table(&page);
    assert!(row(table, &[">Jupiter<", ">R<"]).is_some(), "{table}");
}

/// The drawn chart and its legend.
pub(crate) fn drawn(page: &str) -> &str {
    let start = page.find("<figure>").expect(page);
    &page[start..start + page[start..].find("</figure>").unwrap()]
}

#[tokio::test]
async fn chart_1_is_drawn_in_each_style_in_both_languages() {
    // Chart 1's lagna is in Virgo, its Saturn in Aries (ch. 9 *Arudha Padas*, p. 87); its mean Rahu moves backward.
    for (lang, title, rahu, saturn, legend) in [
        (
            "en",
            "Rasi chart",
            "Rahu, retrograde, 5 degrees 55 minutes in Cancer",
            ">Sa<",
            "R marks a retrograde graha.",
        ),
        (
            "hu",
            "Rasi chakra",
            "Rahu, retrográd, 5 fok 55 perc, Rák",
            ">Sz<",
            "Az R retrográd grahát jelöl.",
        ),
    ] {
        for style in ["south", "north", "east"] {
            let page = body(&format!("/{lang}/chart?{}", with(&[("style", style)]))).await;
            let drawn = drawn(&page);
            for text in [
                title,
                rahu,
                saturn,
                legend,
                r#"<tspan class="lagna">La</tspan>"#,
            ] {
                assert!(drawn.contains(text), "{lang} {style} {text}: {drawn}");
            }
            // Only North Indian numbers its areas, the 1st bhava Virgo's 6, and explains them.
            let north = drawn.contains(r#"y="222" class="mark" aria-hidden="true">6<"#);
            assert_eq!(north, style == "north", "{lang} {style}: {drawn}");
            assert_eq!(
                drawn.matches(" class=\"mark\" aria-hidden").count() == 12,
                north
            );
        }
    }
}

#[tokio::test]
async fn hl_and_gl_are_drawn_in_each_style_but_not_without_a_vedic_day() {
    for (lang, legend, hora, ghati) in [
        (
            "en",
            "HL and GL are the hora and ghati lagnas.",
            "Hora lagna in ",
            "Ghati lagna in ",
        ),
        (
            "hu",
            "A HL és a GL a hora és a ghati lagna.",
            "Hora lagna, ",
            "Ghati lagna, ",
        ),
    ] {
        for style in ["south", "north", "east"] {
            let page = body(&format!("/{lang}/chart?{}", with(&[("style", style)]))).await;
            let chakra = drawn(&page);
            for text in [
                r#"<tspan class="special">HL</tspan>"#,
                r#"<tspan class="special">GL</tspan>"#,
                &format!(r#"aria-label="{hora}"#),
                &format!(r#"aria-label="{ghati}"#),
                legend,
            ] {
                assert!(chakra.contains(text), "{lang} {style} {text}: {chakra}");
            }
            assert!(
                !chakra.contains(">SL<") && !chakra.contains(">BL<"),
                "{chakra}"
            );
            let polar = with(&[TROMSO.as_slice(), &[("style", style)]].concat());
            let polar = body(&format!("/{lang}/chart?{polar}")).await;
            let chakra = drawn(&polar);
            assert!(!chakra.contains("class=\"special\""), "{chakra}");
            assert!(!chakra.contains(legend), "{chakra}");
        }
    }
}

/// The request and display choices the page reads from `query`, a test query whose only escapes are `:` and `+`.
pub(crate) fn parsed(query: &str) -> (hora_swe::Request, Shown) {
    let form = query
        .split('&')
        .filter_map(|pair| pair.split_once('='))
        .map(|(name, value)| {
            let value = value.replace("%3A", ":").replace("%2B", "+");
            (name.to_owned(), value)
        })
        .collect();
    parse(&form, None).ok().expect(query)
}

/// The sky the page calculates for `query`.
pub(crate) async fn sky_of(query: &str) -> Sky {
    hora_swe::calculate(parsed(query).0).await.unwrap()
}

/// What the chart page at `/{lang}/chart?{query}` shows.
pub(crate) async fn view(lang: Lang, query: &str) -> ChartView {
    let (request, shown) = parsed(query);
    let sky = hora_swe::calculate(request.clone()).await.unwrap();
    let uri: Uri = format!("/{}/chart?{query}", lang.code()).parse().unwrap();
    ChartView::new(lang, &uri, &request, &sky, shown).expect(query)
}

/// A message in `lang`.
pub(crate) fn t(lang: Lang, id: &str) -> String {
    LOCALES.lookup(&lang.id(), id)
}

/// An English message.
pub(crate) fn en(id: &str) -> String {
    t(Lang::En, id)
}

/// The grahas table's row of the body `id`.
pub(crate) fn graha<'a>(view: &'a ChartView, id: &str) -> &'a Row {
    view.grahas.iter().find(|row| row.name == id).expect(id)
}

/// A grahas, upagraha or special lagna row as the page writes it in English: name, rasi, bhava, position,
/// nakshatra, pada and Vimsottari lord.
fn row_texts(row: &Row) -> [String; 7] {
    [
        en(&row.name),
        en(row.rasi),
        row.bhava.to_string(),
        row.position.clone(),
        format!("{} ({})", en(row.nakshatra), row.nakshatra_number),
        row.pada.to_string(),
        en(row.lord),
    ]
}

/// The row named `name` in English among `rows`, as `row_texts` writes it.
pub(crate) fn row_named<'a>(rows: impl IntoIterator<Item = &'a Row>, name: &str) -> [String; 7] {
    let mut rows = rows.into_iter();
    row_texts(rows.find(|row| en(&row.name) == name).expect(name))
}

/// The panchaanga's limb `name` as (value, lord's name in `lang`, or ""), or `None` without it.
pub(crate) fn limb(lang: Lang, view: &ChartView, name: &str) -> Option<(String, String)> {
    let limb = view.panchaanga.iter().find(|limb| limb.name == name)?;
    Some((
        limb.value.clone(),
        limb.lord.map_or(String::new(), |l| t(lang, l)),
    ))
}

/// The Vedic day's clock time of `name` (`vedic-day-…`).
pub(crate) fn vedic_day<'a>(view: &'a ChartView, name: &str) -> &'a str {
    let day = view.vedic_day.as_ref().expect("a Vedic day");
    &day.iter().find(|(id, _)| *id == name).expect(name).1
}

/// The (start, end) of the limb `name`.
pub(crate) fn span<'a>(view: &'a ChartView, name: &str) -> (&'a str, &'a str) {
    let span = view.limb_spans.iter().find(|s| s.name == name).expect(name);
    (&span.start, &span.end)
}

/// An English abbreviation.
pub(crate) fn short(id: &str) -> String {
    en(&format!("{id}-short"))
}

/// Graha abbreviations as a table cell writes them: comma-separated, or "—".
pub(crate) fn shorts(ids: &[&str]) -> String {
    match ids {
        [] => "—".to_owned(),
        ids => ids
            .iter()
            .map(|id| short(id))
            .collect::<Vec<_>>()
            .join(", "),
    }
}

#[tokio::test]
async fn the_varga_chakra_is_navamsa_by_default_and_the_link_row_picks_another() {
    for (varga, query, heading) in [
        (Varga::Navamsa, CHART_1.to_owned(), "Navamsa (D-9)"),
        (
            Varga::Dasamsa,
            format!("{CHART_1}&varga=d10"),
            "Dasamsa (D-10)",
        ),
    ] {
        let page = body(&format!("/en/chart?{query}")).await;
        let section = section(&page, "varga");
        let n = varga.parts();
        let title = format!(r#"<a href="/en/kb/vargas#d-{n}">{heading}</a></h2>"#);
        assert!(section.contains(&title), "{section}");
        // Nineteen links, D-2 to D-60, each the chart with its varga; only the shown one is current.
        assert_eq!(section.matches("&#38;varga=d").count(), 19, "{section}");
        // The chakras head the page, so a link to one lands at the top, not past the header.
        assert!(!section.contains("#varga"), "{section}");
        let current = format!(
            r#"href="/en/chart?{}&#38;varga=d{n}" aria-current="page" aria-label="{heading}">D-{n}</a>"#,
            CHART_1.replace('&', "&#38;")
        );
        assert!(section.contains(&current), "{current}: {section}");
        assert_eq!(section.matches("aria-current").count(), 1, "{section}");
        assert!(
            section.contains(r#"aria-label="Shashtyamsa (D-60)">D-60</a>"#),
            "{section}"
        );
        assert!(section.contains("rasis only"), "{section}");
    }
}

#[tokio::test]
async fn each_body_sits_in_its_varga_rasi_in_each_style_but_hl_and_gl_not_without_a_vedic_day() {
    let sky = sky_of(CHART_1).await;
    let chart = hora_chart::chart(&sky);
    let specials: Vec<_> = hora_chart::special_lagnas(&sky)
        .into_iter()
        .filter(|(l, _)| matches!(l, SpecialLagna::HoraLagna | SpecialLagna::GhatiLagna))
        .map(|(l, longitude)| (kb::message_id("special-lagna", l), longitude, false))
        .collect();
    let bodies = chart
        .entries()
        .iter()
        .map(|e| (body_id(e.body).to_owned(), e.longitude, e.retrograde))
        .chain(specials);
    let bodies: Vec<_> = bodies.collect();
    assert_eq!(bodies.len(), 12);
    for varga in [Varga::Navamsa, Varga::Dasamsa, Varga::Shashtyamsa] {
        for style in ["south", "north", "east"] {
            let query = format!("{}&varga=d{}", with(&[("style", style)]), varga.parts());
            let page = body(&format!("/en/chart?{query}")).await;
            let chakra = section(&page, "varga");
            for (id, longitude, retrograde) in &bodies {
                let rasi = en(rasi_id(hora_chart::varga(varga, *longitude)));
                let r = if *retrograde { ", retrograde," } else { "" };
                let label = format!(r#"aria-label="{}{r} in {rasi}""#, en(id));
                assert!(
                    chakra.contains(&label),
                    "{varga:?} {style} {label}: {chakra}"
                );
            }
            assert!(!chakra.contains('°'), "{chakra}");
        }
    }
    let row = section(&body(&format!("/en/chart?{CHART_1}")).await, "varga").to_owned();
    assert!(!row.contains(">D-1<"), "{row}");
    assert!(row.contains(">D-2<"), "{row}");
    let polar = body(&format!("/en/chart?{}", with(&TROMSO))).await;
    let chakra = section(&polar, "varga");
    assert!(chakra.contains("<svg"), "{chakra}");
    assert!(!chakra.contains(r#"class="special""#), "{chakra}");
}

#[tokio::test]
async fn an_unknown_varga_is_refused() {
    // D-1 is the rasi chart, always drawn beside the varga chakra, so it is not one of its choices.
    for varga in ["d1", "d13", "D9", "9", "d"] {
        let response = get(&format!("/en/chart?{CHART_1}&varga={varga}"), None).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST, "{varga}");
        let page = page(response).await;
        let error =
            r#"<strong id="varga-error" class="error">Choose one of the listed options.</strong>"#;
        assert!(page.contains(error), "{varga}: {page}");
        assert!(!page.contains("<table"), "{page}");
    }
}

#[tokio::test]
async fn the_north_indian_varga_chakra_counts_from_the_reference_points_varga_rasi() {
    let query = with(&[("reference", "hora-lagna"), ("style", "north")]);
    let sky = sky_of(&query).await;
    let (_, hl) = hora_chart::special_lagnas(&sky)
        .into_iter()
        .find(|(l, _)| *l == SpecialLagna::HoraLagna)
        .unwrap();
    for varga in [Varga::Navamsa, Varga::Drekkana] {
        let page = body(&format!("/en/chart?{query}&varga=d{}", varga.parts())).await;
        let first = hora_chart::varga(varga, hl) as u8 + 1;
        let top = format!(r#"y="222" class="mark" aria-hidden="true">{first}<"#);
        assert!(section(&page, "varga").contains(&top), "{varga:?} {top}");
    }
}

#[tokio::test]
async fn the_vargas_table_has_a_row_per_varga_and_a_column_per_point() {
    let sky = sky_of(CHART_1).await;
    let chart = hora_chart::chart(&sky);
    let mut points: Vec<_> = chart.entries().iter().map(|e| e.longitude).collect();
    for lagna in [
        SpecialLagna::HoraLagna,
        SpecialLagna::GhatiLagna,
        SpecialLagna::SreeLagna,
    ] {
        let special = hora_chart::special_lagnas(&sky)
            .into_iter()
            .find(|s| s.0 == lagna);
        points.push(special.unwrap().1);
    }
    let page = body(&format!("/en/chart?{CHART_1}&topic=vargas")).await;
    let vargas = section(&page, "vargas");
    assert!(vargas.contains(r#"<a href="/en/kb/vargas">"#), "{vargas}");
    let chart_1 = view(Lang::En, CHART_1).await;
    assert_eq!(chart_1.vargas.len(), 20);
    for (varga, row) in hora_chart::VARGAS.into_iter().zip(&chart_1.vargas) {
        let n = varga.parts();
        let name = en(&kb::message_id("varga", varga));
        assert_eq!((row.parts, &*row.name), (n, &*format!("{name} (D-{n})")));
        let rasis: Vec<_> = points
            .iter()
            .map(|&longitude| rasi_id(hora_chart::varga(varga, longitude)))
            .collect();
        assert_eq!(row.rasis, rasis, "{varga:?}");
    }
    let shorts = |view: &ChartView| -> Vec<String> {
        view.varga_columns.iter().map(|(_, s)| en(s)).collect()
    };
    let columns = shorts(&chart_1);
    for short in ["La", "Su", "Ke", "HL", "GL", "SL"] {
        assert!(columns.iter().any(|c| c == short), "{short}");
    }
    let polar = shorts(&view(Lang::En, &with(&TROMSO)).await);
    assert!(!polar.iter().any(|c| c == "HL" || c == "GL"), "{polar:?}");
    assert!(polar.iter().any(|c| c == "SL"), "{polar:?}");
}

#[tokio::test]
async fn the_amsabala_table_counts_each_graha_over_the_four_varga_groups() {
    let chart = hora_chart::chart(&sky_of(CHART_1).await);
    for lang in [Lang::En, Lang::Hu] {
        let page = body(&format!("/{}/chart?{CHART_1}&topic=vargas", lang.code())).await;
        let amsabala = section(&page, "amsabala");
        assert!(
            amsabala.contains(&format!(r#"<a href="/{}/kb/amsabala">"#, lang.code())),
            "{amsabala}"
        );
        for group in hora_chart::VARGA_GROUPS {
            let name = t(lang, &kb::message_id("varga-group", group));
            assert!(amsabala.contains(&format!(">{name}</th>")), "{name}");
        }
    }
    let view = view(Lang::En, CHART_1).await;
    assert_eq!(view.amsabala.len(), 9);
    for (entry, (id, cells)) in chart.grahas().iter().zip(&view.amsabala) {
        assert_eq!(*id, body_id(entry.body));
        let expected = hora_chart::VARGA_GROUPS.map(|group| {
            let (count, amsa) = hora_chart::amsabala(entry.body, entry.longitude, group);
            (count, amsa.map(|a| kb::message_id("amsa", a)))
        });
        assert_eq!(*cells, expected, "{id}");
    }
}

#[tokio::test]
async fn bhavas_are_counted_from_the_chosen_reference_point() {
    // Chart 1's lagna is in Virgo, its Moon in Gemini, its Sun in Pisces (ch. 9 *Arudha Padas*, p. 87).
    let moon = with(&[("reference", "moon")]);
    for (query, [lagna, moon, sun]) in [(CHART_1, [1, 10, 7]), (&moon, [4, 1, 10])] {
        let view = view(Lang::En, query).await;
        for (id, rasi, bhava) in [
            ("lagna", "rasi-virgo", lagna),
            ("graha-moon", "rasi-gemini", moon),
            ("graha-sun", "rasi-pisces", sun),
        ] {
            let row = graha(&view, id);
            assert_eq!((row.rasi, row.bhava), (rasi, bhava), "{id}");
        }
    }
    let from_moon = body(&format!("/en/chart?{}", with(&[("reference", "moon")]))).await;
    assert!(
        from_moon.contains(r#"value="moon" selected"#),
        "{from_moon}"
    );
}

#[tokio::test]
async fn bhavas_are_counted_from_a_chosen_special_lagna() {
    for reference in ["hora-lagna", "ghati-lagna", "sree-lagna"] {
        let query = with(&[("reference", reference), ("style", "north")]);
        let view = view(Lang::En, &query).await;
        let id = format!("special-lagna-{reference}");
        let row = view.special_lagnas.iter().find(|row| row.name == id);
        let row = row.expect(reference);
        assert_eq!(row.bhava, 1, "{reference}");
        let number = hora_chart::RASIS
            .iter()
            .position(|r| rasi_id(*r) == row.rasi)
            .unwrap()
            + 1;
        let page = body(&format!("/en/chart?{query}")).await;
        let top = format!(r#"y="222" class="mark" aria-hidden="true">{number}<"#);
        assert!(drawn(&page).contains(&top), "{reference}: {top}");
        assert!(page.contains(&format!(r#"value="{reference}" selected"#)));
    }
}

#[tokio::test]
async fn bhavas_are_counted_from_paaka_lagna() {
    // Chart 1's lagna is in Virgo, so its paaka lagna is Mercury's rasi (ch. 7 §7.3.5).
    let query = with(&[("reference", "paaka-lagna"), ("style", "north")]);
    let chart = hora_chart::chart(&sky_of(&query).await);
    let paaka = chart.in_varga(Varga::RasiChart).paaka_lagna();
    let page = body(&format!("/en/chart?{query}")).await;
    let mercury_query = with(&[("reference", "mercury"), ("style", "north")]);
    let mercury = body(&format!("/en/chart?{mercury_query}")).await;
    let lagna = body(&format!("/en/chart?{}", with(&[("style", "north")]))).await;
    // The varga's functional nature, the last column, counts from that varga's own paaka lagna (ADR 0016).
    let rasi_chart = |page: &str| -> Vec<String> {
        tbody(page)
            .lines()
            .map(|line| {
                line.rsplit_once("<td>")
                    .map_or(line, |(row, _)| row)
                    .to_owned()
            })
            .collect()
    };
    assert_ne!(
        rasi_chart(&page),
        rasi_chart(&lagna),
        "Mercury is not in Virgo"
    );
    assert_eq!(rasi_chart(&page), rasi_chart(&mercury));
    let page_lagnas = body(&format!("/en/chart?{query}&topic=lagnas")).await;
    let mercury_lagnas = body(&format!("/en/chart?{mercury_query}&topic=lagnas")).await;
    for id in ["special-lagnas", "upagrahas"] {
        assert_eq!(
            section(&page_lagnas, id),
            section(&mercury_lagnas, id),
            "{id}"
        );
    }
    assert_eq!(drawn(&page), drawn(&mercury));
    let top = format!(
        r#"y="222" class="mark" aria-hidden="true">{}<"#,
        paaka as u8 + 1
    );
    assert!(drawn(&page).contains(&top), "{top}");
    assert!(page.contains(r#"value="paaka-lagna" selected>Paaka lagna<"#));
}

#[tokio::test]
async fn the_north_indian_varga_chakra_counts_from_that_vargas_paaka_lagna() {
    let query = with(&[("reference", "paaka-lagna"), ("style", "north")]);
    let chart = hora_chart::chart(&sky_of(&query).await);
    let paaka = chart.in_varga(Varga::Navamsa).paaka_lagna();
    let mercury = chart.entry(hora_chart::Body::Mercury);
    // The varga's own lagna lord decides, not the rasi chart's (ADR 0003).
    assert_ne!(paaka, hora_chart::varga(Varga::Navamsa, mercury.longitude));
    let page = body(&format!("/en/chart?{query}&varga=d9")).await;
    let top = format!(
        r#"y="222" class="mark" aria-hidden="true">{}<"#,
        paaka as u8 + 1
    );
    assert!(section(&page, "varga").contains(&top), "{top}");
}

/// The arudha table row `id`: a bhava arudha's `bhava-arudha-…` or a graha arudha's `arudha-graha-…`.
pub(crate) fn arudha<'a>(view: &'a ChartView, id: &str) -> &'a ArudhaRow {
    let rows = view.bhava_arudhas.iter().chain(&view.graha_arudhas);
    rows.into_iter().find(|row| row.id == id).expect(id)
}

#[tokio::test]
async fn each_topic_shows_both_chakras_and_only_its_sections() {
    let all = [
        r#"<table id="grahas">"#,
        "vedic-day",
        "panchaanga",
        "limb-spans",
        "special-lagnas",
        "upagrahas",
        "vargas",
        "amsabala",
        "arudha-padas",
        "graha-arudhas",
        "aspects",
        "argalas",
        "yogas",
        "ashtakavarga",
        "baadhakas",
        "yogadas",
        "marakas",
        "rudra",
        "three-pairs",
        "eighth-lord",
    ];
    for (topic, shown) in [
        ("", &[r#"<table id="grahas">"#][..]),
        ("&topic=grahas", &[r#"<table id="grahas">"#]),
        ("&topic=day", &["vedic-day", "panchaanga", "limb-spans"]),
        ("&topic=lagnas", &["special-lagnas", "upagrahas"]),
        ("&topic=vargas", &["vargas", "amsabala"]),
        ("&topic=arudhas", &["arudha-padas", "graha-arudhas"]),
        ("&topic=aspects", &["aspects", "argalas"]),
        ("&topic=yogas", &["yogas"]),
        ("&topic=ashtakavarga", &["ashtakavarga"]),
        ("&topic=interpreting", &["baadhakas", "yogadas"]),
        (
            "&topic=longevity",
            &["marakas", "rudra", "three-pairs", "eighth-lord"],
        ),
    ] {
        let page = body(&format!("/en/chart?{CHART_1}{topic}")).await;
        assert!(
            drawn(&page).contains(r#"aria-labelledby="chakra-title""#),
            "{topic}"
        );
        assert!(section(&page, "varga").contains("<svg"), "{topic}");
        assert!(
            page.contains(r#"<a href="/en/kb/ayanamsa">Ayanamsa</a>: "#),
            "{topic}"
        );
        for id in all {
            let marker = if id.starts_with('<') {
                id.to_owned()
            } else {
                format!(r#"<section aria-labelledby="{id}">"#)
            };
            assert_eq!(page.contains(&marker), shown.contains(&id), "{topic}: {id}");
        }
    }
}

#[tokio::test]
async fn the_contents_and_every_link_keep_the_whole_view() {
    let keys = [
        "grahas",
        "day",
        "lagnas",
        "vargas",
        "arudhas",
        "aspects",
        "yogas",
        "ashtakavarga",
        "interpreting",
        "longevity",
        "strength",
    ];
    let state = "&varga=d10&sort=karaka";
    for (i, topic) in keys.into_iter().enumerate() {
        let page = body(&format!("/en/chart?{CHART_1}{state}&topic={topic}")).await;
        // Askama escapes `&` in attributes.
        let href =
            |key: &str| format!("/en/chart?{CHART_1}{state}&topic={key}").replace('&', "&#38;");
        let start = page.find(r#"<nav class="toc""#).unwrap();
        let toc = &page[start..start + page[start..].find("</nav>").unwrap()];
        assert!(toc.contains("<h3>Chart Analysis</h3>"), "{toc}");
        let links: Vec<_> = toc.split("<li>").skip(1).collect();
        assert_eq!(links.len(), keys.len(), "{toc}");
        for (j, (link, key)) in links.iter().zip(keys).enumerate() {
            assert!(
                link.starts_with(&format!(r#"<a href="{}""#, href(key))),
                "{link}"
            );
            assert_eq!(
                link.contains(r#"aria-current="page""#),
                i == j,
                "{topic}: {link}"
            );
        }
        for (key, fact) in [
            (
                "grahas",
                "<small>ch. 1–3, 7–8</small><span>Lagna Virgo</span>",
            ),
            ("day", "<small>ch. 1</small><span>Sunday</span>"),
            ("vargas", "<small>ch. 6</small><span>Dasamsa (D-10)</span>"),
            ("aspects", "<small>ch. 10</small></a>"),
            ("ashtakavarga", "<small>ch. 12</small></a>"),
            ("interpreting", "<small>ch. 13</small></a>"),
            ("longevity", "<small>ch. 14</small></a>"),
        ] {
            let link = links[keys.iter().position(|k| *k == key).unwrap()];
            assert!(link.contains(fact), "{link}");
        }
        assert!(links[6].contains(" found</span>"), "{}", links[6]);
        let start = page.find(r#"<nav class="path">"#).unwrap();
        let path = &page[start..start + page[start..].find("</nav>").unwrap()];
        let previous = format!(
            r#"<a rel="prev" href="{}">"#,
            href(keys.get(i.wrapping_sub(1)).unwrap_or(&""))
        );
        assert_eq!(path.contains(&previous), i > 0, "{topic}: {path}");
        assert_eq!(path.contains(r#"rel="prev""#), i > 0, "{topic}: {path}");
        let next = keys
            .get(i + 1)
            .map(|key| format!(r#"<a rel="next" href="{}">"#, href(key)));
        assert_eq!(
            next.is_some(),
            path.contains(r#"rel="next""#),
            "{topic}: {path}"
        );
        assert!(
            next.is_none_or(|next| path.contains(&next)),
            "{topic}: {path}"
        );
        // The varga row and the grahas table's headers keep the topic too.
        let d9 = format!("/en/chart?{CHART_1}&sort=karaka&topic={topic}&varga=d9\"");
        assert!(page.contains(&d9.replace('&', "&#38;")), "{topic}");
        for hidden in [("varga", "d10"), ("sort", "karaka"), ("topic", topic)] {
            let input = format!(
                r#"<input type="hidden" name="{}" value="{}">"#,
                hidden.0, hidden.1
            );
            assert!(page.contains(&input), "{topic}: {input}");
        }
        // Each topic is its own page, so the tab and history name it.
        let title = links[i]
            .split("<span>")
            .nth(1)
            .unwrap()
            .split("</span>")
            .next()
            .unwrap();
        assert!(
            page.contains(&format!("<title>{title} – hora</title>")),
            "{topic}"
        );
        // Cast, the birth form folds into one closed line.
        let fold = r#"<details class="birth">
<summary><span>2000-04-09 · 17:55 · 42.5, -71.2</span> <span>Change birth data</span></summary>"#;
        assert!(page.contains(fold), "{topic}");
    }
    let sort = format!("/en/chart?{CHART_1}&varga=d10&topic=day&sort=graha#grahas");
    let page = body(&format!("/en/chart?{CHART_1}{state}&topic=day")).await;
    // The grahas table is not on the day topic, so its sort links are not either.
    assert!(!page.contains(&sort.replace('&', "&#38;")));
}

#[tokio::test]
async fn without_a_vedic_day_the_contents_name_no_vaara_and_no_hora_lagna() {
    let page = body(&format!("/hu/chart?{}", with(&TROMSO))).await;
    let start = page.find(r#"<nav class="toc""#).unwrap();
    let toc = &page[start..start + page[start..].find("</nav>").unwrap()];
    assert!(toc.contains("<h3>Képletelemzés</h3>"), "{toc}");
    assert!(toc.contains("<small>1. fej.</small></a>"), "{toc}");
    assert!(toc.contains("<small>4–5. fej.</small></a>"), "{toc}");
    assert!(toc.contains(" találat</span>"), "{toc}");
}

#[tokio::test]
async fn bhavas_are_counted_from_the_atma_karaka() {
    // Chart 1's AK is the Sun, 26°29′ Pisces (ch. 9 *Arudha Padas*, p. 87; ch. 8 §8.2).
    let query = with(&[("reference", "atma-karaka"), ("style", "north")]);
    let sky = sky_of(&query).await;
    let karakas = hora_chart::chart(&sky).chara_karakas();
    assert!(karakas.contains(&(hora_chart::Body::Sun, CharaKaraka::Atma)));
    let page = body(&format!("/en/chart?{query}")).await;
    let sun_query = with(&[("reference", "sun"), ("style", "north")]);
    let sun = body(&format!("/en/chart?{sun_query}")).await;
    assert_eq!(tbody(&page), tbody(&sun));
    assert_eq!(graha(&view(Lang::En, &query).await, "lagna").bhava, 7);
    let page_lagnas = body(&format!("/en/chart?{query}&topic=lagnas")).await;
    let sun_lagnas = body(&format!("/en/chart?{sun_query}&topic=lagnas")).await;
    for id in ["special-lagnas", "upagrahas"] {
        assert_eq!(section(&page_lagnas, id), section(&sun_lagnas, id), "{id}");
    }
    assert_eq!(drawn(&page), drawn(&sun));
    assert!(page.contains(r#"value="atma-karaka" selected>Atma karaka<"#));
}

#[tokio::test]
async fn the_north_indian_navamsa_counts_from_the_karakamsa() {
    // Karakamsa: the AK's navamsa rasi (ch. 7 §7.3.6, ADR 0003).
    let query = with(&[("reference", "atma-karaka"), ("style", "north")]);
    let sky = sky_of(&query).await;
    let karakamsa = hora_chart::varga(Varga::Navamsa, sky.grahas.sun.longitude);
    assert_ne!(karakamsa, hora_chart::rasi(sky.grahas.sun.longitude).0);
    let page = body(&format!("/en/chart?{query}&varga=d9")).await;
    let top = format!(
        r#"y="222" class="mark" aria-hidden="true">{}<"#,
        karakamsa as u8 + 1
    );
    assert!(section(&page, "varga").contains(&top), "{top}");
}

#[test]
fn every_chara_karaka_has_a_name_and_an_abbreviation() {
    let en = keys(include_str!("../../locales/en/main.ftl"));
    let missing: Vec<_> = hora_chart::CHARA_KARAKAS
        .into_iter()
        .flat_map(|k| {
            let id = kb::message_id("chara-karaka", k);
            [format!("{id}-short"), id]
        })
        .filter(|id| !en.contains(id.as_str()))
        .collect();
    assert_eq!(missing, [""; 0]);
}

#[tokio::test]
async fn a_polar_birth_offers_and_takes_only_the_sree_lagna_among_special_lagnas() {
    let chart = body(&format!("/en/chart?{}", with(&TROMSO))).await;
    let start = chart.find(r#"<select id="reference""#).unwrap();
    let select = &chart[start..start + chart[start..].find("</select>").unwrap()];
    assert!(select.contains(r#"value="sree-lagna""#), "{select}");
    assert!(!select.contains(r#"value="hora-lagna""#), "{select}");
    assert!(!select.contains(r#"value="ghati-lagna""#), "{select}");
    for reference in ["hora-lagna", "ghati-lagna"] {
        let query = with(&[TROMSO.as_slice(), &[("reference", reference)]].concat());
        let response = get(&format!("/en/chart?{query}"), None).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST, "{reference}");
        let page = page(response).await;
        assert!(
                page.contains(r#"<strong id="reference-error" class="error">There is no Vedic day here, so only the sree lagna: the others count from sunrise.</strong>"#),
                "{page}"
            );
        assert!(!page.contains("<figure>"), "{reference}");
        // The refused choice stays selected beside its error.
        assert!(page.contains(&format!(r#"value="{reference}" selected"#)));
    }
    let query = with(&[TROMSO.as_slice(), &[("reference", "sree-lagna")]].concat());
    assert_eq!(
        get(&format!("/en/chart?{query}"), None).await.status(),
        StatusCode::OK
    );
}

/// The section headed `id`, which follows the chart table.
pub(crate) fn section<'a>(page: &'a str, id: &str) -> &'a str {
    let start = page
        .find(&format!(r#"<section aria-labelledby="{id}">"#))
        .expect(id);
    &page[start..start + page[start..].find("</section>").expect("a section end")]
}

#[tokio::test]
async fn an_overloaded_pool_answers_busy_with_retry_after() {
    for (lang, text) in [
        (Lang::En, "Try again in a few seconds."),
        (Lang::Hu, "néhány másodperc múlva"),
    ] {
        let response = failure(
            lang,
            &Uri::from_static("/en/chart"),
            hora_swe::Error::Overloaded,
        );
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(response.headers()[header::RETRY_AFTER], "5");
        assert!(page(response).await.contains(text));
    }
}

#[tokio::test]
async fn an_ephemeris_failure_is_a_generic_500_page() {
    let response = failure(
        Lang::Hu,
        &Uri::from_static("/hu/chart"),
        hora_swe::Error::Ephemeris,
    );
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert!(!response.headers().contains_key(header::RETRY_AFTER));
    let page = page(response).await;
    assert!(
        page.contains("A chakrát nem sikerült kiszámítani."),
        "{page}"
    );
    assert!(!page.contains("ephemeris"), "{page}");
}

/// The Fluent message (and term) ids: lines that start an entry at column 0.
pub(crate) fn keys(ftl: &str) -> BTreeSet<&str> {
    ftl.lines()
        .filter(|line| !line.starts_with([' ', '#', '.']))
        .filter_map(|line| line.split_once('=').map(|(id, _)| id.trim()))
        .collect()
}

#[test]
fn degrees_are_truncated_and_never_shown_as_30() {
    assert_eq!(dms(29.0 + 59.0 / 60.0 + 59.9 / 3600.0), "29°59′59″");
    assert_eq!(dms(30.0 - 1e-15), "29°59′59″");
    assert_eq!(dms(0.0), "0°00′00″");
}

#[test]
fn clock_times_are_truncated_and_never_shown_as_24() {
    // JD 2451644.5 is 2000-04-09 0h; it holds about 40 µs, so 1 ms before midnight is representable.
    let at = |seconds: f64| 2_451_644.5 + seconds / 86400.0;
    assert_eq!(hms(at(6.0 * 3600.0 + 13.0 * 60.0 + 7.9)), "06:13:07");
    assert_eq!(hms(at(86400.0 - 1e-3)), "23:59:59");
    assert_eq!(hms(at(0.0)), "00:00:00");
}

#[test]
fn both_languages_have_the_same_messages() {
    let en = keys(include_str!("../../locales/en/main.ftl"));
    let hu = keys(include_str!("../../locales/hu/main.ftl"));
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

#[test]
fn every_rasi_nakshatra_and_panchaanga_value_has_a_message() {
    use hora_chart::{KARANAS, LUNAR_MONTHS, NAKSHATRAS, RASIS, SUN_MOON_YOGAS, TITHIS};
    let en = keys(include_str!("../../locales/en/main.ftl"));
    let vaaras = [
        Vaara::Sunday,
        Vaara::Monday,
        Vaara::Tuesday,
        Vaara::Wednesday,
        Vaara::Thursday,
        Vaara::Friday,
        Vaara::Saturday,
    ];
    let ids = RASIS
        .map(rasi_id)
        .into_iter()
        .chain(NAKSHATRAS.map(nakshatra_id))
        .chain(TITHIS.map(tithi_id))
        .chain([Paksha::Sukla, Paksha::Krishna].map(paksha_id))
        .chain(SUN_MOON_YOGAS.map(sun_moon_yoga_id))
        .chain(KARANAS.map(karana_id))
        .chain(vaaras.map(vaara_id))
        .chain(LUNAR_MONTHS.map(lunar_month_id))
        .chain([TwinMonth::Adhika, TwinMonth::Nija].map(twin_month_id));
    let missing: Vec<_> = ids.filter(|id| !en.contains(id)).collect();
    assert_eq!(missing, [""; 0]);
}

#[test]
fn every_special_lagna_has_a_name_and_an_abbreviation() {
    let en = keys(include_str!("../../locales/en/main.ftl"));
    let missing: Vec<_> = hora_chart::SPECIAL_LAGNAS
        .into_iter()
        .flat_map(|l| ["special-lagna", "special-lagna-abbreviation"].map(|p| kb::message_id(p, l)))
        .filter(|id| !en.contains(id.as_str()))
        .collect();
    assert_eq!(missing, [""; 0]);
}

#[test]
fn every_varga_has_a_name() {
    let en = keys(include_str!("../../locales/en/main.ftl"));
    let missing: Vec<_> = hora_chart::VARGAS
        .map(|v| kb::message_id("varga", v))
        .into_iter()
        .filter(|id| !en.contains(id.as_str()))
        .collect();
    assert_eq!(missing, [""; 0]);
}

#[test]
fn every_varga_group_and_amsa_has_a_name() {
    let en = keys(include_str!("../../locales/en/main.ftl"));
    let missing: Vec<_> = hora_chart::VARGA_GROUPS
        .into_iter()
        .flat_map(|g| {
            [kb::message_id("varga-group", g)]
                .into_iter()
                .chain(g.amsas().iter().map(|a| kb::message_id("amsa", a)))
        })
        .filter(|id| !en.contains(id.as_str()))
        .collect();
    assert_eq!(missing, [""; 0]);
}
