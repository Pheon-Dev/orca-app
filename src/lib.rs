use leptos::{html::Input, leptos_dom::helpers::location_hash, *};
use storage::PaySerialized;
use uuid::Uuid;

mod storage;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Pays(pub Vec<Pay>);

const STORAGE_KEY: &str = "pays-leptos";

impl Pays {
    pub fn new(cx: Scope) -> Self {
        let starting_pay = if let Ok(Some(storage)) = window().local_storage() {
            storage
                .get_item(STORAGE_KEY)
                .ok()
                .flatten()
                .and_then(|value| serde_json::from_str::<Vec<PaySerialized>>(&value).ok())
                .map(|values| {
                    values
                        .into_iter()
                        .map(|stored| stored.into_pay(cx))
                        .collect()
                })
                .unwrap_or_default()
        } else {
            Vec::new()
        };
        Self(starting_pay)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn add(&mut self, pay: Pay) {
        self.0.push(pay);
    }

    pub fn remove(&mut self, id: Uuid) {
        self.0.retain(|pay| pay.id != id);
    }

    pub fn balance(&self) -> usize {
        self.0.iter().filter(|pay| !pay.paid.get()).count()
    }

    pub fn paid(&self) -> usize {
        self.0.iter().filter(|pay| pay.paid.get()).count()
    }

    pub fn toggle_all(&self) {
        if self.balance() == 0 {
            for pay in &self.0 {
                pay.paid.update(|paid| {
                    if *paid {
                        *paid = false
                    }
                });
            }
        } else {
            for pay in &self.0 {
                pay.paid.set(true);
            }
        }
    }

    fn clear_paid(&mut self) {
        self.retain(|pay| !pay.paid.get());
    }

    fn retain(&mut self, mut f: impl FnMut(&Pay) -> bool) {
        self.0.retain(|pay| {
            let retain = f(pay);

            if !retain {
                pay.amount.dispose();
                pay.sender.dispose();
                pay.currency.dispose();
                pay.receiver.dispose();
                pay.paid.dispose();
            }

            retain
        })
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Pay {
    pub id: Uuid,
    pub amount: RwSignal<f64>,
    pub receiver: RwSignal<String>,
    pub currency: RwSignal<String>,
    pub sender: RwSignal<String>,
    pub paid: RwSignal<bool>,
}

impl Pay {
    pub fn new(
        cx: Scope,
        id: Uuid,
        amount: f64,
        receiver: String,
        currency: String,
        sender: String,
    ) -> Self {
        Self::new_with_paid(cx, id, amount, receiver, currency, sender, false)
    }

    pub fn new_with_paid(
        cx: Scope,
        id: Uuid,
        amount: f64,
        receiver: String,
        currency: String,
        sender: String,
        paid: bool,
    ) -> Self {
        let amount = create_rw_signal(cx, amount);
        let receiver = create_rw_signal(cx, receiver);
        let currency = create_rw_signal(cx, currency);
        let sender = create_rw_signal(cx, sender);
        let paid = create_rw_signal(cx, paid);

        Self {
            id,
            amount,
            receiver,
            currency,
            sender,
            paid,
        }
    }
    pub fn toggle(&self) {
        self.paid.update(|paid| *paid = !*paid);
    }
}

const ESCAPE_KEY: u32 = 27;
const ENTER_KEY: u32 = 13;

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    let (pays, set_pays) = create_signal(cx, Pays::new(cx));

    provide_context(cx, set_pays);

    let (mode, set_mode) = create_signal(cx, Mode::All);

    window_event_listener(ev::hashchange, move |_| {
        let new_mode = location_hash().map(|hash| route(&hash)).unwrap_or_default();
        set_mode(new_mode);
    });

    let input_ref = create_node_ref::<Input>(cx);
    let add_pay = move |ev: web_sys::KeyboardEvent| {
        let input = input_ref.get().unwrap();
        ev.stop_propagation();
        let key_code = ev.key_code();
        if key_code == ENTER_KEY {
            let amount = input.value();
            let amount = amount.trim();
            let sender = input.value();
            let sender = sender.trim();
            let receiver = input.value();
            let receiver = receiver.trim();
            let currency = input.value();
            let currency = currency.trim();

            if !amount.is_empty() {
                let new = Pay::new(
                    cx,
                    Uuid::new_v4(),
                    amount.parse::<f64>().unwrap(),
                    receiver.to_string(),
                    sender.to_string(),
                    currency.to_string(),
                );
                set_pays.update(|p| p.add(new));
                input.set_value("");
            }
        }
    };

    let filtered_pays = move || {
        pays.with(|pays| match mode.get() {
            Mode::All => pays.0.to_vec(),
            Mode::Active => pays
                .0
                .iter()
                .filter(|pay| !pay.paid.get())
                .cloned()
                .collect(),
            Mode::Paid => pays
                .0
                .iter()
                .filter(|pay| pay.paid.get())
                .cloned()
                .collect(),
        })
    };

    create_effect(cx, move |_| {
        if let Ok(Some(storage)) = window().local_storage() {
            let objs = pays
                .get()
                .0
                .iter()
                .map(PaySerialized::from)
                .collect::<Vec<_>>();
            let json = serde_json::to_string(&objs).expect("couldn't serialize pays");
            if storage.set_item(STORAGE_KEY, &json).is_err() {
                log::error!("error while trying to set item in local storage");
            }
        }
    });

    create_effect(cx, move |_| {
        if let Some(input) = input_ref.get() {
            request_animation_frame(move || {
                let _ = input.focus();
            });
        }
    });

    view! { cx,
        <main>
            <section class="orca-app">
                <header class="header">
                    <h1>"Orca App"</h1>
                    <input
                        class="new-pay"
                        placeholder="What amount is to be sent?"
                        autofocus
                        on:keydown=add_pay
                        node_ref=input_ref
                    />
                </header>
                <section
                    class="main"
                    class:hidden={move || pays.with(|p| p.is_empty())}
                >
                    <input id="toggle-all" class="toggle-all" type="checkbox"
                        prop:checked={move || pays.with(|p| p.balance() > 0)}
                        on:input=move |_| pays.with(|p| p.toggle_all())
                    />
                    <label for="toggle-all">"Mark all as paid"</label>
                    <ul class="pay-list">
                        <For
                            each=filtered_pays
                            key=|pay| pay.id
                            view=move |cx, pay: Pay| view! { cx, <Pay pay /> }
                        />
                    </ul>
                </section>
                <footer
                    class="footer"
                    class:hidden={move || pays.with(|p| p.is_empty())}
                >
                    <span class="pay-count">
                        <strong>{move || pays.with(|p| p.balance().to_string())}</strong>
                        {move || if pays.with(|p| p.balance()) == 1 {
                            " item"
                        } else {
                            " items"
                        }}
                        " left"
                    </span>
                    <ul class="filters">
                        <li><a href="#/" class="selected" class:selected={move || mode() == Mode::All}>"All"</a></li>
                        <li><a href="#/active" class:selected={move || mode() == Mode::Active}>"Active"</a></li>
                        <li><a href="#/paid" class:selected={move || mode() == Mode::Paid}>"Paid"</a></li>
                    </ul>
                    <button
                        class="clear-paid hidden"
                        class:hidden={move || pays.with(|p| p.paid() == 0)}
                        on:click=move |_| set_pays.update(|p| p.clear_paid())
                    >
                        "Clear Paid"
                    </button>
                </footer>
            </section>
            <footer class="info">
                <p>"Double-click to edit a pay"</p>
            </footer>
        </main>
    }
}

#[component]
pub fn Pay(cx: Scope, pay: Pay) -> impl IntoView {
    let (paying, set_paying) = create_signal(cx, false);
    let set_pays = use_context::<WriteSignal<Pays>>(cx).unwrap();

    let pay_input = create_node_ref::<Input>(cx);

    let save = move |value: &str| {
        let value = value.trim();
        if value.is_empty() {
            set_pays.update(|t| t.remove(pay.id));
        } else {
            pay.amount.set(value.parse::<f64>().unwrap());
            pay.sender.set(value.to_string());
            pay.receiver.set(value.to_string());
            pay.currency.set(value.to_string());
        }
        set_paying(false);
    };

    view! { cx,
        <li
            class="pay"
            class:paying={paying}
            class:paid={move || pay.paid.get()}
        >
            <div class="view">
                <input
                    node_ref=pay_input
                    class="toggle"
                    type="checkbox"
                    prop:checked={move || (pay.paid)()}
                    on:input={move |ev| {
                        let checked = event_target_checked(&ev);
                        pay.paid.set(checked);
                    }}
                />
                <label on:dblclick=move |_| {
                        set_paying(true);

                        if let Some(input) = pay_input.get() {
                        _ = input.focus();
                    }
                }>
                    {move || pay.currency.get()}
                    {move || pay.amount.get()}
                    {move || pay.receiver.get()}
                    {move || pay.paid.get()}
                </label>
                <button class="cancel" on:click=move |_| set_pays.update(|p| p.remove(pay.id)) />
            </div>
            {move || paying().then(|| view! { cx,
                <input
                    class="edit"
                    class:hidden={move || !(paying)()}
                    prop:value={move || pay.sender.get()}
                    on:focusout=move |ev: web_sys::FocusEvent| save(&event_target_value(&ev))
                    on:keyup={move |ev: web_sys::KeyboardEvent| {
                        let key_code = ev.key_code();
                        if key_code == ENTER_KEY {
                            save(&event_target_value(&ev));
                        } else if key_code == ESCAPE_KEY {
                            set_paying(false);
                        }
                    }}
                />
            })
            }
        </li>
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Active,
    Paid,
    #[default]
    All,
}

pub fn route(hash: &str) -> Mode {
    match hash {
        "/active" => Mode::Active,
        "/paid" => Mode::Paid,
        _ => Mode::All,
    }
}
