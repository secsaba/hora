//! Birth places from GeoNames, searched by name in-process so a birth place never leaves the server (ADR 0005).

use std::cmp::Reverse;
use std::collections::HashMap;
use std::path::Path;

use anyhow::Context;
use jiff::tz::{TimeZone, TimeZoneDatabase};

/// One row of the flake's `places` file.
pub struct Place {
    pub id: u32,
    /// The primary name first, then the ASCII and alternate names; tab-separated, since a few primary names hold commas.
    names: Box<str>,
    pub latitude: f64,
    pub longitude: f64,
    pub country: Box<str>,
    population: u64,
    zone: u16,
}

impl Place {
    pub fn name(&self) -> &str {
        self.names.split('\t').next().unwrap_or_default()
    }

    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.names.split('\t').filter(|name| !name.is_empty())
    }
}

pub struct Places {
    /// Sorted by id.
    places: Vec<Place>,
    /// Each zone named in the file, resolved once so a zone missing from tzdata stops startup.
    zones: Vec<(Box<str>, TimeZone)>,
    /// Every place's lowercased names, back to back.
    keys: String,
    /// (start, end, place index) of each key in `keys`, sorted by key, for prefix search.
    index: Vec<(u32, u32, u32)>,
}

/// At most this many places per search, so a short prefix stays a short page.
const RESULTS: usize = 20;

impl Places {
    pub fn load(path: &Path, tzdb: &TimeZoneDatabase) -> anyhow::Result<Places> {
        let text =
            std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
        Places::parse(&text, tzdb).with_context(|| format!("reading {}", path.display()))
    }

    /// The file's rows: geonameid, name, asciiname, alternatenames, latitude, longitude, country, population, zone.
    fn parse(text: &str, tzdb: &TimeZoneDatabase) -> anyhow::Result<Places> {
        let mut places = Places {
            places: Vec::new(),
            zones: Vec::new(),
            keys: String::new(),
            index: Vec::new(),
        };
        let mut seen = HashMap::new();
        for (number, line) in text.lines().enumerate() {
            let place = places
                .row(line, tzdb, &mut seen)
                .with_context(|| format!("line {}", number + 1))?;
            places.places.push(place);
        }
        places.places.sort_unstable_by_key(|place| place.id);
        for (i, place) in places.places.iter().enumerate() {
            let mut keys: Vec<String> = place.names().map(str::to_lowercase).collect();
            keys.sort_unstable();
            keys.dedup();
            for key in keys {
                let start = places.keys.len();
                places.keys.push_str(&key);
                places
                    .index
                    .push((start as u32, places.keys.len() as u32, i as u32));
            }
        }
        let keys = &places.keys;
        places
            .index
            .sort_unstable_by_key(|&(start, end, _)| &keys[start as usize..end as usize]);
        Ok(places)
    }

    /// `seen` maps each zone name already in `zones` to its index.
    fn row<'a>(
        &mut self,
        line: &'a str,
        tzdb: &TimeZoneDatabase,
        seen: &mut HashMap<&'a str, usize>,
    ) -> anyhow::Result<Place> {
        let fields: Vec<&str> = line.split('\t').collect();
        let [
            id,
            name,
            ascii,
            alternates,
            latitude,
            longitude,
            country,
            population,
            zone,
        ] = fields[..]
        else {
            anyhow::bail!("{} fields, not 9", fields.len());
        };
        let zone = match seen.get(zone) {
            Some(&i) => i,
            None => {
                let tz = tzdb.get(zone).with_context(|| format!("zone {zone}"))?;
                self.zones.push((zone.into(), tz));
                seen.insert(zone, self.zones.len() - 1);
                self.zones.len() - 1
            }
        };
        Ok(Place {
            id: id.parse().context("geonameid")?,
            names: [name, ascii, &alternates.replace(',', "\t")]
                .join("\t")
                .into(),
            latitude: latitude.parse().context("latitude")?,
            longitude: longitude.parse().context("longitude")?,
            country: country.into(),
            population: population.parse().context("population")?,
            zone: zone.try_into().context("too many zones")?,
        })
    }

    pub fn get(&self, id: u32) -> Option<&Place> {
        let i = self
            .places
            .binary_search_by_key(&id, |place| place.id)
            .ok()?;
        self.places.get(i)
    }

    pub fn zone(&self, place: &Place) -> (&str, &TimeZone) {
        let (name, tz) = &self.zones[usize::from(place.zone)];
        (name, tz)
    }

    /// The places with a name starting with `query`, ignoring case, each with its first such name: exact matches first, then the most populous.
    pub fn search(&self, query: &str) -> Vec<(&Place, &str)> {
        let query = query.to_lowercase();
        let key = |&(start, end, _): &(u32, u32, u32)| &self.keys[start as usize..end as usize];
        let first = self
            .index
            .partition_point(|entry| key(entry) < query.as_str());
        let mut found: Vec<(bool, u32)> = self.index[first..]
            .iter()
            .take_while(|entry| key(entry).starts_with(&query))
            .map(|entry| (key(entry) != query, entry.2))
            .collect();
        // Exact first within each place, so dedup keeps its best match.
        found.sort_unstable_by_key(|&(inexact, i)| (i, inexact));
        found.dedup_by_key(|&mut (_, i)| i);
        found.sort_by_key(|&(inexact, i)| (inexact, Reverse(self.places[i as usize].population)));
        found
            .into_iter()
            .take(RESULTS)
            .map(|(_, i)| {
                let place = &self.places[i as usize];
                let names = || place.names();
                let name = names()
                    .find(|name| name.to_lowercase() == query)
                    .or_else(|| names().find(|name| name.to_lowercase().starts_with(&query)))
                    .unwrap_or(place.name());
                (place, name)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn places(rows: &[&str]) -> Places {
        let tzdb = TimeZoneDatabase::from_env();
        Places::parse(&rows.join("\n"), &tzdb).unwrap()
    }

    const ROWS: &[&str] = &[
        "3054643\tBudapest\tBudapest\tBudapeste,Budapesht\t47.49835\t19.04045\tHU\t1741041\tEurope/Budapest",
        "681290\tCluj-Napoca\tCluj-Napoca\tKolozsvar,Kolozsvár,Klausenburg\t46.76667\t23.6\tRO\t286598\tEurope/Bucharest",
        "4791259\tVienna\tVienna\t\t38.90122\t-77.26526\tUS\t16522\tAmerica/New_York",
        "2761369\tVienna\tVienna\tBécs,Wien\t48.20849\t16.37208\tAT\t1691468\tEurope/Vienna",
        "3045188\tSopronhorpács\tSopronhorpacs\tSopronhorpacs,Sopronhorpács\t47.48342\t16.7353\tHU\t0\tEurope/Budapest",
        "2761370\tViennale\tViennale\t\t48.2\t16.3\tAT\t9999999\tEurope/Vienna",
    ];

    fn found<'a>(places: &'a Places, query: &str) -> Vec<(u32, &'a str)> {
        places
            .search(query)
            .into_iter()
            .map(|(place, name)| (place.id, name))
            .collect()
    }

    #[test]
    fn a_search_finds_names_by_prefix_ignoring_case_exact_ones_first_then_by_population() {
        let places = places(ROWS);
        assert_eq!(
            found(&places, "vienna"),
            [
                (2761369, "Vienna"),
                (4791259, "Vienna"),
                (2761370, "Viennale")
            ]
        );
        assert_eq!(found(&places, "BUDA"), [(3054643, "Budapest")]);
        assert!(found(&places, "Prague").is_empty());
    }

    #[test]
    fn a_search_finds_alternate_and_hungarian_names_and_shows_the_one_matched() {
        let places = places(ROWS);
        assert_eq!(found(&places, "kolozsvár"), [(681290, "Kolozsvár")]);
        assert_eq!(found(&places, "Bécs"), [(2761369, "Bécs")]);
        // The ASCII name and the alternates repeat the primary name; the place is listed once.
        assert_eq!(found(&places, "sopron"), [(3045188, "Sopronhorpács")]);
    }

    #[test]
    fn a_place_is_found_by_id_with_its_zone() {
        let places = places(ROWS);
        let place = places.get(681290).unwrap();
        assert_eq!((place.name(), &*place.country), ("Cluj-Napoca", "RO"));
        assert_eq!(places.zone(place).0, "Europe/Bucharest");
        assert!(places.get(1).is_none());
    }

    #[test]
    fn a_bad_row_or_unknown_zone_is_refused_with_its_line() {
        let tzdb = TimeZoneDatabase::from_env();
        for text in [
            "1\tA\tA\t\t1\t2\tHU\t0",
            "1\tA\tA\t\tnorth\t2\tHU\t0\tEurope/Budapest",
            "1\tA\tA\t\t1\t2\tHU\t0\tEurope/Atlantis",
        ] {
            let error = Places::parse(&format!("{}\n{text}", ROWS[0]), &tzdb)
                .err()
                .unwrap();
            assert!(format!("{error:#}").starts_with("line 2"), "{error:#}");
        }
    }
}
