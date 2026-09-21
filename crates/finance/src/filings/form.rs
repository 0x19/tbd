//! What each form's body holds: which containers are rows, which keys are
//! the headline, and how the tax return's rows map to the figures in
//! `docs/accountant/expected/pd.csv`. Data, not code, so a schema revision
//! is an edit here.

use super::Form;

/// Containers (local-name paths below the body) whose children are rows
/// even when there is only one of them.
#[must_use]
pub const fn row_paths(form: Form) -> &'static [&'static str] {
    match form {
        Form::Pd => &["Primatelji", "Racuni"],
        Form::Pdv => &["Preknjizenja/Stavke", "PodaciZaUstup"],
        Form::PdvS | Form::Zp => &["Isporuke", "AranzmaniPremjestanjaDobara"],
        Form::Joppd => &["Primatelji"],
        Form::PdIpo => &[
            "Podaci/Podaci1/Osobe",
            "Podaci/Podaci2/Osobe",
            "Podaci/Podaci3/Osobe",
            "Podaci/Podaci4/Osobe",
        ],
        Form::Tz => &[],
    }
}

/// The few keys a listing shows for the form, in display order.
#[must_use]
pub const fn headline_keys(form: Form) -> &'static [&'static str] {
    match form {
        Form::Pd => &[
            "1", "2", "3", "26", "35", "38", "43", "44", "55", "56", "57", "59",
        ],
        Form::Pdv => &[
            "000",
            "104",
            "105",
            "200.Vrijednost",
            "200.Porez",
            "300.Vrijednost",
            "300.Porez",
            "400",
            "630",
            "640",
        ],
        Form::PdvS => &["IsporukeUkupno.I1", "IsporukeUkupno.I2"],
        Form::Zp => &[
            "IsporukeUkupno.I1",
            "IsporukeUkupno.I2",
            "IsporukeUkupno.I3",
            "IsporukeUkupno.I4",
        ],
        Form::Joppd => &[
            "A.BrojOsoba",
            "A.BrojRedaka",
            "A.PredujamPoreza.P1",
            "A.Doprinosi.GeneracijskaSolidarnost.P1",
            "A.Doprinosi.KapitaliziranaStednja.P1",
            "A.Doprinosi.ZdravstvenoOsiguranje.P1",
        ],
        Form::PdIpo => &[
            "Podaci.Podaci1.Sveukupno.S1",
            "Podaci.Podaci2.Sveukupno.S1",
            "Podaci.Podaci3.Sveukupno.S1",
            "Podaci.Podaci4.Sveukupno.S1",
        ],
        Form::Tz => &["01", "02", "03", "04", "05", "06", "07"],
    }
}

/// The key a row of the tax return's printed form has in `values`: the
/// numbered rows are their number, section X rows are `Podatak110` to
/// `Podatak150`.
#[must_use]
pub fn pd_csv_key(row: &str) -> Option<&str> {
    match row.trim() {
        "X.1" => Some("110"),
        "X.1.1" => Some("111"),
        "X.1.2" => Some("112"),
        "X.2" => Some("120"),
        "X.3" => Some("130"),
        "X.4" => Some("140"),
        "X.5" => Some("150"),
        r if !r.is_empty() && r.bytes().all(|b| b.is_ascii_digit()) => Some(r),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_form_has_a_headline() {
        for form in Form::ALL {
            assert!(!headline_keys(form).is_empty(), "{form}");
        }
        assert_eq!(pd_csv_key("57"), Some("57"));
        assert_eq!(pd_csv_key("X.1.2"), Some("112"));
        assert_eq!(pd_csv_key("row"), None);
    }
}
