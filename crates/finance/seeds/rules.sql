-- A starting taxonomy and rule set, derived from the real Erste data.
--
-- Run per party:  psql -v party="'<uuid>'" -f crates/finance/seeds/rules.sql
--
-- Idempotent on both tables: categories key on (party_id, slug), rules on
-- (party_id, name). Re-running corrects a rule in place rather than adding a
-- second copy, so this file stays the source of truth for the starting set
-- while the UI edits build on top of it.
--
-- Three things the data forced, none of which were in the first version:
--
-- 1. **The largest single bucket has no counterparty at all.** Cash withdrawals
--    arrive with `counterparty_name` null and `ERSTE ATM ...` in the remittance
--    text. A rule matching on the name scores zero against EUR 38k. Anything
--    the bank writes as free text needs a `match_remittance_like` rule.
--
-- 2. **`GOOGLE` is two completely different things.** `GOOGLE WORKSPACE_INORBIT`
--    is a business subscription; `GOOGLE PLAY APPS` is 313 personal charges
--    worth EUR 8.2k. One rule matching `GOOGLE` filed all of it as business
--    hosting. Vendor prefixes are not categories.
--
-- 3. **Cash is an expense, not a transfer.** Calling it a transfer implies the
--    money reappears somewhere we track, and it does not. It is the point where
--    the trail ends, and the honest label says so.

\set ON_ERROR_STOP on

-- The taxonomy. `kind` is what it does to the books; `name` is what a person
-- reads. Anything that is not clearly one of these stays uncategorised on
-- purpose -- a wrong category is worse than a visible gap.
insert into finance.categories (id, party_id, slug, name, kind, deductible)
select gen_random_uuid(), :party, v.slug, v.name, v.kind, v.deductible
  from (values
    ('groceries',    'Groceries',              'expense',  false),
    ('dining',       'Dining & takeaway',      'expense',  false),
    ('fuel',         'Fuel',                   'expense',  true),
    ('vehicle',      'Vehicle & transport',    'expense',  true),
    ('leasing',      'Vehicle leasing',        'expense',  true),
    ('software',     'Software & hosting',     'expense',  true),
    ('digital',      'Apps, games & media',    'expense',  false),
    ('entertainment','Entertainment',          'expense',  false),
    ('shopping',     'Shopping & goods',       'expense',  false),
    ('pets',         'Pets',                   'expense',  false),
    ('health',       'Health',                 'expense',  false),
    ('fitness',      'Fitness & sport',        'expense',  false),
    ('telco',        'Telecoms',               'expense',  true),
    ('utilities',    'Utilities & home',       'expense',  true),
    ('travel',       'Travel & accommodation', 'expense',  true),
    ('accounting',   'Accounting',             'expense',  true),
    ('legal',        'Legal',                  'expense',  true),
    ('banking',      'Bank fees',              'expense',  true),
    ('education',    'Education & books',      'expense',  true),
    ('taxes',        'Taxes & contributions',  'tax',      false),
    ('cash',         'Cash withdrawals',       'expense',  false),
    ('savings',      'Savings transfers',      'transfer', false),
    ('family',       'Family & friends',       'transfer', false),
    ('client',       'Client income',          'income',   false),
    ('crypto',       'Crypto & investments',   'capital',  false),
    ('wallet',       'Wallets & top-ups',      'transfer', false),
    ('owner_draw',   'Owner draw',             'capital',  false),
    ('loan',         'Loans & credit',         'capital',  false),
    ('kiosk',        'Kiosks & newsstands',    'expense',  false),
    ('equipment',    'Equipment & hardware',   'expense',  true),
    ('insurance',    'Insurance',              'expense',  true)
  ) as v(slug, name, kind, deductible)
on conflict (party_id, slug) do update
  set name = excluded.name, kind = excluded.kind,
      deductible = excluded.deductible, archived_at = null;

-- `retail` was the first version's catch-all. Its rules are repointed below by
-- name; archiving it keeps the row so any `declared` reference still resolves.
update finance.categories set archived_at = now()
 where party_id = :party and slug = 'retail' and archived_at is null;

-- The rules. Priority ascending wins; a lower number is a more specific claim.
--
-- Ordering is the whole design here. Government and bank move first because
-- their names are unambiguous. Generic retail patterns move last, because
-- `STUDENAC` is only groceries when nothing better has already claimed the row.
insert into finance.rules
    (id, party_id, priority, name, category_id,
     match_counterparty_like, match_remittance_like, match_credit_debit)
select gen_random_uuid(), :party, v.priority, v.name, c.id,
       v.cp, v.rem, v.dc
  from (values
    -- 10: the state. Unambiguous, and must beat anything else naming a tax.
    (10, 'proracun',        'taxes',   'DRZAVNI PRORACUN',        null, null),
    (10, 'porez',           'taxes',   'POREZ',                   null, null),
    (10, 'hzzo',            'taxes',   'HRVATSKI ZAVOD',          null, null),
    (10, 'mirovinsko',      'taxes',   'MIROVINSK',               null, null),
    (10, 'doprinos',        'taxes',   'DOPR.ZA',                 null, null),
    (10, 'opcina',          'taxes',   'OPCINA',                  null, null),

    -- 15: people. Named individuals, before any pattern that could swallow them.
    (15, 'family_sara',     'family',  'SARA VESIC',              null, null),
    (15, 'family_sanjin',   'family',  'SANJIN VESIC',            null, null),
    (15, 'family_jovana',   'family',  'JOVANA SOSKIC',           null, null),
    (15, 'keks_pay',        'family',  null,                      'KEKS PAY', null),

    -- 20: movements between own accounts the IBAN cannot see. `internal` in
    -- `transactions_enriched` only catches a counterparty we hold; an a'vista
    -- sweep names no account at all.
    (20, 'avista',          'savings', null,                      'NAPLATA S A''VISTA', null),

    -- 25: cash. Matched on remittance, which is the only place the bank puts it.
    (25, 'atm_remittance',  'cash',    null,                      'ERSTE ATM', null),
    (25, 'atm',             'cash',    'ERSTE ATM',               null, null),
    -- Cash going the other way. Same category: the point is that the balance
    -- moved between the account and a pocket, in whichever direction.
    (25, 'atm_deposit',     'cash',    null,                      'ATM UPLATA', null),
    -- A withdrawal abroad names the other bank, not the ATM.
    (25, 'atm_wien',        'cash',    null,                      'ERSTE BANK WIEN', null),

    -- 22: own money moving, where no IBAN says so. `Kasica` is Erste's savings
    -- pot and `Revolut**NNNN*` is a top-up of a wallet in the same name --
    -- neither is spending, and counting them as such overstates the month.
    (22, 'kasica',          'savings', null,                      'KASICE', null),
    (22, 'revolut',         'wallet',  'REVOLUT*',                null, null),
    (22, 'kredit',          'loan',    null,                      'ZATVARANJE KREDITA', null),
    -- `ISPLATA ZAJMA`: a loan paid out, on whichever side it lands.
    (22, 'zajam',           'loan',    null,                      'ZAJMA', null),

    -- 18: taking money out of the company. A real capital movement on the
    -- business side, not a trading expense -- the accountant needs it apart.
    (18, 'dohodak_kapital', 'owner_draw', null,                   'DOHODAK OD KAPITALA', null),

    -- 30: the bank charging for being the bank.
    (30, 'bank_fee',        'banking', null,                      'NAKNAD', null),
    (30, 'erste',           'banking', 'ERSTE&STEIERMARKISCHE',   null, null),
    (30, 'eracuni',         'banking', 'ELEKTRONICKI RACUNI',     null, null),

    -- 40: the car. Leasing is its own line because it is a financing cost.
    (40, 'leasing',         'leasing', 'LEASING',                 null, null),
    (40, 'toyota',          'leasing', 'TOYOTA TSUSHO',           null, null),
    (40, 'ruting',          'vehicle', 'RUTING',                  null, null),
    (40, 'bozic_auto',      'vehicle', 'BOZIC AUTO',              null, null),
    (40, 'moto',            'vehicle', 'MOTO ',                   null, null),
    (40, 'autoskola',       'vehicle', 'AUTOSKOLA',               null, null),
    (40, 'toll',            'vehicle', 'DIONICA ',                null, null),
    (40, 'autocesta',       'vehicle', 'AUTOCESTA',               null, null),
    (40, 'tokic',           'vehicle', 'TOKIC',                   null, null),
    -- NYX is the processor behind fuel-station car washes and vending, so the
    -- merchant name is a site code. The prefix is the only stable part.
    (40, 'nyx',             'vehicle', 'NYX*',                    null, null),
    -- The whole word: `DUBRAVICA-AUTOBUSNI KO` is the bakery at the bus
    -- station, and the short form filed every pastry as transport.
    (40, 'kolodvor',        'vehicle', 'AUTOBUSNI KOLODVOR',      null, null),
    (40, 'parking',         'vehicle', 'PARKIR',                  null, null),
    (40, 'parking_en',      'vehicle', 'PARKING',                 null, null),
    (40, 'garaza',          'vehicle', 'GARAZ',                   null, null),
    (40, 'garage',          'vehicle', 'GARAGE',                  null, null),
    -- Rijeka plus runs the city's parking.
    (40, 'rijeka_plus',     'vehicle', 'RIJEKA PLUS',             null, null),
    (40, 'putevi',          'vehicle', 'PUTEVI',                  null, null),
    (40, 'bina_istra',      'vehicle', 'BINA-ISTRA',              null, null),

    -- 50: what the business actually runs on.
    (50, 'anthropic',       'software','ANTHROPIC',               null, null),
    (50, 'claude',          'software','CLAUDE',                  null, null),
    (50, 'openai',          'software','OPENAI',                  null, null),
    (50, 'github',          'software','GITHUB',                  null, null),
    (50, 'jetbrains',       'software','JETBRAINS',               null, null),
    (50, 'hetzner',         'software','HETZNER',                 null, null),
    (50, 'cloudflare',      'software','CLOUDFLARE',              null, null),
    (50, 'google_workspace','software','GOOGLE WORKSPACE',        null, null),
    (50, 'google_cloud',    'software','GOOGLE CLOUD',            null, null),
    (50, 'medium',          'software','MEDIUM',                  null, null),
    (50, 'itranslator',     'software','ITRANSLATOR',             null, null),
    (50, 'excalidraw',      'software','EXCALIDRAW',              null, null),
    (50, 'slack',           'software','SLACK',                   null, null),
    (50, 'namecheap',       'software','NAME-CHEAP',              null, null),
    (50, 'namecheap_alt',   'software','NAMECHEAP',               null, null),

    -- 55: consumer digital. Kept apart from hosting on purpose -- see the note
    -- at the top; this is the split that was silently wrong before.
    (55, 'google_play',     'digital', 'GOOGLE PLAY',             null, null),
    (55, 'google_play_alt', 'digital', 'GOOGLE PLAY AP',          null, null),
    (55, 'apple',           'digital', 'APPLE.COM',               null, null),
    (55, 'google_one',      'digital', 'GOOGLE ONE',              null, null),
    (55, 'playbook',        'digital', 'PLAYBOOK TECHNOLOGIES',   null, null),
    (55, 'brighthinki',     'digital', 'BRIGHTHINKI',             null, null),
    (55, 'vibeshort',       'digital', 'VIBESHORT',               null, null),
    (55, 'crownbill',       'digital', 'CROWNBILL',               null, null),
    (55, 'audible',         'digital', 'AUDIBLE',                 null, null),
    (55, 'cudesna',         'entertainment','CUDESNA ZEMLJA',     null, null),
    (55, 'playstation',     'digital', 'PLAYSTATION',             null, null),
    (55, 'netflix',         'entertainment','NETFLIX',            null, null),
    (55, 'hrt',             'entertainment','HRT',                null, null),
    (55, 'patreon',         'entertainment','PATREON',            null, null),
    (55, 'spotify',         'entertainment','SPOTIFY',            null, null),
    (55, 'youtube',         'entertainment','YOUTUBE',            null, null),
    (55, 'events_pula',     'entertainment','EVENTS AND PRODUCTION', null, null),

    -- 60: bills.
    (60, 'a1',              'telco',   'A1 HRVATSKA',             null, null),
    (60, 'ht',              'telco',   'HT D.D.',                 null, null),
    (60, 'tcom',            'telco',   'T-COM',                   null, null),
    (60, 'tmobile',         'telco',   'T-MOBILE',                null, null),
    (60, 'hep',             'utilities','HEP ',                   null, null),
    (60, 'vodovod',         'utilities','VODOVOD',                null, null),
    (60, 'komunal',         'utilities','KOMUNAL',                null, null),
    (60, 'osiguranje',      'insurance','OSIGURANJE',             null, null),
    (60, 'saily',           'telco',   'SAILY',                   null, null),

    -- 65: the professionals.
    (65, 'racunovodstvo',   'accounting','RACUNOVODSTVO',         null, null),
    (65, 'odvjetnicki',     'legal',   'ODVJETNICK',              null, null),

    -- 70: body and household.
    (70, 'pet_centar',      'pets',    'PET CENTAR',              null, null),
    (70, 'pet_shop',        'pets',    'PET SHOP',                null, null),
    (70, 'veterinar',       'pets',    'VETERINARSK',             null, null),
    (70, 'poliklinika',     'health',  'POLIKLINIK',              null, null),
    (70, 'ljekarna',        'health',  'LJEKARN',                 null, null),
    (70, 'ghetaldus',       'health',  'GHETALDUS',               null, null),
    (70, 'gym',             'fitness', 'GYM',                     null, null),
    (70, 'polleo',          'fitness', 'POLLEO',                  null, null),
    (70, 'capoeira',        'fitness', 'CAPOEIRA',                null, null),
    (70, 'coursera',        'education','COURSERA',               null, null),
    (70, 'naklad',          'education','NAKLADNISTVO',           null, null),

    -- 75: away from home.
    (75, 'booking',         'travel',  'BOOKING.COM',             null, null),
    (75, 'airbnb',          'travel',  'AIRBNB',                  null, null),
    (75, 'hotel',           'travel',  'HOTEL',                   null, null),
    (75, 'aerodrom',        'travel',  'MZLZ',                    null, null),
    (75, 'uber',            'travel',  'UBER',                    null, null),
    (75, 'bolt',            'travel',  'BOLT.EU',                 null, null),
    -- CarGo, the Belgrade ride app, bills as its company.
    (75, 'cargo',           'travel',  'GO TECHNOLOGIES',         null, null),
    (75, 'ucka',            'travel',  'NP UCKA',                 null, null),

    -- 80: fuel. Distinct from `vehicle` because it is the variable cost.
    (80, 'ina',             'fuel',    'INA ',                    null, null),
    (80, 'petrol',          'fuel',    'PETROL',                  null, null),
    (80, 'adria_oil',       'fuel',    'ADRIA OIL',               null, null),
    (80, 'petrol_bp',       'fuel',    'PBZTPETROL',              null, null),
    (80, 'crodux',          'fuel',    'CRODUX',                  null, null),
    (80, 'tifon',           'fuel',    'TIFON',                   null, null),
    (80, 'shell',           'fuel',    'SHELL',                   null, null),

    -- 85: food in.
    (85, 'studenac',        'groceries','STUDENAC',               null, null),
    (85, 'plodine',         'groceries','PLODINE',                null, null),
    (85, 'konzum',          'groceries','KONZUM',                 null, null),
    (85, 'lidl',            'groceries','LIDL',                   null, null),
    (85, 'kaufland',        'groceries','KAUFLAND',               null, null),
    -- No trailing space: the real charges arrive as `SPAR8714`, a store code.
    (85, 'spar',            'groceries','SPAR',                   null, null),
    (85, 'vocarnica',       'groceries','VOCARNICA',              null, null),
    (85, 'tommy',           'groceries','TOMMY',                  null, null),
    (85, 'mlinar',          'groceries','MLINAR',                 null, null),
    (85, 'mesnica',         'groceries','MESNICA',                null, null),
    -- Kiosks sell papers, tobacco and a coffee; none of it is groceries.
    (85, 'tisak',           'kiosk',   'TISAK',                   null, null),
    (85, 'inovine',         'kiosk',   'INOVINE',                 null, null),
    (85, 'kiosk',           'kiosk',   'KIOSK',                   null, null),

    -- 90: food out.
    (90, 'wolt',            'dining',  'WOLT',                    null, null),
    (90, 'glovo',           'dining',  'GLOVO',                   null, null),
    (90, 'restoran',        'dining',  'RESTORAN',                null, null),
    (90, 'konoba',          'dining',  'KONOBA',                  null, null),
    (90, 'bistro',          'dining',  'BISTRO',                  null, null),
    (90, 'gostionica',      'dining',  'GOSTIONICA',              null, null),
    (90, 'pizzeria',        'dining',  'PIZZER',                  null, null),
    (90, 'caffe',           'dining',  'CAFFE',                   null, null),
    (90, 'restaurant',      'dining',  'RESTAURANT',              null, null),
    (90, 'kavana',          'dining',  'KAVANA',                  null, null),
    (90, 'mcdonalds',       'dining',  'MCDONALD',                null, null),
    (90, 'marche',          'dining',  'MARCHE',                  null, null),
    (90, 'degustacija',     'dining',  'SALA ZA DEGUS',           null, null),
    (90, 'bar',             'dining',  ' BAR',                    null, null),
    (90, 'slasticarn',      'dining',  'SLASTICARN',              null, null),
    -- A bakery is eaten on the way, not cooked at home: the owner filed
    -- Dubravica as dining by hand, and the rule follows the person.
    (88, 'pekar',           'dining',  'PEKAR',                   null, null),
    (88, 'dubravica',       'dining',  'DUBRAVICA',               null, null),
    -- A refund from the same till names nobody; the text still says who.
    (88, 'dubravica_rem',   'dining',  null,                      'DUBRAVICA', null),
    (90, 'pub',             'dining',  ' PUB',                    null, null),
    (90, 'pivnica',         'dining',  'PIVNICA',                 null, null),
    (90, 'kod_brace',       'dining',  'KOD BRACE',               null, null),
    (90, 'doner',           'dining',  'DONER',                   null, null),
    (90, 'kebab',           'dining',  'KEBAB',                   null, null),
    (90, 'burek',           'dining',  'BUREK',                   null, null),
    (90, 'fast_food',       'dining',  'FAST FOOD',               null, null),
    (90, 'snack',           'dining',  'SNACK',                   null, null),
    (90, 'grill',           'dining',  'GRILL',                   null, null),
    (90, 'buffet',          'dining',  'BUFFET',                  null, null),
    (90, 'gastro',          'dining',  'GASTRO',                  null, null),
    (90, 'klub_mladih',     'dining',  'KLUB MLADIH',             null, null),

    -- 95: things.
    (95, 'lesnina',         'shopping','LESNINA',                 null, null),
    (95, 'pevex',           'shopping','PEVEX',                   null, null),
    (95, 'elipso',          'shopping','ELIPSO',                  null, null),
    (95, 'istyle',          'equipment','ISTYLE',                 null, null),
    (95, 'harvey_norman',   'equipment','HARVEY NORMAN',          null, null),
    (95, 'tern_setups',     'equipment','TERN SETUPS',            null, null),
    (95, 'veertee',         'equipment','VEERTEE',                null, null),
    (95, 'decathlon',       'shopping','DECATHLON',               null, null),
    (95, 'intersport',      'shopping','INTERSPORT',              null, null),
    (95, 'sport_vision',    'shopping','SPORT VISION',            null, null),
    (95, 'zara',            'shopping','ZARA',                    null, null),
    (95, 'furla',           'shopping','FURLA',                   null, null),
    (95, 'boss',            'shopping','BOSS',                    null, null),
    (95, 'tom_tailor',      'shopping','TOM TAILOR',              null, null),
    (95, 'mass',            'shopping','MASS ',                   null, null),
    (95, 'fashionfriends',  'shopping','FASHIONFRIENDS',          null, null),
    (95, 'thomann',         'shopping','THOMANN',                 null, null),
    (95, 'instar',          'equipment','INSTAR INFORMATIKA',     null, null),
    (95, 'desigual',        'shopping','DESIGUAL',                null, null),
    (95, 'peek',            'shopping','PEEK & CLOPPENBURG',      null, null),
    (95, 'tabacco',         'shopping','TABACCO',                 null, null),
    (95, 'sextasy',         'shopping','SEXTASY',                 null, null),

    -- 110: money coming in. `CRDT` only: a refund to a client is not income.
    (110,'tenderly',        'client',  'TENDERLY',                null, 'CRDT'),

    -- 120: not spending at all.
    (120,'binance',         'crypto',  'BINANCE',                 null, null)
  ) as v(priority, name, slug, cp, rem, dc)
  join finance.categories c on c.party_id = :party and c.slug = v.slug
on conflict (party_id, name) do update
  set priority = excluded.priority,
      category_id = excluded.category_id,
      match_counterparty_like = excluded.match_counterparty_like,
      match_remittance_like = excluded.match_remittance_like,
      match_credit_debit = excluded.match_credit_debit,
      enabled = true;

-- The first version's `GOOGLE` rule filed 458 personal Play Store charges as
-- business hosting. Disabled rather than deleted, so the correction is visible
-- to anyone who goes looking for why.
update finance.rules set enabled = false
 where party_id = :party and name = 'google';
