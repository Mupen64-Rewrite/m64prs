use std::{env::args, error::Error, time::Duration};
use iced::{event, subscription, time, widget::{container, text}, window, Application, Command, Event, Length, Settings, Size, Subscription, Theme};
use m64prs_core::{Core, Plugin, ctypes::PluginType};


#[derive(Clone, Debug)]
enum MainMessage {
    EventOccurred(Event),
    Tick,
}

#[derive(Debug, Default)]
struct MainApp {

}

impl Application for MainApp {
    type Executor = iced::executor::Default;

    type Message = MainMessage;

    type Theme = Theme;

    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        (MainApp::default(), Command::none())
    }

    fn title(&self) -> String {
        "Test app with events".to_owned()
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        match message {
            MainMessage::EventOccurred(evt) =>  {
                println!("event {:?}", evt);
                Command::none()
            },
            MainMessage::Tick => {
                println!("tick");
                Command::none()
            },
        }
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        Subscription::batch([
            time::every(Duration::from_millis(1000)).map(|_| MainMessage::Tick)
        ])
    }

    fn view(&self) -> iced::Element<'_, Self::Message, Self::Theme, iced::Renderer> {
        container(
            text("Hello, world!")
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .into()
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    MainApp::run(Settings {
        window: window::Settings {
            size: Size { width: 640.0, height: 480.0 },
            ..window::Settings::default()
        },
        ..Settings::default()
    })?;
    Ok(())
}

/// quick test function showcasing the rough workflow when using core.
#[allow(dead_code)]
fn encode_test() -> Result<(), Box<dyn Error>> {

    let _args: Vec<String> = args().skip(1).collect();

    let mut core = Core::load("/usr/lib/libmupen64plus.so.2")?;

    core.load_rom(&_args[0])?;
    println!("Loaded ROM");

    core.attach_plugin(PluginType::GFX, Plugin::load("/usr/lib/mupen64plus/mupen64plus-video-rice.so")?)?;
    core.attach_plugin(PluginType::AUDIO, Plugin::load("/usr/lib/mupen64plus/mupen64plus-audio-sdl.so")?)?;
    core.attach_plugin(PluginType::INPUT, Plugin::load("/usr/lib/mupen64plus/mupen64plus-input-sdl.so")?)?;
    core.attach_plugin(PluginType::RSP, Plugin::load("/usr/lib/mupen64plus/mupen64plus-rsp-hle.so")?)?;
    println!("Loaded plugins");

    core.execute_sync()?;

    core.detach_plugin(PluginType::GFX)?;
    core.detach_plugin(PluginType::AUDIO)?;
    core.detach_plugin(PluginType::INPUT)?;
    core.detach_plugin(PluginType::RSP)?;

    core.close_rom()?;

    Ok(())
}