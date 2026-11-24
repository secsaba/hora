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

/// The KB's worked examples: noon UT, 1 January 2000, at Greenwich.
pub(crate) const GREENWICH_2000: &str = "date=2000-01-01&time=12%3A00&calendar=gregorian&utc=stated&offset=%2B00%3A00\
        &latitude=51.48&longitude=0&altitude=&ayanamsa=lahiri&node=mean&sunrise=upper-limb-apparent&reference=lagna&style=south";

/// hora's own example chart, the chart KB page's, without a style.
pub(crate) const HORAS_CHART: &str = "date=2000-01-01&time=06%3A00&calendar=gregorian&utc=stated&offset=%2B00%3A00\
        &latitude=51.48&longitude=0&altitude=&ayanamsa=lahiri&node=mean&sunrise=upper-limb-apparent&reference=lagna";

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

/// The drawn chart and its legend.
pub(crate) fn drawn(page: &str) -> &str {
    let start = page.find("<figure>").expect(page);
    &page[start..start + page[start..].find("</figure>").unwrap()]
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

/// The arudha table row `id`: a bhava arudha's `bhava-arudha-…` or a graha arudha's `arudha-graha-…`.
pub(crate) fn arudha<'a>(view: &'a ChartView, id: &str) -> &'a ArudhaRow {
    let rows = view.bhava_arudhas.iter().chain(&view.graha_arudhas);
    rows.into_iter().find(|row| row.id == id).expect(id)
}

/// The section headed `id`, which follows the chart table.
pub(crate) fn section<'a>(page: &'a str, id: &str) -> &'a str {
    let start = page
        .find(&format!(r#"<section aria-labelledby="{id}">"#))
        .expect(id);
    &page[start..start + page[start..].find("</section>").expect("a section end")]
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
