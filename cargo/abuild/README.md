# abuild

> a tool for building

## Language Support

1. **Rust**
2. **C++ Modules**
3. **C++**
4. **C**
5. **C#**

> [!NOTE]
>
> Sort by development priority

## Usage

```shell
$ abuild new -w my-worksapce
...
$ cd my-workspace
$ abuild new -j my-project
...
$ vi ./my-project/src/main.rs # edit your code
$ abuild build
...
$ ./target/debug/my-project
Hello, world!
$ abuild clean
...
$ 
```

## Config

### Project

```rust
// abuild.rs
// use ::abuild::prelude::*;
use ::abuild::{project::Project, profile::Profiles, target::Targets};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Project::default().parse_args().run()?;
    Project::builder()
        .src_dir("./src")
        .rc_dir("./rc")
        .profiles(
            // Profiles::default()?
            Profiles::builder()
                .dev("debug")
                .release("release")
                .build()?
        )
        .targets(
            // Targets::host()?
            Targets::builder()
                .host()
                .build()?
        )
        .build()?
        .parse_args()
        .run()?;
    Ok(())
}
```

### Workspace

```rust
// abuild.rs
// use ::abuild::prelude::*;
use ::abuild::{workspace::Workspace, project::Projects};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Workspace::default().parse_args().run()?;
    Workspace::builder()
        .projects(
            // Projects::default()?
            Projects::with_dirs([
                "./project_a",
                "./project_b"
            ])
                .build()?
        )
        .build()?
        .parse_args()
        .run()?;
    Ok(())
}
```

### Workspace & Module

```rust
// abuild.rs
// use ::abuild::prelude::*;
use ::abuild::workspace::Workspace;
mod project_a;
mod project_b;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Workspace::with_projects([project_a::project()?, project_b::project()?]).parse_args().run()?;
    Workspace::builder()
        .projects(
            Projects::from_slice([
                project_a::project()?,
                project_b::project()?,
            ])
        )
        .build()?
        .parse_args()
        .run()?;
    Ok(())
}
```

### Targets

```rust
// use ::abuild::prelude::*;
use ::abuild::target::Targets;
fn targets() -> Result<Targets, Box<dyn std::error::Error>> {
    Ok(
        // Targets::host()?
        Targets::builder()
            .host()
            .build()?
    )
}
```

### Profiles

```rust
// use ::abuild::prelude::*;
use ::abuild::profile::Profiles;
fn profiles() -> Result<Profiles, Box<dyn std::error::Error>> {
    Ok(
        // Profiles::default()?
        Profiles::builder()
            .dev("debug")
            .release("release")
            .target_dir("./target")
            .build_dir("build") // ./target/build
            .deps_dir("deps") // ./target/deps
            .bin_dir("bin") // ./target/bin
            .build()?
    )
}
```
