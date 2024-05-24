use std::collections::HashMap;
use std::hash::Hash;
use std::time::Duration;
use tokio::select;
use tokio::sync::oneshot::{channel, Sender};
use tokio::task::JoinHandle;
use tokio::time::sleep;

#[derive(Debug)]
enum Error {
    Timeout,
    Dropped,
}

#[derive(Debug)]
enum Command<K, V> {
    Wait(K, Duration, Sender<Result<V, Error>>),
    Ready(K, V),
    Remove(K),
    Check(K, Sender<bool>),
}

pub struct Waiter<K, V>
where
    K: Eq + Hash,
{
    command_tx: tokio::sync::mpsc::UnboundedSender<Command<K, V>>,
    handle: JoinHandle<()>,
}

impl<K, V> Drop for Waiter<K, V>
where
    K: Eq + Hash,
{
    fn drop(&mut self) {
        self.handle.abort();
    }
}

struct Context<K, V> {
    remember: HashMap<K, Sender<Result<V, Error>>>,
    command_rx: tokio::sync::mpsc::UnboundedReceiver<Command<K, V>>,
    expire_tx: tokio::sync::mpsc::UnboundedSender<K>,
    expire_rx: tokio::sync::mpsc::UnboundedReceiver<K>,
}

impl<K, V> Context<K, V>
where
    K: Eq + Hash + Send + Sync + Clone + 'static,
{
    fn new(command_rx: tokio::sync::mpsc::UnboundedReceiver<Command<K, V>>) -> Self {
        let (expire_tx, expire_rx) = tokio::sync::mpsc::unbounded_channel::<K>();
        Context {
            remember: HashMap::new(),
            command_rx,
            expire_tx,
            expire_rx,
        }
    }

    async fn process_loop(&mut self) {
        loop {
            select! {
                key_opt = self.expire_rx.recv() => {
                    let Some(key) = key_opt else {
                        return;
                    };
                    self.remember.remove(&key);
                }
                cmd_opt = self.command_rx.recv() => {
                    let Some(cmd) = cmd_opt else {
                        return;
                    };
                    self.handle_command(cmd).await;
                }
            }
        }
    }

    async fn handle_command(&mut self, cmd: Command<K, V>) {
        match cmd {
            Command::Wait(key, timeout, tx) => {
                self.remember.insert(key.clone(), tx);
                let shared_expire_tx = self.expire_tx.clone();
                tokio::spawn(async move {
                    sleep(timeout).await;
                    let _ = shared_expire_tx.send(key);
                });
            }
            Command::Ready(key, value) => {
                if let Some(tx) = self.remember.remove(&key) {
                    let _ = tx.send(Ok(value)); // ignore error, receiver may be dropped
                }
            }
            Command::Remove(key) => {
                self.remember.remove(&key);
            }
            Command::Check(key, tx) => {
                let result = self.remember.contains_key(&key);
                let _ = tx.send(result); // ignore error, receiver may be dropped
            }
        }
    }
}

impl<K, V> Waiter<K, V>
where
    K: Eq + Hash + Send + Sync + Clone + 'static,
    V: Send + 'static,
{
    fn new() -> Self {
        let (command_tx, command_rx) = tokio::sync::mpsc::unbounded_channel();
        let handle = tokio::spawn(async move {
            let mut ctx: Context<K, V> = Context::new(command_rx);
            ctx.process_loop().await
        });
        Waiter { command_tx, handle }
    }

    async fn wait(&self, key: K, timeout: Duration) -> Result<V, Error> {
        let (tx, rx) = channel::<Result<V, Error>>();
        let cmd = Command::Wait(key, timeout, tx);
        if let Err(_) = self.command_tx.send(cmd) {
            return Err(Error::Dropped);
        }
        // ignore error, receiver may be dropped
        select! {
            _ = sleep(timeout) => {
                Err(Error::Timeout)
            }
            res = rx => {
                res.unwrap_or(Err(Error::Timeout))
            }
        }
    }

    async fn wake(&self, key: K, data: V) {
        let cmd = Command::Ready(key, data);
        let _ = self.command_tx.send(cmd); // ignore error, receiver may be dropped
    }

    /// Check if the key is in the waiter
    async fn exists(&self, key: K) -> bool {
        let (tx, rx) = channel::<bool>();
        if let Err(_) = self.command_tx.send(Command::Check(key, tx)) {
            // ignore error, receiver may be dropped
            return false;
        }
        match rx.await {
            Ok(res) => res,
            Err(_) => false,
        }
    }

    /// Remove a key from the waiter
    async fn remove(&self, key: K) {
        let cmd = Command::Remove(key);
        let _ = self.command_tx.send(cmd); // ignore error, receiver may be dropped
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::time::Duration;
    use futures_util::future::{join, join_all};
    use rand::distributions::Alphanumeric;
    use rand::Rng;
    use tokio::select;
    use tokio::sync::oneshot::{channel, Sender};
    use tokio::time::sleep;
    use crate::commons::waiter::Waiter;

    #[tokio::test]
    async fn one_shot() {
        let (tx, rx) = channel::<i32>();
        let f1 = async {
            sleep(Duration::from_secs(1)).await;
            tx.send(155558).unwrap();
        };
        let f2 = async {
            let res = rx.await.unwrap();
            assert_eq!(res, 155558);
        };
        join(f2, f1).await;
    }

    #[tokio::test]
    async fn remember() {
        let mut receivers: HashMap<String, Sender<i32>> = HashMap::new();
        let (tx, rx) = channel::<i32>();
        receivers.insert("1".to_string(), tx);
        let f1 = async {
            sleep(Duration::from_secs(1)).await;
            receivers.remove("1").unwrap().send(155558).unwrap();
        };
        let f2 = async {
            let res = rx.await.unwrap();
            assert_eq!(res, 155558);
        };
        join(f2, f1).await;
    }

    fn key() -> String {
        // random string
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect()
    }

    struct Resp {
        data: i32,
    }

    fn value() -> Resp {
        Resp { data: 155558 }
    }

    #[tokio::test]
    async fn waiter_once() {
        let waiter = Waiter::<String, Resp>::new();
        let key = key();
        let wait = async {
            let res = waiter.wait(key.clone(), Duration::from_secs(2)).await;
            assert_eq!(res.unwrap().data, 155558);
        };
        let shot = async {
            sleep(Duration::from_secs(1)).await;
            let value = value();
            waiter.wake(key.clone(), value).await;
        };
        join(shot, wait).await;
    }

    #[tokio::test]
    async fn waiter_many() {
        let waiter = Arc::new(Waiter::<String, String>::new());
        let waiter_ref = &waiter;
        let keys = (0..100)
            .map(|_| {
                rand::thread_rng()
                    .sample_iter(&Alphanumeric)
                    .take(7)
                    .collect::<Vec<u8>>()
            })
            .map(String::from_utf8)
            .filter_map(Result::ok)
            .collect::<Vec<_>>();
        let waits = keys
            .iter()
            .map(|key| async move {
                let res = waiter_ref
                    .clone()
                    .wait(key.clone(), Duration::from_secs(2))
                    .await;
                assert_eq!(res.unwrap(), key.clone());
            })
            .collect::<Vec<_>>();
        let shots = keys
            .iter()
            .rev()
            .into_iter()
            .map(|key| async move {
                sleep(Duration::from_millis(100)).await;
                waiter_ref.clone().wake(key.clone(), key.clone()).await;
            })
            .collect::<Vec<_>>();
        join(join_all(waits), join_all(shots)).await;
    }

    #[tokio::test]
    async fn waiter_drop_wait_early() {
        let waiter = Waiter::<String, Resp>::new();
        let key = key();
        select! {
            _ = waiter.wait(key.clone(), Duration::from_millis(200)) => {
                panic!("should not return normally");
            },
            _ = sleep(Duration::from_millis(100)) => {
                println!("timeout");
            }
        }
        // todo: need another drop mechanism to drop dead waiter
        sleep(Duration::from_millis(300)).await;
        assert!(!waiter.exists(key).await, "key should be dropped");
    }

    #[tokio::test]
    async fn waiter_remove_key() {
        let waiter = Waiter::<String, Resp>::new();
        let key = key();
        let wait = async {
            let res = waiter.wait(key.clone(), Duration::from_secs(2)).await;
            assert!(res.is_err(), "should return error");
        };
        let remove = async {
            sleep(Duration::from_millis(100)).await;
            waiter.remove(key.clone()).await;
        };
        join(wait, remove).await;
        assert!(!waiter.exists(key).await, "key should be removed");
    }
}