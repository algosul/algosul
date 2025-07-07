use std::{
    collections::HashSet,
    fmt::Debug,
    fs::{read_dir, read_link, DirEntry},
    io::ErrorKind,
    path::PathBuf,
};

use crate::{lang::FileType, profile::CompileOptions};
pub trait ProjectSources: Debug {
    fn sources(
        &self,
    ) -> std::io::Result<Box<dyn Iterator<Item = std::io::Result<Source>>>>;
}
#[derive(Debug, Clone)]
pub struct Source {
    pub file_path:       PathBuf,
    pub file_type:       FileType,
    pub compile_options: CompileOptions,
}
#[derive(Debug, Clone)]
pub struct ProjectSourcesInDir {
    src_dir: PathBuf,
}
#[derive(Debug)]
pub struct ProjectSourcesInDirIter {
    already:  HashSet<PathBuf>,
    no_ready: Vec<DirEntry>,
}
impl ProjectSourcesInDirIter {
    fn from_dir(dir: impl Into<PathBuf>) -> std::io::Result<Self> {
        let mut no_ready = Vec::new();
        for entry in read_dir(dir.into())? {
            let entry = entry?;
            no_ready.push(entry);
        }
        Ok(Self { already: HashSet::new(), no_ready })
    }

    fn file(&mut self, file_path: PathBuf) -> std::io::Result<Source> {
        let file_type =
            FileType::from_file(&file_path).ok_or(std::io::Error::new(
                ErrorKind::InvalidFilename,
                format!("unknown the path '{}' type", file_path.display()),
            ))?;
        Ok(Source {
            file_path,
            file_type,
            compile_options: CompileOptions::default(),
        })
    }

    fn dir(&mut self, file_path: PathBuf) -> std::io::Result<()> {
        read_dir(file_path)?.try_for_each(|x| {
            self.no_ready.push(x?);
            Ok::<(), std::io::Error>(())
        })
    }

    fn link(&mut self, file_path: PathBuf) -> std::io::Result<Option<Source>> {
        let file_path = read_link(file_path)?;
        if self.already.contains(&file_path) {
            return Ok(None);
        }
        if file_path.is_file() {
            self.file(file_path).map(Some)
        } else if file_path.is_dir() {
            self.dir(file_path).map(|()| None)
        } else if file_path.is_symlink() {
            self.link(file_path)
        } else {
            panic!("unknown the path '{}' type", file_path.display())
        }
    }
}
impl Iterator for ProjectSourcesInDirIter {
    type Item = std::io::Result<Source>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(entry) = self.no_ready.pop() {
                let file_path = entry.path();
                let file_type = match entry.file_type() {
                    Ok(ok) => ok,
                    Err(e) => break Some(Err(e)),
                };
                self.already.insert(file_path.clone());
                if self.already.contains(&file_path) {
                    continue;
                }
                if file_type.is_file() {
                    break Some(self.file(file_path));
                } else if file_type.is_dir() {
                    match self.dir(file_path) {
                        Err(e) => break Some(Err(e)),
                        Ok(()) => continue,
                    }
                } else if file_type.is_symlink() {
                    match self.link(file_path) {
                        Err(e) => break Some(Err(e)),
                        Ok(None) => continue,
                        Ok(Some(s)) => break Some(Ok(s)),
                    }
                } else {
                    panic!("unknown the path '{}' type", file_path.display())
                }
            } else {
                break None;
            }
        }
    }
}
impl ProjectSourcesInDir {
    pub fn new(src_dir: impl Into<PathBuf>) -> Self {
        Self { src_dir: src_dir.into() }
    }
}
impl ProjectSources for ProjectSourcesInDir {
    fn sources(
        &self,
    ) -> std::io::Result<Box<dyn Iterator<Item = std::io::Result<Source>>>>
    {
        Ok(Box::new(ProjectSourcesInDirIter::from_dir(self.src_dir.clone())?))
    }
}
