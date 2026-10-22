/// Declaration Package — what the Generator submits.
///
/// The Generator declares:
///   - The raw observable source (implementing `Observable`)
///   - The name of the authorized mapping to apply
///   - The topology under which evaluation is requested
///   - The provenance chain (named steps — NOT computed results)
///
/// The Generator does NOT compute the mapping's output.
/// The Generator does NOT construct `AdmittedObservation<T>`.
/// The Generator names; the gate executes.

use crate::observable::Observable;

/// A single named link in a provenance chain.
/// Names a transformation step. Does not supply its numerical output.
#[derive(Debug, Clone)]
pub struct ProvenanceLink {
    pub step_name:   &'static str,
    pub description: &'static str,
}

/// An ordered sequence of named transformation steps.
/// Declaration only — not a computation.
#[derive(Debug, Clone, Default)]
pub struct ProvenanceChain {
    links: Vec<ProvenanceLink>,
}

impl ProvenanceChain {
    pub fn new() -> Self { ProvenanceChain { links: Vec::new() } }

    pub fn link(mut self, step_name: &'static str, description: &'static str) -> Self {
        self.links.push(ProvenanceLink { step_name, description });
        self
    }

    pub fn links(&self) -> &[ProvenanceLink] { &self.links }
    pub fn is_empty(&self) -> bool { self.links.is_empty() }

    pub fn verify_no_silent_links(&self) -> Result<(), ProvenanceError> {
        for link in &self.links {
            if link.step_name.is_empty() {
                return Err(ProvenanceError::SilentLink { description: link.description });
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum ProvenanceError {
    SilentLink { description: &'static str },
}

impl std::fmt::Display for ProvenanceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProvenanceError::SilentLink { description } =>
                write!(f, "unnamed transformation step: '{}'", description),
        }
    }
}

/// A declaration package submitted by the Generator to the gate.
///
/// `O` is the observable source type (implements `Observable`).
///
/// Fields:
///   - `source`:        the raw observation
///   - `mapping_name`:  which authorized mapping to apply (gate executes it)
///   - `topology_id`:   the declared topology
///   - `provenance`:    named transformation steps (declaration only)
pub struct DeclarationPackage<O: Observable> {
    pub source:       O,
    pub mapping_name: &'static str,
    pub topology_id:  &'static str,
    pub provenance:   ProvenanceChain,
}
