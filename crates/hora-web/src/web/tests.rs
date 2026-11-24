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
    app(places(), None, jiff::Timestamp::now)
        .oneshot(request.body(Body::empty()).unwrap())
        .await
        .unwrap()
}

/// The dev shell's places file, loaded once for every test.
fn places() -> Arc<Places> {
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
fn tbody(page: &str) -> &str {
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
const HORAS_CHART: &str = "date=2000-01-01&time=06%3A00&calendar=gregorian&utc=stated&offset=%2B00%3A00\
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

/// Saturn in Aries is neecha, sama with Mars (ch. 3 §3.3–3.4); the graha-relationships page works its example on this chart.
#[tokio::test]
async fn the_chart_page_shows_each_grahas_dignity_and_rasi_lord_as_the_kb_example_says() {
    let view = view(Lang::En, &format!("{HORAS_CHART}&style=south")).await;
    for (id, dignity, rasi_lord, relationship) in [
        (
            "graha-saturn",
            Some("dignity-neecha"),
            "graha-mars",
            "relationship-sama",
        ),
        (
            "graha-jupiter",
            None,
            "graha-mars",
            "relationship-adhimitra",
        ),
        ("graha-mercury", None, "graha-jupiter", "relationship-satru"),
        ("graha-venus", None, "graha-mars", "relationship-mitra"),
    ] {
        let row = graha(&view, id);
        assert_eq!(
            (
                row.dignity.as_deref(),
                row.rasi_lord,
                row.relationship.as_deref()
            ),
            (dignity, rasi_lord, Some(relationship)),
            "{id}"
        );
    }
    // The lagna, in Scorpio, and Rahu, in Cancer, have no dignity and only their rasi's lord.
    for (point, rasi_lord) in [("lagna", "graha-mars"), ("graha-rahu", "graha-moon")] {
        let row = graha(&view, point);
        assert_eq!(
            (
                row.dignity.as_deref(),
                row.rasi_lord,
                row.relationship.as_deref()
            ),
            (None, rasi_lord, None),
            "{point}"
        );
    }
    // Each links to its knowledge base section.
    for (lang, saturn) in [("en", "Saturn"), ("hu", "Szaturnusz")] {
        let page = body(&format!("/{lang}/chart?{HORAS_CHART}&style=south")).await;
        let kb = format!("/{lang}/kb");
        let cells = [
            format!(">{saturn}<"),
            format!(r#"<td><a href="{kb}/dignity#neecha">neecha</a></td>"#),
            format!(
                r#"<td><a href="{kb}/graha-characteristics#mars">Mars</a> (<a href="{kb}/graha-relationships#compound">sama</a>)</td>"#
            ),
        ];
        let cells = cells.each_ref().map(String::as_str);
        assert!(row(table(&page), &cells).is_some(), "{lang}: {page}");
    }
    for (lang, source) in [
        ("en", include_str!("../../kb/en/graha-relationships.md")),
        ("hu", include_str!("../../kb/hu/graha-relationships.md")),
    ] {
        let link = format!("(/{lang}/chart?{HORAS_CHART}&style=south)");
        assert!(source.contains(&link), "{lang}");
        for term in ["neecha", "sama", "adhimitra", "satru"] {
            assert!(source.contains(term), "{lang}: {term}");
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

/// Ch. 4: eleven upagrahas with a Vedic day, only the five Sun-based without one.
#[tokio::test]
async fn the_chart_page_shows_each_upagrahas_rasi_nakshatra_and_lords() {
    let tromso = with(&TROMSO);
    for (lang, query, upagrahas) in [
        ("en", HORAS_CHART, 11),
        ("hu", HORAS_CHART, 11),
        ("en", tromso.as_str(), 5),
    ] {
        let page = body(&format!("/{lang}/chart?{query}&style=south&topic=lagnas")).await;
        let section = section(&page, "upagrahas");
        let kb = format!("/{lang}/kb");
        let rows: Vec<_> = section.split("<tr>").skip(2).collect();
        assert_eq!(rows.len(), upagrahas, "{section}");
        // Only a chart without a Vedic day says why six are missing.
        assert_eq!(section.contains("<p>"), upagrahas == 5, "{section}");
        for (i, row) in rows.iter().enumerate() {
            let group = if hora_chart::UPAGRAHAS[i].is_sun_based() {
                "sun-based"
            } else {
                "time-based"
            };
            for (href, count) in [
                (format!("{kb}/upagraha#{group}"), 1),
                (format!("{kb}/rasi-characteristics#"), 1),
                (format!("{kb}/bhava"), 1),
                (format!("{kb}/nakshatra"), 1),
                // The Vimsottari lord and the rasi lord.
                (format!("{kb}/graha-characteristics#"), 2),
            ] {
                let href = format!(r#"<a href="{href}"#);
                assert_eq!(row.matches(&href).count(), count, "{href}: {row}");
            }
        }
    }
}

/// Ch. 5: HL, GL, SL and BL with a Vedic day, only SL without one.
#[tokio::test]
async fn the_chart_page_shows_each_special_lagnas_rasi_nakshatra_and_lords() {
    let tromso = with(&TROMSO);
    for (lang, query, names) in [
        (
            "en",
            HORAS_CHART,
            &["hora-lagna", "ghati-lagna", "sree-lagna", "bhaava-lagna"][..],
        ),
        (
            "hu",
            HORAS_CHART,
            &["hora-lagna", "ghati-lagna", "sree-lagna", "bhaava-lagna"],
        ),
        ("en", tromso.as_str(), &["sree-lagna"]),
    ] {
        let page = body(&format!("/{lang}/chart?{query}&style=south&topic=lagnas")).await;
        let section = section(&page, "special-lagnas");
        let kb = format!("/{lang}/kb");
        let rows: Vec<_> = section.split("<tr").skip(2).collect();
        assert_eq!(rows.len(), names.len(), "{section}");
        assert_eq!(section.contains("<p>"), names.len() == 1, "{section}");
        for (row, name) in rows.iter().zip(names) {
            for (href, count) in [
                (format!("{kb}/special-lagna#{name}\""), 1),
                (format!("{kb}/rasi-characteristics#"), 1),
                (format!("{kb}/bhava"), 1),
                (format!("{kb}/nakshatra"), 1),
                // The Vimsottari lord and the rasi lord.
                (format!("{kb}/graha-characteristics#"), 2),
            ] {
                let href = format!(r#"<a href="{href}"#);
                assert_eq!(row.matches(&href).count(), count, "{href}: {row}");
            }
        }
    }
}

/// The upagraha page works its example on hora's chart: a Friday night, Saturn's part the 5th, the birth in the Sun's 7th.
#[tokio::test]
async fn the_upagraha_pages_example_is_horas_chart() {
    let view = view(Lang::En, &format!("{HORAS_CHART}&style=south")).await;
    let upagraha =
        |view: &ChartView, name| row_named(view.upagrahas.iter().map(|(_, row)| row), name);
    // Bhavas from the lagna, in Scorpio.
    let examples = [
        ("Dhuma", "Aries", 6, "29°35′59″"),
        ("Indrachaapa", "Libra", 12, "29°35′59″"),
        ("Vyatipaata", "Pisces", 5, "0°24′"),
        ("Parivesha", "Virgo", 11, "0°24′"),
        ("Upaketu", "Scorpio", 1, "16°15′59″"),
        ("Gulika", "Virgo", 11, "13°41′"),
        ("Maandi", "Virgo", 11, "24°20′"),
        ("Kaala", "Scorpio", 1, "6°55′"),
    ];
    for (name, rasi, bhava, position) in examples {
        let row = upagraha(&view, name);
        assert_eq!([&*row[1], &*row[2]], [rasi, &*bhava.to_string()], "{name}");
        assert!(row[3].starts_with(position), "{name}: {row:?}");
    }
    // From the Moon, in Libra, Upaketu in Scorpio is in the 2nd.
    let from_moon = HORAS_CHART.replace("reference=lagna", "reference=moon");
    let moon_view = self::view(Lang::En, &format!("{from_moon}&style=south")).await;
    assert_eq!(upagraha(&moon_view, "Upaketu")[1..3], ["Scorpio", "2"]);
    let sun = row_named(&view.grahas, "Sun");
    assert_eq!(sun[1], "Sagittarius");
    assert!(sun[3].starts_with("16°15′59″"), "{sun:?}");
    let day = view.vedic_day.as_ref().expect("a Vedic day");
    assert!(
        day.iter()
            .any(|(_, time)| time == "15:59:58 (December 31, 1999)")
    );
    assert!(day.iter().any(|(_, time)| time.ends_with("08:05:46")));
    assert_eq!(
        limb(Lang::En, &view, "vaara")
            .map(|(value, _)| value)
            .as_deref(),
        Some("Friday")
    );
    for (lang, source) in [
        ("en", include_str!("../../kb/en/upagraha.md")),
        ("hu", include_str!("../../kb/hu/upagraha.md")),
    ] {
        let link = format!("(/{lang}/chart?{HORAS_CHART}&style=south)");
        assert!(source.contains(&link), "{lang}");
        for (_, _, _, position) in examples {
            assert!(source.contains(position), "{lang}: {position}");
        }
        for text in [
            "256°15′59″",
            "389°35′59″",
            "330°24′",
            "15:59:58",
            "08:05:46",
        ] {
            assert!(source.contains(text), "{lang}: {text}");
        }
    }
}

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

/// Chart 1 with `field` set to `value` (URL-encoded) gives 400 with `message` next to that field, keeping `value`.
async fn assert_field_error(field: &str, value: &str, message: &str) {
    let query = with(&[(field, value)]);
    let response = get(&format!("/en/chart?{query}"), None).await;
    assert_eq!(
        response.status(),
        StatusCode::BAD_REQUEST,
        "{field}={value}"
    );
    let page = page(response).await;
    let error = format!(r#"<strong id="{field}-error" class="error">{message}</strong>"#);
    assert!(page.contains(&error), "{field}={value}: {page}");
    assert!(page.contains(r#"aria-invalid="true""#), "{page}");
    assert!(!page.contains("<table"), "{page}");
    // Every other field keeps its value.
    assert!(
        page.contains(r#"value="42.5""#) || field == "latitude",
        "{page}"
    );
}

#[tokio::test]
async fn a_bad_date_is_refused_next_to_its_field() {
    for date in [
        "2000-02-30",
        "2000-4-31",
        "9-04-09",
        "2000-04",
        "2000-04-09x",
        "",
    ] {
        assert_field_error("date", date, DATE_ERROR).await;
    }
}

const DATE_ERROR: &str = "Enter a real date as YYYY-MM-DD, from 1200-01-04 to 2399-12-28 (in the Julian calendar, to 2399-12-12).";

#[tokio::test]
async fn a_julian_date_past_the_window_is_refused() {
    // Julian 2399-12-20 is Gregorian 2400-01-05, past the files' end.
    let query = with(&[("calendar", "julian"), ("date", "2399-12-20")]);
    let response = get(&format!("/en/chart?{query}"), None).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let page = page(response).await;
    let error = format!(r#"<strong id="date-error" class="error">{DATE_ERROR}</strong>"#);
    assert!(page.contains(&error), "{page}");
}

#[tokio::test]
async fn every_out_of_range_field_is_refused_at_once() {
    let query = with(&[
        ("latitude", "95"),
        ("offset", "%2B15%3A00"),
        ("time", "24%3A00"),
    ]);
    let response = get(&format!("/en/chart?{query}"), None).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let page = page(response).await;
    for field in ["latitude", "offset", "time"] {
        assert!(
            page.contains(&format!(r#"id="{field}-error""#)),
            "{field}: {page}"
        );
    }
    assert!(!page.contains(r#"id="date-error""#), "{page}");
}

#[tokio::test]
async fn an_unknown_calendar_does_not_judge_the_date() {
    let query = with(&[("calendar", "bogus"), ("date", "1500-03-01")]);
    let response = get(&format!("/en/chart?{query}"), None).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let page = page(response).await;
    assert!(page.contains(r#"id="calendar-error""#), "{page}");
    assert!(!page.contains(r#"id="date-error""#), "{page}");
}

#[tokio::test]
async fn the_dates_the_messages_name_are_charts() {
    let first_gregorian = with(&[("date", "1582-10-15")]);
    body(&format!("/en/chart?{first_gregorian}")).await;
    // The ends `error-date` names, at the clock time and offset that push them furthest out.
    for (calendar, date, time, offset) in [
        ("julian", "1200-01-04", "00%3A00", "%2B14%3A00"),
        ("gregorian", "2399-12-28", "23%3A59%3A59", "-12%3A00"),
        ("julian", "2399-12-12", "23%3A59%3A59", "-12%3A00"),
    ] {
        let query = with(&[
            ("calendar", calendar),
            ("date", date),
            ("time", time),
            ("offset", offset),
        ]);
        let response = get(&format!("/en/chart?{query}"), None).await;
        assert_eq!(response.status(), StatusCode::OK, "{query}");
    }
}

#[tokio::test]
async fn every_option_in_the_form_is_a_chart() {
    let home = body("/en/").await;
    for name in [
        "calendar",
        "utc",
        "ayanamsa",
        "node",
        "sunrise",
        "reference",
        "style",
    ] {
        let start = home.find(&format!(r#"<select id="{name}""#)).expect(name);
        let select = &home[start..start + home[start..].find("</select>").expect(name)];
        let values: Vec<_> = select
            .split(r#"<option value=""#)
            .skip(1)
            .filter_map(|option| option.split_once('"').map(|(value, _)| value))
            .collect();
        assert!(!values.is_empty(), "{select}");
        for value in values {
            // A zone needs a chosen place at its coordinates: Boston, near chart 1's.
            let query = match value {
                "zone" => format!(
                    "{}&place=Boston&place-id=4930956",
                    with(&[
                        (name, value),
                        ("latitude", "42.35843"),
                        ("longitude", "-71.05977")
                    ])
                ),
                _ => with(&[(name, value)]),
            };
            let response = get(&format!("/en/chart?{query}"), None).await;
            assert_eq!(response.status(), StatusCode::OK, "{name}={value}");
        }
    }
}

#[tokio::test]
async fn a_gregorian_date_before_1582_10_15_is_refused() {
    let message = "The Gregorian calendar starts on 1582-10-15; choose Julian for an earlier date.";
    assert_field_error("date", "1582-10-14", message).await;
    // The same day in the Julian calendar is a chart.
    let julian = with(&[("date", "1582-10-04"), ("calendar", "julian")]);
    body(&format!("/en/chart?{julian}")).await;
}

#[tokio::test]
async fn a_latitude_out_of_range_is_refused() {
    for latitude in ["90", "-90", "91", "north", ""] {
        assert_field_error(
            "latitude",
            latitude,
            "Enter the latitude in degrees, between −90 and 90.",
        )
        .await;
    }
    assert_field_error(
        "longitude",
        "180.5",
        "Enter the longitude in degrees, from −180 to 180.",
    )
    .await;
    assert_field_error(
        "altitude",
        "9001",
        "Enter the altitude in metres, from −500 to 9000, or leave it empty.",
    )
    .await;
}

#[tokio::test]
async fn nan_and_infinity_are_refused() {
    for value in ["NaN", "nan", "inf", "-infinity"] {
        assert_field_error(
            "latitude",
            value,
            "Enter the latitude in degrees, between −90 and 90.",
        )
        .await;
        assert_field_error(
            "longitude",
            value,
            "Enter the longitude in degrees, from −180 to 180.",
        )
        .await;
        assert_field_error(
            "altitude",
            value,
            "Enter the altitude in metres, from −500 to 9000, or leave it empty.",
        )
        .await;
    }
}

#[tokio::test]
async fn a_bad_or_out_of_range_offset_is_refused() {
    let message = "Enter the offset as +HH:MM or −HH:MM, from −12:00 to +14:00.";
    for offset in [
        "-12%3A00%3A01",
        "%E2%88%9212%3A00%3A01",
        "%2B14%3A01",
        "%2B5",
        "%2B05%3A60",
        "05%3A30%3A00%3A00",
        "--4%3A00",
        "",
    ] {
        assert_field_error("offset", offset, message).await;
    }
    assert_field_error("time", "24%3A00", "Enter the time as HH:MM or HH:MM:SS.").await;
}

#[test]
fn each_choice_starts_on_its_adr_0003_default() {
    assert_eq!(CALENDARS[0].1, Calendar::default());
    assert_eq!(AYANAMSAS[0].1, Ayanamsa::default());
    assert_eq!(NODES[0].1, NodeType::default());
    assert_eq!(SUNRISES[0].1, SunriseDefinition::default());
}

#[tokio::test]
async fn a_choice_left_out_is_its_default() {
    let home = body("/en/").await;
    for value in [
        "gregorian",
        "stated",
        "lahiri",
        "mean",
        "upper-limb-apparent",
        "lagna",
        "south",
    ] {
        assert!(
            home.contains(&format!(r#"value="{value}" selected"#)),
            "{value}"
        );
    }
    let blank = with(&[
        ("calendar", ""),
        ("utc", ""),
        ("ayanamsa", ""),
        ("node", ""),
        ("sunrise", ""),
        ("reference", ""),
        ("style", ""),
    ]);
    let blank_query = blank;
    let (blank, chart_1) = (
        body(&format!("/en/chart?{blank_query}")).await,
        body(&format!("/en/chart?{CHART_1}")).await,
    );
    assert_eq!(tbody(&blank), tbody(&chart_1));
    assert_eq!(drawn(&blank), drawn(&chart_1));
    assert!(
        drawn(&blank).contains("M0 0H480V480H0ZM120 0V480"),
        "South Indian"
    );
    let day = |query: &str| format!("/en/chart?{query}&topic=day");
    let (blank_day, chart_1_day) = (body(&day(&blank_query)).await, body(&day(CHART_1)).await);
    assert_eq!(
        section(&blank_day, "vedic-day"),
        section(&chart_1_day, "vedic-day")
    );
}

/// The drawn chart and its legend.
fn drawn(page: &str) -> &str {
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
fn parsed(query: &str) -> (hora_swe::Request, Shown) {
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
async fn sky_of(query: &str) -> Sky {
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
fn short(id: &str) -> String {
    en(&format!("{id}-short"))
}

/// Graha abbreviations as a table cell writes them: comma-separated, or "—".
fn shorts(ids: &[&str]) -> String {
    match ids {
        [] => "—".to_owned(),
        ids => ids
            .iter()
            .map(|id| short(id))
            .collect::<Vec<_>>()
            .join(", "),
    }
}

/// An aspects table's rows as the page writes them in English: the graha, then each drishti's rasis and grahas.
fn aspect_texts(rows: &[AspectRow]) -> Vec<[String; 3]> {
    let aspected = |a: &Aspected| match a.rasis.as_slice() {
        [] => "—".to_owned(),
        rasis => {
            let rasis: Vec<_> = rasis
                .iter()
                .map(|(rasi, bhava)| format!("{} ({bhava})", short(rasi)))
                .collect();
            format!("{} / {}", rasis.join(", "), shorts(&a.grahas))
        }
    };
    rows.iter()
        .map(|r| {
            [
                en(r.graha),
                aspected(&r.graha_drishti),
                aspected(&r.rasi_drishti),
            ]
        })
        .collect()
}

/// An argala table's rows as the page writes them in English: bhava, rasi (`*` when reversed), then each pair.
fn argala_texts(rows: &[ArgalaRow]) -> Vec<Vec<String>> {
    rows.iter()
        .map(|r| {
            let mark = if r.reversed { " *" } else { "" };
            let mut cells = vec![r.bhava.to_string(), format!("{}{mark}", en(r.rasi))];
            for (argala, virodhargala) in &r.pairs {
                cells.extend([shorts(argala), shorts(virodhargala)]);
            }
            cells
        })
        .collect()
}

/// A yoga table's rows across its classes as the page writes them in `lang`: name, grahas, conditions to judge, notes,
/// closeness and amsas, "—" for each one missing.
fn yoga_texts(lang: Lang, groups: &[YogaGroup]) -> Vec<Vec<String>> {
    let or_dash = |cells: Vec<String>, by: &str| match cells.is_empty() {
        true => "—".to_owned(),
        false => cells.join(by),
    };
    groups
        .iter()
        .flat_map(|group| &group.rows)
        .map(|row| {
            let grahas = row
                .grahas
                .iter()
                .map(|(id, place)| format!("{} ({place})", t(lang, id)))
                .collect();
            let (closeness, amsas) = row.magnitude.as_ref().map_or_else(
                || ("—".to_owned(), "—".to_owned()),
                |(closeness, amsas)| {
                    let amsas = amsas
                        .iter()
                        .map(|(id, count, amsa)| {
                            let amsa = amsa.as_ref().map_or("—".to_owned(), |a| t(lang, a));
                            format!("{}: {count} {amsa}", t(lang, id))
                        })
                        .collect();
                    (closeness.clone(), or_dash(amsas, " / "))
                },
            );
            vec![
                row.name.clone(),
                or_dash(grahas, ", "),
                or_dash(row.to_judge.clone(), " / "),
                or_dash(row.notes.clone(), " / "),
                closeness,
                amsas,
            ]
        })
        .collect()
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
async fn an_unknown_choice_is_refused() {
    for field in [
        "calendar",
        "utc",
        "ayanamsa",
        "node",
        "sunrise",
        "reference",
        "style",
    ] {
        assert_field_error(field, "fagan", "Choose one of the listed options.").await;
    }
}

#[tokio::test]
async fn the_name_sound_is_one_of_table_37s_five_groups_or_not_given() {
    // Ch. 15 §15.4.4, Table 37 (p. 193); ADR 0018.
    for sound in ["", "1", "2", "3", "4", "5"] {
        let page = body(&format!("/en/chart?{}", with(&[("sound", sound)]))).await;
        let option = format!(r#"<option value="{sound}" selected>"#);
        assert!(page.contains(&option), "{sound}: {page}");
    }
    for sound in ["0", "6", "01", "x", "-1"] {
        assert_field_error("sound", sound, "Choose one of the listed options.").await;
        let response = get(&format!("/hu/chart?{}", with(&[("sound", sound)])), None).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST, "{sound}");
        let page = page(response).await;
        assert!(
            page.contains("Válasszon a felsorolt lehetőségek közül."),
            "{sound}: {page}"
        );
    }
}

#[test]
fn a_url_without_a_sound_is_a_chart_without_one() {
    let query = CHART_1.replace("&sound=", "");
    assert_eq!(parsed(&query).1.sound, None);
    assert_eq!(parsed(&with(&[("sound", "5")])).1.sound, Some(5));
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

/// Chart 3 (ch. 9 *Arudha Padas*, p. 97), at the seconds its golden test is cast with.
const CHART_3: &str = "date=1926-12-25&time=05%3A12%3A30&utc=stated&offset=%2B05%3A30\
        &latitude=26.233333333333334&longitude=78.16666666666667";

/// The `<tr` of the body or arudha `id` in `html`.
fn body_row<'a>(html: &'a str, id: &str) -> &'a str {
    let at = format!(r#" data-body="{id}">"#);
    html.split("<tr").find(|r| r.starts_with(&at)).expect(id)
}

/// The arudha table row `id`: a bhava arudha's `bhava-arudha-…` or a graha arudha's `arudha-graha-…`.
fn arudha<'a>(view: &'a ChartView, id: &str) -> &'a ArudhaRow {
    let rows = view.bhava_arudhas.iter().chain(&view.graha_arudhas);
    rows.into_iter().find(|row| row.id == id).expect(id)
}

#[tokio::test]
async fn the_arudha_tables_show_chart_3s_arudha_padas() {
    // Ch. 20, p. 265: "Cp contains AL and A5"; Example 32, p. 96: Jupiter's and Rahu's arudha padas in Libra. Lagna in Scorpio.
    let view = view(Lang::En, CHART_3).await;
    for id in ["bhava-arudha-1", "bhava-arudha-5"] {
        let row = arudha(&view, id);
        assert_eq!((row.rasi, row.bhava), ("rasi-capricorn", 3), "{id}");
    }
    assert_eq!(view.bhava_arudhas.len(), 12);
    for id in ["arudha-graha-jupiter", "arudha-graha-rahu"] {
        let row = arudha(&view, id);
        assert_eq!((row.rasi, row.bhava), ("rasi-libra", 12), "{id}");
    }
    assert_eq!(view.graha_arudhas.len(), 9);
    let page = body(&format!("/en/chart?{CHART_3}&topic=arudhas")).await;
    let arudha_padas = section(&page, "arudha-padas");
    assert!(
        arudha_padas.contains(r#"<a href="/en/kb/arudha-padas">Arudha padas</a></h2>"#),
        "{arudha_padas}"
    );
    for (name, short) in [
        ("Arudha lagna", "AL"),
        ("Arudha pada of the 5th bhava", "A5"),
        ("Darapada", "A7"),
        ("Upapada lagna", "UL"),
    ] {
        assert!(
            arudha_padas.contains(&format!(r#"<abbr title="{name}">{short}</abbr>"#)),
            "{arudha_padas}"
        );
    }
    let grahas = section(&page, "graha-arudhas");
    assert!(
        grahas.contains(r#"<a href="/en/kb/arudha-padas">Graha arudhas</a></h2>"#),
        "{grahas}"
    );
    assert!(
        grahas.contains(
            r##"<th scope="row"><a href="/en/kb/graha-characteristics#jupiter">Jupiter</a></th>"##
        ),
        "{grahas}"
    );
    // Hover lights the row, which needs its own rule.
    for id in ["bhava-arudha-1", "arudha-graha-rahu"] {
        assert!(page.contains(&format!(r#"<tr data-body="{id}">"#)), "{id}");
        assert!(
            page.contains(&format!("[data-body={id}] {{ --lit: 1; }}")),
            "{id}"
        );
    }
    let hu = body(&format!("/hu/chart?{CHART_3}&topic=arudhas")).await;
    assert!(section(&hu, "arudha-padas").contains("Arudha padák"));
    assert!(
        section(&hu, "arudha-padas")
            .contains(r#"<abbr title="Az 5. bhava arudha padája">A5</abbr>"#)
    );
    assert!(section(&hu, "graha-arudhas").contains("Graha arudhák"));
}

#[tokio::test]
async fn the_arudha_tables_varga_columns_hold_exercises_12_and_13() {
    // Chart 2: chart 1's birth at UTC−5, in D-16 (ch. 9 §9.8, p. 99); its D-16 lagna counts the varga's Bhava with the lagna reference.
    let query = with(&[("offset", "-05%3A00"), ("time", "17%3A55%3A51")]);
    let query = format!("{query}&varga=d16");
    let view = view(Lang::En, &query).await;
    let sky = sky_of(&query).await;
    let lagna = hora_chart::varga(Varga::Shodasamsa, hora_chart::chart(&sky).lagna().longitude);
    let d16 = "Shodasamsa (D-16)";
    assert_eq!(view.varga_name, d16);
    let page = body(&format!("/en/chart?{query}&topic=arudhas")).await;
    for id in ["arudha-padas", "graha-arudhas"] {
        let header = format!(r#"<th scope="colgroup" colspan="2">{d16}</th>"#);
        assert!(section(&page, id).contains(&header), "{id}");
    }
    use Rasi::*;
    for (n, rasi) in [
        Aquarius,
        Aries,
        Sagittarius,
        Scorpio,
        Sagittarius,
        Aquarius,
        Pisces,
        Virgo,
        Taurus,
        Sagittarius,
        Sagittarius,
        Aquarius,
    ]
    .into_iter()
    .enumerate()
    {
        let row = arudha(&view, &arudha_id(n));
        let shown = (rasi_id(rasi), hora_chart::bhava(lagna, rasi));
        assert_eq!((row.varga_rasi, row.varga_bhava), shown, "{}", row.id);
    }
    // The page writes each row's cells as the chakra's rasi and bhava, then the varga's.
    let al = arudha(&view, &arudha_id(0));
    let cell = |rasi: &str, bhava: u8| {
        let slug = rasi.trim_start_matches("rasi-");
        format!(
            r##"<td><a href="/en/kb/rasi-characteristics#{slug}">{}</a></td><td><a href="/en/kb/bhava">{bhava}</a></td>"##,
            en(rasi)
        )
    };
    let cells = cell(al.rasi, al.bhava) + &cell(al.varga_rasi, al.varga_bhava);
    assert!(section(&page, "arudha-padas").contains(&cells), "{cells}");
    use hora_chart::Body::*;
    for (graha, rasi) in [
        (Sun, Scorpio),
        (Moon, Sagittarius),
        (Mars, Gemini),
        (Mercury, Pisces),
        (Jupiter, Scorpio),
        (Venus, Aquarius),
        (Saturn, Sagittarius),
        (Rahu, Virgo),
        (Ketu, Pisces),
    ] {
        let row = arudha(&view, &format!("arudha-{}", body_id(graha)));
        let shown = (rasi_id(rasi), hora_chart::bhava(lagna, rasi));
        assert_eq!((row.varga_rasi, row.varga_bhava), shown, "{}", row.id);
    }
}

/// Chart 5 (ch. 10 *Aspects and Argalas*, p. 101), at the seconds its golden test is cast with.
fn chart_5() -> String {
    with(&[
        ("date", "2000-04-19"),
        ("time", "19%3A21%3A57"),
        ("latitude", "42.666666666666664"),
        ("longitude", "-71.13333333333334"),
    ])
}

/// Chart 6 (ch. 10 *Aspects and Argalas*, p. 107) as the book prints it.
fn chart_6() -> String {
    with(&[
        ("date", "1921-06-28"),
        ("time", "12%3A49"),
        ("offset", "%2B05%3A17"),
        ("latitude", "18.433333333333334"),
        ("longitude", "79.15"),
    ])
}

/// A section's table for the rasi chart and, in its `<details>`, for the varga shown.
fn both_tables(section: &str) -> (&str, &str) {
    section.split_once("<details>").expect("a details")
}

#[tokio::test]
async fn the_aspects_tables_hold_exercises_14_and_15_in_chart_5s_navamsa() {
    let query = format!("{}&varga=d9", chart_5());
    let page = body(&format!("/en/chart?{query}&topic=aspects")).await;
    let aspects = section(&page, "aspects");
    assert!(
        aspects.contains(r#"<a href="/en/kb/aspects-and-argalas">Aspects</a></h2>"#),
        "{aspects}"
    );
    assert!(
        aspects.contains("<details>\n<summary>Navamsa (D-9)</summary>"),
        "{aspects}"
    );
    let view = view(Lang::En, &query).await;
    assert_eq!(view.rasi.aspects.len(), 9);
    let rows = aspect_texts(&view.varga.aspects);
    // Ch. 10 §10.8, pp. 109–110: the rasis and houses in count order, the planets as the book lists them.
    assert_eq!(
        rows[2],
        [
            "Mars",
            "Aq (4), Ta (7), Ge (8) / Ke, Su",
            "Cp (3), Ar (6), Cn (9) / Ve, Mo",
        ]
    );
    assert_eq!(
        rows[4],
        [
            "Jupiter",
            "Cp (3), Pi (5), Ta (7) / Ve, Su",
            "Sg (2), Pi (5), Ge (8) / Me",
        ]
    );
    // Exercise 14 leaves out the nodes (ADR 0013); Exercise 15 lists them.
    assert_eq!(rows[7], ["Rahu", "—", "Li (12), Cp (3), Ar (6) / Ve, Mo"]);
    assert_eq!(rows[8], ["Ketu", "—", "Ar (6), Cn (9), Li (12) / Mo"]);
    // Moon's graha drishti falls on Libra, which holds no graha.
    assert_eq!(rows[1][1], "Li (12) / —");
    assert!(!aspects.contains("Lagna"), "{aspects}");
    // Hover lights the chakra and the grahas table through the rules their rows already have.
    assert!(
        aspects.contains(r#"<tr data-body="graha-mars">"#),
        "{aspects}"
    );
    assert!(aspects.contains(r#"<th scope="col">Graha drishti</th>"#));
    assert!(aspects.contains(r#"<th scope="col">Rasi drishti</th>"#));
    let hu = body(&format!("/hu/chart?{query}&topic=aspects")).await;
    assert!(section(&hu, "aspects").contains("Aspektusok"));
}

#[tokio::test]
async fn the_aspects_tables_bhavas_follow_the_reference_point_and_nothing_else_does() {
    let at = |reference: &'static str| async move {
        let view = view(Lang::En, &format!("{}&reference={reference}", chart_6())).await;
        aspect_texts(&view.rasi.aspects)
    };
    let (lagna, saturn) = (at("lagna").await, at("saturn").await);
    // Saturn in Leo: the Sun in Gemini aspects Sagittarius, the 5th from him.
    assert!(saturn[0][1].starts_with("Sg (5)"), "{:?}", saturn[0]);
    assert_ne!(lagna, saturn);
    let without_bhavas = |rows: Vec<[String; 3]>| {
        rows.concat()
            .join("|")
            .replace(|c: char| c.is_ascii_digit(), "")
    };
    assert_eq!(without_bhavas(lagna), without_bhavas(saturn));
}

#[tokio::test]
async fn the_argala_tables_hold_exercise_16_in_chart_5s_navamsa() {
    let query = format!("{}&varga=d9", chart_5());
    let page = body(&format!("/en/chart?{query}&topic=aspects")).await;
    let argalas = section(&page, "argalas");
    assert!(
        argalas.contains(r#"<a href="/en/kb/aspects-and-argalas">Argalas</a></h2>"#),
        "{argalas}"
    );
    assert!(
        argalas.contains("<details>\n<summary>Navamsa (D-9)</summary>"),
        "{argalas}"
    );
    let (rasi_chart, navamsa) = both_tables(argalas);
    let view = view(Lang::En, &query).await;
    assert_eq!(view.rasi.argalas.len(), 12);
    let rows = argala_texts(&view.varga.argalas);
    assert_eq!(rows.len(), 12);
    // Ch. 10 §10.8, pp. 110–111, each argala beside its obstruction: 2nd/12th, 4th/10th, 5th/9th, 11th/3rd.
    assert_eq!(
        rows[10],
        [
            "11", "Virgo", "—", "Ra", "Me", "—", "Ve", "Su", "—", "Ma, Sa"
        ]
    );
    // The book's asterisk: Ketu is in Aquarius, so its counts run in reverse.
    assert_eq!(
        rows[3],
        [
            "4",
            "Aquarius *",
            "Ve",
            "—",
            "Ma, Sa",
            "Su",
            "—",
            "—",
            "Mo",
            "Me"
        ]
    );
    assert_eq!(rows.iter().filter(|r| r[1].ends_with('*')).count(), 1);
    assert!(navamsa.contains("Counted in reverse: this rasi holds Ketu."));
    assert!(!argalas.contains("Lagna"), "{argalas}");
    // Real headers, one group per pair, for screen readers.
    for pair in ["2nd / 12th", "4th / 10th", "5th / 9th", "11th / 3rd"] {
        let header = format!(r#"<th scope="colgroup" colspan="2">{pair}</th>"#);
        assert!(rasi_chart.contains(&header), "{header}: {rasi_chart}");
    }
    assert_eq!(rasi_chart.matches(r#"<colgroup span="2">"#).count(), 5);
    assert_eq!(
        rasi_chart.matches(r#"<th scope="col">Argala</th>"#).count(),
        4
    );
    assert_eq!(
        rasi_chart
            .matches(r#"<th scope="col">Virodhargala</th>"#)
            .count(),
        4
    );
    // §10.6's several malefics are not computed (ADR 0013).
    assert!(
        argalas.contains(r##"<a href="/en/kb/aspects-and-argalas#virodhargala">"##),
        "{argalas}"
    );
    let hu = body(&format!("/hu/chart?{query}&topic=aspects")).await;
    let hu = section(&hu, "argalas");
    assert!(hu.contains("Argalák"), "{hu}");
    assert!(hu.contains(r#"<th scope="colgroup" colspan="2">11. / 3.</th>"#));
    assert!(hu.contains("Visszafelé számolva"), "{hu}");
}

#[tokio::test]
async fn the_varga_tables_count_from_the_reference_point_in_the_varga() {
    // Chart 5's D-9 (p. 101): the Moon in Aries, the Sun in Taurus aspecting Scorpio, the 8th from Aries.
    let view = view(Lang::En, &format!("{}&varga=d9&reference=moon", chart_5())).await;
    let argalas = argala_texts(&view.varga.argalas);
    assert_eq!(argalas[0][..2], ["1", "Aries"]);
    let aspects = aspect_texts(&view.varga.aspects);
    assert!(aspects[0][1].starts_with("Sc (8)"), "{:?}", aspects[0]);
}

#[tokio::test]
async fn example_35_counting_from_saturn_makes_leo_the_first_argala_row() {
    // Ch. 10 §10.7, pp. 107–108: "Mercury, Mars and Sun have an argala on Saturn, as they are in the 11th".
    let view = view(Lang::En, &format!("{}&reference=saturn", chart_6())).await;
    let rows = argala_texts(&view.rasi.argalas);
    assert_eq!(rows[0][..2], ["1", "Leo"]);
    assert_eq!(rows[0][8], "Su, Ma, Me");
}

/// The aspects page works its example on hora's chart, counted from Saturn.
#[tokio::test]
async fn the_aspects_pages_example_is_horas_chart() {
    let from_saturn = HORAS_CHART.replace("reference=lagna", "reference=saturn");
    let view_from_saturn = view(Lang::En, &format!("{from_saturn}&style=south")).await;
    let aspects = aspect_texts(&view_from_saturn.rasi.aspects);
    assert_eq!(
        aspects[6],
        [
            "Saturn",
            "Ge (3), Li (7), Cp (10) / Mo, Ke",
            "Le (5), Sc (8), Aq (11) / Ve, Ma",
        ]
    );
    assert_eq!(
        aspects[4],
        [
            "Jupiter",
            "Le (5), Li (7), Sg (9) / Mo, Su, Me",
            "Le (5), Sc (8), Aq (11) / Ve, Ma",
        ]
    );
    assert_eq!(aspects[1][1], "Ar (1) / Ju, Sa");
    // The Moon alone aspects Saturn by graha drishti; Mars and Venus by rasi drishti.
    for (i, row) in aspects.iter().enumerate() {
        assert_eq!(row[1].contains("Ar (1)"), i == 1, "{row:?}");
        assert_eq!(
            row[2].contains("Ar (1)"),
            row[0] == "Mars" || row[0] == "Venus",
            "{row:?}"
        );
    }
    let rows = argala_texts(&view_from_saturn.rasi.argalas);
    assert_eq!(
        rows[0],
        ["1", "Aries", "—", "—", "Ra", "Ke", "—", "Su, Me", "Ma", "—"]
    );
    // Ketu's rasi, counted in reverse.
    assert_eq!(
        rows[9],
        [
            "10",
            "Capricorn *",
            "Su, Me",
            "Ma",
            "Mo",
            "Ju, Sa",
            "—",
            "—",
            "—",
            "Ve"
        ]
    );
    // From the lagna, in Scorpio, Saturn's rasi is the 6th row, with the same grahas.
    let from_lagna = view(Lang::En, &format!("{HORAS_CHART}&style=south")).await;
    let rows = argala_texts(&from_lagna.rasi.argalas);
    assert_eq!(
        rows[5],
        ["6", "Aries", "—", "—", "Ra", "Ke", "—", "Su, Me", "Ma", "—"]
    );
    for (lang, source) in [
        ("en", include_str!("../../kb/en/aspects-and-argalas.md")),
        ("hu", include_str!("../../kb/hu/aspects-and-argalas.md")),
    ] {
        let link = format!("(/{lang}/chart?{from_saturn}&style=south)");
        assert!(source.contains(&link), "{lang}");
    }
}

#[test]
fn the_arudha_padas_page_sends_rasi_drishti_to_the_aspects_page() {
    for (lang, source) in [
        ("en", include_str!("../../kb/en/arudha-padas.md")),
        ("hu", include_str!("../../kb/hu/arudha-padas.md")),
    ] {
        let link = format!("(/{lang}/kb/aspects-and-argalas#rasi-drishti)");
        assert_eq!(source.matches(&link).count(), 2, "{lang}");
    }
}

#[tokio::test]
async fn each_bhava_arudha_is_drawn_once_in_each_chakra_in_its_tables_rasi() {
    for style in ["south", "north", "east"] {
        let query = format!("{CHART_3}&style={style}&varga=d10");
        let page = body(&format!("/en/chart?{query}")).await;
        let view = view(Lang::En, &query).await;
        let chart = hora_chart::chart(&sky_of(CHART_3).await);
        for (chakra, d) in [
            (drawn(&page), Varga::RasiChart),
            (section(&page, "varga"), Varga::Dasamsa),
        ] {
            let arudhas = chart.in_varga(d).bhava_arudhas();
            for (n, rasi) in arudhas.into_iter().enumerate() {
                let id = arudha_id(n);
                let opening = format!(
                    r#"data-body="{id}" role="img" aria-label="{} in {}""#,
                    en(&id),
                    en(rasi_id(rasi))
                );
                assert_eq!(
                    chakra.matches(&opening).count(),
                    1,
                    "{style} {d:?} {opening}: {chakra}"
                );
                assert_eq!(
                    chakra.matches(&format!(r#"data-body="{id}""#)).count(),
                    1,
                    "{style} {d:?} {id}"
                );
                // The row's rasi in the rasi chart, then in the varga.
                let row = arudha(&view, &id);
                let shown = if d == Varga::RasiChart {
                    row.rasi
                } else {
                    row.varga_rasi
                };
                assert_eq!(shown, rasi_id(rasi), "{id} {d:?}");
            }
            // Graha arudhas are not drawn.
            assert!(!chakra.contains("arudha-graha"), "{chakra}");
        }
        for chakra in [drawn(&page), section(&page, "varga")] {
            assert!(
                chakra.contains(r#"<tspan class="arudha-lagna">AL</tspan></text>"#),
                "{chakra}"
            );
            assert!(
                chakra.contains(r#"<tspan class="arudha">A7</tspan></text>"#),
                "{chakra}"
            );
        }
        assert!(page.contains("are the bhava arudhas"), "{style}");
    }
}

#[tokio::test]
async fn bhavas_are_counted_from_the_arudha_and_upapada_lagnas_each_chart_its_own() {
    let chart = hora_chart::chart(&sky_of(CHART_3).await);
    for (reference, name, n) in [
        ("arudha-lagna", "Arudha lagna", 0),
        ("upapada-lagna", "Upapada lagna", 11),
    ] {
        let query = format!("{CHART_3}&reference={reference}&style=north&varga=d10");
        let page = body(&format!("/en/chart?{query}")).await;
        let view = view(Lang::En, &query).await;
        let rasi = chart.in_varga(Varga::RasiChart).bhava_arudhas()[n];
        let d10 = chart.in_varga(Varga::Dasamsa).bhava_arudhas()[n];
        assert!(
            page.contains(&format!(r#"value="{reference}" selected>{name}<"#)),
            "{reference}"
        );
        // The row of the reference point itself is 1st in both charts.
        let row = arudha(&view, &arudha_id(n));
        assert_eq!(
            [(row.rasi, row.bhava), (row.varga_rasi, row.varga_bhava)],
            [(rasi_id(rasi), 1), (rasi_id(d10), 1)],
            "{reference}"
        );
        let top = |rasi: Rasi| {
            format!(
                r#"y="222" class="mark" aria-hidden="true">{}<"#,
                rasi as u8 + 1
            )
        };
        assert!(drawn(&page).contains(&top(rasi)), "{reference}");
        assert!(section(&page, "varga").contains(&top(d10)), "{reference}");
        // Every other table counts from it too: the lagna's row, Scorpio from Capricorn is 11th.
        let lagna = hora_chart::bhava(rasi, Rasi::Scorpio);
        assert_eq!(graha(&view, "lagna").bhava, lagna, "{reference}");
    }
    // The varga chakra's 1st bhava is D-10's AL: Virgo, from a count (p. 303).
    let page = body(&format!(
        "/en/chart?{CHART_3}&reference=arudha-lagna&style=north&varga=d10"
    ))
    .await;
    assert!(section(&page, "varga").contains(r#"y="222" class="mark" aria-hidden="true">6<"#));
    // AL in Capricorn: the rasi chakra's 1st bhava is 10.
    assert!(drawn(&page).contains(r#"y="222" class="mark" aria-hidden="true">10<"#));
}

#[tokio::test]
async fn the_grahas_table_shows_chart_3s_chara_karakas() {
    // Chart 3: AK Mercury and PK Saturn (ch. 8 §8.2, p. 265).
    let view = view(Lang::En, CHART_3).await;
    for (id, karaka) in [
        ("graha-mercury", Some(CharaKaraka::Atma)),
        ("graha-saturn", Some(CharaKaraka::Putra)),
        ("lagna", None),
        ("graha-ketu", None),
    ] {
        assert_eq!(graha(&view, id).karaka, karaka, "{id}");
    }
    for lang in ["en", "hu"] {
        let page = body(&format!("/{lang}/chart?{CHART_3}")).await;
        let cell = format!(
            r#"</th><td><a href="/{lang}/kb/karakas#atma-karaka"><abbr title="Atma karaka">AK</abbr></a></td>"#
        );
        assert!(body_row(&page, "graha-mercury").contains(&cell), "{lang}");
        assert!(
            body_row(&page, "lagna").contains("</th><td>—</td>"),
            "{lang}"
        );
    }
}

#[tokio::test]
async fn the_grahas_table_sorts_by_graha_or_by_chara_karaka() {
    let by_graha = [
        "lagna",
        "graha-sun",
        "graha-moon",
        "graha-mars",
        "graha-mercury",
        "graha-jupiter",
        "graha-venus",
        "graha-saturn",
        "graha-rahu",
        "graha-ketu",
    ];
    // Chart 3's karakas from its golden positions: Me, Ve, Ra, Mo, Ma, Sa, Su, Ju (ch. 8 §8.2).
    let by_karaka = [
        "lagna",
        "graha-mercury",
        "graha-venus",
        "graha-rahu",
        "graha-moon",
        "graha-mars",
        "graha-saturn",
        "graha-sun",
        "graha-jupiter",
        "graha-ketu",
    ];
    for (sort, order, active) in [
        ("", by_graha, 0),
        ("&sort=graha", by_graha, 0),
        ("&sort=karaka", by_karaka, 1),
    ] {
        let query = format!("{CHART_3}{sort}");
        let view = view(Lang::En, &query).await;
        let names: Vec<_> = view.grahas.iter().map(|row| row.name.as_str()).collect();
        assert_eq!(names, order, "{sort}");
        let page = body(&format!("/en/chart?{query}")).await;
        let start = page.find(r#"<table id="grahas">"#).unwrap();
        let head = &page[start..start + page[start..].find("</thead>").unwrap()];
        let headers: Vec<_> = head.split("<th ").skip(1).collect();
        let sorted: Vec<_> = headers
            .iter()
            .map(|h| h.contains(r#"aria-sort="ascending""#))
            .collect();
        assert_eq!(sorted.iter().filter(|s| **s).count(), 1, "{sort}");
        let (graha, karaka) = (headers[0], headers[1]);
        assert!([graha, karaka][active].contains("aria-sort"), "{sort}");
        // Askama escapes `&` in attributes.
        let base = query.replace("&sort=graha", "").replace("&sort=karaka", "");
        for (header, value) in [(graha, "graha"), (karaka, "karaka")] {
            let href = format!(r#"href="/en/chart?{base}&sort={value}#grahas""#);
            let href = href.replace('&', "&#38;");
            assert!(header.contains(&href), "{sort}: {header}");
        }
        let hidden = sort.trim_start_matches("&sort=");
        assert!(page.contains(&format!(
            r#"<input type="hidden" name="sort" value="{hidden}">"#
        )));
    }
}

#[tokio::test]
async fn an_unknown_sort_is_refused_and_dropped_from_the_form() {
    for sort in ["rasi", "Karaka", "1"] {
        let response = get(&format!("/en/chart?{CHART_1}&sort={sort}"), None).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST, "{sort}");
        let page = page(response).await;
        let error =
            r#"<strong id="sort-error" class="error">Choose one of the listed options.</strong>"#;
        assert!(page.contains(error), "{sort}: {page}");
        assert!(page.contains(r#"<input type="hidden" name="sort" value="">"#));
        assert!(!page.contains("<table"), "{page}");
    }
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
async fn an_unknown_topic_is_refused_and_dropped_from_the_form() {
    for topic in ["nonsense", "Grahas", "dasas"] {
        let response = get(&format!("/en/chart?{CHART_1}&topic={topic}"), None).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST, "{topic}");
        let page = page(response).await;
        let error =
            r#"<strong id="topic-error" class="error">Choose one of the listed options.</strong>"#;
        assert!(page.contains(error), "{topic}: {page}");
        assert!(page.contains(r#"<input type="hidden" name="topic" value="">"#));
        assert!(!page.contains("<table"), "{page}");
        assert!(page.contains("<title>hora</title>"), "{page}");
        // No chart, so the form shows unfolded with its error.
        assert!(!page.contains(r#"class="birth""#), "{page}");
    }
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

#[tokio::test]
async fn a_refused_form_keeps_the_chosen_sunrise_definition() {
    let query = with(&[("latitude", "95"), ("sunrise", "centre-true")]);
    let response = get(&format!("/en/chart?{query}"), None).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let page = page(response).await;
    assert!(page.contains(r#"value="centre-true" selected"#), "{page}");
}

/// Chart 9 (ch. 11 *Yogas*, p. 128), at the date, time and seconds its golden test is cast with.
fn chart_9() -> String {
    with(&[
        ("date", "1630-02-19"),
        ("time", "18%3A25%3A32"),
        ("calendar", "julian"),
        ("utc", "local-mean"),
        ("offset", ""),
        ("latitude", "18.533333333333335"),
        ("longitude", "73.88333333333334"),
    ])
}

/// Chart 10 (ch. 11 *Yogas*, p. 137), entered as the Julian date the book also prints.
fn chart_10() -> String {
    with(&[
        ("date", "1542-11-24"),
        ("time", "03%3A39"),
        ("calendar", "julian"),
        ("utc", "local-mean"),
        ("offset", ""),
        ("latitude", "25.316666666666666"),
        ("longitude", "69.78333333333333"),
    ])
}

/// Chart 18 (ch. 16 *Vimsottari Dasa*, p. 217) as the book prints it.
fn chart_18() -> String {
    with(&[
        ("date", "1972-06-01"),
        ("time", "04%3A16"),
        ("offset", "%2B05%3A30"),
        ("latitude", "16.25"),
        ("longitude", "81.2"),
    ])
}

/// Chart 90 (ch. 36, p. 493), at the seconds its golden test is cast with.
fn chart_90() -> String {
    with(&[
        ("date", "1968-01-25"),
        ("time", "11%3A14%3A45"),
        ("offset", "%2B05%3A30"),
        ("latitude", "17.483333333333334"),
        ("longitude", "78.5"),
    ])
}

/// The row of a yoga table whose first cell is `name`.
fn yoga_row(rows: &[Vec<String>], name: &str) -> Vec<String> {
    let row = rows.iter().find(|row| row[0] == name);
    row.unwrap_or_else(|| panic!("no {name} in {rows:?}"))
        .clone()
}

#[tokio::test]
async fn chart_10_lists_its_mahaapurusha_yogas_and_the_raaja_yoga_of_venus_and_saturn() {
    let page = body(&format!("/en/chart?{}&topic=yogas", chart_10())).await;
    let yogas = section(&page, "yogas");
    assert!(
        yogas.contains(r#"<a href="/en/kb/yogas">Yogas</a></h2>"#),
        "{yogas}"
    );
    assert!(
            yogas.contains(r#"<th scope="rowgroup" colspan="6"><a href="/en/kb/mahaapurusha-yogas">Mahaapurusha yogas</a></th>"#),
            "{yogas}"
        );
    assert!(
        yogas.contains(r##"<a href="/en/kb/yogas#checked">"##),
        "{yogas}"
    );
    let rows = yoga_texts(Lang::En, &view(Lang::En, &chart_10()).await.rasi.yogas);
    // Ch. 11 §11.7.2, p. 137: "Mahapurusha yogas (Maalavya and Sasa)" and "another Mahapurusha Yoga – Ruchaka Yoga".
    for (name, graha) in [
        ("Maalavya yoga", "Venus"),
        ("Sasa yoga", "Saturn"),
        ("Ruchaka yoga", "Mars"),
    ] {
        let row = yoga_row(&rows, name);
        assert!(row[1].starts_with(&format!("{graha} (")), "{row:?}");
        assert_eq!(row[2..], ["—", "—", "—", "—"], "{row:?}");
    }
    // P. 137: Venus and Saturn "less than a degree apart"; their printed amsas are not golden (ADR 0014).
    let row = rows.iter().find(|row| {
        row[0] == "Raaja yoga" && row[1] == "Venus (1L, 8L, in 1), Saturn (4L, 5L, in 1)"
    });
    let row = row.unwrap_or_else(|| panic!("{rows:?}"));
    assert!(row[4].starts_with("0°"), "{row:?}");
    // P. 137: Jupiter, a functional malefic, in their rasi "22° away" from Venus (ADR 0016).
    assert!(
        row[3].contains("Venus afflicted by Jupiter, a functional malefic, 22°28′ apart"),
        "{row:?}"
    );
    let amsas: Vec<_> = row[5].split(" / ").collect();
    assert!(
        amsas.len() == 2 && amsas[0].starts_with("Venus: ") && amsas[1].starts_with("Saturn: "),
        "{row:?}"
    );
}

#[tokio::test]
async fn chart_6s_vipareeta_raaja_yoga_of_mars_and_the_sun_is_17_apart() {
    let rows = yoga_texts(Lang::En, &view(Lang::En, &chart_6()).await.rasi.yogas);
    // Ch. 17, p. 230: Mars and the Sun "within a 17' from each other".
    let row = rows
        .iter()
        .find(|row| row[0] == "Vipareeta Raaja yoga" && row[1].starts_with("Sun ("));
    assert_eq!(row.map(|row| row[4].as_str()), Some("0°17′"), "{rows:?}");
}

#[tokio::test]
async fn chart_18s_saptamsa_lists_the_raaja_yoga_of_venus_and_rahu() {
    let query = format!("{}&varga=d7", chart_18());
    let page = body(&format!("/en/chart?{query}&topic=yogas")).await;
    let yogas = section(&page, "yogas");
    assert!(
        yogas.contains("<details>\n<summary>Saptamsa (D-7)</summary>"),
        "{yogas}"
    );
    // A varga's table has no magnitude columns (ADR 0014).
    assert!(
        !yogas
            .split_once("<details>")
            .unwrap()
            .1
            .contains("Closeness"),
        "{yogas}"
    );
    // Ch. 16, p. 218: Rahu "acts as the 5th lord", "with lagna lord Venus. His conjunction with Venus – conjunction of 1st and
    // 5th lords – consists of a raja yoga", which is also §11.7.3 (1)'s second half.
    let rows = yoga_texts(Lang::En, &view(Lang::En, &query).await.varga.yogas);
    for (name, grahas) in [
        ("Raaja yoga", "Venus (1L, 8L, in 11), Rahu (5L, in 11)"),
        (
            "Raaja yoga §11.7.3 (1)",
            "Venus (1L, in 11), Rahu (5L, in 11)",
        ),
    ] {
        let row = rows.iter().find(|row| row[0] == name && row[1] == grahas);
        assert!(row.is_some_and(|row| row[2] == "—"), "{name}: {rows:?}");
    }
}

#[tokio::test]
async fn chart_3s_raaja_yoga_1_names_its_ak_and_pk() {
    // Ch. 20, p. 265: "a powerful raja yoga between AK (Mercury) and PK (Saturn)", in the lagna.
    for (lang, grahas) in [
        (Lang::En, "Mercury (AK, in 1), Saturn (PK, in 1)"),
        (
            Lang::Hu,
            "Merkúr (AK, 1. bhavában), Szaturnusz (PK, 1. bhavában)",
        ),
    ] {
        let rows = yoga_texts(lang, &view(lang, CHART_3).await.rasi.yogas);
        assert_eq!(yoga_row(&rows, "Raaja yoga §11.7.3 (1)")[1], grahas);
    }
}

#[tokio::test]
async fn chart_90s_dasamsa_has_the_vipareeta_raaja_yoga_of_the_moon_and_jupiter_to_judge() {
    let query = format!("{}&varga=d10", chart_90());
    // Real-life Example 3, p. 494: the 8th and 12th lords conjoined in a kendra, effective only "in strength".
    let rows = yoga_texts(Lang::En, &view(Lang::En, &query).await.varga.yogas);
    let row = rows
        .iter()
        .find(|row| row[0] == "Vipareeta Raaja yoga" && row[1].starts_with("Moon ("));
    let row = row.unwrap_or_else(|| panic!("{rows:?}"));
    assert!(row[1].contains("Jupiter ("), "{row:?}");
    assert_eq!(row[2], "Whether Moon is strong / Whether Jupiter is strong");
    let page = body(&format!("/en/chart?{query}&topic=yogas")).await;
    assert!(
        page.contains(r#"<a href="/en/kb/raaja-yogas">Whether Moon is strong</a>"#),
        "{page}"
    );
    let rows = yoga_texts(Lang::Hu, &view(Lang::Hu, &query).await.varga.yogas);
    assert!(
        rows.iter()
            .any(|row| row[2] == "Erős-e a Hold / Erős-e a Jupiter"),
        "{rows:?}"
    );
}

#[tokio::test]
async fn the_rasi_only_yogas_are_never_in_the_vargas_table() {
    let view = view(Lang::En, &format!("{}&varga=d9", chart_9())).await;
    let names = |groups| -> Vec<String> {
        yoga_texts(Lang::En, groups)
            .into_iter()
            .map(|row| row[0].clone())
            .collect()
    };
    let (rasi_chart, navamsa) = (names(&view.rasi.yogas), names(&view.varga.yogas));
    // Ch. 11 §11.6, pp. 128–129: Kalpadruma mixes the rasi chart and the navamsa (ADR 0014).
    assert!(
        rasi_chart.iter().any(|name| name == "Kalpadruma yoga"),
        "{rasi_chart:?}"
    );
    for name in [
        "Kalpadruma",
        "Vishnu",
        "Gouri",
        "Chandikaa",
        "Bhaarathi",
        "Vidyut",
        "Jaya",
    ] {
        let yoga = format!("{name} yoga");
        assert!(!navamsa.iter().any(|n| n.starts_with(&yoga)), "{name}");
    }
    for numbered in [
        "Raaja yoga §11.7.3 (9)",
        "Daridra yoga §11.10 (8)",
        "Daridra yoga §11.10 (9)",
    ] {
        assert!(!navamsa.iter().any(|n| n.contains(numbered)), "{numbered}");
    }
}

#[tokio::test]
async fn the_yoga_tables_count_from_the_lagna_whatever_the_reference() {
    for chart in [chart_10(), chart_90()] {
        let yogas = |view: ChartView| {
            [view.rasi.yogas, view.varga.yogas].map(|groups| yoga_texts(Lang::En, &groups))
        };
        let from_lagna = yogas(view(Lang::En, &format!("{chart}&varga=d10")).await);
        let from_moon = yogas(view(Lang::En, &format!("{chart}&varga=d10&reference=moon")).await);
        assert_eq!(from_lagna, from_moon);
    }
}

/// The row of `id` ("graha-mercury", "av-sav", ...) in an ashtakavarga's BAV table.
fn av_row<'a>(av: &'a Ashtakavarga, id: &str) -> &'a [AvCell; 12] {
    &av.bav.iter().find(|row| row.0 == id).expect(id).1
}

#[tokio::test]
async fn chart_6s_ashtakavarga_holds_mercurys_bav_and_pinda_and_the_sav() {
    use Rasi::*;
    let view = view(Lang::En, &chart_6()).await;
    let av = &view.rasi.ashtakavarga;
    // Ch. 12 Exercise 19, p. 163 (Mercury in Ge), Exercise 20's SAV and Example 43's sodhya pinda, p. 160.
    let mercury = av_row(av, "graha-mercury");
    assert_eq!(
        [mercury[Aries as usize].0, mercury[Gemini as usize].0],
        [7, 7]
    );
    let natal = "av-natal".to_owned();
    assert!(mercury[Gemini as usize].1.contains(&natal));
    assert!(!mercury[Aries as usize].1.contains(&natal));
    let sav = &av_row(av, "av-sav")[Aries as usize];
    assert_eq!(
        (sav.0, sav.1.as_slice()),
        (34, ["av-sav-strong".to_owned()].as_slice())
    );
    let pinda = av.pindas.iter().find(|p| p.0 == "graha-mercury").unwrap();
    assert_eq!(pinda.1.sodhya(), 152);
    // The marker is text, with words for a screen reader (ADR 0015).
    let page = body(&format!("/en/chart?{}&topic=ashtakavarga", chart_6())).await;
    let section = section(&page, "ashtakavarga");
    assert!(
        section.contains(
            r#"<h2 id="ashtakavarga"><a href="/en/kb/ashtakavarga">Ashtakavarga</a></h2>"#
        ),
        "{section}"
    );
    let row = section
            .split("<tr>")
            .find(|row| row.contains(r#"<th scope="row"><a href="/en/kb/graha-characteristics#mercury">Mercury</a></th><td>"#))
            .unwrap();
    assert!(
            row.contains(r#"<td>7 <span aria-hidden="true">+</span><span class="sr">favourable</span> <span aria-hidden="true">•</span><span class="sr">its rasi</span></td>"#),
            "{row}"
        );
}

#[tokio::test]
async fn chart_3s_dasamsa_has_a_strong_35_in_scorpio() {
    // Ch. 12 Example 39, p. 154: the D-10's SAV.
    let page = body(&format!("/en/chart?{CHART_3}&varga=d10&topic=ashtakavarga")).await;
    let section = section(&page, "ashtakavarga");
    let (_, varga) = section
        .split_once("<details>\n<summary>Dasamsa (D-10)</summary>")
        .expect(section);
    let sav = varga
        .split("<tr>")
        .find(|row| row.starts_with(r#"<th scope="row">SAV</th>"#))
        .unwrap();
    let cells: Vec<_> = sav.split("<td>").skip(1).collect();
    assert_eq!(
        cells[Rasi::Scorpio as usize],
        r#"35 <span aria-hidden="true">+</span><span class="sr">strong</span></td>"#
    );
}

#[tokio::test]
async fn the_ashtakavarga_tables_count_from_the_lagna_whatever_the_reference() {
    let tables = |query: String| async move {
        section(
            &body(&format!("/en/chart?{query}&topic=ashtakavarga")).await,
            "ashtakavarga",
        )
        .to_owned()
    };
    let from_lagna = tables(format!("{CHART_3}&varga=d10")).await;
    assert!(from_lagna.contains("Dasamsa (D-10)"));
    for reference in ["moon", "arudha-lagna"] {
        let from = tables(format!("{CHART_3}&varga=d10&reference={reference}")).await;
        assert_eq!(from, from_lagna, "{reference}");
    }
}

#[test]
fn every_ashtakavarga_band_has_a_name_and_each_marked_one_a_mark() {
    use hora_chart::{BavBand, SavBand};
    let en = keys(include_str!("../../locales/en/main.ftl"));
    let names = [BavBand::Favourable, BavBand::Neutral, BavBand::Unfavourable]
        .map(|b| kb::message_id("av-bav", b))
        .into_iter()
        .chain(
            [SavBand::Strong, SavBand::Average, SavBand::Weak].map(|b| kb::message_id("av-sav", b)),
        );
    let marks = [
        "av-bav-favourable",
        "av-bav-unfavourable",
        "av-sav-strong",
        "av-sav-weak",
        "av-natal",
    ]
    .map(|id| format!("{id}-mark"));
    let missing: Vec<_> = names
        .chain(marks)
        .filter(|id| !en.contains(id.as_str()))
        .collect();
    // both_languages_have_the_same_messages holds Hungarian to the same ids.
    assert_eq!(missing, [""; 0]);
}

/// Every yoga with a name of its own, without the numbered entries of §11.7.3–11.10.
fn named_yogas() -> Vec<hora_chart::YogaName> {
    use hora_chart::YogaName::*;
    vec![
        Vesi,
        Vosi,
        Ubhayachara,
        BudhaAaditya,
        Sunaphaa,
        Anaphaa,
        Duradhara,
        Kemadruma,
        ChandraMangala,
        Adhi,
        Ruchaka,
        Bhadra,
        Sasa,
        Maalavya,
        Hamsa,
        Rajju,
        Musala,
        Nala,
        Maalaa,
        Sarpa,
        Gadaa,
        Sakata,
        Vihanga,
        Sringaataka,
        Hala,
        Vajra,
        Yava,
        Kamala,
        Vaapi,
        Yoopa,
        Sara,
        Sakti,
        Danda,
        Naukaa,
        Koota,
        Chatra,
        Chaapa,
        Ardhachandra,
        Chakra,
        Samudra,
        Veenaa,
        Daama,
        Paasa,
        Kedaara,
        Soola,
        Yuga,
        Gola,
        Subha,
        Asubha,
        GajaKesari,
        GuruMangala,
        Amala,
        Parvata,
        Kaahala,
        Chaamara,
        Sankha,
        Bheri,
        Mridanga,
        Sreenaatha,
        Matsya,
        Koorma,
        Khadga,
        Kusuma,
        Kalaanidhi,
        Kalpadruma,
        Lagnaadhi,
        Hari,
        Hara,
        Brahma,
        Vishnu,
        Siva,
        Trilochana,
        Gouri,
        Chandikaa,
        Lakshmi,
        Saarada,
        Bhaarathi,
        Saraswathi,
        Amsaavatara,
        Devendra,
        Indra,
        Ravi,
        Bhaaskara,
        Kulavardhana,
        Vasumati,
        Gandharva,
        Go,
        Vidyut,
        Chapa,
        Pushkala,
        Makuta,
        Jaya,
        Harsha,
        Sarala,
        Vimala,
        Tapaswi,
        BasicRaaja,
        DharmaKarmadhipati,
        VipareetaRaaja,
        BasicDhana,
    ]
}

#[test]
fn every_yoga_class_and_condition_has_a_name() {
    use hora_chart::{YogaClass, YogaName};
    let en = keys(include_str!("../../locales/en/main.ftl"));
    let names = named_yogas();
    let classes = [
        YogaClass::Ravi,
        YogaClass::Chandra,
        YogaClass::Mahaapurusha,
        YogaClass::Naabhasa,
        YogaClass::OtherPopular,
        YogaClass::Raaja,
        YogaClass::RaajaSambandha,
        YogaClass::Dhana,
        YogaClass::Daridra,
    ];
    let numbered =
        ["raaja", "raaja-sambandha", "dhana", "daridra"].map(|class| format!("yoga-{class}-n"));
    let conditions = [
        "mercurys-nature",
        "may-not-operate-well",
        "strong",
        "combust",
        "deep-exaltation",
        "malefic-in-upachaya",
        "all-in-one-rasi",
    ]
    .map(|c| format!("yoga-to-judge-{c}"));
    let missing: Vec<_> = names
        .into_iter()
        .map(|n: YogaName| kb::message_id("yoga", n))
        .chain(classes.map(|c| kb::message_id("yoga-class", c)))
        .chain(numbered)
        .chain(conditions)
        .filter(|id| !en.contains(id.as_str()))
        .collect();
    assert_eq!(missing, [""; 0]);
}

/// Every ch. 11 entry under its own heading, and ch. 13's Tapaswi: hora's 100 named yogas (the basic Dhana among them) and every numbered entry, Raaja (18) and Daridra (10) included though not computed.
#[tokio::test]
async fn every_yoga_has_a_section_on_its_class_page_in_both_languages() {
    use hora_chart::YogaName as N;
    let named = named_yogas().into_iter().map(|n| {
        let id = kb::message_id("yoga", n);
        (kb::yoga_page(n.class()), id["yoga-".len()..].to_owned())
    });
    let numbered = [
        (N::Raaja(1), "raaja", 18),
        (N::RaajaSambandha(1), "raaja-sambandha", 15),
        (N::Dhana(1), "dhana", 12),
        (N::Daridra(1), "daridra", 13),
    ]
    .into_iter()
    .flat_map(|(n, prefix, last)| {
        (1..=last).map(move |i| (kb::yoga_page(n.class()), format!("{prefix}-{i}")))
    });
    let anchors: Vec<_> = named.chain(numbered).collect();
    assert_eq!(anchors.len(), 100 + 18 + 15 + 12 + 13);
    for lang in ["en", "hu"] {
        let overview = body(&format!("/{lang}/kb/yogas")).await;
        assert!(overview.contains(r#"id="checked""#), "{lang}");
        let mut pages = std::collections::HashMap::new();
        for (page, _) in &anchors {
            if !pages.contains_key(page) {
                pages.insert(*page, body(&format!("/{lang}/kb/{page}")).await);
            }
        }
        let missing: Vec<_> = anchors
            .iter()
            .filter(|(page, id)| !pages[page].contains(&format!(r#"id="{id}""#)))
            .collect();
        assert!(missing.is_empty(), "{lang}: {missing:?}");
    }
}

/// The section headed `id`, which follows the chart table.
pub(crate) fn section<'a>(page: &'a str, id: &str) -> &'a str {
    let start = page
        .find(&format!(r#"<section aria-labelledby="{id}">"#))
        .expect(id);
    &page[start..start + page[start..].find("</section>").expect("a section end")]
}

#[tokio::test]
async fn errors_are_shown_in_hungarian_too() {
    let response = get(&format!("/hu/chart?{}", with(&[("latitude", "95")])), None).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert!(
        page(response)
            .await
            .contains("A szélességet fokban adja meg, −90 és 90 között.")
    );
}

#[tokio::test]
async fn a_decimal_comma_is_read_as_a_point() {
    let comma = with(&[("latitude", "42%2C5")]);
    assert_eq!(
        tbody(&body(&format!("/en/chart?{comma}")).await),
        tbody(&body(&format!("/en/chart?{CHART_1}")).await)
    );
}

#[tokio::test]
async fn local_mean_time_is_the_stated_offset_of_its_longitude() {
    // 71°12′ W is 71.2 × 4 min = 4 h 44 min 48 s behind UT.
    let stated = with(&[("offset", "-04%3A44%3A48")]);
    let mean = with(&[("utc", "local-mean"), ("offset", "")]);
    assert_eq!(
        tbody(&body(&format!("/en/chart?{mean}")).await),
        tbody(&body(&format!("/en/chart?{stated}")).await)
    );
    let minus = with(&[("offset", "%E2%88%9204%3A00")]);
    assert_eq!(
        tbody(&body(&format!("/en/chart?{minus}")).await),
        tbody(&body(&format!("/en/chart?{CHART_1}")).await)
    );
    // The comparison would pass vacuously if the offset did not move the chart.
    assert_ne!(
        tbody(&body(&format!("/en/chart?{mean}")).await),
        tbody(&body(&format!("/en/chart?{CHART_1}")).await)
    );
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
fn keys(ftl: &str) -> BTreeSet<&str> {
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

/// A birth at Budapest (geonameid 3054643) with its zone, on `date` at `time`.
fn in_budapest(date: &str, time: &str) -> String {
    format!(
        "/en/chart?date={date}&time={time}&utc=zone&place=Budapest&place-id=3054643\
            &latitude=47.49835&longitude=19.04045"
    )
}

#[tokio::test]
async fn a_zone_gives_its_modern_and_historical_offsets_and_names_them() {
    let page = body(&in_budapest("2000-07-01", "12%3A00")).await;
    assert!(
        page.contains("UTC offset +02:00, from the time zone Europe/Budapest of Budapest, HU."),
        "{page}"
    );
    assert!(!page.contains("Before 1970"), "{page}");
    // tzdb's wartime India: +06:30 from 1942-09 to 1945-10.
    let page = body(
        "/en/chart?date=1943-06-01&time=12%3A00&utc=zone&place=Kolkata&place-id=1275004\
            &latitude=22.56263&longitude=88.36304",
    )
    .await;
    assert!(
        page.contains("UTC offset +06:30, from the time zone Asia/Kolkata of Kolkata, IN."),
        "{page}"
    );
    assert!(
        page.contains("Before 1970 the time zone database"),
        "{page}"
    );
}

#[tokio::test]
async fn a_julian_date_takes_the_zones_offset_on_its_gregorian_day() {
    // Julian 2000-06-18 is Gregorian 2000-07-01, in summer time.
    let page = body(&format!(
        "{}&calendar=julian",
        in_budapest("2000-06-18", "12%3A00")
    ))
    .await;
    assert!(page.contains("UTC offset +02:00"), "{page}");
}

#[tokio::test]
async fn a_time_in_a_clock_change_is_refused_with_both_offsets_to_choose_from() {
    for (date, message, offsets) in [
        (
            "2000-03-26",
            "were put forward at this time",
            ["+01:00", "+02:00"],
        ),
        (
            "2000-10-29",
            "were put back at this time",
            ["+02:00", "+01:00"],
        ),
    ] {
        let response = get(&in_budapest(date, "02%3A30"), None).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST, "{date}");
        let page = page(response).await;
        assert!(
            page.contains(r#"<strong id="time-error" class="error">"#),
            "{page}"
        );
        assert!(page.contains(message), "{page}");
        for offset in offsets {
            assert!(
                page.contains(&format!(r#"name="offset-pick" value="{offset}""#)),
                "{date} {offset}: {page}"
            );
        }
        assert!(!page.contains("<table"), "{page}");
    }
    // Picking one makes it a stated offset: no zone is named.
    let page = body(&format!(
        "{}&offset-pick=%2B02%3A00",
        in_budapest("2000-03-26", "02%3A30")
    ))
    .await;
    assert!(page.contains("<table"), "{page}");
    assert!(
        page.contains(r#"<option value="stated" selected>"#),
        "{page}"
    );
    assert!(page.contains(r#"value="+02:00""#), "{page}");
    assert!(!page.contains("from the time zone"), "{page}");
    // West of Greenwich the buttons carry U+2212, which the offset field reads back.
    let new_york = "/en/chart?date=2000-10-29&time=01%3A30&utc=zone&place=New%20York%20City\
            &place-id=5128581&latitude=40.71427&longitude=-74.00597";
    let response = get(new_york, None).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let page = self::page(response).await;
    for offset in ["−04:00", "−05:00"] {
        assert!(
            page.contains(&format!(r#"name="offset-pick" value="{offset}""#)),
            "{offset}: {page}"
        );
    }
    let page = body(&format!("{new_york}&offset-pick=%E2%88%9205%3A00")).await;
    assert!(page.contains("<table"), "{page}");
    assert!(page.contains(r#"value="−05:00""#), "{page}");
}

#[test]
fn an_offset_is_written_with_its_seconds_only_when_it_has_some() {
    // Budapest's local mean time until 1890, and New York's standard time.
    assert_eq!(offset_text(4580), "+01:16:20");
    assert_eq!(offset_text(-18000), "−05:00");
    assert_eq!(offset(&offset_text(-4580)), Some(-4580));
}

#[tokio::test]
async fn a_place_name_lists_the_matching_places_in_both_languages() {
    let page = body("/en/chart?date=2000-07-01&time=12%3A00&place=Kolozsv%C3%A1r").await;
    assert!(
            page.contains(
                r#"<button type="submit" name="pick" value="681290" formnovalidate>Kolozsvár (Cluj-Napoca), RO (Europe/Bucharest; 46.77, 23.60)</button>"#
            ),
            "{page}"
        );
    assert!(page.contains("Choose the birth place"), "{page}");
    assert!(!page.contains("<table"), "{page}");
    // A village cities500 lacks, from the full Hungarian file.
    let page = body("/hu/chart?date=2000-07-01&time=12%3A00&place=sopronhorp%C3%A1cs").await;
    assert!(page.contains(r#"value="3045188""#), "{page}");
    assert!(page.contains("Válassza ki a születési helyet"), "{page}");
    // A name with digits and no spelled-out alternate.
    let page = body("/en/chart?date=2000-07-01&time=12%3A00&place=28%20May").await;
    assert!(page.contains(r#"value="12238134""#), "{page}");
}

#[tokio::test]
async fn picking_a_place_fills_its_coordinates_and_zone() {
    let page =
        body("/en/chart?date=2000-07-01&time=12%3A00&utc=stated&place=Buda&pick=3054643").await;
    assert!(page.contains("<table"), "{page}");
    for value in ["Budapest", "3054643", "47.49835", "19.04045"] {
        assert!(
            page.contains(&format!(r#"value="{value}""#)),
            "{value}: {page}"
        );
    }
    assert!(page.contains(r#"<option value="zone" selected>"#), "{page}");
    assert!(
        page.contains("from the time zone Europe/Budapest"),
        "{page}"
    );
    // Edited, the place's text is a new search.
    let page =
        body("/en/chart?date=2000-07-01&time=12%3A00&utc=zone&place=Vienna&place-id=3054643").await;
    assert!(page.contains(r#"name="pick""#), "{page}");
    assert!(!page.contains("<table"), "{page}");
}

#[tokio::test]
async fn the_script_finds_what_it_swaps_and_is_cached_by_its_etag() {
    // hora.js swaps in the list by these ids; without a place-id typing only searches.
    let list = body("/en/chart?date=2000-07-01&time=12%3A00&place=Budap").await;
    assert!(
        list.contains(r#"<div id="places" aria-live="polite">"#),
        "{list}"
    );
    assert!(list.contains(r#"name="pick""#), "{list}");
    assert!(!list.contains("<table"), "{list}");
    let none = page(get("/en/chart?date=2000-07-01&time=12%3A00&place=Budapxx", None).await).await;
    assert!(none.contains(r#"id="place-error""#), "{none}");
    assert!(
        none.contains(r#"<script type="module" src="/hora.js"></script>"#),
        "{none}"
    );

    let response = get("/hora.js", None).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers()[header::CONTENT_TYPE],
        "text/javascript; charset=utf-8"
    );
    assert_eq!(response.headers()[header::CACHE_CONTROL], "no-cache");
    let etag = response.headers()[header::ETAG]
        .to_str()
        .unwrap()
        .to_owned();
    for tags in [etag.clone(), format!("W/{etag}"), format!("\"x\", {etag}")] {
        let revalidated = app(places(), None, jiff::Timestamp::now)
            .oneshot(
                Request::get("/hora.js")
                    .header(header::IF_NONE_MATCH, &tags)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(revalidated.status(), StatusCode::NOT_MODIFIED, "{tags}");
        let body = to_bytes(revalidated.into_body(), 1 << 20).await.unwrap();
        assert!(body.is_empty());
    }
}

#[tokio::test]
async fn a_bad_or_unknown_place_or_a_zone_without_one_is_refused() {
    let long = "a".repeat(101);
    for (query, field, message) in [
        ("place=%3Cscript%3E", "place", "Enter 2 to 100 letters"),
        ("place=a", "place", "Enter 2 to 100 letters"),
        (
            &format!("place={long}") as &str,
            "place",
            "Enter 2 to 100 letters",
        ),
        ("place=Xyzzyqx", "place", "No place of that name"),
        (
            "place=Buda&pick=1",
            "place",
            "Choose one of the listed options.",
        ),
        (
            "utc=zone&latitude=47.5&longitude=19",
            "utc",
            "To use a time zone",
        ),
    ] {
        let response = get(
            &format!("/en/chart?date=2000-07-01&time=12%3A00&{query}"),
            None,
        )
        .await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST, "{query}");
        let page = page(response).await;
        let error = format!(r#"<strong id="{field}-error" class="error">{message}"#);
        assert!(page.contains(&error), "{query}: {page}");
    }
}

#[tokio::test]
async fn the_form_credits_geonames_in_both_languages() {
    for lang in ["en", "hu"] {
        let page = body(&format!("/{lang}/")).await;
        assert!(page.contains(r#"<a href="https://www.geonames.org/">GeoNames</a> (<a href="https://creativecommons.org/licenses/by/4.0/">CC BY 4.0</a>)"#), "{page}");
    }
}

#[tokio::test]
async fn typed_coordinates_replace_the_chosen_places_zone() {
    let edited = in_budapest("2000-07-01", "12%3A00").replace("47.49835", "47.6");
    let response = get(&edited, None).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let page = page(response).await;
    assert!(
        page.contains(r#"<strong id="utc-error" class="error">To use a time zone"#),
        "{page}"
    );
    // Stated, the offset needs no zone, and the kept place text is no new search.
    let page = body(&edited.replace("utc=zone", "utc=stated&offset=%2B02%3A00")).await;
    assert!(page.contains("<table"), "{page}");
    assert!(!page.contains("from the time zone"), "{page}");
}

#[tokio::test]
async fn a_zone_refusal_comes_with_the_other_fields_errors() {
    for (date, time, field) in [
        ("2000-03-26", "02%3A30", "time"),
        ("2000-02-30", "12%3A00", "date"),
    ] {
        let query = format!("{}&altitude=99999", in_budapest(date, time));
        let response = get(&query, None).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST, "{date}");
        let page = page(response).await;
        for field in [field, "altitude"] {
            assert!(
                page.contains(&format!(r#"<strong id="{field}-error" class="error">"#)),
                "{date} {field}: {page}"
            );
        }
    }
}

#[tokio::test]
async fn the_before_1970_warning_goes_by_the_gregorian_date() {
    let warned = |page: &str| page.contains("Before 1970 the time zone database");
    assert!(warned(&body(&in_budapest("1969-12-31", "12%3A00")).await));
    // Julian 1969-12-25 is Gregorian 1970-01-07.
    let page = body(&format!(
        "{}&calendar=julian",
        in_budapest("1969-12-25", "12%3A00")
    ))
    .await;
    assert!(
        page.contains("from the time zone Europe/Budapest"),
        "{page}"
    );
    assert!(!warned(&page), "{page}");
}

/// Chart 13 (ch. 13 *Interpreting Charts*, p. 173) as the book prints it.
fn chart_13() -> String {
    with(&[
        ("date", "1894-05-20"),
        ("time", "13%3A22"),
        ("offset", "%2B05%3A30"),
        ("latitude", "11.95"),
        ("longitude", "79.53333333333333"),
    ])
}

/// Chart 14 (ch. 13, p. 175), at the seconds of its footnote 37.
fn chart_14() -> String {
    with(&[
        ("date", "1944-08-20"),
        ("time", "07%3A11%3A40"),
        ("offset", "%2B05%3A30"),
        ("latitude", "18.966666666666665"),
        ("longitude", "72.81666666666666"),
    ])
}

#[tokio::test]
async fn a_relatives_lagna_counts_the_vargas_bhavas_from_it() {
    // Ch. 13 §13.4.2, p. 176: in chart 14's D-3 the younger brother's lagna is Li, where the 3rd lord Saturn is (ADR 0016).
    let query = format!("{}&varga=d3&reference=younger-sibling-1", chart_14());
    let view = view(Lang::En, &query).await;
    let first = &view.varga.argalas[0];
    assert_eq!((first.bhava, first.rasi), (1, "rasi-libra"));
    for (lang, label) in [
        ("en", "1st younger sibling (D-3)"),
        ("hu", "1. fiatalabb testvér (D-3)"),
    ] {
        let page = body(&format!("/{lang}/chart?{query}")).await;
        let option = format!(r#"<option value="younger-sibling-1" selected>{label}</option>"#);
        assert!(page.contains(&option), "{lang}");
    }
}

#[tokio::test]
async fn chart_13_has_the_tapaswi_yoga_with_its_two_notes() {
    // Ch. 13 §13.5, Example 44, p. 174: Saturn and Ketu in Vi, Venus aspecting from Pi, the 8th; "AK is also involved".
    for (lang, name, notes) in [
        (
            Lang::En,
            "Tapaswi yoga",
            "AK Saturn joins or aspects another of the three: more fruitful / Venus in the 8th: particularly powerful",
        ),
        (
            Lang::Hu,
            "Tapaswi yoga",
            "Az AK (Szaturnusz) együtt áll a másik kettő egyikével, vagy aspektálja azt: gyümölcsözőbb / A Vénusz a 8. bhavában: különösen erős",
        ),
    ] {
        let rows = yoga_texts(lang, &view(lang, &chart_13()).await.rasi.yogas);
        let row = yoga_row(&rows, name);
        assert_eq!(row[2..4], ["—", notes], "{row:?}");
    }
    let page = body(&format!("/en/chart?{}&topic=yogas", chart_13())).await;
    let note =
        r#"<a href="/en/kb/other-popular-yogas">Venus in the 8th: particularly powerful</a>"#;
    assert!(section(&page, "yogas").contains(note), "{page}");
}

#[tokio::test]
async fn chart_14s_raaja_yoga_lists_its_afflicting_functional_malefics() {
    // Ch. 11 §11.7.2, pp. 136–137: Mercury and Venus, Leo functional malefics, with the Sun and Jupiter, and Saturn's
    // "very close aspect" on Jupiter (ADR 0016).
    let rows = yoga_texts(Lang::En, &view(Lang::En, &chart_14()).await.rasi.yogas);
    let row = rows.iter().find(|row| {
        row[0] == "Raaja yoga" && row[1].starts_with("Sun (") && row[1].contains("Jupiter (")
    });
    let row = row.unwrap_or_else(|| panic!("{rows:?}"));
    for graha in ["Mercury", "Venus"] {
        assert!(
            row[3].contains(&format!("Sun afflicted by {graha}, a functional malefic, ")),
            "{row:?}"
        );
    }
    assert!(
        row[3].contains("Jupiter afflicted by Saturn, a functional malefic, 2°01′ apart"),
        "{row:?}"
    );
    // A varga's afflictions have no closeness.
    let rows = yoga_texts(Lang::En, &view(Lang::En, &chart_14()).await.varga.yogas);
    assert!(
        rows.iter().all(|row| !row[3].contains(" apart")),
        "{rows:?}"
    );
}

#[tokio::test]
async fn chart_90s_venus_is_a_yogada_for_money_and_power() {
    // Ch. 36, p. 496: lagna Pi, HL Ta, GL Sg, Venus in Sg "aspects lagna (by rasi aspect)… Venus is a yogada" (ADR 0016).
    let view = view(Lang::En, &chart_90()).await;
    let venus: Vec<_> = view
        .rasi
        .yogadas
        .iter()
        .filter(|y| y.graha == "graha-venus")
        .map(|y| {
            let links = y
                .links
                .each_ref()
                .map(|l| l.iter().map(|id| en(id)).collect::<Vec<_>>().join(", "));
            (en(&y.gives), links)
        })
        .collect();
    assert_eq!(
        venus,
        [
            (
                "Money (HL)".to_owned(),
                ["rasi drishti".to_owned(), "owns".to_owned()]
            ),
            (
                "Power (GL)".to_owned(),
                ["rasi drishti".to_owned(), "in it".to_owned()]
            ),
        ]
    );
    let page = body(&format!("/hu/chart?{}&topic=interpreting", chart_90())).await;
    let yogadas = section(&page, "yogadas");
    assert!(yogadas.contains("<td>Pénz (HL)</td>"), "{yogadas}");
}

#[tokio::test]
async fn without_a_vedic_day_the_yogadas_say_so() {
    let page = body(&format!("/en/chart?{}&topic=interpreting", with(&TROMSO))).await;
    let yogadas = section(&page, "yogadas");
    assert!(!yogadas.contains("<table"), "{yogadas}");
    assert!(yogadas.contains(&en("yogadas-no-vedic-day")), "{yogadas}");
}

#[tokio::test]
async fn the_grahas_table_shows_functional_nature_from_the_reference_point() {
    // Ch. 13 §13.2, Table 30 (p. 167): from Le, Mars the yogakaraka, Venus a malefic; from the Moon's Cn too, Mars the
    // yogakaraka, Jupiter a benefic. Nodes and the lagna have none (ADR 0016).
    let nature = |view: &ChartView, id: &str| -> [String; 2] {
        graha(view, id).functional.clone().map(|cell| {
            cell.map_or("—".to_owned(), |(id, yogakaraka)| {
                let name = en(&id);
                if yogakaraka {
                    format!("{name}, {}", en("yogakaraka"))
                } else {
                    name
                }
            })
        })
    };
    let view = view(Lang::En, &chart_14()).await;
    assert_eq!(nature(&view, "graha-mars")[0], "Benefic, Yogakaraka");
    assert_eq!(nature(&view, "graha-venus")[0], "Malefic");
    for id in ["lagna", "graha-rahu", "graha-ketu"] {
        assert_eq!(nature(&view, id), ["—", "—"], "{id}");
    }
    let page = body(&format!("/en/chart?{}", chart_14())).await;
    assert!(
            page.contains(r#"<th scope="colgroup" colspan="2"><a href="/en/kb/functional-nature">Functional nature</a></th>"#),
            "{page}"
        );
}

#[tokio::test]
async fn every_kb_link_on_every_topic_reaches_a_listed_page() {
    for lang in ["en", "hu"] {
        let mut pages = std::collections::HashMap::new();
        for topic in &TOPICS {
            let page = body(&format!("/{lang}/chart?{}&topic={}", chart_14(), topic.key)).await;
            let prefix = format!(r#"href="/{lang}/kb/"#);
            let mut links = page.split(prefix.as_str()).skip(1).peekable();
            assert!(links.peek().is_some(), "{lang} {}", topic.key);
            for link in links {
                let link = &link[..link.find('"').unwrap()];
                let (slug, anchor) = link.split_once('#').unwrap_or((link, ""));
                assert!(
                    slug.is_empty() || kb::PAGES.iter().any(|p| p.slug == slug),
                    "{lang} {}: {slug}",
                    topic.key
                );
                if !anchor.is_empty() {
                    if !pages.contains_key(slug) {
                        pages.insert(slug.to_owned(), body(&format!("/{lang}/kb/{slug}")).await);
                    }
                    assert!(
                        pages[slug].contains(&format!(r#"id="{anchor}""#)),
                        "{lang} {}: {slug}#{anchor}",
                        topic.key
                    );
                }
            }
        }
    }
}

#[tokio::test]
async fn the_baadhakas_table_counts_from_the_reference_point() {
    // Ch. 13 §13.3, Table 31 (pp. 168–169): chart 1's Vi lagna, a dwiswabhava, has its baadhaka sthaana in the 7th, Pi,
    // and Jupiter its baadhaka.
    let view = view(Lang::En, CHART_1).await;
    assert_eq!(view.rasi.baadhakas.len(), 12);
    let first = &view.rasi.baadhakas[0];
    assert_eq!(
        (
            first.bhava,
            first.rasi,
            first.sthaana,
            first.baadhakas.as_slice()
        ),
        (1, "rasi-virgo", ("rasi-pisces", 7), &["graha-jupiter"][..])
    );
    assert!(
        first
            .arudhas
            .iter()
            .all(|id| id.starts_with("bhava-arudha-"))
    );
    let arudhas: usize = view.rasi.baadhakas.iter().map(|r| r.arudhas.len()).sum();
    assert_eq!(arudhas, 12);
    let page = body(&format!("/en/chart?{CHART_1}&topic=interpreting")).await;
    let baadhakas = section(&page, "baadhakas");
    assert!(
        baadhakas.contains(r#"<a href="/en/kb/baadhaka-and-yogada">Baadhakas</a></h2>"#),
        "{baadhakas}"
    );
    assert!(
        baadhakas.contains("<summary>Navamsa (D-9)</summary>"),
        "{baadhakas}"
    );
}

#[test]
fn every_chapter_13_value_has_a_message() {
    use hora_chart::{FunctionalNature, Gives, Link};
    let en = keys(include_str!("../../locales/en/main.ftl"));
    let missing: Vec<_> = [
        FunctionalNature::Benefic,
        FunctionalNature::Neutral,
        FunctionalNature::Malefic,
    ]
    .map(|n| kb::message_id("functional", n))
    .into_iter()
    .chain([Gives::Money, Gives::Power].map(|g| kb::message_id("yogada-gives", g)))
    .chain([Link::Owns, Link::Conjoins, Link::Aspects].map(|l| kb::message_id("yogada-link", l)))
    .chain(REFERENCES.iter().map(|r| r.2.to_owned()))
    .chain(
        [
            "yogakaraka",
            "functional-nature",
            "yoga-notes",
            "yoga-affliction",
            "yoga-affliction-closeness",
            "yoga-note-atma-karaka",
            "yoga-note-in-the-eighth",
            "topic-interpreting-charts",
            "baadhakas",
            "baadhaka-sthaana",
            "baadhaka-arudhas",
            "baadhaka-grahas",
            "yogadas",
            "yogada-gives",
            "yogada-hl-or-gl",
            "yogadas-none",
            "yogadas-no-vedic-day",
        ]
        .map(str::to_owned),
    )
    .filter(|id| !en.contains(id.as_str()))
    .collect();
    assert_eq!(missing, [""; 0]);
}

/// Chart 8 (ch. 10, p. 109; Exercise 23, p. 186) as the book prints it.
fn chart_8() -> String {
    with(&[
        ("date", "1946-12-02"),
        ("time", "06%3A45"),
        ("offset", "%2B01%3A00"),
        ("latitude", "38.1"),
        ("longitude", "15.65"),
    ])
}

#[tokio::test]
async fn chart_8_shows_its_marakas_rudra_and_longevity() {
    // Exercise 23's answer (ch. 14 §14.7, p. 186): Jupiter, Venus and Rahu the main marakas; Rudra Mercury, Trishoola Ge, Li, Aq;
    // Maheswara Jupiter by exception (2); middle life; the eighth lord method's short life from Sc, Mercury in its 12th. Jupiter as
    // the afflicted weaker and the paramaayush of 64 are hora's readings (ADR 0017).
    let view = view(Lang::En, &chart_8()).await;
    let marakas: Vec<_> = view.rasi.marakas.iter().map(|m| m.0).collect();
    for graha in ["graha-jupiter", "graha-venus", "graha-rahu"] {
        assert!(marakas.contains(&graha), "{marakas:?}");
    }
    assert_eq!(
        view.rasi.maraka_sthanas,
        [("rasi-sagittarius", 2), ("rasi-taurus", 7)]
    );
    let rahu = view.rasi.marakas.iter().find(|m| m.0 == "graha-rahu");
    assert!(rahu.unwrap().1.contains(&"in 7".to_owned()), "{rahu:?}");
    let l = &view.longevity;
    assert_eq!(
        l.candidates,
        [
            ("rasi-sagittarius", "graha-jupiter"),
            ("rasi-gemini", "graha-mercury")
        ]
    );
    assert_eq!(
        (l.rudra, l.decided_by.as_str(), l.weaker, l.weaker_afflicted),
        (
            "graha-mercury",
            "rudra-rule-advanced",
            "graha-jupiter",
            true
        )
    );
    assert_eq!(l.trishoola, ["rasi-gemini", "rasi-libra", "rasi-aquarius"]);
    assert_eq!(
        (l.maheswara, l.maheswara_exceptions.as_slice()),
        (
            "graha-jupiter",
            &["maheswara-exception-node".to_owned()][..]
        )
    );
    let longevity: Vec<_> = l.pairs.iter().map(|p| p.longevity.as_str()).collect();
    assert_eq!(
        longevity,
        ["longevity-middle", "longevity-middle", "longevity-short"]
    );
    assert_eq!(
        l.result,
        Some((
            "Middle life (36–72 years)".to_owned(),
            "pairs-rule-agreement".to_owned()
        ))
    );
    assert_eq!(l.paramaayush, "64 years");
    assert_eq!(
        l.eighth_lord,
        (
            "rasi-scorpio",
            "graha-mercury",
            12,
            "Short life (0–36 years)".to_owned()
        )
    );
    for (lang, middle, caution) in [
        ("en", "Middle life (36–72 years)", "not infallible"),
        ("hu", "Közepes élet (36–72 év)", "nem tévedhetetlenek"),
    ] {
        let page = body(&format!("/{lang}/chart?{}&topic=longevity", chart_8())).await;
        let pairs = section(&page, "three-pairs");
        assert!(pairs.contains(&format!("<td>{middle}</td>")), "{pairs}");
        assert!(
            pairs.contains(&format!(r#"<a href="/{lang}/kb/longevity#caution">"#))
                && pairs.contains(caution),
            "{pairs}"
        );
        let rudra = section(&page, "rudra");
        assert!(
            rudra.contains(&format!(
                r#"<h2 id="rudra"><a href="/{lang}/kb/marakas-and-rudra#rudra">Rudra</a></h2>"#
            )),
            "{rudra}"
        );
    }
}

#[tokio::test]
async fn a_varga_shows_marakas_and_no_rudra_or_longevity() {
    // Ch. 14 §14.2 reads marakas in any chart, D-11 and D-30 by name; Rudra and both methods in the rasi chart only (ADR 0017).
    let query = format!("{}&varga=d11&topic=longevity", chart_8());
    let view = view(Lang::En, &query).await;
    assert!(!view.varga.marakas.is_empty());
    let page = body(&format!("/en/chart?{query}")).await;
    let marakas = section(&page, "marakas");
    assert!(
        marakas.contains("<summary>Rudramsa (D-11)</summary>"),
        "{marakas}"
    );
    assert_eq!(marakas.matches("<table>").count(), 2, "{marakas}");
    for id in ["rudra", "three-pairs", "eighth-lord"] {
        let shown = section(&page, id);
        assert!(
            !shown.contains("<details") && !shown.contains("D-11"),
            "{shown}"
        );
        assert!(shown.contains("<caption>Rasi chart</caption>"), "{shown}");
    }
}

#[tokio::test]
async fn without_a_vedic_day_the_three_pairs_say_so() {
    let view = view(Lang::En, &with(&TROMSO)).await;
    assert_eq!(view.longevity.pairs.len(), 2);
    assert!(view.longevity.result.is_none());
    let page = body(&format!("/en/chart?{}&topic=longevity", with(&TROMSO))).await;
    let pairs = section(&page, "three-pairs");
    assert!(pairs.contains(&en("pairs-no-vedic-day")), "{pairs}");
    assert!(!pairs.contains(&en("paramaayush")), "{pairs}");
}

/// Example 49 (ch. 15 §15.4.4, pp. 199–200), chart 27's birth, with the name sound `sound`.
fn example_49(sound: &str) -> String {
    with(&[
        ("date", "1970-04-04"),
        ("time", "17%3A50"),
        ("offset", "%2B05%3A30"),
        ("latitude", "16.25"),
        ("longitude", "81.2"),
        ("sound", sound),
    ])
}

/// The Sayanaadi cell pair of `graha`'s avasthas row: (avastha id, strength id).
fn sayanaadi<'a>(view: &'a ChartView, graha: &str) -> &'a Option<(String, Option<String>)> {
    let row = view.strength.avasthas.iter().find(|r| r.graha == graha);
    &row.expect(graha).sayanaadi
}

#[tokio::test]
async fn example_49_shows_the_books_sayanaadi_avasthas_and_strengths() {
    // Example 49's answer, sound 1 (p. 200): Mercury Gamana and cheshta, Jupiter Nriyalipsaa and drishti, Venus Sabhaa and cheshta.
    let view = view(Lang::En, &example_49("1")).await;
    for (graha, avastha, strength) in [
        ("graha-mercury", "gamana", "cheshta"),
        ("graha-jupiter", "nriyalipsaa", "drishti"),
        ("graha-venus", "sabhaa", "cheshta"),
    ] {
        assert_eq!(
            sayanaadi(&view, graha),
            &Some((
                format!("sayanaadi-avastha-{avastha}"),
                Some(format!("activity-strength-{strength}"))
            )),
            "{graha}"
        );
    }
    // The state names are the same in both languages (GLOSSARY.md).
    for lang in ["en", "hu"] {
        let query = format!("{}&topic=strength", example_49("1"));
        let page = body(&format!("/{lang}/chart?{query}")).await;
        let avasthas = section(&page, "avasthas");
        assert!(
            avasthas.contains("<td>Gamana</td><td>cheshta</td>"),
            "{avasthas}"
        );
        assert!(avasthas.contains(r#"<th scope="col">"#), "{avasthas}");
        assert!(avasthas.contains(r#"<th scope="row">"#), "{avasthas}");
        let results = format!(r#"href="/{lang}/kb/avasthas""#);
        assert!(avasthas.contains(&results), "{avasthas}");
        let lords = section(&page, "stronger-lords");
        assert_eq!(lords.matches("<tr>").count(), 1 + 2, "{lords}");
        assert_eq!(lords.matches(r#"<th scope="col">"#).count(), 2, "{lords}");
        let karakas = section(&page, "sthira-karakas");
        assert_eq!(karakas.matches("<tr>").count(), 1 + 7, "{karakas}");
        // The reference point does not move ch. 15's tables (ADR 0018).
        let from_moon = body(&format!("/{lang}/chart?{query}&reference=moon")).await;
        for id in ["avasthas", "stronger-lords", "sthira-karakas"] {
            assert_eq!(section(&from_moon, id), section(&page, id), "{lang} {id}");
        }
    }
}

#[tokio::test]
async fn without_a_sound_the_sayanaadi_avasthas_show_and_their_strength_says_why_not() {
    let view = view(Lang::En, &example_49("")).await;
    assert_eq!(
        sayanaadi(&view, "graha-mercury"),
        &Some(("sayanaadi-avastha-gamana".to_owned(), None))
    );
    for lang in [Lang::En, Lang::Hu] {
        let query = format!("{}&topic=strength", example_49(""));
        let page = body(&format!("/{}/chart?{query}", lang.code())).await;
        let avasthas = section(&page, "avasthas");
        let needs = t(lang, "activity-strength-needs-sound");
        assert!(
            avasthas.contains(&format!("<td>Gamana</td><td>{needs}</td>")),
            "{avasthas}"
        );
    }
}

#[tokio::test]
async fn without_a_vedic_day_the_sayanaadi_avasthas_say_so() {
    let query = format!("{}&sound=1&topic=strength", with(&TROMSO));
    let view = view(Lang::En, &query).await;
    assert!(view.strength.avasthas.iter().all(|r| r.sayanaadi.is_none()));
    let page = body(&format!("/en/chart?{query}")).await;
    let avasthas = section(&page, "avasthas");
    assert!(
        avasthas.contains(&en("sayanaadi-no-vedic-day")),
        "{avasthas}"
    );
}

#[tokio::test]
async fn a_node_reads_a_dash_where_a_state_does_not_apply() {
    // Example 49's Ketu is in Leo, neither his own rasi, exaltation nor debilitation (ADR 0018).
    let view = view(Lang::En, &example_49("1")).await;
    let ketu = view
        .strength
        .avasthas
        .iter()
        .find(|r| r.graha == "graha-ketu");
    assert_eq!(ketu.unwrap().alertness, None);
    let page = body(&format!("/en/chart?{}&topic=strength", example_49("1"))).await;
    let avasthas = section(&page, "avasthas");
    let row = &avasthas[avasthas.find(r#"data-body="graha-ketu""#).unwrap()..];
    let row = &row[..row.find("</tr>").unwrap()];
    assert!(row.contains("<td>—</td>"), "{row}");
}

#[tokio::test]
async fn a_kopita_graha_says_its_mood_is_to_judge() {
    // §15.4.3: Kopita is being joined by the Sun, and how closely is left to judge (ADR 0018). Chart 1's Sun joins
    // Mercury and Venus in Pisces.
    let view = view(Lang::En, CHART_1).await;
    let kopita: Vec<_> = view
        .strength
        .avasthas
        .iter()
        .filter(|r| r.moods.iter().any(|m| m == "mood-avastha-kopita"))
        .map(|r| r.graha)
        .collect();
    assert_eq!(kopita, ["graha-mercury", "graha-venus"]);
    for lang in [Lang::En, Lang::Hu] {
        let page = body(&format!("/{}/chart?{CHART_1}&topic=strength", lang.code())).await;
        let avasthas = section(&page, "avasthas");
        let note = t(lang, "kopita-to-judge");
        for row in view.strength.avasthas.iter() {
            let cells = &avasthas[avasthas
                .find(&format!(r#"data-body="{}""#, row.graha))
                .unwrap()..];
            let cells = &cells[..cells.find("</tr>").unwrap()];
            assert_eq!(
                cells.contains(&note),
                kopita.contains(&row.graha),
                "{cells}"
            );
        }
    }
}

#[test]
fn every_chapter_15_value_has_a_message() {
    use hora_chart::{
        ActivityStrength as S, AgeAvastha as A, Alertness as L, Kin as K, MoodAvastha as M,
        SayanaadiAvastha as Y, StrongerRule as R,
    };
    let en = keys(include_str!("../../locales/en/main.ftl"));
    let hu = keys(include_str!("../../locales/hu/main.ftl"));
    let ages = [A::Saisava, A::Kumaara, A::Yuva, A::Vriddha, A::Mrita]
        .map(|a| kb::message_id("age-avastha", a));
    let ids: Vec<String> = ages
        .iter()
        .flat_map(|id| [id.clone(), format!("{id}-share")])
        .chain([L::Jaagrita, L::Swapna, L::Sushupta].map(|a| kb::message_id("alertness", a)))
        .chain(
            [
                M::Deepta,
                M::Swastha,
                M::Mudita,
                M::Saanta,
                M::Deena,
                M::Duhkhita,
                M::Vikala,
                M::Khala,
                M::Kopita,
                M::Lajjita,
                M::Garvita,
                M::Kshudhita,
                M::Trishita,
                M::Kshobhita,
            ]
            .map(|m| kb::message_id("mood-avastha", m)),
        )
        .chain(
            [
                Y::Sayana,
                Y::Upavesana,
                Y::Netrapaani,
                Y::Prakaasana,
                Y::Gamana,
                Y::Aagamana,
                Y::Sabhaa,
                Y::Aagama,
                Y::Bhojana,
                Y::Nriyalipsaa,
                Y::Kautuka,
                Y::Nidraa,
            ]
            .map(|y| kb::message_id("sayanaadi-avastha", y)),
        )
        .chain(
            [S::Cheshta, S::Drishti, S::Vicheshta].map(|s| kb::message_id("activity-strength", s)),
        )
        .chain(
            [
                R::Basic,
                R::Joined,
                R::JupiterMercuryDispositor,
                R::Exalted,
                R::Mobility,
                R::Advancement,
            ]
            .map(|r| kb::message_id("stronger-rule", r)),
        )
        .chain(
            [
                K::Father,
                K::Mother,
                K::YoungerSiblings,
                K::MaternalRelatives,
                K::Husband,
                K::Wife,
                K::ElderSiblings,
            ]
            .map(|k| kb::message_id("kin", k)),
        )
        .collect();
    let missing: Vec<_> = ids
        .iter()
        .filter(|id| !en.contains(id.as_str()) || !hu.contains(id.as_str()))
        .collect();
    assert!(missing.is_empty(), "{missing:?}");
}

#[test]
fn every_chapter_14_value_has_a_message() {
    use hora_chart::{Longevity, MaheswaraException as M, PairsRule, RudraRule as R};
    let en = keys(include_str!("../../locales/en/main.ftl"));
    let missing: Vec<_> = [Longevity::Short, Longevity::Middle, Longevity::Long]
        .map(|l| kb::message_id("longevity", l))
        .into_iter()
        .chain(
            [
                R::Conjoins,
                R::ExaltedOrOwn,
                R::JoinsExalted,
                R::Aspected,
                R::Advanced,
            ]
            .map(|r| kb::message_id("rudra-rule", r)),
        )
        .chain(
            [M::OwnOrExalted, M::Node, M::NodeReplaced]
                .map(|e| kb::message_id("maheswara-exception", e)),
        )
        .chain(
            [
                PairsRule::Agreement,
                PairsRule::LagnaAndHl,
                PairsRule::MoonAndSaturn,
            ]
            .map(|r| kb::message_id("pairs-rule", r)),
        )
        .chain(
            [
                "topic-longevity",
                "marakas",
                "maraka",
                "maraka-reasons",
                "maraka-sthana",
                "maraka-owns",
                "maraka-occupies",
                "maraka-aspects",
                "maraka-conjoins-lord",
                "maraka-aspects-lord",
                "rudra",
                "rudra-from-lagna",
                "rudra-from-seventh",
                "rudra-counted-by-table",
                "rudra-counted-normally",
                "rudra-decided-by",
                "rudra-rule-tie",
                "rudra-weaker",
                "rudra-weaker-afflicted",
                "rudra-weaker-unafflicted",
                "trishoola-rasis",
                "maheswara",
                "three-pairs",
                "longevity-caution",
                "pair",
                "pair-rasis",
                "pair-lords",
                "pair-moon-saturn",
                "pair-lagna-hl",
                "longevity",
                "longevity-years",
                "pairs-result",
                "pairs-rule",
                "paramaayush",
                "paramaayush-years",
                "paramaayush-none",
                "pairs-no-vedic-day",
                "eighth-lord-method",
                "eighth-lord-reference",
                "eighth-lord",
                "eighth-lord-house",
            ]
            .map(str::to_owned),
        )
        .filter(|id| !en.contains(id.as_str()))
        .collect();
    assert_eq!(missing, [""; 0]);
}
