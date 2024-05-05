use anyhow::Context;
use icebreaker::*;
use std::io::BufRead;

fn main() -> anyhow::Result<()> {
    let mut lines = std::io::stdin().lines();
    let first_line = lines
        .next()
        .expect("no message received")
        .context("failed to read message from stdin")?;
    drop(lines);

    let init_msg: Message<Payload> = serde_json::from_str(&first_line)?;

    let node = Node::new(&init_msg);

    let mut init_reply = init_msg.reply(Some(0));
    init_reply.body.payload = Payload::InitOk;

    init_reply
        .send(&mut std::io::stdout())
        .context("failed to send reply")?;

    let (tx, rx) = std::sync::mpsc::channel();

    let join_handle = std::thread::spawn(move || {
        let stdin_lock = std::io::stdin().lock();
        for line in stdin_lock.lines() {
            let line = line.context("failed to read line from stdin")?;
            let message: Message<Payload> =
                serde_json::from_str(&line).context("failed to deserialize input")?;

            let _  = tx.send(message);
        }

        return Ok::<_, anyhow::Error>(());
    });

    for message in rx {
        node.handle(&message, std::io::stdout()).context("message handling by node failed")?;
    }

    join_handle.join().expect("thread failed").context("thread failed")?;

    Ok(())
}
