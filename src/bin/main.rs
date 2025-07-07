#![feature(lock_value_accessors)]
use std::{
    fmt::{Debug, Display},
    str::FromStr,
    sync::LazyLock,
};

use annasul::app::{
    AppOper,
    apps::rust::{
        HostTriple,
        InstallCustomInfo,
        InstallInfo,
        Profile,
        Rustup,
        Toolchain,
    },
};
use gtk4::{
    Application,
    ApplicationWindow,
    Box,
    Button,
    ComboBoxText,
    DropDown,
    Entry,
    Label,
    MessageDialog,
    Notebook,
    Orientation,
    StringList,
    StringObject,
    glib::ExitCode,
    prelude::*,
};
use tokio::{join, runtime::Runtime, sync::Mutex};
#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
struct DronDownBuilder<'a, T> {
    all: &'a [T],
}
impl<'a, T> DronDownBuilder<'a, T> {
    fn new() -> Self { Self { all: &[] } }

    fn all(mut self, all: &'a [T]) -> Self {
        self.all = all;
        self
    }
}
struct NotebookBuilder<'a, T> {
    all: &'a [T],
    f:   std::boxed::Box<dyn Fn(&T) -> bool>,
}
impl<'a, T> NotebookBuilder<'a, T> {
    fn new() -> Self { Self { all: &[], f: std::boxed::Box::new(|_| true) } }

    fn all(mut self, all: &'a [T]) -> Self {
        self.all = all;
        self
    }

    fn on_change(mut self, f: impl Fn(&T) -> bool + 'static) -> Self {
        self.f = std::boxed::Box::new(f);
        self
    }
}
trait Builder {
    type Output;
    type Error;
    type Info;
    fn build(self, info: Self::Info) -> Result<Self::Output, Self::Error>;
}
#[deprecated]
#[allow(deprecated)]
fn build_combo<T: Default + Debug + Display + FromStr + 'static>(
    combo: ComboBoxText, all: &[T], f: fn(&T),
) -> ComboBoxText {
    for i in all.iter() {
        combo.append(Some(&i.to_string()), &i.to_string());
    }
    combo.set_active_id(Some(&T::default().to_string()));
    combo.connect_changed(move |combo| {
        let active_id = combo.active_id().and_then(|id| id.parse().ok());
        if let Some(id) = active_id {
            f(&id);
        }
    });
    combo
}
fn build_dropdown<
    T: Default + Debug + Display + FromStr + PartialEq + Eq + 'static,
>(
    all: &[T], f: fn(&T),
) -> DropDown
where <T as FromStr>::Err: Debug {
    let model = StringList::from_iter(all.iter().map(|x| x.to_string()));
    let dropdown = DropDown::builder()
        .model(&model)
        .selected(all.iter().position(|t| t == &T::default()).unwrap() as u32)
        .build();
    dropdown.connect_notify_local(Some("selected"), move |dropdown, _| {
        if let Some(item) = dropdown.selected_item() {
            let text = item.downcast::<StringObject>().unwrap().string();
            f(&T::from_str(text.as_str()).unwrap());
        }
    });
    dropdown
}
impl<T> Builder for NotebookBuilder<'_, T>
where T: Default + Debug + PartialEq + FromStr + Eq + Display + 'static + Clone
{
    type Error = ();
    type Info = ();
    type Output = Notebook;

    fn build(self, info: Self::Info) -> Result<Self::Output, Self::Error> {
        let model =
            self.all.iter().map(|x| x.to_string()).collect::<Vec<String>>();
        let notebook = Notebook::builder().build();
        for i in model {
            notebook.append_page(
                &Label::new(Some(&i)),
                Some(&Label::new(Some(&i))),
            );
        }
        notebook.set_current_page(Some(
            self.all.iter().position(|t| t == &T::default()).unwrap() as u32,
        ));
        let f = self.f;
        let all = self.all.to_vec();
        notebook.connect_change_current_page(move |_notebook: &Notebook, i| {
            f(&all[i as usize])
        });
        Ok(notebook)
    }
}
static DEFAULT_HOST_TRIPLE: LazyLock<Mutex<HostTriple>> =
    LazyLock::new(|| Mutex::new(Default::default()));
static DEFAULT_TOOLCHAIN: LazyLock<Mutex<Toolchain>> =
    LazyLock::new(|| Mutex::new(Default::default()));
static PROFILE: LazyLock<Mutex<Profile>> =
    LazyLock::new(|| Mutex::new(Default::default()));
static MODIFY_PATH_VARIABLE: LazyLock<Mutex<bool>> = LazyLock::new(|| {
    Mutex::new(InstallCustomInfo::default().modify_path_variable)
});
static INSTALL_THREAD: std::sync::Mutex<Option<tokio::task::JoinHandle<()>>> =
    std::sync::Mutex::new(None);
fn main() -> ExitCode {
    env_logger::init();
    let app =
        Application::builder().application_id("yuanair.github.io").build();
    app.connect_activate(|app| {
        let window = ApplicationWindow::builder()
            .application(app)
            .title(env!("CARGO_PKG_NAME"))
            .default_width(800)
            .default_height(600)
            .build();
        window.present();
        let notebook = Notebook::builder().build();
        window.set_child(Some(&notebook));
        for i in 1..=3 {
            let vbox = Box::new(Orientation::Vertical, 10);
            vbox.append(&build_dropdown(
                &[Profile::Minimal, Profile::Default, Profile::Complete],
                |profile| {
                    *Runtime::new()
                        .unwrap()
                        .block_on(async { PROFILE.lock().await }) = *profile;
                    println!("profile: {profile}");
                },
            ));
            vbox.append(&build_dropdown(
                &[
                    Toolchain::Stable,
                    Toolchain::Beta,
                    Toolchain::Nightly,
                    Toolchain::None,
                ],
                |toolchain| {
                    *Runtime::new()
                        .unwrap()
                        .block_on(async { DEFAULT_TOOLCHAIN.lock().await }) =
                        *toolchain;
                    println!("toolchain: {toolchain}");
                },
            ));
            vbox.append(
                &NotebookBuilder::new()
                    .all(&[
                        HostTriple::Host,
                        HostTriple::Target("".to_string()),
                    ])
                    .on_change(|host_triple| {
                        *Runtime::new().unwrap().block_on(async {
                            DEFAULT_HOST_TRIPLE.lock().await
                        }) = host_triple.clone();
                        println!("host_triple: {host_triple}");
                        true
                    })
                    .build(())
                    .unwrap(),
            );
            let user_input_args = Entry::builder()
                .placeholder_text("args e.g. `-h`")
                .margin_top(20)
                .build();
            vbox.append(&user_input_args);
            let button = Button::with_label("Install");
            vbox.append(&button);
            button.connect_clicked(move |button| {
                if let Some(ref install_thread) =
                    *INSTALL_THREAD.lock().unwrap()
                {
                    if install_thread.is_finished() {
                        button.set_label("Installed（已安装）");
                    } else {
                        button.set_label("Already Installing（正在安装）");
                    }
                }
                INSTALL_THREAD
                    .replace(Some(Runtime::new().unwrap().spawn(async move {
                        Rustup::install(InstallInfo::Custom(
                            InstallCustomInfo {
                                default_host_triple:  (*DEFAULT_HOST_TRIPLE
                                    .lock()
                                    .await)
                                    .clone(),
                                default_toolchain:    *DEFAULT_TOOLCHAIN
                                    .lock()
                                    .await,
                                profile:              *PROFILE.lock().await,
                                modify_path_variable: *MODIFY_PATH_VARIABLE
                                    .lock()
                                    .await,
                            },
                        ))
                        .await
                        .unwrap();
                        let dialog = MessageDialog::builder()
                            .title(env!("CARGO_PKG_NAME"))
                            .text("Done! 安装成功！")
                            .build();
                        dialog.run_async(|obj, answer| {
                            obj.close();
                        });
                    })))
                    .unwrap();
                println!("Input {i}: {}", user_input_args.text());
            });
            notebook.append_page(
                &vbox,
                Some(&Label::new(Some(&format!("Preset {i}")))),
            );
        }
    });
    app.connect_shutdown(|_| {
        if let Some(install_thread) = INSTALL_THREAD.lock().unwrap().take() {
            Runtime::new().unwrap().block_on(async {
                join!(install_thread).0.unwrap();
            });
        }
    });
    app.run()
}
