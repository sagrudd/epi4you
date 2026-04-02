use crate::epi4you_errors::Epi4youError;

pub struct NextflowArtefact {}

impl NextflowArtefact {
    pub fn init() -> Result<NextflowArtefact, Epi4youError> {
        return Ok(NextflowArtefact {});
    }
}
