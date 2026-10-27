site-name = hora
home-intro = Védikus (Jyotish) asztrológia: chakrák és tudástár, magyarul és angolul.
# The other language's name, in that language.
other-language = English
source-code = Forráskód (AGPL-3.0-or-later)

## The chart form
field-date = Születési dátum (ÉÉÉÉ-HH-NN)
field-time = Születési idő
field-calendar = Naptár
calendar-gregorian = Gergely-naptár
calendar-julian = Julián-naptár
field-utc = UTC-eltérés
utc-stated = Alább megadva
utc-local-mean = Helyi középidő
utc-local-apparent = Helyi valódi idő
field-offset = Megadott UTC-eltérés (+ÓÓ:PP vagy −ÓÓ:PP, másodperc is megadható)
field-latitude = Szélesség (fok, északra pozitív)
field-longitude = Hosszúság (fok, keletre pozitív)
field-altitude = Tengerszint feletti magasság (méter, nem kötelező)
field-ayanamsa = Ayanamsa
ayanamsa-lahiri = Lahiri
ayanamsa-true-chitra = True Chitra
ayanamsa-raman = Raman
ayanamsa-true-pushya = True Pushya
ayanamsa-true-mula = True Mula
field-node = Holdcsomó típusa
node-mean = Közép
node-true = Valódi
field-sunrise = Napkelte-meghatározás
sunrise-upper-limb-apparent = A napkorong felső széle, látszólagos (első sugár)
sunrise-centre-true = A napkorong közepe, valódi
sunrise-upper-limb-true = A napkorong felső széle, valódi
sunrise-centre-apparent = A napkorong közepe, látszólagos
field-reference = A bhavák viszonyítási pontja
field-style = Chakra rajzmódja
style-south = Dél-indiai
style-north = Észak-indiai
style-east = Kelet-indiai
submit = Chakra számítása
error-date = Valós dátumot adjon meg ÉÉÉÉ-HH-NN alakban, 1200-01-04 és 2399-12-28 között (Julián-naptárban 2399-12-12-ig).
error-before-gregorian = A Gergely-naptár 1582-10-15-én kezdődik; korábbi dátumhoz válassza a Julián-naptárt.
error-time = Az időt ÓÓ:PP vagy ÓÓ:PP:MM alakban adja meg.
error-offset = Az eltérést +ÓÓ:PP vagy −ÓÓ:PP alakban adja meg, −12:00 és +14:00 között.
error-latitude = A szélességet fokban adja meg, −90 és 90 között.
error-longitude = A hosszúságot fokban adja meg, −180 és 180 között.
error-altitude = A magasságot méterben adja meg, −500 és 9000 között, vagy hagyja üresen.
error-choice = Válasszon a felsorolt lehetőségek közül.

## The chart
chart = Chakra
graha = Graha
rasi = Rasi
bhava = Bhava
position = Fok
retrograde = retrográd
# The mark after a retrograde graha's degree.
retrograde-mark = R
lagna = Lagna
graha-sun = Nap
graha-moon = Hold
graha-mars = Mars
graha-mercury = Merkúr
graha-jupiter = Jupiter
graha-venus = Vénusz
graha-saturn = Szaturnusz
graha-rahu = Rahu
graha-ketu = Ketu
rasi-aries = Kos
rasi-taurus = Bika
rasi-gemini = Ikrek
rasi-cancer = Rák
rasi-leo = Oroszlán
rasi-virgo = Szűz
rasi-libra = Mérleg
rasi-scorpio = Skorpió
rasi-sagittarius = Nyilas
rasi-capricorn = Bak
rasi-aquarius = Vízöntő
rasi-pisces = Halak
nakshatra = Nakshatra
pada = Pada
vimsottari-lord = Vimsottari ura
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
chakra-title = Rasi chakra
# Two letters on the drawn chart; the lagna's is bold.
lagna-short = La
graha-sun-short = Na
graha-moon-short = Ho
graha-mars-short = Ma
graha-mercury-short = Me
graha-jupiter-short = Ju
graha-venus-short = Ve
graha-saturn-short = Sz
graha-rahu-short = Ra
graha-ketu-short = Ke
# The column of these letters in the knowledge base's graha table.
short-label = Jelölés
# A label's accessible name.
chakra-label = { $graha }{ $retrograde ->
    [yes] , retrográd
   *[no] {""}
}, { $degrees } fok { $minutes } perc, { $rasi }
chakra-legend = A La a lagna. Az R retrográd grahát jelöl. A kis számok a rasin belüli fok és perc, levágva, nem kerekítve.
chakra-legend-north = A felső rombusz az 1. bhava; a bhavák az óramutató járásával ellentétesen követik egymást. Az egyes mezőkben álló szám a mező rasija, a Kos 1-től a Halak 12-ig.

## The Vedic day
vedic-day = Védikus nap
vedic-day-sunrise = Napkelte
vedic-day-sunset = Napnyugta
vedic-day-next-sunrise = Következő napkelte
vedic-day-date = { $year }. { $month ->
    [1] január
    [2] február
    [3] március
    [4] április
    [5] május
    [6] június
    [7] július
    [8] augusztus
    [9] szeptember
    [10] október
    [11] november
   *[12] december
} { $day }.
vedic-day-none = Ezen a helyen ezen a napon nincs védikus nap: a Nap nem kel fel vagy nem nyugszik le.

## The panchaanga
panchaanga = Panchaanga
panchaanga-limb = Elem
panchaanga-value = Érték
lord = Ura
panchaanga-no-vedic-day = Itt nincs védikus nap, így vaara és hora sincs.
panchaanga-far-offset = A megadott UTC-eltérés messze van e hely helyi idejétől, így a vaara és a hora, amelyek a napkelte ebben az időben vett dátumát követik, egy nappal eltérhetnek.
limb-spans = Az elemek kezdete és vége
limb-spans-start = Kezdete
limb-spans-end = Vége
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
vaara-sunday = vasárnap
vaara-monday = hétfő
vaara-tuesday = kedd
vaara-wednesday = szerda
vaara-thursday = csütörtök
vaara-friday = péntek
vaara-saturday = szombat
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
lunar-month = Holdhónap
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
busy-heading = Foglalt
busy-text = A szerver most más chakrákat számol. Próbálja újra néhány másodperc múlva.
failure-heading = Hiba történt
failure-text = A chakrát nem sikerült kiszámítani. Próbálja újra később.

## The knowledge base
kb = Tudástár
kb-intro = Az oldalak abban a sorrendben, ahogyan a könyv a fogalmaikat tanítja.
# Where a page's concept is in the book.
kb-chapter-section = { $chapter }. fejezet, § { $section }
kb-reference = Hivatkozás a könyvre
kb-book = P.V.R. Narasimha Rao, Vedic Astrology: An Integrated Approach
kb-book-link = Hol szerezhető be a könyv
kb-not-found-heading = Az oldal nem található
kb-not-found-text = Ezen a címen nincs tudástár-oldal.
longitude = Hosszúság
