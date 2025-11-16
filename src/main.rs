pub mod display;
pub mod invoice;
pub mod menu_item;
pub(crate) mod page;
pub(crate) mod utilities;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let screen_size = display::get_primary_display_size()
        .map(|size| size.to_size())
        .unwrap_or(cosmic::iced::Size::new(1024.0, 768.0));

    let settings = cosmic::app::Settings::default()
        .size(screen_size)
        .antialiasing(true)
        .client_decorations(true)
        .exit_on_close(true)
        .debug(false);

    cosmic::app::run::<invoice::App>(settings, ())?;

    Ok(())
}
