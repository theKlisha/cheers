#![allow(unused)]

use std::{io, marker::PhantomData};

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
    B: Board,
    S: Search,
    E: Eval,
{
    fn read(&self) -> Option<UciResponse> {
        todo!()
    }

    fn write(&self, request: UciRequest) -> io::Result<()> {
        todo!()
    }
}

pub struct StdioHost;

impl UciHost for StdioHost {
    fn read(&self) -> Option<UciRequest> {
        todo!()
    }

    fn write(&self, response: UciResponse) -> io::Result<()> {
        todo!()
    }
}

fn main() {
    let engine = Engine::<Nil, Nil, Nil>::default();
    let host = StdioHost;

    connect(host, engine);
}
