use std::{collections::HashMap, fmt::Debug, sync::Arc};

use commands::ProjectCommand;
use sources::ProjectSources;

use crate::{
    profile::Profile,
    project::sources::ProjectSourcesInDir,
    utils::{Builder, HasBuilder},
};
pub mod commands;
pub mod sources;
/// ```
/// # use std::sync::Arc;
/// use abuild::{
///     profile::{DevProfile, ReleaseProfile},
///     project::{Project, commands::BuildCommand},
///     utils::{Builder, HasBuilder},
/// };
/// Project::builder()
///     .command(Arc::new(BuildCommand::default()))
///     .profile("debug", Arc::new(DevProfile::default()))
///     .profile("release", Arc::new(ReleaseProfile::default()))
///     .build()
///     .unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct Project {
    commands: HashMap<String, Arc<dyn ProjectCommand>>,
    profiles: HashMap<String, Arc<dyn Profile>>,
    sources:  Arc<dyn ProjectSources>,
}
#[derive(Debug, Clone)]
pub struct ProjectBuilder {
    commands: HashMap<String, Arc<dyn ProjectCommand>>,
    profiles: HashMap<String, Arc<dyn Profile>>,
    sources:  Arc<dyn ProjectSources>,
}
impl Builder for ProjectBuilder {
    type Error = ();
    type Output = Project;

    fn new() -> Self {
        Self {
            commands: HashMap::new(),
            profiles: HashMap::new(),
            sources:  Arc::new(ProjectSourcesInDir::new("src")),
        }
    }

    fn build(&self) -> Result<Self::Output, Self::Error> {
        Ok(Project {
            commands: self.commands.clone(),
            profiles: self.profiles.clone(),
            sources:  self.sources.clone(),
        })
    }
}
impl HasBuilder for Project {
    type Builder = ProjectBuilder;
}
impl ProjectBuilder {
    pub fn command(&mut self, command: Arc<dyn ProjectCommand>) -> &mut Self {
        self.commands.insert(command.tag(), command);
        self
    }

    pub fn commands(
        &mut self, commands: impl Iterator<Item = Arc<dyn ProjectCommand>>,
    ) -> &mut Self {
        for command in commands {
            self.command(command);
        }
        self
    }

    pub fn profile(
        &mut self, name: impl Into<String>, profile: Arc<dyn Profile>,
    ) -> &mut Self {
        self.profiles.insert(name.into(), profile);
        self
    }

    pub fn profiles(
        &mut self, profiles: impl Iterator<Item = (String, Arc<dyn Profile>)>,
    ) -> &mut Self {
        for (name, profile) in profiles {
            self.profile(name, profile);
        }
        self
    }

    pub fn sources(&mut self, source: Arc<dyn ProjectSources>) -> &mut Self {
        self.sources = source;
        self
    }
}
