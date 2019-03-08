use std::thread::*;
use std::io::{Error, ErrorKind};
use std::sync::{Arc, Weak, Mutex};
use std::sync::mpsc::{SyncSender, Receiver, sync_channel};

use mirror::Remote;

pub struct Instance<R: Remote + Send + 'static> {
    control: Weak<usize>,
    sender: SyncSender<R>,
}

pub struct InstanceContainer<R: Remote + Send + 'static> {
    instances: Vec<Instance<R>>,
}

impl<R: Remote + Send + 'static> InstanceContainer<R> {
    pub fn new() -> Self {
        Self {
            instances: Vec::new(),
        }
    }

    pub fn create<F>(&mut self, f: F, c: Arc<Mutex<Self>>) -> usize where
        F: FnOnce(Receiver<R>, Arc<Mutex<Self>>) + Send + 'static
    {
        let id = self.instances
            .iter()
            .enumerate()
            .filter_map(|(j, i)| if i.control.upgrade().is_none() { Some(j) } else { None })
            .next()
            .unwrap_or(self.instances.len());

        let (tx, rx) = sync_channel(8);
        let alive = Arc::new(0);
        let control = Arc::downgrade(&alive);
        spawn(move || {
            f(rx, c);
            let _ = *alive;
        });
        let i = Instance {
            control,
            sender: tx,
        };

        if id == self.instances.len() {
            self.instances.push(i);
        } else {
            self.instances[id] = i;
        }

        id
    }

    pub fn submit(&mut self, instance: usize, remote: R) -> ::std::io::Result<()> {
        if let Some(inst) = self.instances.get_mut(instance) {
            inst.sender
                .send(remote)
                .map_err(|_e| Error::from(ErrorKind::BrokenPipe))?;
            Ok(())
        } else {
            Err(Error::from(ErrorKind::NotFound))
        }
    }
}
