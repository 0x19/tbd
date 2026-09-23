/**
 * The Croatian of what a person reads in `site.ts`. Keyed by what identifies an
 * entry there, never by position where a name exists, so the two files can be
 * edited apart: an entry without a twin here reads in English until it gets
 * one (`src/lib/i18n/site.ts` does the merge). Names, numbers, years, stacks
 * and links are not repeated here; they are the same in both languages.
 */
export const hr = {
  company: {
    tagline: "Backend i blockchain sustavi.",
    headline: "Sustavi koji ostaju gore kad je važno, i eksperimenti koji ne moraju.",
    summary:
      "Dvadeset godina u neglamuroznoj polovici sustava: najprije glasovni poslužitelji i SMS pristupnici, zatim anycast mreže koje nose promet u stvarnom vremenu pri šezdeset gigabita, pa rollupi, mostovi i indekseri, a najnovije RPC infrastruktura ispred stotinjak blockchain mreža. Deset od tih godina u Gou, a u zadnje vrijeme jednako toliko u Rustu. Što god gradim iz zabave, završi i ovdje.",
    availability:
      "Dostupan od listopada 2026. za protokole, infrastrukturu i distribuirane sustave u Rustu i Gou, na B2B ugovor preko vlastite tvrtke, InOrbit d.o.o.",
    now: "Samostalno, kroz InOrbit",
  },
  experience: {
    Tenderly: {
      role: "Softverski inženjer, mrežna infrastruktura",
      where: "Na daljinu",
      body: "Razvojna infrastruktura za Ethereum: simulacija, otklanjanje grešaka i sustavi iza toga, u produkcijskom obujmu. Backend rad u Gou i Rustu na sustavima koji to nose. Ugovor je završio u rujnu 2026.",
    },
    "(Un)Pack": {
      role: "Osnivač",
      where: "Na daljinu",
      body: "Vlastiti proizvod: platforma koja rastavlja Ethereum ugovore u velikom broju -- izvorni kod, AST i IR, bytecode, grafovi toka -- s uslugom otkrivanja preko GraphQL-a koja je jezičnim modelima opisivala što ugovor radi i sliči li na prijevaru. Ugašen kad je počeo rad za Tenderly, jer je pogon koštao više nego što je zarađivao. Biblioteke su ostale javne.",
      highlights: [
        "Osmišljen, izgrađen i vođen sam: crawler, pohrana, analiza, API i račun.",
        "solgo -- prvi Solidity AST i IR parser u Gou, s izgradnjom grafa toka; drugi ga otad koriste.",
        "Crawler koji je dosezao više od 30 tisuća zahtjeva u sekundi prema lancu i punio Postgres i ClickHouse s jednim do jedan i pol terabajta dnevno.",
        "Cijene tokena izravno iz rezervi bazena, bez Chainlinka i vanjskih API-ja.",
      ],
    },
    Eiger: {
      role: "Stariji softverski / protokolni inženjer",
      where: "Na daljinu",
      body: "Rad na protokolima na više lanaca, od istraživanja do produkcije: vlasnički optimistički rollup kompatibilan s EVM-om odveden od zamisli do produkcije, jedan od prvih WASM portova čvora druge razine u Gou i most likvidnosti između Ethereuma i Bitcoina izgrađen na višestranačkom računanju i pragovnom ECDSA. Vodio timove do pet ljudi, vodio istraživanje i pisao prijave za potpore koje su dio toga financirale.",
    },
    InOrbit: {
      role: "Vlasnik",
      where: "Hrvatska",
      body: "Tvrtka kroz koju ide B2B rad; Subspace, Eiger i Tenderly bili su angažirani tako.",
    },
    Subspace: {
      role: "Stariji softverski inženjer",
      where: "Na daljinu · Los Angeles",
      body: "Mreža izgrađena za promet koji ne može čekati. Pisao sam korisničke servise koji sjede između kernela i upravljačke ravnine, i dio kernelske strane -- IP filtriranje, prepisivanje paketa, predmemorija mrežne kartice, upravljanje mapama. Suizgradio prvu verziju anycast TURN mreže, žive na više od 150 točaka prisutnosti, i prvu SIP anycast mrežu na Kamailiju i FreeSWITCH-u; te Elixir upravljačku ravninu koja je dizala tunele i naplaćivala korištenje iz geografski svjesne baze.",
      highlights: [
        "Jedan od prva tri inženjera. Mreža od prvog do sedmog sloja izgrađena u šest mjeseci za regiju MENA, s više od 60 Gbps od početka; osigurala je ugovor vrijedan više od tri milijuna dolara godišnje i sljedeći krug financiranja.",
        "Suosmislio i suizgradio prvu globalnu anycast TURN mrežu, više od 150 točaka prisutnosti, i SIP anycast mrežu na Kamailiju.",
        "Korisnički servisi između kernela i upravljačke ravnine, i sama kernelska strana u eBPF-u: IP filtriranje, prepisivanje paketa, predmemorija kartice, upravljanje mapama. Prijava patenta na eBPF dizajnu.",
        "Elixir upravljačka ravnina koja je zahtjeve korisničkog API-ja pretvarala u IPv4 tunele, s naplatom korištenja iz geografski svjesne distribuirane baze.",
      ],
    },
    Avaya: {
      role: "Stariji inženjer, zatim softverski inženjer na CPaaS-u",
      where: "Hrvatska",
      body: "Platforma TelAPI nakon preuzimanja, kao Zang Cloud pa Avaya CPaaS. Go mikroservisi na platformi, zatim arhitektura sljedeće generacije njezina sučelja i vođenje tima koji ga je gradio, uključujući stranu sigurnosti i usklađenosti -- skeniranje koda, HIPAA, GDPR, SOC.",
      highlights: [
        "Vodio tim za servise sučelja: planiranje, odblokiranje, isporuka.",
        "Arhitektura sljedeće generacije CPaaS sučelja (React, Node.js, Go, Kubernetes, GCP, AWS).",
        "Odgovoran za sigurnosnu stranu: skeniranje koda i rad na usklađenosti s HIPAA-om, GDPR-om i SOC-om.",
      ],
    },
    "TelTech Systems · TelAPI": {
      role: "Stariji softverski inženjer",
      where: "Na daljinu · New York",
      body: "Telekom na razini protokola: glasovni poslužitelji na FreeSWITCH-u i Kamailiju, SMS stog preko SMPP-a s SMSC i SMSE stranom, usluge brojeva i operatera, i potpuno prepisivanje tih servisa iz Pythona u Go. Potrošački proizvodi na istim temeljima, među njima spoofcard.com i tapeacall.com.",
      highlights: [
        "Glasovni poslužitelji na FreeSWITCH-u i Kamailiju; SMS stog preko SMPP-a s SMSC i SMSE stranom; usluge brojeva i operatera.",
        "Potpuno prepisivanje servisa iz Pythona u Go.",
      ],
    },
    "TelAPI Adriatica": {
      role: "Direktor",
      where: "Hrvatska",
      body: "Hrvatska podružnica, i dva inženjera u njoj. Zatvorena kad je Avaya preuzela matičnu tvrtku.",
    },
    Earlier: {
      role: "Web razvoj i administracija poslužitelja",
      where: "Rijeka · New Jersey",
      body: "TelTech Systems, CLKCLK, Adria24, Web Factory, In-tech, Design Strategist i Skin29 -- gdje dvadeset godina počinje, i gdje sam naučio da netko mora i pokretati poslužitelj.",
    },
  } as Record<string, { role?: string; where?: string; body?: string; highlights?: string[] }>,
  earlier: {
    "TelTech Systems": {
      role: "Web programer -- CPaaS sučelje u Zendu, njegova dokumentacija i API preglednik",
    },
    ClkClk: { role: "Web programer -- SaaS tvrtke, zatim njezina interna administracija od nule" },
    Adria24: { role: "Web programer -- interni sustav rezervacija turističke agencije, sučelje i pozadina" },
    "In-tech, WebFactory": { role: "Vodeći web programer -- platforma za e-trgovinu knjigama; WordPress" },
    "Skin29, Design Strategist": {
      role: "Web programer -- CMS koji je poslije koristilo više velikih hrvatskih tvrtki",
    },
  } as Record<string, { role?: string }>,
  achievements: [
    "Jedan od prva tri inženjera u Subspaceu: anycast mreža od prvog do sedmog sloja izgrađena u šest mjeseci, više od 60 Gbps od prvog dana, više od 150 točaka prisutnosti; donijela je ugovor vrijedan više od tri milijuna dolara godišnje i sljedeći krug financiranja. Prijava patenta na eBPF radu.",
    "Optimistički EVM rollup u Gou, od zamisli do produkcije; jedan od prvih WASM portova čvora druge razine u Gou.",
    "Most likvidnosti između Ethereuma i Bitcoina na višestranačkom računanju i pragovnom ECDSA.",
    "EVM indekser koji struji cijeli lanac za manje od deset sati; crawler pri 30 tisuća zahtjeva u sekundi koji puni jedan do jedan i pol terabajta dnevno.",
    "solgo, prvi Solidity AST/IR parser u Gou s grafovima toka -- otvorenog koda, drugi ga otad koriste.",
  ],
  about: [
    "Dvadeset godina gradnje softvera, većinom infrastrukture: distribuirani sustavi, pohrana, protokoli i telekomunikacije. Deset od tih godina u Gou, a u zadnje vrijeme jednako toliko u Rustu. Usput sam vodio timove do pet ljudi i radio onaj dio tog posla koji je razgovor s upravom i klijentima, a ne s prevoditeljem koda.",
    "Luk ide telekom, pa mreže u stvarnom vremenu, pa blockchain protokoli. Najprije glasovni poslužitelji, SMS i usluge operatera; zatim anycast TURN i SIP mreže i kernelski rad s paketima ispod njih; zatim optimistički EVM rollup, most između lanaca i indekseri koji drže korak s lancem; pa dvije godine razvojne infrastrukture za Ethereum u Tenderlyju, RPC sloj ispred stotinjak mreža.",
    "Dalje od ekrana: gitara, više filozofije i psihologije nego što je strogo korisno, psi i moja djevojka.",
  ],
  projects: {
    solgo: {
      what: "Solidity parser u Gou koji izvorni kod ugovora pretvara u strukturirani oblik koji se može analizirati -- temelj za detektore, rad s ABI-jem i otkrivanje standarda.",
    },
    "sourcify-go": {
      what: "Go klijent za Sourcify API: provjeri ugovor, dohvati njegove metapodatke i izvore, vidi što lanac već zna o adresi.",
    },
    fdb: {
      what: "Transportni sloj visokih performansi ispred ugrađenih key-value baza poput MDBX-a, za čitanja na koja čvor ili indekser ne mogu čekati.",
    },
    "solc-switch": {
      what: "Upravlja svim verzijama Solidity prevoditelja odjednom i prevodi pravom, paralelno, umjesto ručnog žongliranja alatima.",
    },
    "go-clickhouse-orm": {
      what: "Podrška za modele i migracije za ClickHouse u Gou, da analitička shema ima verzije kao i svaki drugi dio servisa.",
    },
    gotostruct: {
      what: "Pretvara JSON objekt u Go strukturu, kao biblioteka i kao javni alat koji je radio na jsonstruct.com. Napisan jer mi je dojadilo raditi to ručno.",
    },
    disposable: {
      what: "JSON i gRPC API koji odgovara na jedno pitanje -- je li ovo jednokratna adresa e-pošte? Malen, javan, i otad ga tiho koriste stranci.",
    },
    goesl: {
      what: "FreeSWITCH Event Socket biblioteka za Go. Napisana 2015. i još je drugi granaju i isporučuju -- telefonija je bila prvi sustav koji sam morao držati na nogama.",
    },
  } as Record<string, { what?: string }>,
  playgrounds: {
    "/playgrounds/break-it/": {
      name: "Sruši ga",
      what: "Četiri prava servisa pod živim prometom s ciljem koji treba održati, i zajednički proračun kvarova koji trošiš pokušavajući ga srušiti. Svi bockaju isti pješčanik; sam se liječi. Gledaj ga preko WebSocketa, server-sent eventa ili običnog pollinga -- isti poziv, tri načina.",
      tag: "Kaos",
    },
    "/playgrounds/tuner/": {
      name: "Štimer za gitaru",
      what: "Trzni žicu i mjerač je imenuje i pokaže koliko si daleko, u centima. Pet štimova, referentni ton po žici, podesiv A4. Sluša kroz mikrofon i ništa ne napušta stranicu.",
      tag: "Zvuk",
    },
    "/playgrounds/fretboard/": {
      name: "Vježbanje vrata",
      what: "Imenuje notu i žicu, ti je odsviraš, a isto uho kao u štimeru kaže jesi li pogodio. Ili označi mjesto na vratu, a ti ga imenuješ. Nizovi pogodaka i popis nota koje stalno promašuješ.",
      tag: "Zvuk",
    },
    "/playgrounds/spectrogram/": {
      name: "Vidi svoj glas",
      what: "Spektrogram uživo svega što mikrofon čuje: vrijeme slijeva nadesno, visina uz rub, svjetlina za glasnoću, osnovni ton pronađen i imenovan u hodu. Mumljaj, zviždi, šišti.",
      tag: "Zvuk",
    },
    "/playgrounds/chords/": {
      name: "Imenovanje akorda",
      what: "Dodirni pragove koje držiš i imenuje akord, s drugim imenima koja bi mogao nositi. Upiši akord i rasporedi oblike uz vrat, svaki odsviran na dodir. Mikrofon nije potreban.",
      tag: "Teorija",
    },
    "/playgrounds/metronome/": {
      name: "Metronom",
      what: "Klikovi na zvučnom satu pa nikad ne odlutaju, tap tempo, dobe u taktu i podjele, i dnevnik vježbanja koji živi samo u tvom pregledniku.",
      tag: "Zvuk",
    },
    "/playgrounds/ear/": {
      name: "Vježbanje sluha",
      what: "Odsvira dvije note ili akord na sintetiziranim žicama, a ti imenuješ interval ili akord. Rezultat se skuplja po odgovoru, pa oni koji te varaju isplivaju.",
      tag: "Teorija",
    },
  } as Record<string, { name?: string; what?: string; tag?: string }>,
  principles: [
    {
      title: "Prvo specifikacija, onda žica",
      body: "RFC kaže što bi se trebalo dogoditi; snimka kaže što se događa. Kad se ne slažu, žica pobjeđuje, a zanimljive greške žive u toj pukotini.",
    },
    {
      title: "Promatrivo po zadanom",
      body: "Svaki zahtjev nosi trace id u zapise, tragove, metrike i kontinuirane profile. Kvar se čita, a ne nagađa.",
    },
    {
      title: "Testirano na kvarove",
      body: "Isti alati koji rade u razvoju ubacuju kašnjenja, greške, izgubljene pakete i kvarove pohrane u CI-ju, pa prvi ispad nije i prvi test.",
    },
    {
      title: "Na otvorenom",
      body: "Otvoreni kod gdje god može. Dobri dijelovi vrijede više kad ih drugi čitaju nego kad stoje u ladici.",
    },
  ] as { title?: string; body?: string }[],
  pipeline: {
    Wire: {
      stage: "Žica",
      name: "paketi, kernel",
      rows: [
        { k: "rad" },
        { k: "mjeri se u", v: "mikrosekundama" },
        { k: "na rubu", v: "anycast, 150+ PoP-ova" },
      ],
    },
    Protocol: {
      stage: "Protokol",
      rows: [
        { k: "govoren", v: "od slova do slova" },
        { k: "provjeren", v: "prema snimci" },
        { k: "nosi", v: "glas, video i blokove" },
      ],
    },
    Service: {
      stage: "Servis",
      rows: [
        { k: "radi", v: "jednu stvar, ponovno pokretljiv" },
        { k: "dolazi s", v: "zdravlje · metrike · tragovi" },
        { k: "pisan u", v: "Go · Rust · Elixir · C" },
      ],
    },
    State: {
      stage: "Stanje",
      name: "birano prema obliku",
      rows: [{ k: "ugrađeno" }, { k: "relacijsko" }, { k: "u velikom" }],
    },
  } as Record<string, { stage?: string; name?: string; rows?: { k?: string; v?: string }[] }>,
  rails: [
    {
      label: "Svaki sloj",
      value: "trace id u zapise, tragove, metrike i kontinuirane profile -- i snimka kad laže",
    },
    {
      label: "Prije produkcije",
      value: "opterećenje, ubačeno kašnjenje, izgubljeni paketi, kvarovi pohrane, testovi svojstava",
    },
  ] as { label?: string; value?: string }[],
  languages: ["engleski", "hrvatski", "bosanski", "srpski", "slovenski"],
  education: "Srednja škola, 2000. — 2003. Sve otad naučeno samostalno, na poslu.",
};
