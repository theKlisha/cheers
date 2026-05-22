#![allow(unused)]

use std::marker::PhantomData;
use std::sync::mpsc::{Receiver, Sender};

use crate::uci::{UciEngine, UciHost, UciRequest, UciResponse, connect};

pub mod board;
pub mod uci;

pub trait Board {}
pub trait Search {}
pub trait Eval {}

#[derive(Default)]
pub struct Nil;

impl Board for Nil {}
impl Search for Nil {}
impl Eval for Nil {}

#[derive(Default)]
pub struct Engine<B, S, E>
where
    B: Board,
    S: Search,
    E: Eval,
{
    _board: PhantomData<B>,
    _search: PhantomData<S>,
    _eval: PhantomData<E>,
}

impl<B, S, E> Engine<B, S, E>
where
    B: Board,
    S: Search,
    E: Eval,
{
}

impl<B, S, E> UciEngine for Engine<B, S, E>
where
    B: Board + Send + 'static,
    S: Search + Send + 'static,
    E: Eval + Send + 'static,
{
    fn start(self) -> (Sender<UciRequest>, Receiver<UciResponse>) {
        todo!()
    }
}

pub struct StdioHost;

impl UciHost for StdioHost {
    fn start(self) -> (Sender<UciResponse>, Receiver<UciRequest>) {
        todo!()
    }
}

fn main() {
    let engine = Engine::<Nil, Nil, Nil>::default();
    let host = StdioHost;

    connect(host, engine);
}
