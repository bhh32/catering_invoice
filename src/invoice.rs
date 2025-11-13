use crate::menu_item::MenuItem;
use chrono::Local;
use cosmic::{
    Action, Application, Core, Element, Task,
    iced::{Alignment, Length},
    widget::{button, column, container, divider, row, scrollable, text, text_input},
};
use printpdf::*;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::{self, Read, Write},
    path::PathBuf,
};

const APP_ID: &str = "com.bhh32.CateringInvoice";

#[derive(Debug, Clone)]
pub enum Message {
    UpdateDesc(String),
    UpdateRate(String),
    UpdateTaxRate(String),
    UpdatePeopleCount(String),
    UpdatePhoneNumber(String),
    UpdateEventDate(String),
    AddMenuItem,
    DeleteMenuItem(u32),
    EditMenuItem(u32),
    CancelEdit,
    UpdateMenuItem(u32),
    UpdateLetterhead(String),
    UpdateTitle(String),
    GeneratePDF,
    SavePDFTo(Option<PathBuf>),
    SaveInvoice,
    SaveInvoiceTo(Option<PathBuf>),
    LoadInvoice,
    LoadInvoiceFrom(Option<PathBuf>),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Invoice {
    desc: String,
    menu_items: Vec<MenuItem>,
    next_id: u32,
    letterhead: String,
    phone_number: String,
    invoice_title: String,
    event_date: String,
    price_per_person: String,
    num_people: u32,
    tax_percentage: String,
    total: f32,

    // Item Edit Mode
    editing_item: Option<u32>,
}

impl Invoice {
    pub fn save(&self, file_path: &str) -> io::Result<()> {
        let serialized = serde_json::to_string(self)?;
        let mut file = File::create(file_path)?;
        file.write_all(serialized.as_bytes())?;

        Ok(())
    }

    pub fn load(file_path: &str) -> io::Result<Self> {
        let mut file = File::open(file_path)?;
        let mut contents = String::new();

        file.read_to_string(&mut contents)?;
        let invoice: Invoice = serde_json::from_str(&contents)?;

        Ok(invoice)
    }

    pub fn subtotal(&self) -> f32 {
        self.price_per_person.parse::<f32>().unwrap_or(0.0) * self.num_people as f32
    }

    pub fn tax_rate_as_dec(&self) -> f32 {
        self.tax_percentage
            .replace("%", "")
            .parse::<f32>()
            .unwrap_or(10.0) // default tax rate is 10%
            / 100.0
    }

    pub fn tax(&self) -> f32 {
        self.subtotal() * self.tax_rate_as_dec()
    }

    pub fn total(&self) -> f32 {
        self.subtotal() + (self.subtotal() * self.tax_rate_as_dec())
    }
}

pub struct App {
    pub core: Core,
    pub invoice: Invoice,
}

impl App {
    fn generate_pdf(&self, file_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let mut doc = PdfDocument::new(&format!(
            "{} - {}",
            self.invoice.invoice_title,
            Local::now().format("%Y-%m-%d")
        ));

        let mut ops = Vec::new();
        let mut y_pos = 270.0;

        ops.push(Op::SaveGraphicsState);

        // Letterhead
        if !self.invoice.letterhead.is_empty() {
            ops.push(Op::StartTextSection);
            ops.push(Op::SetTextCursor {
                pos: Point::new(Mm(20.0), Mm(y_pos)),
            });
            ops.push(Op::SetFontSizeBuiltinFont {
                size: Pt(24.0),
                font: BuiltinFont::HelveticaBold,
            });
            ops.push(Op::WriteTextBuiltinFont {
                items: vec![TextItem::Text(self.invoice.letterhead.clone())],
                font: BuiltinFont::HelveticaBold,
            });
            ops.push(Op::EndTextSection);
            y_pos -= 10.0;

            ops.push(Op::StartTextSection);
            ops.push(Op::SetTextCursor {
                pos: Point::new(Mm(20.0), Mm(y_pos)),
            });
            ops.push(Op::SetFontSizeBuiltinFont {
                size: Pt(14.0),
                font: BuiltinFont::Helvetica,
            });
            ops.push(Op::WriteTextBuiltinFont {
                items: vec![TextItem::Text(self.invoice.phone_number.clone())],
                font: BuiltinFont::Helvetica,
            });
            ops.push(Op::EndTextSection);
            y_pos -= 15.0;
        }

        // Event Date
        if !self.invoice.event_date.is_empty() {
            ops.push(Op::StartTextSection);
            ops.push(Op::SetTextCursor {
                pos: Point::new(Mm(20.0), Mm(y_pos)),
            });
            ops.push(Op::SetFontSizeBuiltinFont {
                size: Pt(12.0),
                font: BuiltinFont::Helvetica,
            });
            ops.push(Op::WriteTextBuiltinFont {
                items: vec![
                    TextItem::Text(String::from("Event Date: ")),
                    TextItem::Text(self.invoice.event_date.clone()),
                ],
                font: BuiltinFont::Helvetica,
            });
            ops.push(Op::EndTextSection);
            y_pos -= 15.0;

            // Menu Items Header
            ops.push(Op::StartTextSection);
            ops.push(Op::SetTextCursor {
                pos: Point::new(Mm(20.0), Mm(y_pos)),
            });
            ops.push(Op::SetFontSizeBuiltinFont {
                size: Pt(14.0),
                font: BuiltinFont::HelveticaBold,
            });
            ops.push(Op::WriteTextBuiltinFont {
                items: vec![TextItem::Text(String::from("Menu"))],
                font: BuiltinFont::HelveticaBold,
            });
            ops.push(Op::EndTextSection);
            y_pos -= 10.0;

            // Menu Items
            for menu_item in &self.invoice.menu_items {
                // Description
                ops.push(Op::StartTextSection);
                ops.push(Op::SetTextCursor {
                    pos: Point::new(Mm(25.0), Mm(y_pos)),
                });
                ops.push(Op::SetFontSizeBuiltinFont {
                    size: Pt(10.0),
                    font: BuiltinFont::Helvetica,
                });
                ops.push(Op::WriteTextBuiltinFont {
                    items: vec![TextItem::Text(menu_item.desc.clone())],
                    font: BuiltinFont::Helvetica,
                });
                ops.push(Op::EndTextSection);
                y_pos -= 6.0;
            }
        }

        y_pos -= 5.0;

        // Pricing Breakdown
        ops.push(Op::StartTextSection);
        ops.push(Op::SetTextCursor {
            pos: Point::new(Mm(20.0), Mm(y_pos)),
        });
        ops.push(Op::SetFontSizeBuiltinFont {
            size: Pt(14.0),
            font: BuiltinFont::HelveticaBold,
        });
        ops.push(Op::WriteTextBuiltinFont {
            items: vec![TextItem::Text(String::from("Pricing Summary:"))],
            font: BuiltinFont::Helvetica,
        });
        ops.push(Op::EndTextSection);
        y_pos -= 8.0;

        ops.push(Op::StartTextSection);
        ops.push(Op::SetTextCursor {
            pos: Point::new(Mm(25.0), Mm(y_pos)),
        });
        ops.push(Op::SetFontSizeBuiltinFont {
            size: Pt(10.0),
            font: BuiltinFont::Helvetica,
        });
        ops.push(Op::WriteTextBuiltinFont {
            items: vec![TextItem::Text(format!(
                "Subtotal: {} people x {:.2}/person: ${:.2}",
                self.invoice.num_people,
                self.invoice.price_per_person,
                self.invoice.subtotal()
            ))],
            font: BuiltinFont::Helvetica,
        });
        ops.push(Op::EndTextSection);
        y_pos -= 6.0;

        ops.push(Op::StartTextSection);
        ops.push(Op::SetTextCursor {
            pos: Point::new(Mm(25.0), Mm(y_pos)),
        });
        ops.push(Op::SetFontSizeBuiltinFont {
            size: Pt(10.0),
            font: BuiltinFont::Helvetica,
        });
        ops.push(Op::WriteTextBuiltinFont {
            items: vec![TextItem::Text(format!(
                "Tax ({}): ${:.2}",
                self.invoice.tax_percentage.clone(),
                self.invoice.tax()
            ))],
            font: BuiltinFont::Helvetica,
        });
        ops.push(Op::EndTextSection);
        y_pos -= 6.0;

        ops.push(Op::StartTextSection);
        ops.push(Op::SetTextCursor {
            pos: Point::new(Mm(25.0), Mm(y_pos)),
        });
        ops.push(Op::SetFontSizeBuiltinFont {
            size: Pt(10.0),
            font: BuiltinFont::Helvetica,
        });
        ops.push(Op::WriteTextBuiltinFont {
            items: vec![TextItem::Text(format!(
                "Total: ${:.2}",
                self.invoice.total(),
            ))],
            font: BuiltinFont::Helvetica,
        });
        ops.push(Op::EndTextSection);

        // Finish operations
        ops.push(Op::RestoreGraphicsState);

        // Create page with operations
        let page = PdfPage::new(Mm(210.0), Mm(297.0), ops);

        // Save the PDF to the chosen location
        let bytes = doc
            .with_pages(vec![page])
            .save(&PdfSaveOptions::default(), &mut Vec::new());

        std::fs::write(file_path, bytes)?;

        Ok(())
    }
}

impl Application for App {
    const APP_ID: &str = APP_ID;

    type Message = Message;
    type Executor = cosmic::executor::Default;
    type Flags = ();

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<Action<Message>>) {
        let invoice = Invoice {
            desc: String::new(),
            menu_items: Vec::new(),
            next_id: 1,
            letterhead: String::new(),
            invoice_title: String::new(),
            phone_number: String::new(),
            event_date: String::new(),
            price_per_person: String::new(),
            num_people: 0,
            tax_percentage: "10%".to_owned(),
            total: 0.0,
            editing_item: None,
        };

        (App { core, invoice }, Task::none())
    }

    fn header_start(&self) -> Vec<Element<'_, Message>> {
        vec![text("Catering Invoice Generator").size(20).into()]
    }

    fn update(&mut self, message: Message) -> Task<Action<Message>> {
        match message {
            Message::UpdateDesc(desc) => self.invoice.desc = desc,
            Message::UpdateRate(rate) => self.invoice.price_per_person = rate,
            Message::UpdateTaxRate(tax_rate) => self.invoice.tax_percentage = tax_rate,
            Message::UpdatePhoneNumber(phone) => self.invoice.phone_number = phone,
            Message::UpdatePeopleCount(new_count) => {
                self.invoice.num_people = new_count.parse().unwrap_or(0)
            }
            Message::UpdateEventDate(date) => self.invoice.event_date = date,
            Message::AddMenuItem => {
                if !self.invoice.desc.is_empty() {
                    let menu_item = MenuItem {
                        id: self.invoice.next_id,
                        desc: self.invoice.desc.clone(),
                    };

                    self.invoice.menu_items.push(menu_item);
                    self.invoice.next_id += 1;

                    // Clear inputs
                    self.invoice.desc.clear();
                }
            }
            Message::DeleteMenuItem(id) => self.invoice.menu_items.retain(|item| item.id != id),
            Message::EditMenuItem(id) => {
                if let Some(menu_item) = self.invoice.menu_items.iter().find(|item| item.id == id) {
                    self.invoice.desc = menu_item.desc.clone();
                    self.invoice.editing_item = Some(id);
                }
            }
            Message::UpdateMenuItem(id) => {
                if let Some(menu_item) = self
                    .invoice
                    .menu_items
                    .iter_mut()
                    .find(|item| item.id == id)
                {
                    menu_item.desc = self.invoice.desc.clone();
                }

                // Clear the inputs and MenuItem being edited
                self.invoice.desc.clear();
                self.invoice.editing_item = None;
            }
            Message::CancelEdit => {
                todo!()
            }
            Message::UpdateLetterhead(val) => self.invoice.letterhead = val,
            Message::UpdateTitle(title) => self.invoice.invoice_title = title,
            Message::SaveInvoice => {
                // Open file dialog to choose save location
                let default_filename = format!(
                    "{}_{}.invoice",
                    self.invoice.invoice_title,
                    Local::now().format("%Y%m%d")
                );

                return cosmic::task::future(async move {
                    let path = rfd::AsyncFileDialog::new()
                        .set_file_name(&default_filename)
                        .add_filter("Invoice", &["invoice"])
                        .save_file()
                        .await
                        .map(|handle| handle.path().to_path_buf());
                    Action::App(Self::Message::SaveInvoiceTo(path))
                });
            }
            Message::SaveInvoiceTo(path) => {
                if let Some(path) = path {
                    let invoice = Invoice {
                        menu_items: self.invoice.menu_items.clone(),
                        next_id: self.invoice.next_id,
                        letterhead: self.invoice.letterhead.clone(),
                        invoice_title: self.invoice.invoice_title.clone(),
                        phone_number: self.invoice.phone_number.clone(),
                        event_date: self.invoice.event_date.clone(),
                        desc: String::new(),
                        price_per_person: self.invoice.price_per_person.clone(),
                        num_people: self.invoice.num_people,
                        tax_percentage: self.invoice.tax_percentage.clone(),
                        total: self.invoice.total,
                        editing_item: None,
                    };

                    match serde_json::to_string_pretty(&invoice) {
                        Ok(json) => {
                            if let Err(e) = fs::write(&path, json) {
                                eprintln!("Error saving invoice: {e}");
                            } else {
                                println!("Invoice saved to :{}", path.display());
                            }
                        }
                        Err(e) => eprintln!("Error serializing invoice: {e}"),
                    }
                }
            }
            Message::LoadInvoice => {
                return cosmic::task::future(async move {
                    let path = rfd::AsyncFileDialog::new()
                        .add_filter("Invoice", &["invoice"])
                        .pick_file()
                        .await
                        .map(|handle| handle.path().to_path_buf());
                    Action::App(Self::Message::LoadInvoiceFrom(path))
                });
            }
            Message::LoadInvoiceFrom(path) => {
                if let Some(path) = path {
                    match fs::read_to_string(&path) {
                        Ok(json) => match serde_json::from_str::<Invoice>(&json) {
                            Ok(invoice) => {
                                self.invoice = invoice;
                                println!("Invoice loaded from: {}", path.display());
                            }
                            Err(e) => eprintln!("Error parsing invoice file: {e}"),
                        },
                        Err(e) => eprintln!("Error reading invoice file: {e}"),
                    }
                }
            }
            Message::GeneratePDF => {
                // Open file dialog to choose save location
                let default_filename = format!(
                    "{}_{}.pdf",
                    self.invoice.invoice_title,
                    Local::now().format("%Y%m%d")
                );

                return cosmic::task::future(async move {
                    let path = rfd::AsyncFileDialog::new()
                        .set_file_name(&default_filename)
                        .add_filter("PDF", &["pdf"])
                        .save_file()
                        .await
                        .map(|handle| handle.path().to_path_buf());
                    Action::App(Self::Message::SavePDFTo(path))
                });
            }
            Message::SavePDFTo(path) => {
                if let Some(path) = path {
                    if let Err(e) = self.generate_pdf(&path) {
                        eprintln!("Error generating PDF: {e}");
                    } else {
                        println!("Invoice saved to: {}", path.display());
                    }
                }
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Self::Message> {
        // Setting section
        let settings = column()
            .push(text("Invoice Settings").size(18))
            .push(
                row()
                    .push(text("Letterhead:").width(Length::Fixed(120.0)))
                    .push(
                        text_input("Business Name", &self.invoice.letterhead)
                            .on_input(Message::UpdateLetterhead)
                            .width(Length::Fill),
                    )
                    .spacing(10),
            )
            .push(
                row()
                    .push(text("Business Phone:").width(Length::Fixed(120.0)))
                    .push(
                        text_input("", &self.invoice.phone_number)
                            .on_input(Message::UpdatePhoneNumber)
                            .width(Length::Fill),
                    )
                    .spacing(10),
            )
            .push(
                row()
                    .push(text("Client Name:").width(Length::Fixed(120.0)))
                    .push(
                        text_input("", &self.invoice.invoice_title)
                            .on_input(Message::UpdateTitle)
                            .width(Length::Fill),
                    )
                    .spacing(10),
            )
            .push(
                row()
                    .push(text("Event Date:").width(Length::Fixed(120.0)))
                    .push(
                        text_input("", &self.invoice.event_date)
                            .on_input(Message::UpdateEventDate)
                            .width(Length::Fill),
                    )
                    .spacing(10),
            )
            .push(
                row()
                    .push(text("Tax Rate:").width(Length::Fixed(120.0)))
                    .push(
                        text_input("10%", &self.invoice.tax_percentage)
                            .on_input(Message::UpdateTaxRate)
                            .width(Length::Fill),
                    )
                    .spacing(10),
            )
            .push(
                row()
                    .push(text("Price/Person").width(Length::Fixed(120.0)))
                    .push(
                        text_input("", &self.invoice.price_per_person)
                            .on_input(Message::UpdateRate)
                            .width(Length::Fill),
                    )
                    .spacing(10),
            )
            .push(
                row()
                    .push(text("Number of People:").width(Length::Fixed(120.0)))
                    .push(
                        text_input("0", self.invoice.num_people.to_string())
                            .on_input(Message::UpdatePeopleCount)
                            .width(Length::Fill),
                    )
                    .spacing(10),
            );

        // Input Section
        let mut input_section = column().push(text("Add Menu Item").size(18)).push(
            row()
                .push(text("Description:").width(Length::Fixed(120.0)))
                .push(
                    text_input("Prime Rib", &self.invoice.desc)
                        .on_input(Message::UpdateDesc)
                        .width(Length::Fill),
                )
                .spacing(10),
        );

        if let Some(editing_id) = self.invoice.editing_item {
            input_section = input_section.push(
                row()
                    .push(
                        button::standard("Update Menu Item")
                            .on_press(Message::UpdateMenuItem(editing_id)),
                    )
                    .push(button::standard("Cancel").on_press(Message::CancelEdit))
                    .spacing(10),
            );
        } else {
            input_section = input_section
                .push(button::standard("Add Menu Item").on_press(Message::AddMenuItem));
        }

        input_section = input_section.spacing(10);

        // Menu Items List
        let mut menu_items_list = column().push(text("Menu Items").size(18)).spacing(10);

        for menu_item in &self.invoice.menu_items {
            let item_desc = menu_item.desc.clone();

            menu_items_list = menu_items_list.push(
                row()
                    .push(text(item_desc).width(Length::Fill))
                    .push(button::standard("Edit").on_press(Message::EditMenuItem(menu_item.id)))
                    .push(
                        button::standard("Delete").on_press(Message::DeleteMenuItem(menu_item.id)),
                    )
                    .spacing(10)
                    .align_y(Alignment::Center),
            );
        }

        // Summary section
        let summary = column()
            .push(text("Summary").size(18))
            .push(text(format!(
                "Subtotal: {} people x ${}/plate: ${:.2}",
                self.invoice.num_people,
                self.invoice.price_per_person,
                self.invoice.subtotal()
            )))
            .push(text(format!(
                "Tax ({}): ${:.2}",
                self.invoice.tax_percentage,
                self.invoice.tax()
            )))
            .push(text(format!("Total: ${:.2}", self.invoice.total())))
            .spacing(10);

        // Actions
        let actions = column().push(
            row()
                .push(
                    button::standard("Generate PDF")
                        .on_press(Message::GeneratePDF)
                        .tooltip("Generate a PDF of the invoice."),
                )
                .push(
                    button::standard("Save Invoice")
                        .on_press(Message::SaveInvoice)
                        .tooltip("Save invoice to .invoice file."),
                )
                .push(
                    button::standard("Load Invoice")
                        .on_press(Message::LoadInvoice)
                        .tooltip("Load invoice from .invoice file."),
                )
                .spacing(10),
        );

        // Put it all together
        let content = column()
            .push(settings)
            .push(divider::horizontal::default())
            .push(input_section)
            .push(divider::horizontal::default())
            .push(menu_items_list)
            .push(divider::horizontal::default())
            .push(summary)
            .push(divider::horizontal::default())
            .push(actions)
            .spacing(20)
            .padding(20);

        container(scrollable(content))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
