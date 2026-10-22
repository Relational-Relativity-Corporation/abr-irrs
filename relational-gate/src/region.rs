/// Region Establishment — domain-agnostic
///
/// A region must be established from observable evidence before any topology
/// or operator can be instantiated. All fields are required; none are defaulted.

#[derive(Debug, Clone)]
pub struct ObservableExtent {
    pub lower: f64,
    pub upper: f64,
    pub unit:  &'static str,
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

    /// True only when `value` is strictly interior — not on the boundary.
    pub fn is_interior(&self, value: f64) -> bool {
        value > self.lower && value < self.upper
    }
}

#[derive(Debug, Clone)]
pub struct CrossFamilyOrientation {
    pub preserved: &'static str,
    pub discarded: &'static str,
}

#[derive(Debug, Clone)]
pub struct RegionalAdmissionRule {
    pub name:        &'static str,
    pub description: &'static str,
}

/// A fully established region. Only `RegionBuilder::establish` produces this.
#[derive(Debug, Clone)]
pub struct EstablishedRegion {
    pub name:            &'static str,
    pub extents:         ObservableExtent,
    pub orientation:     CrossFamilyOrientation,
    pub admission_rule:  RegionalAdmissionRule,
    pub verified_locus:  f64,
}

pub struct RegionBuilder {
    name:           Option<&'static str>,
    extents:        Option<ObservableExtent>,
    orientation:    Option<CrossFamilyOrientation>,
    admission_rule: Option<RegionalAdmissionRule>,
}

impl RegionBuilder {
    pub fn new() -> Self {
        RegionBuilder { name: None, extents: None, orientation: None, admission_rule: None }
    }

    pub fn name(mut self, name: &'static str) -> Self {
        self.name = Some(name); self
    }
    pub fn extents(mut self, extents: ObservableExtent) -> Self {
        self.extents = Some(extents); self
    }
    pub fn orientation(mut self, orientation: CrossFamilyOrientation) -> Self {
        self.orientation = Some(orientation); self
    }
    pub fn admission_rule(mut self, rule: RegionalAdmissionRule) -> Self {
        self.admission_rule = Some(rule); self
    }

    /// Establish the region.
    /// `observable_locus` must be strictly interior to the declared extents.
    /// Returns `RegionError::MissingField` if any declaration is absent.
    /// Nothing is defaulted.
    pub fn establish(self, observable_locus: f64) -> Result<EstablishedRegion, RegionError> {
        let name           = self.name.ok_or(RegionError::MissingField { field: "name" })?;
        let extents        = self.extents.ok_or(RegionError::MissingField { field: "extents" })?;
        let orientation    = self.orientation.ok_or(RegionError::MissingField { field: "orientation" })?;
        let admission_rule = self.admission_rule.ok_or(RegionError::MissingField { field: "admission_rule" })?;

        if !extents.is_interior(observable_locus) {
            return Err(RegionError::LocusNotInterior {
                locus: observable_locus,
                lower: extents.lower,
                upper: extents.upper,
            });
        }

        Ok(EstablishedRegion { name, extents, orientation, admission_rule, verified_locus: observable_locus })
    }
}

impl Default for RegionBuilder {
    fn default() -> Self { Self::new() }
}

#[derive(Debug)]
pub enum RegionError {
    MissingField    { field: &'static str },
    InvalidExtent   { reason: &'static str },
    LocusNotInterior { locus: f64, lower: f64, upper: f64 },
}

impl std::fmt::Display for RegionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegionError::MissingField { field } =>
                write!(f, "region rejected: required field '{}' not declared", field),
            RegionError::InvalidExtent { reason } =>
                write!(f, "invalid observable extent: {}", reason),
            RegionError::LocusNotInterior { locus, lower, upper } =>
                write!(f, "interior-locus condition failed: {} ∉ ({}, {})", locus, lower, upper),
        }
    }
}
