site-name = hora
home-intro = Vedic (Jyotish) astrology: charts and a knowledge base, in English and Hungarian.
# The other language's name, in that language.
other-language = Magyar
source-code = Source code (AGPL-3.0-or-later)
# The running release, e.g. 0.1.2.
version = Version { $version }

## The chart form
field-date = Birth date (YYYY-MM-DD)
field-time = Birth time
field-calendar = Calendar
calendar-gregorian = Gregorian
calendar-julian = Julian
field-utc = UTC offset
utc-stated = As stated below
utc-local-mean = Local mean time
utc-local-apparent = Local apparent time
utc-zone = From the birth place's time zone
field-offset = Stated UTC offset (+HH:MM or −HH:MM, seconds optional)
field-place = Birth place (type its name and choose it from the list; if none appears, calculate to search. Or leave it empty and enter the coordinates)
place-choose = Choose the birth place
# One search result: $name is the name matched, with the place's main name in brackets when it differs; $country is its two-letter country code.
place-result = { $name }, { $country } ({ $zone }; { $latitude }, { $longitude })
# Followed by the linked name of the data source and its licence.
place-attribution = Place names and time zones:
field-latitude = Latitude (degrees, north positive)
field-longitude = Longitude (degrees, east positive)
field-altitude = Altitude (metres, optional)
field-ayanamsa = Ayanamsa
ayanamsa-lahiri = Lahiri
ayanamsa-true-chitra = True Chitra
ayanamsa-raman = Raman
ayanamsa-true-pushya = True Pushya
ayanamsa-true-mula = True Mula
field-node = Node type
node-mean = Mean
node-true = True
field-sunrise = Sunrise definition
sunrise-upper-limb-apparent = Upper edge of the disc, apparent (first ray)
sunrise-centre-true = Centre of the disc, true
sunrise-upper-limb-true = Upper edge of the disc, true
sunrise-centre-apparent = Centre of the disc, apparent
field-reference = Count bhavas from
field-style = Chart style
style-south = South Indian
style-north = North Indian
style-east = East Indian
field-settings = Method and chart style
submit = Calculate the chart
error-date = Enter a real date as YYYY-MM-DD, from 1200-01-04 to 2399-12-28 (in the Julian calendar, to 2399-12-12).
error-before-gregorian = The Gregorian calendar starts on 1582-10-15; choose Julian for an earlier date.
error-time = Enter the time as HH:MM or HH:MM:SS.
error-offset = Enter the offset as +HH:MM or −HH:MM, from −12:00 to +14:00.
error-latitude = Enter the latitude in degrees, between −90 and 90.
error-longitude = Enter the longitude in degrees, from −180 to 180.
error-altitude = Enter the altitude in metres, from −500 to 9000, or leave it empty.
error-choice = Choose one of the listed options.
error-place = Enter 2 to 100 letters or digits of the place name (spaces, hyphens, apostrophes and full stops are allowed).
error-place-none = No place of that name was found. Check the spelling, or leave the field empty and enter the coordinates.
error-utc-zone = To use a time zone, choose a birth place and keep its coordinates; or state the offset.
error-time-gap = The clocks of the birth place's time zone were put forward at this time, so it did not exist on that day. Choose the UTC offset the time was recorded in:
error-time-fold = The clocks of the birth place's time zone were put back at this time, so it happened twice on that day. Choose the UTC offset of the one meant:

## The chart
chart = Chart
graha = Graha
upagraha = Upagraha
upagrahas = Upagrahas
rasi = Rasi
bhava = Bhava
position = Degree
retrograde = retrograde
# The mark after a retrograde graha's degree.
retrograde-mark = R
lagna = Lagna
graha-sun = Sun
graha-moon = Moon
graha-mars = Mars
graha-mercury = Mercury
graha-jupiter = Jupiter
graha-venus = Venus
graha-saturn = Saturn
graha-rahu = Rahu
graha-ketu = Ketu
rasi-aries = Aries
rasi-taurus = Taurus
rasi-gemini = Gemini
rasi-cancer = Cancer
rasi-leo = Leo
rasi-virgo = Virgo
rasi-libra = Libra
rasi-scorpio = Scorpio
rasi-sagittarius = Sagittarius
rasi-capricorn = Capricorn
rasi-aquarius = Aquarius
rasi-pisces = Pisces
nakshatra = Nakshatra
pada = Pada
vimsottari-lord = Vimsottari lord
nakshatra-aswini = Aswini
nakshatra-bharani = Bharani
nakshatra-krittika = Krittika
nakshatra-rohini = Rohini
nakshatra-mrigasira = Mrigasira
nakshatra-aardra = Aardra
nakshatra-punarvasu = Punarvasu
nakshatra-pushyami = Pushyami
nakshatra-aasresha = Aasresha
nakshatra-makha = Makha
nakshatra-poorva-phalguni = Poorva Phalguni
nakshatra-uttara-phalguni = Uttara Phalguni
nakshatra-hasta = Hasta
nakshatra-chitra = Chitra
nakshatra-swaati = Swaati
nakshatra-visaakha = Visaakha
nakshatra-anooraadha = Anooraadha
nakshatra-jyeshtha = Jyeshtha
nakshatra-moola = Moola
nakshatra-poorvaashaadha = Poorvaashaadha
nakshatra-uttaraashaadha = Uttaraashaadha
nakshatra-sravanam = Sravanam
nakshatra-dhanishtha = Dhanishtha
nakshatra-satabhishak = Satabhishak
nakshatra-poorvaabhaadra = Poorvaabhaadra
nakshatra-uttaraabhaadra = Uttaraabhaadra
nakshatra-revati = Revati

## The drawn chart
chakra-title = Rasi chart
# Two letters on the drawn chart; the lagna's is bold.
lagna-short = La
graha-sun-short = Su
graha-moon-short = Mo
graha-mars-short = Ma
graha-mercury-short = Me
graha-jupiter-short = Ju
graha-venus-short = Ve
graha-saturn-short = Sa
graha-rahu-short = Ra
graha-ketu-short = Ke
# The column of these letters in the knowledge base's graha table.
short-label = Chart label
# A label's accessible name.
chakra-label = { $graha }{ $retrograde ->
    [yes] , retrograde
   *[no] {""}
}, { $degrees } { $degrees ->
    [one] degree
   *[other] degrees
} { $minutes } { $minutes ->
    [one] minute
   *[other] minutes
} in { $rasi }
chakra-legend = La is the lagna. R marks a retrograde graha. The small numbers are the degrees and minutes within the rasi, cut off, not rounded.
chakra-legend-special = HL and GL are the hora and ghati lagnas.
chakra-legend-north = The top diamond is the 1st bhava; the bhavas run counter-clockwise. The number in each area is its rasi, from 1 for Aries to 12 for Pisces.

## The Vedic day
vedic-day = Vedic day
vedic-day-sunrise = Sunrise
vedic-day-sunset = Sunset
vedic-day-next-sunrise = Next sunrise
vedic-day-date = { $month ->
    [1] January
    [2] February
    [3] March
    [4] April
    [5] May
    [6] June
    [7] July
    [8] August
    [9] September
    [10] October
    [11] November
   *[12] December
} { $day }, { $year }
vedic-day-none = There is no Vedic day at this place on this date: the Sun does not rise or does not set.

## The panchaanga
panchaanga = Panchaanga
panchaanga-limb = Limb
panchaanga-value = Value
lord = Lord
panchaanga-no-vedic-day = There is no Vedic day here, so no vaara or hora.
panchaanga-far-offset = The stated UTC offset is far from this place's local time, so the vaara and hora, which take the date of sunrise in that time, may be a day off.
zone-used = UTC offset { $offset }, from the time zone { $zone } of { $place }.
zone-before-1970 = Before 1970 the time zone database is often wrong outside a zone's main city. If you know the offset in use at the birth, choose “As stated below” and enter it.
limb-spans = Start and end of the limbs
limb-spans-start = Started
limb-spans-end = Ends
tithi = Tithi
paksha-sukla = Sukla
paksha-krishna = Krishna
tithi-pratipat = Pratipat
tithi-dwitiya = Dwitiya
tithi-tritiya = Tritiya
tithi-chaturthi = Chaturthi
tithi-panchami = Panchami
tithi-shashti = Shashti
tithi-saptami = Saptami
tithi-ashtami = Ashtami
tithi-navami = Navami
tithi-dasami = Dasami
tithi-ekadasi = Ekadasi
tithi-dwadasi = Dwadasi
tithi-trayodasi = Trayodasi
tithi-chaturdasi = Chaturdasi
tithi-paurnami = Paurnami
tithi-amavasya = Amavasya
vaara = Vaara
vaara-sunday = Sunday
vaara-monday = Monday
vaara-tuesday = Tuesday
vaara-wednesday = Wednesday
vaara-thursday = Thursday
vaara-friday = Friday
vaara-saturday = Saturday
sun-moon-yoga = Sun-Moon yoga
sun-moon-yoga-vishkambha = Vishkambha
sun-moon-yoga-preeti = Preeti
sun-moon-yoga-aayushmaan = Aayushmaan
sun-moon-yoga-saubhaagya = Saubhaagya
sun-moon-yoga-sobhana = Sobhana
sun-moon-yoga-atiganda = Atiganda
sun-moon-yoga-sukarman = Sukarman
sun-moon-yoga-dhriti = Dhriti
sun-moon-yoga-shoola = Shoola
sun-moon-yoga-ganda = Ganda
sun-moon-yoga-vriddhi = Vriddhi
sun-moon-yoga-dhruva = Dhruva
sun-moon-yoga-vyaaghaata = Vyaaghaata
sun-moon-yoga-harshana = Harshana
sun-moon-yoga-vajra = Vajra
sun-moon-yoga-siddhi = Siddhi
sun-moon-yoga-vyatipaata = Vyatipaata
sun-moon-yoga-variyan = Variyan
sun-moon-yoga-parigha = Parigha
sun-moon-yoga-shiva = Shiva
sun-moon-yoga-siddha = Siddha
sun-moon-yoga-saadhya = Saadhya
sun-moon-yoga-subha = Subha
sun-moon-yoga-sukla = Sukla
sun-moon-yoga-brahma = Brahma
sun-moon-yoga-indra = Indra
sun-moon-yoga-vaidhriti = Vaidhriti
karana = Karana
karana-bava = Bava
karana-balava = Balava
karana-kaulava = Kaulava
karana-taitula = Taitula
karana-garija = Garija
karana-vanija = Vanija
karana-vishti = Vishti
karana-sakuna = Sakuna
karana-chatushpada = Chatushpada
karana-naga = Naga
karana-kimstughna = Kimstughna
hora = Hora
lunar-month = Lunar month
lunar-month-chaitra = Chaitra
lunar-month-vaisaakha = Vaisaakha
lunar-month-jyeshtha = Jyeshtha
lunar-month-aashaadha = Aashaadha
lunar-month-sraavana = Sraavana
lunar-month-bhaadrapada = Bhaadrapada
lunar-month-aaswayuja = Aaswayuja
lunar-month-kaarteeka = Kaarteeka
lunar-month-maargasira = Maargasira
lunar-month-pushya = Pushya
lunar-month-maagha = Maagha
lunar-month-phaalguna = Phaalguna
lunar-month-adhika = Adhika
lunar-month-nija = Nija

## When a chart cannot be calculated
busy-heading = Busy
busy-text = The server is busy calculating other charts. Try again in a few seconds.
failure-heading = Something went wrong
failure-text = The chart could not be calculated. Try again later.

## The knowledge base
kb = Knowledge base
kb-intro = The pages in the order the book teaches their concepts.
# Where a page's concept is in the book.
kb-chapter-section = chapter { $chapter }, § { $section }
kb-section = § { $section }
# The book's chapter titles, heading the index's groups and each page.
kb-chapter-1 = Chapter 1: Basic concepts
kb-chapter-2 = Chapter 2: Rasis
kb-chapter-3 = Chapter 3: Grahas
kb-chapter-4 = Chapter 4: Upagrahas
# The links from a page to its neighbours on the learning path.
kb-path = Learning path
kb-previous = Previous
kb-next = Next
kb-reference = Book reference
kb-book = P.V.R. Narasimha Rao, Vedic Astrology: An Integrated Approach
kb-book-link = Where to get the book
kb-not-found-heading = Page not found
kb-not-found-text = There is no knowledge base page at this address.
# Columns of the knowledge base's rasi table; the names and symbols are the book's (ch. 1 §1.3.2, Table 1).
rasi-sanskrit-name = Sanskrit name
rasi-symbol = Symbol
rasi-start = Start
rasi-end = End
rasi-aries-sanskrit = Mesha
rasi-taurus-sanskrit = Vrishabha
rasi-gemini-sanskrit = Mithuna
rasi-cancer-sanskrit = Karkataka
rasi-leo-sanskrit = Simha
rasi-virgo-sanskrit = Kanya
rasi-libra-sanskrit = Thula
rasi-scorpio-sanskrit = Vrischika
rasi-sagittarius-sanskrit = Dhanus
rasi-capricorn-sanskrit = Makara
rasi-aquarius-sanskrit = Kumbha
rasi-pisces-sanskrit = Meena
rasi-aries-short = Ar
rasi-taurus-short = Ta
rasi-gemini-short = Ge
rasi-cancer-short = Cn
rasi-leo-short = Le
rasi-virgo-short = Vi
rasi-libra-short = Li
rasi-scorpio-short = Sc
rasi-sagittarius-short = Sg
rasi-capricorn-short = Cp
rasi-aquarius-short = Aq
rasi-pisces-short = Pi
# Columns of the knowledge base's nakshatra table; deities as the book's Table 2 prints them (ch. 1 §1.3.6), but Brahma for its Bramha.
nakshatra-common = Common spelling
nakshatra-start = Start
nakshatra-end = End
nakshatra-deity = Ruling deity
# The main names in Wikipedia's List of Nakshatras, without diacritics (GLOSSARY.md, Nakshatra).
nakshatra-aswini-common = Ashvini
nakshatra-bharani-common = Bharani
nakshatra-krittika-common = Krittika
nakshatra-rohini-common = Rohini
nakshatra-mrigasira-common = Mrigashira
nakshatra-aardra-common = Ardra
nakshatra-punarvasu-common = Punarvasu
nakshatra-pushyami-common = Pushya
nakshatra-aasresha-common = Ashlesha
nakshatra-makha-common = Magha
nakshatra-poorva-phalguni-common = Purva Phalguni
nakshatra-uttara-phalguni-common = Uttara Phalguni
nakshatra-hasta-common = Hasta
nakshatra-chitra-common = Chitra
nakshatra-swaati-common = Svati
nakshatra-visaakha-common = Vishakha
nakshatra-anooraadha-common = Anuradha
nakshatra-jyeshtha-common = Jyeshtha
nakshatra-moola-common = Mula
nakshatra-poorvaashaadha-common = Purva Ashadha
nakshatra-uttaraashaadha-common = Uttara Ashadha
nakshatra-sravanam-common = Shravana
nakshatra-dhanishtha-common = Dhanishta
nakshatra-satabhishak-common = Shatabhisha
nakshatra-poorvaabhaadra-common = Purva Bhadrapada
nakshatra-uttaraabhaadra-common = Uttara Bhadrapada
nakshatra-revati-common = Revati
nakshatra-aswini-deity = Aswini Kumara
nakshatra-bharani-deity = Yama
nakshatra-krittika-deity = Agni
nakshatra-rohini-deity = Brahma
nakshatra-mrigasira-deity = Moon
nakshatra-aardra-deity = Shiva
nakshatra-punarvasu-deity = Aditi
nakshatra-pushyami-deity = Jupiter
nakshatra-aasresha-deity = Rahu
nakshatra-makha-deity = Sun
nakshatra-poorva-phalguni-deity = Aryaman
nakshatra-uttara-phalguni-deity = Sun
nakshatra-hasta-deity = Viswakarma
nakshatra-chitra-deity = Vaayu
nakshatra-swaati-deity = Indra
nakshatra-visaakha-deity = Mitra
nakshatra-anooraadha-deity = Indra
nakshatra-jyeshtha-deity = Nirriti
nakshatra-moola-deity = Varuna
nakshatra-poorvaashaadha-deity = Viswadeva
nakshatra-uttaraashaadha-deity = Brahma
nakshatra-sravanam-deity = Vishnu
nakshatra-dhanishtha-deity = Vasu
nakshatra-satabhishak-deity = Varuna
nakshatra-poorvaabhaadra-deity = Ajacharana
nakshatra-uttaraabhaadra-deity = Ahirbudhanya
nakshatra-revati-deity = Pooshan
# Columns of the knowledge base's lunar month table; the months as the book's Table 4 abbreviates them (ch. 1 §1.3.8.2).
lunar-month-conjunction = Rasi of the conjunction
lunar-month-full-moon = Likely nakshatra of the full Moon
lunar-month-when = Roughly when
gregorian-month-short = { $month ->
    [1] Jan
    [2] Feb
    [3] Mar
    [4] Apr
    [5] May
    [6] June
    [7] July
    [8] Aug
    [9] Sept
    [10] Oct
    [11] Nov
   *[12] Dec
}
# Columns of the knowledge base's Sun-Moon yoga table; the meanings as the book's Table 5 gives them (ch. 1 §1.3.9).
sun-moon-yoga-number = Number
sun-moon-yoga-meaning = Meaning
sun-moon-yoga-vishkambha-meaning = Door bolt, supporting pillar
sun-moon-yoga-preeti-meaning = Love, affection
sun-moon-yoga-aayushmaan-meaning = Long-lived
sun-moon-yoga-saubhaagya-meaning = Good fortune, long life of the spouse
sun-moon-yoga-sobhana-meaning = Splendid, bright
sun-moon-yoga-atiganda-meaning = Great danger
sun-moon-yoga-sukarman-meaning = One who does good deeds
sun-moon-yoga-dhriti-meaning = Firmness
sun-moon-yoga-shoola-meaning = Shiva's weapon of destruction (pain)
sun-moon-yoga-ganda-meaning = Danger
sun-moon-yoga-vriddhi-meaning = Growth
sun-moon-yoga-dhruva-meaning = Fixed, constant
sun-moon-yoga-vyaaghaata-meaning = Great blow
sun-moon-yoga-harshana-meaning = Cheerful
sun-moon-yoga-vajra-meaning = Diamond (strong)
sun-moon-yoga-siddhi-meaning = Accomplishment
sun-moon-yoga-vyatipaata-meaning = Great fall
sun-moon-yoga-variyan-meaning = Chief, best
sun-moon-yoga-parigha-meaning = Obstacle, hindrance
sun-moon-yoga-shiva-meaning = Lord Shiva (purity)
sun-moon-yoga-siddha-meaning = Accomplished, ready
sun-moon-yoga-saadhya-meaning = Possible
sun-moon-yoga-subha-meaning = Auspicious
sun-moon-yoga-sukla-meaning = White, bright
sun-moon-yoga-brahma-meaning = The creator (good knowledge and purity)
sun-moon-yoga-indra-meaning = Ruler of the gods
sun-moon-yoga-vaidhriti-meaning = A class of gods
# Columns of the knowledge base's karana table (ch. 1 §1.3.10).
karana-first-half = First half
karana-second-half = Second half
# Rasi characteristics (ch. 2 §2.2.1–2.2.12), in the glossary's form.
limb-head = head
limb-face = face
limb-arms = arms
limb-heart = heart
limb-stomach = stomach
limb-hip = hip
limb-below-navel = below the navel
limb-private-parts = private parts
limb-thighs = thighs
limb-knees = knees
limb-ankles = ankles
limb-feet = feet
parity-vishama = vishama (odd)
parity-sama = sama (even)
footedness-vishamapada = vishamapada (odd-footed)
footedness-samapada = samapada (even-footed)
mobility-chara = chara (movable)
mobility-sthira = sthira (fixed)
mobility-dwiswabhava = dwiswabhava (dual)
element-agni = agni (fire)
element-bhoo = bhoo (earth)
element-vaayu = vaayu (air)
element-jala = jala (water)
nature-pitta = pitta (bilious)
nature-vaata = vaata (windy)
nature-kapha = kapha (phlegmatic)
nature-mixed = mixed
quality-sattwa = sattwa (purity)
quality-rajas = rajas (energy)
quality-tamas = tamas (darkness)
direction-east = east
direction-south = south
direction-west = west
direction-north = north
colour-blood-red = blood red
colour-white = white
colour-grass-green = grass green
colour-pale-red = pale red
colour-variegated = variegated
colour-black = black
colour-reddish-brown = reddish brown
colour-husk-of-grass = the colour of the husk of grass
colour-brown = brown, that of a mongoose
colour-cream = cream, or the colour of fish
strong-by-divaa = divaa (day)
strong-by-nishaa = nishaa (night)
rising-seershodaya = seershodaya (head-rising)
rising-prishthodaya = prishthodaya (feet-rising)
rising-both = both head and feet
class-brahmana = Brahmana (scholar)
class-kshatriya = Kshatriya (warrior)
class-vaisya = Vaisya (trader)
class-sudra = Sudra (worker)
# Graha characteristics (ch. 3 §3.2.1–3.2.17), in the glossary's form; class-, quality-, direction- and paksha- are shared.
avatara-matsya = Matsya, or Meena (fish)
avatara-koorma = Koorma (tortoise)
avatara-varaaha = Varaaha, or Sookara (boar)
avatara-narasimha = Narasimha, or Nrisimha (half man, half lion)
avatara-vaamana = Vaamana (learned dwarf)
avatara-parasu-rama = Parasu Rama, or Bhaargava Rama
avatara-rama = Rama
avatara-krishna = Krishna
avatara-buddha = Buddha
disposition-saumya = saumya (natural benefic)
disposition-kroora = kroora (natural malefic)
disposition-by-company = saumya alone or with more natural benefics, kroora when joined by more natural malefics
disposition-by-paksha = saumya waxing in Sukla paksha, kroora waning in Krishna paksha
governance-soul = soul
governance-mind = mind
governance-strength = strength
governance-speech = speech
governance-knowledge-and-happiness = knowledge and happiness
governance-potency = potency
governance-grief = grief
colour-tawny = tawny
cabinet-king = king
cabinet-leader = leader (army chief)
cabinet-prince = prince
cabinet-minister = minister
cabinet-servant = servant
cabinet-army = army
deity-agni = Agni (god of fire)
deity-varuna = Varuna (god of rain)
deity-subrahmanya = Subrahmanya (army chief of the gods)
deity-maha-vishnu = Maha Vishnu (supreme sustaining force)
deity-indra = Indra (ruler of the gods)
deity-sachi-devi = Sachi Devi (Indra's wife)
deity-brahma = Brahma (the Creator)
sex-male = male
sex-female = female
element-aakaasa = aakaasa (ether)
abode-temple = temple
abode-watery-place = watery place
abode-sports-ground = sports ground
abode-treasure-house = treasure house
abode-bedroom = bedroom
abode-filthy-area = filthy area
sapta-dhaatu-bones = bones
sapta-dhaatu-blood = blood
sapta-dhaatu-marrow = marrow
sapta-dhaatu-skin = skin
sapta-dhaatu-fat = fat
sapta-dhaatu-semen = semen (the reproductive system's matter)
sapta-dhaatu-muscles = muscles
period-ayana = ayana (half a year)
period-minute = minute
period-week = week
period-ritu = ritu (season of two months)
period-month = month
period-fortnight = fortnight
period-year = year
taste-pungent = pungent
taste-saline = saline
taste-bitter = bitter
taste-mixed = mixed
taste-sweet = sweet
taste-sour = sour
taste-astringent = astringent
strong-by-always = always
ayana-uttara = Uttara (north course)
ayana-dakshina = Dakshina (south course)
ritu-vasanta = vasanta (spring)
ritu-greeshma = greeshma (summer)
ritu-varsha = varsha (rainy season)
ritu-hemanta = hemanta (season of dew)
ritu-seeta = seeta (winter)
ritu-sisira = sisira (fall)
realm-dhatu = dhatu (metals and materials)
realm-moola = moola (roots and vegetables)
realm-jeeva = jeeva (living beings)
# Dignities and relationships (ch. 3 §3.3–3.4), in the glossary's form; relationship- names both natural and compound values.
dignity-uchcha = uchcha (exalted)
dignity-moolatrikona = moolatrikona
dignity-own-rasi = own rasi
dignity-neecha = neecha (debilitated)
relationship-adhimitra = adhimitra (good friend)
relationship-mitra = mitra (friend)
relationship-sama = sama (neutral)
relationship-satru = satru (enemy)
relationship-adhisatru = adhisatru (bad enemy)
tatkaala-mitra = tatkaala mitra (temporary friend)
tatkaala-satru = tatkaala satru (temporary enemy)
# The bare terms, for the chart table's narrow cells.
dignity-uchcha-short = uchcha
dignity-moolatrikona-short = moolatrikona
dignity-own-rasi-short = own rasi
dignity-neecha-short = neecha
relationship-adhimitra-short = adhimitra
relationship-mitra-short = mitra
relationship-sama-short = sama
relationship-satru-short = satru
relationship-adhisatru-short = adhisatru
# Chart table columns.
dignity = Dignity
rasi-lord = Rasi lord
# Column headings of Tables 6–8 and the degree splits on the knowledge base pages (ch. 3 §3.3–3.4).
dignity-own-rasis = Own rasis
dignity-uchcha-rasi = Uchcha (deep exaltation)
dignity-neecha-rasi = Neecha (deep debilitation)
dignity-moolatrikona-rasi = Moolatrikona
dignity-degrees = Degrees
relationship-natural = Natural relationship
# Column and label of each rasi characteristic on the knowledge base page (ch. 2 §2.2).
rasi-characteristic-rasis = Rasis
rasi-characteristic-limb = Limb of Vishnu
rasi-characteristic-vishama-rasi = Odd or even
rasi-characteristic-vishamapada-rasi = Odd- or even-footed
rasi-characteristic-chara-sthira-dwiswabhava = Movement
rasi-characteristic-element = Element
rasi-characteristic-pitta-vaata-kapha = Ayurvedic nature
rasi-characteristic-guna = Guna
rasi-characteristic-direction = Direction
rasi-characteristic-colour = Colour
rasi-characteristic-divaa-rasi = Strong
rasi-characteristic-seershodaya-rasi = Rises
rasi-characteristic-varna = Varna
# Column and label of each graha characteristic on the knowledge base page (ch. 3 §3.2).
graha-characteristic-grahas = Grahas
graha-characteristic-avatara = Avatara
graha-characteristic-saumya-graha = Benefic or malefic
graha-characteristic-governance = Main governance
graha-characteristic-colour = Colour
graha-characteristic-cabinet = Cabinet
graha-characteristic-deity = Deity
graha-characteristic-sex = Sex
graha-characteristic-element = Element
graha-characteristic-varna = Varna
graha-characteristic-guna = Guna
graha-characteristic-abode = Abode
graha-characteristic-sapta-dhaatu = Sapta dhaatu
graha-characteristic-time-period = Time period
graha-characteristic-taste = Taste
graha-characteristic-digbala = Digbala
graha-characteristic-day-and-night = Strong
graha-characteristic-paksha = Strong paksha
graha-characteristic-ayana = Strong ayana
graha-characteristic-ritu = Ritu
graha-characteristic-dhatu-moola-jeeva = Rules over

# Upagrahas in the glossary's spelling, and their natures as far as the book gives them (ch. 4 §4.2–4.3).
upagraha-dhuma = Dhuma
upagraha-vyatipaata = Vyatipaata
upagraha-parivesha = Parivesha
upagraha-indrachaapa = Indrachaapa
upagraha-upaketu = Upaketu
upagraha-kaala = Kaala
upagraha-mrityu = Mrityu
upagraha-arthaprahaara = Arthaprahaara
upagraha-yamaghantaka = Yamaghantaka
upagraha-gulika = Gulika
upagraha-maandi = Maandi
upagraha-nature-very-malefic = very malefic
upagraha-nature-malefic-like-sun = malefic, like the Sun
upagraha-nature-malefic-like-mars = malefic, like Mars
upagraha-nature-like-mercury = like Mercury
upagraha-nature-like-jupiter = like Jupiter
upagraha-nature-like-saturn = like Saturn
upagraha-longitude = Longitude
upagraha-nature = Nature
upagrahas-no-vedic-day = There is no Vedic day here, so only the five Sun-based upagrahas.

special-lagnas = Special lagnas
special-lagna = Special lagna
special-lagnas-no-vedic-day = There is no Vedic day here, so only the sree lagna: the others count from sunrise.
# Special lagnas in the glossary's spelling, and their abbreviations (ch. 5).
special-lagna-hora-lagna = Hora lagna
special-lagna-ghati-lagna = Ghati lagna
special-lagna-sree-lagna = Sree lagna
special-lagna-bhaava-lagna = Bhaava lagna
special-lagna-abbreviation-hora-lagna = HL
special-lagna-abbreviation-ghati-lagna = GL
special-lagna-abbreviation-sree-lagna = SL
special-lagna-abbreviation-bhaava-lagna = BL

## The privacy page
privacy = Privacy
privacy-stored = hora stores nothing about you: no accounts, no cookies, and no birth details once your chakra is drawn.
privacy-cloudflare = Every request passes through Cloudflare, which carries the site's traffic, so Cloudflare sees it, including the birth details in a chakra's address.
privacy-logs = When a chakra cannot be calculated or shown, the error log records its address, birth details included, so the failure can be reproduced. The log keeps it only until newer entries overwrite it.

## The disclaimer page
disclaimer = Disclaimer
disclaimer-text = hora is for study and reflection, not medical, legal, financial or psychological advice.
