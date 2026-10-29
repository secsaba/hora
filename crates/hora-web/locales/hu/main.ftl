site-name = hora
home-intro = Védikus (Jyotish) asztrológia: chakrák és tudástár, magyarul és angolul.
# The other language's name, in that language.
other-language = English
source-code = Forráskód (AGPL-3.0-or-later)
# The running release, e.g. 0.1.2.
version = Verzió { $version }

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
utc-zone = A születési hely időzónájából
field-offset = Megadott UTC-eltérés (+ÓÓ:PP vagy −ÓÓ:PP, másodperc is megadható)
field-place = Születési hely (írja be a nevét, és válassza ki a listából; ha nem jelenik meg, számoljon a kereséshez. Vagy hagyja üresen, és adja meg a koordinátákat)
place-choose = Válassza ki a születési helyet
# One search result: $name is the name matched, with the place's main name in brackets when it differs; $country is its two-letter country code.
place-result = { $name }, { $country } ({ $zone }; { $latitude }, { $longitude })
# Followed by the linked name of the data source and its licence.
place-attribution = Helynevek és időzónák:
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
field-settings = Módszer és rajzmód
submit = Chakra számítása
error-date = Valós dátumot adjon meg ÉÉÉÉ-HH-NN alakban, 1200-01-04 és 2399-12-28 között (Julián-naptárban 2399-12-12-ig).
error-before-gregorian = A Gergely-naptár 1582-10-15-én kezdődik; korábbi dátumhoz válassza a Julián-naptárt.
error-time = Az időt ÓÓ:PP vagy ÓÓ:PP:MM alakban adja meg.
error-offset = Az eltérést +ÓÓ:PP vagy −ÓÓ:PP alakban adja meg, −12:00 és +14:00 között.
error-latitude = A szélességet fokban adja meg, −90 és 90 között.
error-longitude = A hosszúságot fokban adja meg, −180 és 180 között.
error-altitude = A magasságot méterben adja meg, −500 és 9000 között, vagy hagyja üresen.
error-choice = Válasszon a felsorolt lehetőségek közül.
error-place = A helynévből 2–100 betűt vagy számjegyet adjon meg (szóköz, kötőjel, aposztróf és pont megengedett).
error-place-none = Nincs ilyen nevű hely. Ellenőrizze az írásmódot, vagy hagyja üresen a mezőt, és adja meg a koordinátákat.
error-utc-zone = Az időzóna használatához válasszon születési helyet, és tartsa meg a koordinátáit; vagy adja meg az eltérést.
error-time-gap = A születési hely időzónájában ekkor előreállították az órákat, így ez az időpont azon a napon nem létezett. Válassza ki, milyen UTC-eltérés szerint jegyezték fel:
error-time-fold = A születési hely időzónájában ekkor visszaállították az órákat, így ez az időpont azon a napon kétszer volt. Válassza ki a kettő közül a szándékolt UTC-eltérést:

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
zone-used = UTC-eltérés: { $offset }, a születési hely ({ $place }) időzónája ({ $zone }) alapján.
zone-before-1970 = 1970 előtt az időzóna-adatbázis egy zóna névadó városán kívül gyakran téved. Ha ismeri a születéskor használt eltérést, válassza az „Alább megadva” lehetőséget, és adja meg.
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
sun-moon-yoga = Nap–Hold yoga
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
kb-section = § { $section }
# The book's chapter titles, heading the index's groups and each page.
kb-chapter-1 = 1. fejezet: Alapfogalmak
kb-chapter-2 = 2. fejezet: Rasik
kb-chapter-3 = 3. fejezet: Grahák
# The links from a page to its neighbours on the learning path.
kb-path = Tanulási útvonal
kb-previous = Előző
kb-next = Következő
kb-reference = Hivatkozás a könyvre
kb-book = P.V.R. Narasimha Rao, Vedic Astrology: An Integrated Approach
kb-book-link = Hol szerezhető be a könyv
kb-not-found-heading = Az oldal nem található
kb-not-found-text = Ezen a címen nincs tudástár-oldal.
# Columns of the knowledge base's rasi table; the names and symbols are the book's (ch. 1 §1.3.2, Table 1).
rasi-sanskrit-name = Szanszkrit név
rasi-symbol = Jel
rasi-start = Kezdet
rasi-end = Vég
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
nakshatra-common = Szokásos írásmód
nakshatra-start = Kezdet
nakshatra-end = Vég
nakshatra-deity = Uralkodó istenség
# The English common spellings: Hungarian has no established equivalents.
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
nakshatra-mrigasira-deity = Hold
nakshatra-aardra-deity = Shiva
nakshatra-punarvasu-deity = Aditi
nakshatra-pushyami-deity = Jupiter
nakshatra-aasresha-deity = Rahu
nakshatra-makha-deity = Nap
nakshatra-poorva-phalguni-deity = Aryaman
nakshatra-uttara-phalguni-deity = Nap
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
lunar-month-conjunction = Az együttállás rasija
lunar-month-full-moon = A telihold valószínű nakshatrája
lunar-month-when = Nagyjából mikor
gregorian-month-short = { $month ->
    [1] jan.
    [2] febr.
    [3] márc.
    [4] ápr.
    [5] máj.
    [6] jún.
    [7] júl.
    [8] aug.
    [9] szept.
    [10] okt.
    [11] nov.
   *[12] dec.
}
# Columns of the knowledge base's Sun-Moon yoga table; the meanings as the book's Table 5 gives them (ch. 1 §1.3.9).
sun-moon-yoga-number = Sorszám
sun-moon-yoga-meaning = Jelentés
sun-moon-yoga-vishkambha-meaning = Ajtóretesz, tartóoszlop
sun-moon-yoga-preeti-meaning = Szeretet, vonzalom
sun-moon-yoga-aayushmaan-meaning = Hosszú életű
sun-moon-yoga-saubhaagya-meaning = Szerencse, a házastárs hosszú élete
sun-moon-yoga-sobhana-meaning = Pompás, ragyogó
sun-moon-yoga-atiganda-meaning = Nagy veszély
sun-moon-yoga-sukarman-meaning = Jó tetteket végző
sun-moon-yoga-dhriti-meaning = Szilárdság
sun-moon-yoga-shoola-meaning = Shiva pusztító fegyvere (fájdalom)
sun-moon-yoga-ganda-meaning = Veszély
sun-moon-yoga-vriddhi-meaning = Növekedés
sun-moon-yoga-dhruva-meaning = Rögzített, állandó
sun-moon-yoga-vyaaghaata-meaning = Nagy csapás
sun-moon-yoga-harshana-meaning = Vidám
sun-moon-yoga-vajra-meaning = Gyémánt (erős)
sun-moon-yoga-siddhi-meaning = Beteljesülés
sun-moon-yoga-vyatipaata-meaning = Nagy bukás
sun-moon-yoga-variyan-meaning = Legfőbb, legjobb
sun-moon-yoga-parigha-meaning = Akadály, gát
sun-moon-yoga-shiva-meaning = Shiva isten (tisztaság)
sun-moon-yoga-siddha-meaning = Beteljesült, kész
sun-moon-yoga-saadhya-meaning = Lehetséges
sun-moon-yoga-subha-meaning = Kedvező
sun-moon-yoga-sukla-meaning = Fehér, fényes
sun-moon-yoga-brahma-meaning = A teremtő (jó tudás és tisztaság)
sun-moon-yoga-indra-meaning = Az istenek ura
sun-moon-yoga-vaidhriti-meaning = Istenek egy osztálya
# Columns of the knowledge base's karana table (ch. 1 §1.3.10).
karana-first-half = Első fele
karana-second-half = Második fele
# Rasi characteristics (ch. 2 §2.2.1–2.2.12), in the glossary's form.
limb-head = fej
limb-face = arc
limb-arms = karok
limb-heart = szív
limb-stomach = gyomor
limb-hip = csípő
limb-below-navel = a köldök alatti rész
limb-private-parts = nemi szervek
limb-thighs = combok
limb-knees = térdek
limb-ankles = bokák
limb-feet = lábfejek
parity-vishama = vishama (páratlan)
parity-sama = sama (páros)
footedness-vishamapada = vishamapada (páratlan lábú)
footedness-samapada = samapada (páros lábú)
mobility-chara = chara (mozgó)
mobility-sthira = sthira (állandó)
mobility-dwiswabhava = dwiswabhava (kettős)
element-agni = agni (tűz)
element-bhoo = bhoo (föld)
element-vaayu = vaayu (levegő)
element-jala = jala (víz)
nature-pitta = pitta
nature-vaata = vaata
nature-kapha = kapha
nature-mixed = vegyes
quality-sattwa = sattwa (tisztaság)
quality-rajas = rajas (energia)
quality-tamas = tamas (sötétség)
direction-east = kelet
direction-south = dél
direction-west = nyugat
direction-north = észak
colour-blood-red = vérvörös
colour-white = fehér
colour-grass-green = fűzöld
colour-pale-red = halványvörös
colour-variegated = tarka
colour-black = fekete
colour-reddish-brown = vörösesbarna
colour-husk-of-grass = a fűpelyva színe
colour-brown = barna, mint a mongúz
colour-cream = krémszín, vagy a hal színe
strong-by-divaa = divaa (nappali)
strong-by-nishaa = nishaa (éjszakai)
rising-seershodaya = seershodaya (fejjel kelő)
rising-prishthodaya = prishthodaya (lábbal kelő)
rising-both = fejjel és lábbal is
class-brahmana = Brahmana (tudós)
class-kshatriya = Kshatriya (harcos)
class-vaisya = Vaisya (kereskedő)
class-sudra = Sudra (munkás)
# Graha characteristics (ch. 3 §3.2.1–3.2.17), in the glossary's form; class-, quality-, direction- and paksha- are shared.
avatara-matsya = Matsya, vagy Meena (hal)
avatara-koorma = Koorma (teknős)
avatara-varaaha = Varaaha, vagy Sookara (vadkan)
avatara-narasimha = Narasimha, vagy Nrisimha (félig ember, félig oroszlán)
avatara-vaamana = Vaamana (tudós törpe)
avatara-parasu-rama = Parasu Rama, vagy Bhaargava Rama
avatara-rama = Rama
avatara-krishna = Krishna
avatara-buddha = Buddha
disposition-saumya = saumya (jótékony bolygó)
disposition-kroora = kroora (ártó bolygó)
disposition-by-company = saumya egyedül vagy több jótékony bolygóval, kroora, ha több ártó bolygó társul hozzá
disposition-by-paksha = saumya növekvőben, Sukla pakshában, kroora fogyóban, Krishna pakshában
governance-soul = lélek
governance-mind = elme
governance-strength = erő
governance-speech = beszéd
governance-knowledge-and-happiness = tudás és boldogság
governance-potency = nemzőerő
governance-grief = bánat
colour-tawny = sárgásbarna
cabinet-king = király
cabinet-leader = vezér (hadvezér)
cabinet-prince = herceg
cabinet-minister = miniszter
cabinet-servant = szolga
cabinet-army = hadsereg
deity-agni = Agni (a tűz istene)
deity-varuna = Varuna (az eső istene)
deity-subrahmanya = Subrahmanya (az istenek hadvezére)
deity-maha-vishnu = Maha Vishnu (a legfőbb fenntartó erő)
deity-indra = Indra (az istenek uralkodója)
deity-sachi-devi = Sachi Devi (Indra felesége)
deity-brahma = Brahma (a Teremtő)
sex-male = hímnemű
sex-female = nőnemű
element-aakaasa = aakaasa (éter)
abode-temple = templom
abode-watery-place = vizes hely
abode-sports-ground = sportpálya
abode-treasure-house = kincsesház
abode-bedroom = hálószoba
abode-filthy-area = piszkos hely
sapta-dhaatu-bones = csontok
sapta-dhaatu-blood = vér
sapta-dhaatu-marrow = velő
sapta-dhaatu-skin = bőr
sapta-dhaatu-fat = zsír
sapta-dhaatu-semen = ondó (a nemzőszervek anyaga)
sapta-dhaatu-muscles = izmok
period-ayana = ayana (fél év)
period-minute = perc
period-week = hét
period-ritu = ritu (kéthónapos évszak)
period-month = hónap
period-fortnight = két hét
period-year = év
taste-pungent = csípős
taste-saline = sós
taste-bitter = keserű
taste-mixed = vegyes
taste-sweet = édes
taste-sour = savanyú
taste-astringent = fanyar
strong-by-always = mindig
ayana-uttara = Uttara (északi út)
ayana-dakshina = Dakshina (déli út)
ritu-vasanta = vasanta (tavasz)
ritu-greeshma = greeshma (nyár)
ritu-varsha = varsha (esős évszak)
ritu-hemanta = hemanta (a harmat évszaka)
ritu-seeta = seeta (tél)
ritu-sisira = sisira (ősz)
realm-dhatu = dhatu (fémek és anyagok)
realm-moola = moola (gyökerek és zöldségek)
realm-jeeva = jeeva (élőlények)
# Dignities and relationships (ch. 3 §3.3–3.4), in the glossary's form; relationship- names both natural and compound values.
dignity-uchcha = uchcha (emelt)
dignity-moolatrikona = moolatrikona
dignity-own-rasi = saját jegy
dignity-neecha = neecha (esésben)
relationship-adhimitra = adhimitra (jó barát)
relationship-mitra = mitra (barát)
relationship-sama = sama (semleges)
relationship-satru = satru (ellenség)
relationship-adhisatru = adhisatru (nagy ellenség)
tatkaala-mitra = tatkaala mitra (ideiglenes barát)
tatkaala-satru = tatkaala satru (ideiglenes ellenség)
# The bare terms, for the chart table's narrow cells.
dignity-uchcha-short = uchcha
dignity-moolatrikona-short = moolatrikona
dignity-own-rasi-short = saját jegy
dignity-neecha-short = neecha
relationship-adhimitra-short = adhimitra
relationship-mitra-short = mitra
relationship-sama-short = sama
relationship-satru-short = satru
relationship-adhisatru-short = adhisatru
# Chart table columns.
dignity = Méltóság
rasi-lord = Jegyúr
# Column headings of Tables 6–8 and the degree splits on the knowledge base pages (ch. 3 §3.3–3.4).
dignity-own-rasis = Saját jegyek
dignity-uchcha-rasi = Uchcha (legmélyebb emelés)
dignity-neecha-rasi = Neecha (legmélyebb esés)
dignity-moolatrikona-rasi = Moolatrikona
dignity-degrees = Fokok
relationship-natural = Természetes viszony
# Column and label of each rasi characteristic on the knowledge base page (ch. 2 §2.2).
rasi-characteristic-rasis = Rasik
rasi-characteristic-limb = Vishnu testrésze
rasi-characteristic-vishama-rasi = Páratlan vagy páros
rasi-characteristic-vishamapada-rasi = Páratlan vagy páros lábú
rasi-characteristic-chara-sthira-dwiswabhava = Mozgás
rasi-characteristic-element = Elem
rasi-characteristic-pitta-vaata-kapha = Ájurvédikus természet
rasi-characteristic-guna = Guna
rasi-characteristic-direction = Irány
rasi-characteristic-colour = Szín
rasi-characteristic-divaa-rasi = Mikor erős
rasi-characteristic-seershodaya-rasi = Hogyan kel
rasi-characteristic-varna = Varna
# Column and label of each graha characteristic on the knowledge base page (ch. 3 §3.2).
graha-characteristic-grahas = Grahák
graha-characteristic-avatara = Avatara
graha-characteristic-saumya-graha = Jótékony vagy ártó
graha-characteristic-governance = Fő uralma
graha-characteristic-colour = Szín
graha-characteristic-cabinet = Kabinet
graha-characteristic-deity = Istenség
graha-characteristic-sex = Nem
graha-characteristic-element = Elem
graha-characteristic-varna = Varna
graha-characteristic-guna = Guna
graha-characteristic-abode = Lakóhely
graha-characteristic-sapta-dhaatu = Sapta dhaatu
graha-characteristic-time-period = Időszak
graha-characteristic-taste = Íz
graha-characteristic-digbala = Digbala
graha-characteristic-day-and-night = Mikor erős
graha-characteristic-paksha = Erős pakshája
graha-characteristic-ayana = Erős ayanája
graha-characteristic-ritu = Ritu
graha-characteristic-dhatu-moola-jeeva = Uralkodik

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
upagraha-nature-very-malefic = nagyon ártó
upagraha-nature-malefic-like-sun = ártó, a Naphoz hasonló
upagraha-nature-malefic-like-mars = ártó, a Marshoz hasonló
upagraha-nature-like-mercury = a Merkúrhoz hasonló
upagraha-nature-like-jupiter = a Jupiterhez hasonló
upagraha-nature-like-saturn = a Szaturnuszhoz hasonló

## The privacy page
privacy = Adatvédelem
privacy-stored = A hora semmit sem tárol Önről: nincs fiók, nincs süti, és a születési adatokat sem őrzi meg, miután a chakra elkészült.
privacy-cloudflare = Minden kérés a Cloudflare-en keresztül érkezik, amely az oldal forgalmát továbbítja, így a Cloudflare látja a kéréseket, a chakra címében szereplő születési adatokkal együtt.
privacy-logs = Ha egy chakrát nem sikerül kiszámítani vagy megjeleníteni, a hibanapló rögzíti a címét a születési adatokkal együtt, hogy a hiba megismételhető legyen. A napló csak addig őrzi, amíg az újabb bejegyzések felül nem írják.

## The disclaimer page
disclaimer = Felelősségkizárás
disclaimer-text = A hora tanulásra és elmélkedésre szolgál, nem orvosi, jogi, pénzügyi vagy pszichológiai tanácsadás.
