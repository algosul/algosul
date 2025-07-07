use std::{any::Any, error::Error, fmt::Debug, path::PathBuf};

use crate::project::Project;
pub trait ProjectCommand: Debug {
    fn tag(&self) -> String;
    fn try_do(
        &self, project: &mut Project, args: &str,
    ) -> Result<Box<dyn Any>, Box<dyn Error>>;
    fn try_undo(
        &self, project: &mut Project, cache: &mut dyn Any, args: &str,
    ) -> Result<(), Box<dyn Error>>;
    fn try_redo(
        &self, project: &mut Project, cache: &mut dyn Any, args: &str,
    ) -> Result<(), Box<dyn Error>>;
}
#[derive(Default, Debug, Clone)]
pub struct BuildCommand {}
#[derive(Debug, Clone)]
struct BuildCommandCache {
    files: Vec<PathBuf>,
}
impl ProjectCommand for BuildCommand {
    fn tag(&self) -> String { "build".to_string() }

    fn try_do(
        &self, project: &mut Project, args: &str,
    ) -> Result<Box<dyn Any>, Box<dyn Error>> {
        todo!()
    }

    fn try_undo(
        &self, project: &mut Project, cache: &mut dyn Any, args: &str,
    ) -> Result<(), Box<dyn Error>> {
        todo!()
    }

    fn try_redo(
        &self, project: &mut Project, cache: &mut dyn Any, args: &str,
    ) -> Result<(), Box<dyn Error>> {
        todo!()
    }
}
