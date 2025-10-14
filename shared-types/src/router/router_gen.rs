use super::{ObserverImpl, WireResponseSender};
use crate::context::Context;
use crate::*;
use serde::{Deserialize, Serialize};

pub trait CallHandler {
    fn find_shortest_path(
        &self,
        ctx: &Context,
        params: ShortestPathParams,
        tx: ObserverImpl<PathResult>,
    );
}

#[allow(non_camel_case_types)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum CallGen {
    find_shortest_path(ShortestPathParams),
}

#[allow(non_camel_case_types)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ResponseNextGen {
    find_shortest_path(PathResult),
}

pub(crate) fn gen_call(
    ctx: &Context,
    id: usize,
    call: CallGen,
    handler: &dyn CallHandler,
    sender: Box<dyn WireResponseSender>,
) {
    match call {
        CallGen::find_shortest_path(params) => handler.find_shortest_path(
            ctx,
            params,
            ObserverImpl::new(id, sender),
        ),
    }
}
