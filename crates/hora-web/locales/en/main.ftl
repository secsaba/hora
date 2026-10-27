site-name = hora
home-intro = Vedic (Jyotish) astrology: charts and a knowledge base, in English and Hungarian.
# The other language's name, in that language.
other-language = Magyar
source-code = Source code (AGPL-3.0-or-later)

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
field-offset = Stated UTC offset (+HH:MM or −HH:MM, seconds optional)
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
submit = Calculate the chart
error-date = Enter a real date as YYYY-MM-DD, from 1200-01-04 to 2399-12-28 (in the Julian calendar, to 2399-12-12).
error-before-gregorian = The Gregorian calendar starts on 1582-10-15; choose Julian for an earlier date.
error-time = Enter the time as HH:MM or HH:MM:SS.
error-offset = Enter the offset as +HH:MM or −HH:MM, from −12:00 to +14:00.
error-latitude = Enter the latitude in degrees, between −90 and 90.
error-longitude = Enter the longitude in degrees, from −180 to 180.
error-altitude = Enter the altitude in metres, from −500 to 9000, or leave it empty.
error-choice = Choose one of the listed options.

## The chart
chart = Chart
graha = Graha
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
yoga = Yoga
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
kb-reference = Book reference
kb-book = P.V.R. Narasimha Rao, Vedic Astrology: An Integrated Approach
kb-book-link = Where to get the book
kb-not-found-heading = Page not found
kb-not-found-text = There is no knowledge base page at this address.
longitude = Longitude
