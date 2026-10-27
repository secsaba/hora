# Ayanamsa

Every longitude hora shows is measured on the nirayana (sidereal, fixed) zodiac, the one Jyotish uses. Astronomy, and the ephemeris hora calculates with, measures on the sayana (tropical, moving) zodiac instead. The two differ only in where they put 0°, and the angle between those two starting points is the ayanamsa. Subtract it from a sayana longitude and you have the nirayana one.

## Two starting points

The sayana zodiac starts at the March equinox, the point where the Sun crosses the celestial equator going north. The nirayana zodiac starts at a point tied to the stars. Both are cut the same way into [rasis](/en/kb/rasi) (signs) and [nakshatras](/en/kb/nakshatra) (constellations), so a [graha](/en/kb/graha) (planet) can stand in different rasis on the two zodiacs: its sayana longitude is its nirayana longitude plus the ayanamsa.

## Precession

The Earth's axis is not fixed in space. It turns slowly in a cone, like the axis of a spinning top that leans, and takes about 26,000 years to go round once. This turning is called precession, and it drags the equinox backward along the zodiac, against the stars, by about 50″ a year: a degree in about 72 years. The sayana zodiac moves with the equinox and the nirayana zodiac does not, so the ayanamsa grows by those 50″ every year. Roughly seventeen centuries ago the two starting points met; today they are a little over 24° apart.

## Why the schools disagree

The rate of precession is well measured; where the nirayana zodiac was meant to start is a question no measurement can settle. The old texts fix it by tradition, not by an observation good to the arcminute, so each school ties 0° to a star that its reading of tradition places at a known point of the zodiac. The stars do not sit in exactly the places the traditions give them relative to one another, so different stars give different ayanamsas, and with them every longitude in a chart shifts by the same amount.

hora offers five, chosen in the *Ayanamsa* field on the home page:

- **Lahiri**, the default: the value adopted in 1956 by India's Calendar Reform Committee and the one most widely used in India. It was set to put the star Chitra (Spica) opposite the start of the zodiac, at 0° Libra, but it is carried forward by a formula for precession rather than by the star itself.
- **True Chitra**: follows Spica itself and keeps it at 0° Libra at every moment. It stays within a minute or two of Lahiri. Chitra, the nakshatra, runs from 23 Vi 20 to 06 Li 40, so its star sits at its middle.
- **Raman**: the value B. V. Raman taught, used by many who learnt from his books; nearly a degree and a half smaller than Lahiri.
- **True Pushya**: keeps the star δ Cancri, the star of Pushyami, at 16° Cancer, near the end of its nakshatra. The book's author proposed it.
- **True Mula**: keeps the star λ Scorpii, the star of Moola, at 0° Sagittarius, the start of its nakshatra, as Chandra Hari proposed.

## Why Lahiri is the default

The book says it uses the Chitrapaksha, or Lahiri, ayanamsa. hora checks its calculations against the book's example charts, and their grahas agree with hora's Lahiri to within about a minute; with True Chitra every graha comes out one to two minutes further on. So hora starts from Lahiri, and the other four are there for readers who follow another school.

## A worked example

On 1 January 2000 at 12:00 UT, at Greenwich, the five ayanamsas give five values, and the Sun, one and the same body in the same sky, stands at five different places:

- [Lahiri](/en/chart?date=2000-01-01&time=12%3A00&calendar=gregorian&utc=stated&offset=%2B00%3A00&latitude=51.48&longitude=0&altitude=&ayanamsa=lahiri&node=mean&sunrise=upper-limb-apparent&reference=lagna&style=south): 23°51′25″, the Sun at 16°31′17″ in Sagittarius, in pada (quarter) 1 of Poorvaashaadha.
- [True Chitra](/en/chart?date=2000-01-01&time=12%3A00&calendar=gregorian&utc=stated&offset=%2B00%3A00&latitude=51.48&longitude=0&altitude=&ayanamsa=true-chitra&node=mean&sunrise=upper-limb-apparent&reference=lagna&style=south): 23°50′28″, the Sun at 16°32′13″, still pada 1.
- [Raman](/en/chart?date=2000-01-01&time=12%3A00&calendar=gregorian&utc=stated&offset=%2B00%3A00&latitude=51.48&longitude=0&altitude=&ayanamsa=raman&node=mean&sunrise=upper-limb-apparent&reference=lagna&style=south): 22°24′38″, the Sun at 17°58′04″, now in pada 2.
- [True Pushya](/en/chart?date=2000-01-01&time=12%3A00&calendar=gregorian&utc=stated&offset=%2B00%3A00&latitude=51.48&longitude=0&altitude=&ayanamsa=true-pushya&node=mean&sunrise=upper-limb-apparent&reference=lagna&style=south): 22°43′19″, the Sun at 17°39′23″, in pada 2.
- [True Mula](/en/chart?date=2000-01-01&time=12%3A00&calendar=gregorian&utc=stated&offset=%2B00%3A00&latitude=51.48&longitude=0&altitude=&ayanamsa=true-mula&node=mean&sunrise=upper-limb-apparent&reference=lagna&style=south): 24°35′08″, the Sun at 15°47′34″, back in pada 1; the lagna (ascendant) moves back from Aries into Pisces, so the Sun is now in the 10th [bhava](/en/kb/bhava) (house) instead of the 9th.

To go from one ayanamsa to another, pass through the sayana zodiac:

1. With Lahiri, the Sun's longitude is 256°31′17″ (the [rasi](/en/kb/rasi) page works it out). Adding the ayanamsa gives its sayana longitude: 256°31′17″ + 23°51′25″ = 280°22′42″.
2. Subtracting Raman's ayanamsa instead: 280°22′42″ − 22°24′38″ = 257°58′04″, which is 17°58′04″ into Sagittarius, the Raman chart's value.

The chart cuts off seconds rather than rounding them, so a sum like this can come out one second away from the chart; True Chitra's does.

The ayanamsa also changes with the date. At the same hour and place, Lahiri gives [22°27′37″ on 1 January 1900](/en/chart?date=1900-01-01&time=12%3A00&calendar=gregorian&utc=stated&offset=%2B00%3A00&latitude=51.48&longitude=0&altitude=&ayanamsa=lahiri&node=mean&sunrise=upper-limb-apparent&reference=lagna&style=south) and [25°15′15″ on 1 January 2100](/en/chart?date=2100-01-01&time=12%3A00&calendar=gregorian&utc=stated&offset=%2B00%3A00&latitude=51.48&longitude=0&altitude=&ayanamsa=lahiri&node=mean&sunrise=upper-limb-apparent&reference=lagna&style=south): close to 1°24′ a century, the 50″ a year of precession.

## In the chart

The chart page prints the ayanamsa it used, to the second, under its table of grahas; its name there links to this page.

Next comes the [dasa](/en/kb/dasa): how Jyotish divides a life into periods, each ruled by a graha or a rasi.
