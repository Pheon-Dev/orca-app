use crate::Pay;
use leptos::{signal_prelude::*, Scope};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct PaySerialized {
    pub id: Uuid,
    pub amount: f64,
    pub receiver: String,
    pub currency: String,
    pub sender: String,
    pub paid: bool,
}

impl PaySerialized {
    pub fn into_pay(self, cx: Scope) -> Pay {
        Pay::new_with_paid(
            cx,
            self.id,
            self.amount,
            self.receiver,
            self.currency,
            self.sender,
            self.paid,
        )
    }
}

impl From<&Pay> for PaySerialized {
    fn from(pay: &Pay) -> Self {
        Self {
            id: pay.id,
            amount: pay.amount.get(),
            receiver: pay.receiver.get(),
            currency: pay.currency.get(),
            sender: pay.sender.get(),
            paid: pay.paid.get(),
        }
    }
}
