# Email to the accountant

Send once the items in "what we fetch ourselves" are in hand, so the accountant is asked
only for what nobody else holds. Croatian, in the register of the first mail. Fill the
bracketed parts.

## What we fetch ourselves first (not in the email)

| Item | Where | How |
|---|---|---|
| PD, PD-IPO, PDV (12), PDV-S, ZP, JOPPD (12), TZ for 2025 | ePorezna, "Pregled podataka > Poslani obrasci", download as XML | one session with the director's e-Građani credentials; the files go to the finance service's drop-zone connector (plan.md phase 1) |
| PKK (porezno-knjigovodstvena kartica) | ePorezna, PKK | export; reconciles 1430, 24311, 2420 to 2423, 1450 |
| Filed 2025 and 2024 statements, notes, both decisions | FINA RGFI javna objava (e-RGFI through e-Građani for the company's own) | download PDFs and the machine-readable BIL/RDG |
| Bank movements 2025 | finance service (`finance.bank_transactions`) | already there |
| Issued invoices 2025 | finance service (`finance.invoices`) and the PDFs in `~/finance-data/Firma` | already there; residence split from client records |
| Distribution decision, cash receipts | own records | scan into the documents store |

## Email

```
Predmet: Dokumentacija za 2025. – ono što samo vi imate

Bok Bruno,

nastavno na prethodnu poruku: velik dio dokumenata (predane PD, PDV, JOPPD, PD-IPO obrasce,
PKK i GFI s bilješkama) preuzeli smo sami iz ePorezne i FINA-e, pa vas molimo samo za ono
što postoji jedino u vašem programu, plus nekoliko pojašnjenja.

Dokumenti, kao izvoz iz programa (Excel ili CSV):

1. Kartice svih konta (ili dnevnik knjiženja) za cijelu 2025. godinu: datum, dokument,
   konto, duguje, potražuje, opis.
2. Kontni plan koji koristite, s nazivima konta.
3. Registar dugotrajne imovine: po svakom sredstvu nabavna vrijednost, datum nabave,
   datum početka amortizacije, stopa, ispravak vrijednosti na 1.1.2025. i na 31.12.2025.,
   te popis onoga što je na kontu 0371 (1.407,28 EUR) i još nije stavljeno u uporabu.
4. Zaključna bruto bilanca prethodnog računovođe na dan preuzimanja knjiga (ako je
   1.7.2025., molim potvrdu datuma), ili početna stanja po kontima na 1.1.2025.

Pojašnjenja:

5. Zajam članu društva (konto 11506, 97.387,68 EUR na 31.12.2025.): korištena kamatna
   stopa, način obračuna dana, te postoji li ugovor s planom otplate. Vidimo kamatu od
   1.721,40 EUR, što odgovara stopi od 2 %.
6. Članarina TZ: u 2025. plaćeno je 192,31 EUR, a na kontu 1450 je 257,62 EUR. Prema
   NN 52/19 djelatnost 62 nije obveznik od 1.1.2020. Je li to preplata i hoće li se
   tražiti povrat?
7. AOP 278 (naknade članovima uprave) u Dodatnim podacima iznosi 2.878,75 EUR, što je
   konto 4686 "prava uporabe računalnih programa". Po RRiF planu 4686 su naknade
   vanjskim članovima uprave. Koje je ispravno?
8. Konta 14032 (138,69) i 24032 (171,45): na koju se uslugu iz EU odnosilo ograničenje
   odbitka pretporeza?
9. AOP 279: predano 2.597,45, a na kontu 4199 je 2.597,48.
10. Što je na kontima 2399 (542,84), 1252 (13,88) i 4850 (236,46), i postoji li
    blagajnički izvještaj za 1.950,00 podignutih u gotovini.
11. Usporedni podaci 2024.: iz kojih su konta AOP 133 (4.095,52), AOP 156 (3.932,29) i
    AOP 279 (4.140,00).

Za 2026.: bi li vam odgovaralo da mjesečno razmjenjujemo izvoz knjiženja (točka 1), tako
da sustav prati stanje tijekom godine, a ne tek u travnju?

Hvala unaprijed.

Lijep pozdrav,
[ime]
```

## After the reply

Items 1 to 4 go into the golden directory (`chaos.md`); items 5 to 11 close the open
questions in `findings.md`, which is updated with the answers and the date.
