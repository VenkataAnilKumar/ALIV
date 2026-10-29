//! Rule-based field extraction for legal and finance documents.
//!
//! Every value records the byte offset it was read from, so the caller can
//! cite the exact passage. Rules prefer leaving a field empty to guessing: a
//! date only becomes an expiry date when the text around it says so.

use chrono::NaiveDate;
use regex::Regex;
use std::sync::OnceLock;

pub const DOC_TYPE: &str = "doc_type";
pub const PARTIES: &str = "parties";
pub const EFFECTIVE_DATE: &str = "effective_date";
pub const EXPIRY_DATE: &str = "expiry_date";
pub const RENEWAL_DEADLINE: &str = "renewal_deadline";
pub const AMOUNT: &str = "amount";
pub const GOVERNING_LAW: &str = "governing_law";
pub const NOTICE_PERIOD: &str = "notice_period";
pub const PENALTY: &str = "penalty";

#[derive(Debug, Clone, PartialEq)]
pub struct Extracted {
    pub key: &'static str,
    pub value: String,
    pub confidence: f64,
    /// Byte offset of the evidence in the source text.
    pub offset: usize,
    pub snippet: String,
}

pub fn extract_all(text: &str) -> Vec<Extracted> {
    let normalized = join_table_labels(text);
    let text = normalized.as_str();
    let mut out = Vec::new();
    out.push(classify(text));
    out.extend(parties(text));
    out.extend(dates(text));
    out.extend(amount(text));
    out.extend(governing_law(text));
    out.extend(notice_period(text));
    out.extend(penalty(text));
    out
}

/// Labels that word-processor tables put in their own cell.
const TABLE_LABELS: &[&str] = &[
    "vendor", "client", "landlord", "tenant", "lessor", "lessee", "supplier", "customer", "buyer", "seller",
    "bill to", "billed to", "disclosing party", "receiving party", "document type", "invoice no.", "invoice number",
    "issue date", "invoice date", "due date", "effective date", "start date", "end date", "expiry date",
    "expiration date", "term end", "renewal date", "renewal deadline", "amount", "amount due", "total", "total due",
    "contract value", "fee", "governing law", "notice period",
];

/// Turns a table cell pair rendered as "Vendor\nAcme LLC" into "Vendor:Acme LLC".
/// The newline is replaced by a single ':' so byte offsets stay identical.
fn join_table_labels(text: &str) -> String {
    let mut bytes = text.as_bytes().to_vec();
    let mut line_start = 0;
    for (i, ch) in text.char_indices() {
        if ch != '\n' {
            continue;
        }
        let line = text[line_start..i].trim().to_lowercase();
        let next_has_text = text[i + 1..].lines().next().is_some_and(|l| !l.trim().is_empty());
        if next_has_text && TABLE_LABELS.contains(&line.as_str()) {
            bytes[i] = b':';
        }
        line_start = i + 1;
    }
    String::from_utf8(bytes).unwrap_or_else(|_| text.to_string())
}

// ---------------------------------------------------------------- doc type

pub fn classify(text: &str) -> Extracted {
    if let Some(c) = re(r"(?im)^\s*document type\s*:\s*(.+?)\s*$").captures(text) {
        let m = c.get(1).unwrap();
        return found(DOC_TYPE, normalize_doc_type(m.as_str()), 0.95, text, m.start());
    }
    let head: String = text.chars().take(3000).collect::<String>().to_lowercase();
    if re(r"(?i)\A\s*(?:from|to|date|subject)\s*:").is_match(text) && head.contains("subject:") {
        return found(DOC_TYPE, "Email".into(), 0.85, text, 0);
    }
    const RULES: &[(&str, &str)] = &[
        ("NDA", r"non-?disclosure|confidentiality agreement|\bnda\b"),
        ("Settlement", r"\bsettlement\b|\brelease agreement\b"),
        ("Lease", r"\blease agreement\b|\bcommercial lease\b|\bresidential lease\b|\blandlord\b|\blease\b"),
        ("Statement of Work", r"statement of work|\bsow\b"),
        ("Purchase Order", r"purchase order"),
        ("Service Agreement", r"services? agreement|master services"),
        ("Employment Agreement", r"employment agreement|offer of employment"),
        ("Loan Agreement", r"loan agreement|promissory note|credit agreement"),
        ("Invoice", r"\binvoice\b"),
        ("Contract", r"\bagreement\b|\bcontract\b"),
    ];
    // The title line is the strongest signal; the body is a fallback.
    let title_end = text.trim_start().find('\n').map(|i| i + (text.len() - text.trim_start().len())).unwrap_or(text.len());
    let title = text[..title_end].to_lowercase();
    for (scope, base_conf) in [(title.as_str(), 0.9), (head.as_str(), 0.75)] {
        for (label, pattern) in RULES {
            if let Some(m) = re(pattern).find(scope) {
                let conf = if *label == "Contract" { 0.6 } else { base_conf };
                return found(DOC_TYPE, (*label).to_string(), conf, text, char_to_byte(text, scope[..m.start()].chars().count()));
            }
        }
    }
    found(DOC_TYPE, "Document".into(), 0.3, text, 0)
}

fn normalize_doc_type(raw: &str) -> String {
    let l = raw.to_lowercase();
    for (needle, label) in [
        ("nda", "NDA"),
        ("non-disclosure", "NDA"),
        ("settlement", "Settlement"),
        ("statement of work", "Statement of Work"),
        ("lease", "Lease"),
        ("invoice", "Invoice"),
        ("purchase order", "Purchase Order"),
        ("service", "Service Agreement"),
        ("employment", "Employment Agreement"),
        ("loan", "Loan Agreement"),
    ] {
        if l.contains(needle) {
            return label.to_string();
        }
    }
    raw.trim().to_string()
}

// ----------------------------------------------------------------- parties

fn parties(text: &str) -> Option<Extracted> {
    // Labelled header lines ("Vendor / Landlord: Acme LLC").
    let label = re(r"(?im)^\s*(?:vendor|client|landlord|tenant|lessor|lessee|disclosing party|receiving party|buyer|seller|supplier|customer|bill to|billed to)(?:\s*/\s*[a-z ]+)?\s*:\s*(.+?)\s*$");
    let mut names: Vec<String> = Vec::new();
    let mut first_offset = None;
    for c in label.captures_iter(text).take(4) {
        let m = c.get(1).unwrap();
        let name = clean_party(m.as_str());
        if !name.is_empty() && !names.contains(&name) {
            first_offset.get_or_insert(m.start());
            names.push(name);
        }
    }
    if names.len() >= 2 {
        return Some(found(PARTIES, names.join(" · "), 0.9, text, first_offset.unwrap()));
    }
    // "between X ("Landlord") and Y ("Tenant")"
    let between = re(r#"(?is)\bbetween\s+(.{3,80}?)\s*(?:\(|,)\s*.{0,40}?\band\s+(.{3,80}?)\s*(?:\(|,|\n|\.\s+[A-Z]|\.$)"#);
    if let Some(c) = between.captures(text) {
        let a = clean_party(c.get(1).unwrap().as_str());
        let b = clean_party(c.get(2).unwrap().as_str());
        if !a.is_empty() && !b.is_empty() {
            return Some(found(PARTIES, format!("{a} · {b}"), 0.8, text, c.get(0).unwrap().start()));
        }
    }
    // Email sender as the counterparty.
    if let Some(c) = re(r"(?im)^from\s*:\s*(.+?)\s*$").captures(text) {
        let m = c.get(1).unwrap();
        let name = clean_party(m.as_str());
        if !name.is_empty() {
            names.insert(0, name);
            return Some(found(PARTIES, names.join(" · "), 0.6, text, m.start()));
        }
    }
    if let (Some(name), Some(off)) = (names.first(), first_offset) {
        return Some(found(PARTIES, name.clone(), 0.7, text, off));
    }
    None
}

fn clean_party(raw: &str) -> String {
    let s = raw.split('<').next().unwrap_or(raw); // drop "<email>"
    // Keep a trailing period: it usually ends an abbreviation ("Inc.", "Ltd.").
    let s = s.trim().trim_matches(|c: char| c == '"' || c == ',' || c == ';' || c == ':');
    let s = s.trim_start_matches("the ").trim_start_matches("The ");
    if (s.contains('@') && !s.contains(' ')) || s.contains("__") || !s.starts_with(|c: char| c.is_alphanumeric()) {
        return String::new();
    }
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

// ------------------------------------------------------------------- dates

#[derive(Debug, Clone)]
pub struct DateMention {
    pub date: NaiveDate,
    pub start: usize,
    pub end: usize,
}

pub fn find_dates(text: &str) -> Vec<DateMention> {
    let months = "(January|February|March|April|May|June|July|August|September|October|November|December|Jan|Feb|Mar|Apr|Jun|Jul|Aug|Sep|Sept|Oct|Nov|Dec)";
    let mut out = Vec::new();
    for c in re(r"\b(\d{4})-(\d{2})-(\d{2})\b").captures_iter(text) {
        push_date(&mut out, &c, c[1].parse().ok(), c[2].parse().ok(), c[3].parse().ok());
    }
    let mdy = format!(r"\b{months}\.?\s+(\d{{1,2}})(?:st|nd|rd|th)?,?\s+(\d{{4}})\b");
    for c in re(&mdy).captures_iter(text) {
        push_date(&mut out, &c, c[3].parse().ok(), month_num(&c[1]), c[2].parse().ok());
    }
    let dmy = format!(r"\b(\d{{1,2}})(?:st|nd|rd|th)?\s+(?:of\s+)?{months},?\s+(\d{{4}})\b");
    for c in re(&dmy).captures_iter(text) {
        push_date(&mut out, &c, c[3].parse().ok(), month_num(&c[2]), c[1].parse().ok());
    }
    for c in re(r"\b(\d{1,2})/(\d{1,2})/(\d{4})\b").captures_iter(text) {
        // US convention (MM/DD/YYYY) unless the first part cannot be a month.
        let (a, b): (u32, u32) = (c[1].parse().unwrap_or(0), c[2].parse().unwrap_or(0));
        let (m, d) = if a > 12 { (b, a) } else { (a, b) };
        push_date(&mut out, &c, c[3].parse().ok(), Some(m), Some(d));
    }
    out.sort_by_key(|d| d.start);
    out
}

fn push_date(
    out: &mut Vec<DateMention>,
    c: &regex::Captures,
    y: Option<i32>,
    m: Option<u32>,
    d: Option<u32>,
) {
    let whole = c.get(0).unwrap();
    if let (Some(y), Some(m), Some(d)) = (y, m, d) {
        if let Some(date) = NaiveDate::from_ymd_opt(y, m, d) {
            out.push(DateMention { date, start: whole.start(), end: whole.end() });
        }
    }
}

fn month_num(name: &str) -> Option<u32> {
    let n = name.to_lowercase();
    let idx = [
        "jan", "feb", "mar", "apr", "may", "jun", "jul", "aug", "sep", "oct", "nov", "dec",
    ]
    .iter()
    .position(|m| n.starts_with(m))?;
    Some(idx as u32 + 1)
}

fn dates(text: &str) -> Vec<Extracted> {
    let mentions = find_dates(text);
    let mut best: Vec<(&'static str, f64, &DateMention)> = Vec::new();
    for m in &mentions {
        let (role, conf) = match date_role(text, m) {
            Some(r) => r,
            None => continue,
        };
        match best.iter_mut().find(|(k, _, _)| *k == role) {
            Some(slot) if conf > slot.1 => *slot = (role, conf, m),
            Some(_) => {}
            None => best.push((role, conf, m)),
        }
    }
    best.into_iter()
        .map(|(key, conf, m)| found(key, m.date.format("%Y-%m-%d").to_string(), conf, text, m.start))
        .collect()
}

/// Decides what a date means from the words just before it on the same line.
fn date_role(text: &str, m: &DateMention) -> Option<(&'static str, f64)> {
    let mut line_start = text[..m.start].rfind('\n').map(|i| i + 1).unwrap_or(0);
    // Prose wraps ("... no later than\n   November 15, 2026"): when the date's
    // line has no label of its own and the previous line runs on, read both.
    if line_start > 0 && !text[line_start..m.start].contains(':') {
        let prev_end = line_start - 1;
        let prev_start = text[..prev_end].rfind('\n').map(|i| i + 1).unwrap_or(0);
        let prev = text[prev_start..prev_end].trim_end();
        if !prev.is_empty() && !prev.ends_with(['.', ':', ';']) {
            line_start = prev_start;
        }
    }
    let lead_start = floor_boundary(text, m.start.saturating_sub(90)).max(line_start);
    let lead = text[lead_start..m.start].to_lowercase();
    let labelled = lead.trim_end().ends_with(':');
    let conf = |strong: f64| if labelled { 0.95 } else { strong };

    const RENEWAL: &[&str] = &["renewal", "renew", "no later than", "notice deadline", "option must be exercised"];
    const EXPIRY: &[&str] = &["expir", "terminat", "end date", "until", "due date", "due by", "by ", "ends on", "valid through", "maturity"];
    const EFFECTIVE: &[&str] = &["effective", "commenc", "dated", "entered into", "start date", "invoice date", "issue date", "issued", "as of"];

    let last = |needles: &[&str]| needles.iter().filter_map(|n| lead.rfind(n)).max();
    let candidates = [
        (RENEWAL_DEADLINE, last(RENEWAL)),
        (EXPIRY_DATE, last(EXPIRY)),
        (EFFECTIVE_DATE, last(EFFECTIVE)),
    ];
    // A renewal deadline is often phrased with generic words ("delivered by",
    // "no later than"), so renewal wins unless an explicit expiry word follows it.
    let explicit_expiry = last(&["expir", "terminat", "end date"]);
    let role = match (candidates[0].1, explicit_expiry) {
        (Some(r), e) if e.map_or(true, |e| e < r) => RENEWAL_DEADLINE,
        // Otherwise the keyword closest to the date wins
        // ("effective March 1 and expires on ...").
        _ => candidates
            .iter()
            .filter_map(|(k, pos)| pos.map(|p| (*k, p)))
            .max_by_key(|(_, p)| *p)?
            .0,
    };
    let strength = match role {
        r if r == EXPIRY_DATE && lead.contains("by ") && !lead.contains("expir") && !lead.contains("due") => 0.6,
        _ => 0.8,
    };
    Some((role, conf(strength)))
}

// ------------------------------------------------------------------ amount

fn amount(text: &str) -> Option<Extracted> {
    let money = re(r"(?i)(?:USD|US\$|EUR|GBP|INR|\$|€|£|₹)\s?\d{1,3}(?:,\d{3})*(?:\.\d{1,2})?(?:\s?(?:USD|EUR|GBP|INR))?|\b\d{1,3}(?:,\d{3})*(?:\.\d{2})?\s?(?:USD|EUR|GBP|INR)\b");
    let mut best: Option<(f64, f64, usize, String)> = None; // (priority, amount, offset, text)
    for m in money.find_iter(text) {
        let line_start = text[..m.start()].rfind('\n').map(|i| i + 1).unwrap_or(0);
        let line = text[line_start..m.start()].to_lowercase();
        let labelled = line.trim_end().ends_with(':');
        let priority = if ["total due", "amount due", "total:", "grand total", "balance due"].iter().any(|k| line.contains(k)) {
            3.0
        } else if labelled && ["amount", "total", "fee", "rent", "price", "consideration", "contract value", "value"].iter().any(|k| line.contains(k)) {
            2.5
        } else if ["amount", "total", "fee", "rent", "price", "consideration", "contract value"].iter().any(|k| line.contains(k)) {
            2.0
        } else {
            1.0
        };
        let value = parse_money(m.as_str());
        let better = match &best {
            None => true,
            Some((p, v, _, _)) => priority > *p || (priority == *p && value > *v),
        };
        if better {
            best = Some((priority, value, m.start(), m.as_str().trim().to_string()));
        }
    }
    best.map(|(p, _, off, s)| {
        let conf = if p >= 3.0 { 0.9 } else if p >= 2.5 { 0.85 } else if p >= 2.0 { 0.75 } else { 0.5 };
        found(AMOUNT, s, conf, text, off)
    })
}

fn parse_money(s: &str) -> f64 {
    let digits: String = s.chars().filter(|c| c.is_ascii_digit() || *c == '.').collect();
    digits.parse().unwrap_or(0.0)
}

// ------------------------------------------------------- law / notice / penalty

fn governing_law(text: &str) -> Option<Extracted> {
    let c = re(r"(?i)governed by,?\s+(?:and construed in accordance with\s+)?the\s+laws?\s+of\s+(?:the\s+)?(?:state\s+of\s+|commonwealth\s+of\s+|province\s+of\s+)?([A-Z][A-Za-z .]{1,40}?)(?:\s*[,.;)]|\s+without|\s+and\b|\s*$)")
        .captures(text)?;
    let m = c.get(1).unwrap();
    Some(found(GOVERNING_LAW, m.as_str().trim().to_string(), 0.85, text, m.start()))
}

fn notice_period(text: &str) -> Option<Extracted> {
    // "60 days' notice", "60 (sixty) days written notice", "thirty (30) days' notice"
    let c = re(r"(?i)(?:\b(\d{1,3})|\((\d{1,3})\))\s*(?:\([a-z\- ]+\)\s*)?(?:calendar\s+|business\s+)?days'?\s+(?:prior\s+|advance\s+)?(?:written\s+)?notice")
        .captures(text)
        .or_else(|| re(r"(?i)notice[^.\n]{0,40}?\b(\d{1,3})\s+days").captures(text))?;
    let days = c.get(1).or_else(|| c.get(2))?.as_str();
    Some(found(NOTICE_PERIOD, format!("{days} days"), 0.8, text, c.get(0).unwrap().start()))
}

fn penalty(text: &str) -> Option<Extracted> {
    // Strongest signal first: an explicit penalty or damages clause beats a late fee.
    let m = [
        r"(?i)penalt|liquidated damages|holdover",
        r"(?i)late (?:payment )?fee|late payments? accrue|interest at \d",
    ]
    .iter()
    .find_map(|p| first_in_body(text, re(p)))?;
    let (start, end) = sentence_around(text, m);
    let sentence = collapse_ws(&text[start..end]);
    let value = truncate_chars(&sentence, 220);
    Some(found(PENALTY, value, 0.7, text, start))
}

/// First match that is not inside an all-caps heading ("5. PENALTY CLAUSE");
/// a match in a heading is moved to the start of the text below it.
fn first_in_body(text: &str, pattern: &Regex) -> Option<usize> {
    for m in pattern.find_iter(text) {
        let line_start = text[..m.start()].rfind('\n').map(|i| i + 1).unwrap_or(0);
        let line_end = text[m.start()..].find('\n').map(|i| m.start() + i).unwrap_or(text.len());
        let line = &text[line_start..line_end];
        let is_heading = line.chars().any(|c| c.is_alphabetic()) && !line.chars().any(|c| c.is_lowercase());
        if !is_heading {
            return Some(m.start());
        }
        let body = text[line_end..].find(|c: char| !c.is_whitespace()).map(|i| line_end + i);
        if let Some(b) = body {
            return Some(b);
        }
    }
    None
}

// ----------------------------------------------------------------- helpers

fn found(key: &'static str, value: String, confidence: f64, text: &str, offset: usize) -> Extracted {
    let (s, e) = sentence_around(text, offset);
    Extracted {
        key,
        value,
        confidence,
        offset,
        snippet: truncate_chars(&collapse_ws(&text[s..e]), 280),
    }
}

/// The sentence (or line, for header-style documents) containing `offset`.
fn sentence_around(text: &str, offset: usize) -> (usize, usize) {
    let offset = floor_boundary(text, offset.min(text.len()));
    let before = &text[..offset];
    let start = before
        .rfind('\n')
        .map(|i| i + 1)
        .into_iter()
        .chain(before.rfind(". ").map(|i| i + 2))
        .max()
        .unwrap_or(0);
    // Header lines end at the newline; prose can wrap, so allow one wrap.
    let after = &text[offset..];
    let mut end = text.len();
    if let Some(i) = after.find(". ") {
        end = end.min(offset + i + 1);
    }
    if let Some(i) = after.find("\n\n") {
        end = end.min(offset + i);
    }
    if let Some(i) = after.find('\n') {
        let line_end = offset + i;
        // A labelled header line ("Expiry Date: ...") ends at the newline.
        if text[start..line_end].contains(':') {
            end = end.min(line_end);
        }
    }
    (start, end.max(offset))
}

fn collapse_ws(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn truncate_chars(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let mut out: String = s.chars().take(max - 1).collect();
    out.push('…');
    out
}

fn floor_boundary(text: &str, mut idx: usize) -> usize {
    while idx > 0 && !text.is_char_boundary(idx) {
        idx -= 1;
    }
    idx
}

fn char_to_byte(text: &str, char_idx: usize) -> usize {
    text.char_indices().nth(char_idx).map(|(b, _)| b).unwrap_or(0)
}

/// Compiled once per distinct pattern and kept for the life of the process.
fn re(pattern: &str) -> &'static Regex {
    use std::collections::HashMap;
    use std::sync::Mutex;
    static CACHE: OnceLock<Mutex<HashMap<String, &'static Regex>>> = OnceLock::new();
    let mut map = CACHE.get_or_init(|| Mutex::new(HashMap::new())).lock().unwrap();
    if let Some(r) = map.get(pattern) {
        return r;
    }
    let compiled: &'static Regex = Box::leak(Box::new(Regex::new(pattern).expect("valid regex")));
    map.insert(pattern.to_string(), compiled);
    compiled
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get<'a>(fields: &'a [Extracted], key: &str) -> Option<&'a Extracted> {
        fields.iter().find(|f| f.key == key)
    }

    const LEASE: &str = "COMMERCIAL LEASE AGREEMENT\n\nDocument Type: Lease Agreement\nVendor / Landlord: Acme Commercial Properties LLC\nClient / Tenant: Redstone Enterprises Inc.\nEffective Date: 2025-03-01\nExpiry Date: 2026-03-01\n\n2. TERM\n   The lease term commences on March 1, 2025 and expires on March 1, 2026.\n   Tenant must provide written notice of renewal no later than 60 days prior to expiry.\n\n3. RENEWAL\n   Renewal notice must be delivered in writing by January 1, 2026.\n\n4. RENT\n   Monthly base rent: $8,400 USD, payable on the first of each month.\n\n5. PENALTY CLAUSE\n   Failure to vacate the premises within 5 business days after lease expiry shall result in a\n   holdover penalty of $2,800 per day until the premises are fully vacated.\n";

    #[test]
    fn lease_fields() {
        let f = extract_all(LEASE);
        assert_eq!(get(&f, DOC_TYPE).unwrap().value, "Lease");
        assert_eq!(get(&f, PARTIES).unwrap().value, "Acme Commercial Properties LLC · Redstone Enterprises Inc.");
        assert_eq!(get(&f, EFFECTIVE_DATE).unwrap().value, "2025-03-01");
        assert_eq!(get(&f, EXPIRY_DATE).unwrap().value, "2026-03-01");
        assert_eq!(get(&f, EXPIRY_DATE).unwrap().confidence, 0.95);
        assert_eq!(get(&f, RENEWAL_DEADLINE).unwrap().value, "2026-01-01");
        assert_eq!(get(&f, AMOUNT).unwrap().value, "$8,400 USD");
        assert_eq!(get(&f, NOTICE_PERIOD).unwrap().value, "60 days");
        let p = get(&f, PENALTY).unwrap();
        assert!(p.value.contains("holdover penalty of $2,800 per day"), "{}", p.value);
    }

    #[test]
    fn evidence_offsets_point_at_the_value() {
        let f = extract_all(LEASE);
        let exp = get(&f, EXPIRY_DATE).unwrap();
        assert!(LEASE[exp.offset..].starts_with("2026-03-01"));
        assert!(exp.snippet.contains("Expiry Date: 2026-03-01"), "{}", exp.snippet);
    }

    #[test]
    fn prose_contract_without_labels() {
        let text = "SERVICES AGREEMENT\nThis Services Agreement is entered into as of January 5, 2025 between Delta Advisory Group LLC (\"Provider\") and Orion Retail Inc. (\"Client\"). This Agreement shall terminate on 31 December 2026 unless renewed. This Agreement shall be governed by the laws of the State of New York, without regard to conflicts. Either party may terminate upon thirty (30) days' written notice. Late payments accrue interest at 1.5% per month.";
        let f = extract_all(text);
        assert_eq!(get(&f, DOC_TYPE).unwrap().value, "Service Agreement");
        assert_eq!(get(&f, PARTIES).unwrap().value, "Delta Advisory Group LLC · Orion Retail Inc.");
        assert_eq!(get(&f, EFFECTIVE_DATE).unwrap().value, "2025-01-05");
        assert_eq!(get(&f, EXPIRY_DATE).unwrap().value, "2026-12-31");
        assert_eq!(get(&f, GOVERNING_LAW).unwrap().value, "New York");
        assert_eq!(get(&f, NOTICE_PERIOD).unwrap().value, "30 days");
        assert!(get(&f, PENALTY).unwrap().value.starts_with("Late payments accrue interest"));
    }

    #[test]
    fn invoice_total_beats_line_items() {
        let text = "INVOICE\nVendor: Harbourview IT Services Ltd.\nClient: Redstone Enterprises Inc.\nInvoice Date: 2026-02-01\nDue Date: 2026-03-01\n1. Audit   $6,500.00\n2. Testing $5,250.00\nTOTAL DUE:  $14,750.00\n";
        let f = extract_all(text);
        assert_eq!(get(&f, DOC_TYPE).unwrap().value, "Invoice");
        assert_eq!(get(&f, AMOUNT).unwrap().value, "$14,750.00");
        assert_eq!(get(&f, EFFECTIVE_DATE).unwrap().value, "2026-02-01");
        assert_eq!(get(&f, EXPIRY_DATE).unwrap().value, "2026-03-01");
    }

    #[test]
    fn date_context_follows_a_line_wrap() {
        let f = extract_all("3. RENEWAL\n   Client may renew by giving written notice no later than\n   November 15, 2026.\n");
        assert_eq!(get(&f, RENEWAL_DEADLINE).unwrap().value, "2026-11-15");
        let f = extract_all("Effective Date: 2025-03-01\nExpiry Date: 2026-03-01\n");
        assert_eq!(get(&f, EFFECTIVE_DATE).unwrap().value, "2025-03-01", "labelled lines never borrow context");
    }

    #[test]
    fn unlabelled_dates_are_not_guessed() {
        let f = extract_all("Meeting notes. We met on 2025-04-02 and again on 2025-05-09.");
        assert!(get(&f, EXPIRY_DATE).is_none());
        assert!(get(&f, EFFECTIVE_DATE).is_none());
    }

    #[test]
    fn date_formats() {
        let d: Vec<String> = find_dates("on 2025-01-02, Feb. 3rd, 2025, 4 March 2025, 12/25/2025 and 31/12/2025")
            .iter()
            .map(|d| d.date.to_string())
            .collect();
        assert_eq!(d, vec!["2025-01-02", "2025-02-03", "2025-03-04", "2025-12-25", "2025-12-31"]);
    }

    #[test]
    fn docx_table_cells_and_release_is_not_lease() {
        let text = "Statement of Work #7\nField\nValue\nVendor\nAcme Legal LLC\nClient\nNovaBio Systems\nIssue Date\n2025-11-27\nDue Date\n2027-01-11\nAmount Due\n$105,362.00\n";
        let f = extract_all(text);
        assert_eq!(get(&f, DOC_TYPE).unwrap().value, "Statement of Work");
        assert_eq!(get(&f, PARTIES).unwrap().value, "Acme Legal LLC · NovaBio Systems");
        assert_eq!(get(&f, EXPIRY_DATE).unwrap().value, "2027-01-11");
        assert_eq!(get(&f, AMOUNT).unwrap().value, "$105,362.00");
        assert!(text[get(&f, AMOUNT).unwrap().offset..].starts_with("$105,362.00"), "offsets survive normalization");
        assert_eq!(classify("Settlement and Release Agreement #12\nVendor: X").value, "Settlement");
    }

    #[test]
    fn email_sender_is_a_party() {
        let text = "From: Vexlar Consulting Group <legal@vexlar.com>\nTo: ClearPath <a@b.com>\nSubject: Invoice #INV-147\n\nAmount: $53,690.00\nExpiry / Due Date: 2027-02-25\n";
        let f = extract_all(text);
        assert_eq!(get(&f, PARTIES).unwrap().value, "Vexlar Consulting Group");
        assert_eq!(get(&f, EXPIRY_DATE).unwrap().value, "2027-02-25");
        assert_eq!(get(&f, AMOUNT).unwrap().value, "$53,690.00");
    }
}
