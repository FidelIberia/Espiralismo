//! Inflection keys and agreement for epithet pieces (`m`/`f`/`n`, `s`/`p`, …).

/// Head gender for agreement (Spanish/Russian); English ignores for invariant pieces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Gender {
    M,
    F,
    /// Neuter / epicene (Russian сущ., English "Orb").
    N,
}

/// Grammatical number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Number {
    Singular,
    Plural,
}

/// Agreement target carried by the chosen stem.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AgreementKey {
    pub gender: Gender,
    pub number: Number,
}

impl AgreementKey {
    #[must_use]
    pub fn from_tags(g: &str, n: &str) -> Self {
        Self {
            gender: match g {
                "f" | "F" => Gender::F,
                "n" | "N" => Gender::N,
                _ => Gender::M,
            },
            number: match n {
                "p" | "P" => Number::Plural,
                _ => Number::Singular,
            },
        }
    }
}

/// One inflected surface form set (adjective, curse, title).
#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct InflectedWord {
    /// Single form for all keys (English, invariant phrases).
    #[serde(default)]
    pub invariant: Option<String>,
    #[serde(default)]
    pub ms: Option<String>,
    #[serde(default)]
    pub fs: Option<String>,
    #[serde(default)]
    pub mp: Option<String>,
    #[serde(default)]
    pub fp: Option<String>,
    /// Russian neuter singular / plural.
    #[serde(default)]
    pub ns: Option<String>,
    #[serde(default)]
    pub np: Option<String>,
    /// Free-form tags (`incorporeo`, `metal`, …) for compatibility checks.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Stem must carry one of these tags / semantic classes (empty = no constraint).
    #[serde(default, alias = "requires_stem")]
    pub requires: Vec<String>,
    /// Skip when a modifier with this tag is already present.
    #[serde(default)]
    pub forbids: Vec<String>,
    /// Redundancy family (`damage`, `decay`, …) — one per phrase.
    #[serde(default)]
    pub group: Option<String>,
    /// `object_adj` | `living_trait` | `character_epithet` (titles default to epithet in code).
    #[serde(default)]
    pub kind: Option<String>,
}

impl InflectedWord {
    fn inflected(&self, key: AgreementKey) -> Option<&str> {
        match (key.gender, key.number) {
            (Gender::M, Number::Singular) => self.ms.as_deref(),
            (Gender::F, Number::Singular) => self.fs.as_deref(),
            (Gender::M, Number::Plural) => self.mp.as_deref(),
            (Gender::F, Number::Plural) => self.fp.as_deref(),
            (Gender::N, Number::Singular) => self
                .ns
                .as_deref()
                .or(self.ms.as_deref())
                .or(self.fs.as_deref()),
            (Gender::N, Number::Plural) => self
                .np
                .as_deref()
                .or(self.mp.as_deref())
                .or(self.fp.as_deref()),
        }
    }

    /// Inflected surface for the head noun; falls back to [`invariant`] only when no keyed form exists.
    #[must_use]
    pub fn resolve_agreeing(&self, key: AgreementKey) -> Option<&str> {
        self.inflected(key)
            .or_else(|| self.invariant.as_deref())
    }

    /// Prefer invariant when set (English tables); otherwise inflect.
    #[must_use]
    pub fn resolve(&self, key: AgreementKey) -> Option<&str> {
        if let Some(ref inv) = self.invariant {
            return Some(inv.as_str());
        }
        self.inflected(key)
    }
}

/// Noun stem with lexical gender/number.
///
/// **family** — English snake_case id shared across `locales/*.toml` (`filament`, `black_candle`).
/// Used only to pick one concept per roll (singular/plural variants share the same family).
///
/// **tags** (alias **traits** in TOML) — semantic archetypes for filters (`edge`, `vessel`, `weapon`).
/// Same vocabulary as adjective `requires` / archetype rules; not for cross-locale identity.
///
/// **groups** — damage/decay already in the surface (`roto` → `damage`); blocks redundant modifiers.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct StemEntry {
    pub text: String,
    pub g: String,
    pub n: String,
    /// English snake_case id (`filament`, `black_candle`) — same across all locale files.
    pub family: String,
    /// Semantic archetypes for filters; in TOML you may write `traits = [...]` instead.
    #[serde(default, alias = "traits")]
    pub tags: Vec<String>,
    /// Broad class when not expressed via [`tags`]: `object`, `person`, `divine`, `creature`, `abstract`.
    #[serde(default)]
    pub semantic: Option<String>,
    /// Redundancy groups already expressed in the stem (`damage`, `emptiness`, …).
    #[serde(default)]
    pub groups: Vec<String>,
    /// Singular relic/throne/etc. — plural form must not take `de Patron`.
    #[serde(default)]
    pub unique: bool,
}

impl StemEntry {
    #[must_use]
    pub fn key(&self) -> AgreementKey {
        AgreementKey::from_tags(&self.g, &self.n)
    }
}

/// Qualifier phrase (often invariant in Spanish: `del abismo`).
#[derive(Debug, Clone, serde::Deserialize)]
pub struct QualifierEntry {
    pub text: String,
    /// When set, only pairs with this gender on the head stem.
    #[serde(default)]
    pub g: Option<String>,
    /// When set, only pairs with this number.
    #[serde(default)]
    pub n: Option<String>,
}

/// Past participle / verbal state (concords with the **head** noun).
#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct VerbalState {
    #[serde(default)]
    pub invariant: Option<String>,
    #[serde(default)]
    pub ms: Option<String>,
    #[serde(default)]
    pub fs: Option<String>,
    #[serde(default)]
    pub mp: Option<String>,
    #[serde(default)]
    pub fp: Option<String>,
    #[serde(default)]
    pub ns: Option<String>,
    #[serde(default)]
    pub np: Option<String>,
    /// `object_fate` | `physical` | `agent_experience` (see [`super::semantic::VerbalKind`]).
    #[serde(default)]
    pub kind: Option<String>,
    /// Participial class for agent pairing (`seal`, `bury`, `drag`, …).
    #[serde(default)]
    pub verb_tags: Vec<String>,
    /// Stem must carry one of these tags / classes (empty = inferred from [`kind`]).
    #[serde(default, alias = "requires_stem")]
    pub requires: Vec<String>,
    /// When true, the participle is only used with an agent (`tocada por X`), never solo.
    #[serde(default)]
    pub requires_agent: bool,
}

impl VerbalState {
    #[must_use]
    pub fn participle(&self, key: AgreementKey) -> Option<&str> {
        if let Some(ref inv) = self.invariant {
            return Some(inv.as_str());
        }
        match (key.gender, key.number) {
            (Gender::M, Number::Singular) => self.ms.as_deref(),
            (Gender::F, Number::Singular) => self.fs.as_deref(),
            (Gender::M, Number::Plural) => self.mp.as_deref(),
            (Gender::F, Number::Plural) => self.fp.as_deref(),
            (Gender::N, Number::Singular) => self
                .ns
                .as_deref()
                .or(self.ms.as_deref())
                .or(self.fs.as_deref()),
            (Gender::N, Number::Plural) => self
                .np
                .as_deref()
                .or(self.mp.as_deref())
                .or(self.fp.as_deref()),
        }
    }
}

/// Optional agent for a verbal piece (`por la sombra`, `por los orcos`, indefinite `alguien`).
#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct AgentEntry {
    /// Full phrase, usually with article: `la sombra`, `los orcos`.
    pub text: String,
    /// Semantic gender tag (metadata / filtering).
    #[serde(default)]
    pub g: Option<String>,
    #[serde(default)]
    pub n: Option<String>,
    /// Linker before agent (`por`, `de`, `by`, …).
    #[serde(default = "default_linker_por")]
    pub linker: String,
    /// Indefinite actor (`alguien`, `quien sabe quién`) — spiralismo: not always someone concrete.
    #[serde(default)]
    pub indefinite: bool,
    /// `entity` (default) | `abstract` (judgment, madness as concept — cannot act).
    #[serde(default)]
    pub kind: Option<String>,
    /// Agent class tags (`storm`, `titan`, `divine`, …) for verb pairing weights.
    #[serde(default)]
    pub tags: Vec<String>,
}

fn default_linker_por() -> String {
    "por".to_string()
}

impl QualifierEntry {
    #[must_use]
    pub fn matches(&self, key: AgreementKey) -> bool {
        if let Some(ref g) = self.g {
            let want = AgreementKey::from_tags(g, "s").gender;
            if want != key.gender && want != Gender::N {
                return false;
            }
        }
        if let Some(ref n) = self.n {
            let want = AgreementKey::from_tags("m", n).number;
            if want != key.number {
                return false;
            }
        }
        true
    }
}
