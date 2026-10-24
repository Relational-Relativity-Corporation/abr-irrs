/// Kernel V8 — Region Establishment
///
/// Region establishment is the precondition for declaring directed relations
/// through M. Before a relation can be declared as a primitive, the observable
/// region in which it exists must be identified, bounded, and admitted. A
/// relation declared outside an established region has no grounding in the
/// bounded domain — one of the three characteristics from which the primitive
/// itself derives.
///
/// The primitive is defined in `lib.rs`. Region establishment is its
/// prerequisite, not its product.
///
/// ───────────────────────────────────────────────────────────────────────────
///
/// Regional context is a prerequisite to topology instantiation.
/// No topology or operator may be instantiated without a verified region.
///
/// C3: The admission gate enforces the complete regional M prerequisite:
///   1. Region identified from observable evidence.
///   2. Observable extents declared.
///   3. Cross-family orientation declared where applicable.
///   4. Regional admission rule established.
///   5. Structural interior-locus condition satisfied.
///
/// None of these are defaults. No missing value is silently replaced with zero.

/// A declared observable extent. Must be supplied; no default exists.
#[derive(Debug, Clone)]
pub struct ObservableExtent {
    /// The lower bound of the observable range (inclusive).
    pub lower: f64,
    /// The upper bound of the observable range (inclusive).
    pub upper: f64,
    /// The unit of the observable.
    pub unit: &'static str,
}

impl ObservableExtent {
    pub fn new(lower: f64, upper: f64, unit: &'static str) -> Result<Self, RegionError> {
        if lower >= upper {
            return Err(RegionError::InvalidExtent {
                reason: "lower bound must be strictly less than upper bound",
            });
        }
        Ok(ObservableExtent { lower, upper, unit })
    }

    /// Test whether a value lies strictly within the declared interior.
    /// The interior-locus condition requires that the locus is interior to
    /// the declared extents — not on the boundary.
    pub fn is_interior(&self, value: f64) -> bool {
        value > self.lower && value < self.upper
    }
}

/// Cross-family orientation: what is preserved across a transformation, and
/// what is discarded. Both must be explicitly declared.
#[derive(Debug, Clone)]
pub struct CrossFamilyOrientation {
    pub preserved: &'static str,
    pub discarded: &'static str,
}

/// A regional admission rule: a named predicate that must hold for the region
/// to be considered established from observable evidence.
#[derive(Debug, Clone)]
pub struct RegionalAdmissionRule {
    pub name: &'static str,
    pub description: &'static str,
}

/// A fully established region, produced only by RegionBuilder::establish().
/// The interior-locus condition is verified at construction.
#[derive(Debug, Clone)]
pub struct EstablishedRegion {
    pub name: &'static str,
    pub extents: ObservableExtent,
    pub orientation: CrossFamilyOrientation,
    pub admission_rule: RegionalAdmissionRule,
    /// The locus value that satisfied the interior-locus condition.
    pub verified_locus: f64,
}

/// Builder for region establishment.
/// All fields are required. No step may be skipped.
pub struct RegionBuilder {
    name: Option<&'static str>,
    extents: Option<ObservableExtent>,
    orientation: Option<CrossFamilyOrientation>,
    admission_rule: Option<RegionalAdmissionRule>,
}

impl RegionBuilder {
    pub fn new() -> Self {
        RegionBuilder {
            name: None,
            extents: None,
            orientation: None,
            admission_rule: None,
        }
    }

    pub fn name(mut self, name: &'static str) -> Self {
        self.name = Some(name);
        self
    }

    pub fn extents(mut self, extents: ObservableExtent) -> Self {
        self.extents = Some(extents);
        self
    }

    pub fn orientation(mut self, orientation: CrossFamilyOrientation) -> Self {
        self.orientation = Some(orientation);
        self
    }

    pub fn admission_rule(mut self, rule: RegionalAdmissionRule) -> Self {
        self.admission_rule = Some(rule);
        self
    }

    /// Establish the region.
    ///
    /// `observable_locus` is the value from observable evidence that must lie
    /// strictly interior to the declared extents (the interior-locus condition).
    ///
    /// Returns `RegionError::MissingField` if any required declaration is absent.
    /// Returns `RegionError::LocusNotInterior` if the locus condition fails.
    /// No missing value is replaced with a default.
    pub fn establish(self, observable_locus: f64) -> Result<EstablishedRegion, RegionError> {
        let name = self.name.ok_or(RegionError::MissingField { field: "name" })?;
        let extents = self.extents.ok_or(RegionError::MissingField { field: "extents" })?;
        let orientation = self.orientation.ok_or(RegionError::MissingField { field: "orientation" })?;
        let admission_rule = self.admission_rule.ok_or(RegionError::MissingField { field: "admission_rule" })?;

        if !extents.is_interior(observable_locus) {
            return Err(RegionError::LocusNotInterior {
                locus: observable_locus,
                lower: extents.lower,
                upper: extents.upper,
            });
        }

        Ok(EstablishedRegion {
            name,
            extents,
            orientation,
            admission_rule,
            verified_locus: observable_locus,
        })
    }
}

impl Default for RegionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub enum RegionError {
    MissingField { field: &'static str },
    InvalidExtent { reason: &'static str },
    LocusNotInterior { locus: f64, lower: f64, upper: f64 },
}

impl std::fmt::Display for RegionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegionError::MissingField { field } =>
                write!(f, "regional establishment rejected: required field '{}' not declared", field),
            RegionError::InvalidExtent { reason } =>
                write!(f, "invalid observable extent: {}", reason),
            RegionError::LocusNotInterior { locus, lower, upper } =>
                write!(f, "interior-locus condition failed: {} is not interior to ({}, {})", locus, lower, upper),
        }
    }
}
