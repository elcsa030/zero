// Copyright 2022 Risc0, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

mod board;
mod bus;
mod contract;
mod ffi;
mod game;
mod journal;
mod layout;
mod lobby;
mod near;
// mod mock;

use std::rc::Rc;

use bus::EventBus;
// use mock::Mock;
use near::{NearContract, NearWallet};
use yew::prelude::*;
use yew_agent::{Dispatched, Dispatcher};
use yew_router::prelude::*;

use crate::{game::GameProvider, journal::Journal, layout::Layout, lobby::Lobby};

#[derive(Debug, Clone, PartialEq, Routable)]
enum Route {
    #[at("/")]
    Lobby,
    #[at("/new/:name")]
    NewGame { name: String },
    #[at("/join/:name")]
    JoinGame { name: String },
    #[not_found]
    #[at("/404")]
    NotFound,
}

enum Msg {
    SignIn,
    SignOut,
}

struct App {
    // contract: Rc<Mock>,
    event_bus: Dispatcher<EventBus>,
    wallet: Rc<NearWallet>,
    contract: Option<Rc<NearContract>>,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let wallet = Rc::new(NearWallet::new().unwrap());
        let user = wallet.current_user().unwrap();
        let contract = if user.is_empty() {
            None
        } else {
            Some(Rc::new(wallet.get_contract().unwrap()))
        };

        Self {
            event_bus: EventBus::dispatcher(),
            wallet,
            contract,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SignIn => {
                self.event_bus.send("App::SignIn".into());
                self.wallet.sign_in().unwrap();
                self.contract = Some(Rc::new(self.wallet.get_contract().unwrap()));
                true
            }
            Msg::SignOut => {
                self.event_bus.send("App::SignOut".into());
                self.wallet.sign_out().unwrap();
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let user = self.wallet.current_user().unwrap();
        let has_user = !user.is_empty();
        html! {
            <BrowserRouter>
                <div class="container">
                    <nav class="navbar navbar-expand-lg navbar-light bg-light">
                        <div class="container-fluid">
                            <a class="navbar-brand" href="https://risczero.com">
                                {"RISC Zero"}
                            </a>
                            <div class="collapse navbar-collapse">
                                <div class="navbar-nav">
                                    <Link<Route> to={Route::Lobby}>
                                        {"Lobby"}
                                    </Link<Route>>
                                </div>
                            </div>
                            <div class="d-flex">
                                if !has_user {
                                    <button
                                        class="button is-primary"
                                        onclick={ctx.link().callback(|_| Msg::SignIn)}>
                                        {"Sign In"}
                                    </button>
                                } else {
                                    <span class="navbar-text col-sm-6">
                                        {"Account: "} {user}
                                    </span>
                                    <button
                                        class="btn btn-primary"
                                        onclick={ctx.link().callback(|_| Msg::SignOut)}>
                                        {"Sign Out"}
                                    </button>
                                }
                            </div>
                        </div>
                    </nav>
                    if !has_user {
                        <p>{"Please Sign In to start."}</p>
                    } else {
                        {self.view_main()}
                    }
                    <hr/>
                    <Journal />
                    <footer class="py-3 my-4 border-top">
                        <p class="text-center text-muted">{"Battleship!"}</p>
                    </footer>
                </div>
            </BrowserRouter>
        }
    }
}

impl App {
    fn view_main(&self) -> Html {
        let contract = self.contract.as_ref().unwrap().clone();
        let render = Switch::render(move |routes| switch(routes, contract.clone()));
        html! {
            <Switch<Route> {render} />
        }
    }
}

fn switch(routes: &Route, contract: Rc<NearContract>) -> Html {
    match routes.clone() {
        Route::Lobby => html! { <Lobby {contract} /> },
        Route::NewGame { name } => html! {
            <GameProvider {name} {contract} until={1}>
                <Layout />
            </GameProvider>
        },
        Route::JoinGame { name } => html! {
            <GameProvider {name} {contract} until={2}>
                <Layout />
            </GameProvider>
        },
        Route::NotFound => html! { <h1>{ "404" }</h1> },
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<App>();
}
