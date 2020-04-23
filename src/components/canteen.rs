use anyhow::{Error, Result};
use gtk::prelude::*;
use gtk::{Box, Builder, Spinner, Label, Stack};
use ovgu_canteen::{Canteen, CanteenDescription};

use crate::components::{get, glib_yield, DayComponent, WindowComponent, GLADE};
use crate::util::{enclose, AdjustingVec};

#[derive(Debug)]
pub struct CanteenComponent {
    canteen_stack: Stack,
    canteen_error_label: Label,
    canteen_spinner: Spinner,
    days: AdjustingVec<DayComponent, Error>,
}

impl CanteenComponent {
    pub fn new(description: &CanteenDescription, window: &WindowComponent) -> Result<Self> {
        let builder = Builder::new_from_string(GLADE);
        let canteen_stack: Stack = get!(&builder, "canteen-stack")?;
        let canteen_error_label: Label = get!(&builder, "canteen-error-label")?;
        let canteen_spinner: Spinner = get!(&builder, "canteen-spinner")?;
        let days_box: Box = get!(&builder, "days-box")?;
        let canteen_name = description.to_german_str();

        window.add_canteen(&canteen_stack, canteen_name)?;

        let days = AdjustingVec::new(
            enclose! { (days_box) move || {
                enclose! { (days_box) async move {
                    let comp = DayComponent::new().await?;
                    days_box.pack_start(comp.root_widget(), false, true, 0);

                    glib_yield!();
                    Ok(comp)
                }}
            }},
            |day| async move {
                day.root_widget().destroy();
                glib_yield!();
                Ok(())
            },
        );

        Ok(Self {
            canteen_stack,
            canteen_error_label,
            canteen_spinner,
            days,
        })
    }

    pub async fn load(&mut self, load_result: Result<Canteen, Error>) {
        self.canteen_spinner.start();
        self.canteen_spinner.show();

        let canteen = match load_result {
            Ok(canteen) => {
                self.canteen_stack.set_visible_child_name("canteen-menu");
                canteen
            },
            Err(e) => {
                self.canteen_stack.set_visible_child_name("canteen-error");
                self.canteen_error_label.set_text(&format!("error: {:#}", e));
                return;
            }
        };

        let days_result = self
            .days
            .adjust(&canteen.days, |mut comp, day| async move {
                comp.load(day).await;
                glib_yield!();
                Ok(comp)
            })
            .await;

        if let Err(e) = days_result {
            self.canteen_stack.set_visible_child_name("canteen-error");
            self.canteen_error_label.set_text(&format!("error: {:#}", e));
        } else {
            self.canteen_stack.set_visible_child_name("canteen-menu");
        }

        self.canteen_spinner.stop();
        self.canteen_spinner.hide();
    }
}
