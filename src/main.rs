use futures_util::stream::StreamExt;
use swayipc_async::{Connection, EventType, Event, Fallible};


mod nodes;

#[tokio::main]
async fn main() -> Fallible<()> {

    // Only Workspace event is interesting
    let subs = [EventType::Workspace];

    let mut connection = Connection::new().await?;
    let mut events = connection.subscribe(&subs).await?;
    while let Some(event) = events.next().await {
        let nodes = match event? {
            Event::Workspace(ev) => ev.current.unwrap().nodes,
            _ => unreachable!("Unsubscribed events unreachable"),
        };

        for n in nodes.iter() {
            let sn = nodes::SwayNodes::from(n).flatten();
            dbg!(sn);
        }
    }

    Ok(())
}
