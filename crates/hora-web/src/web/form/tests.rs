use super::*;
use crate::web::tests::get;
use crate::web::tests::*;
use axum::body::{Body, to_bytes};
use axum::http::Request;
use tower::ServiceExt;

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
async fn a_refused_form_keeps_the_chosen_sunrise_definition() {
    let query = with(&[("latitude", "95"), ("sunrise", "centre-true")]);
    let response = get(&format!("/en/chart?{query}"), None).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let page = page(response).await;
    assert!(page.contains(r#"value="centre-true" selected"#), "{page}");
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
