use std::io::Write;
use anyhow::Context;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Message<Payload> {
    pub src: String,
    pub dest: String,
    pub body: Body<Payload>,
}

impl Message<Payload> {
    pub fn reply(&self, id: Option<usize>) -> Self {
        return Message {
            src: self.dest.clone(),
            dest: self.src.clone(),
            body: Body {
                msg_id: id,
                in_reply_to: self.body.msg_id,
                payload: self.body.payload.clone(),
            },
        };
    }

    pub fn send(self, out: &mut std::io::Stdout) -> anyhow::Result<()> {
        let mut out_lock = out.lock();
        serde_json::to_writer(&mut out_lock, &self).context("failed to write to stdout")?;
        out_lock.write_all(b"\n")?;

        return Ok(());
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Body<Payload> {
    pub msg_id: Option<usize>,      // a unique integer identifier
    pub in_reply_to: Option<usize>, // for req/response, the msgId of the request
    #[serde(flatten)]
    pub payload: Payload,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum Payload {
    Init {
        node_id: String,
        node_ids: Vec<String>,
    },
    InitOk,
    Echo {
        echo: String,
    },
    EchoOk {
        echo: String,
    },
}

#[derive(Debug)]
pub struct Node {
    pub node_id: String,
    pub node_ids: Vec<String>,
}

impl Node {
    pub fn new(msg: &Message<Payload>) -> Self {
        if let Payload::Init { node_id, node_ids } = &msg.body.payload {
            return Self {
                node_id: node_id.clone(),
                node_ids: node_ids.clone(),
            };
        }

        panic!("Payload is not of type init");
    }

    pub fn handle(&self, msg: &Message<Payload>, mut out: std::io::Stdout) -> anyhow::Result<()> {
        let mut reply = msg.reply(Some(0));
        match reply.body.payload {
            Payload::Echo { echo }=> {
                reply.body.payload = Payload::EchoOk { echo };
                reply.send(&mut out)?;
            },
            _ => println!("unknown msg type {:?}", msg.body.payload),
        }
        Ok(())
    }
}
