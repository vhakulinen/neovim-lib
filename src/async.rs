use std::marker::PhantomData;

use rmpv::Value;

use session::ClientConnection;
use neovim;
use rpc::model::FromVal;

pub struct AsyncCall<'a, R: FromVal<Value>> {
    method: String,
    args: Vec<Value>,
    client: &'a mut ClientConnection,
    cb: Option<Box<FnMut(Result<Value, Value>) + Send + 'static>>,
    marker: PhantomData<R>,
}

impl<'a, R: FromVal<Value>> AsyncCall<'a, R> {
    pub fn new(client: &'a mut ClientConnection, method: String, args: Vec<Value>) -> Self {
        AsyncCall {
            method,
            args,
            client,
            cb: None,
            marker: PhantomData,
        }
    }

    pub fn cb<F>(mut self, cb: F) -> Self
    where
        F: FnOnce(Result<R, neovim::CallError>) + Send + 'static,
    {
        let mut cb = Some(cb);

        self.cb = Some(Box::new(move |res| {
            let res = res.map(|v| R::from_val(v)).map_err(
                neovim::map_generic_error,
            );
            cb.take().unwrap()(res);
        }));
        self
    }

    /// Async call. Call can be made only after event loop begin processing
    pub fn call(self) {
        match self.client {
            &mut ClientConnection::Child(ref mut client, _) => {
                client.call_async(self.method, self.args, self.cb)
            }
            &mut ClientConnection::Parent(ref mut client) => {
                client.call_async(self.method, self.args, self.cb)
            }
            &mut ClientConnection::Tcp(ref mut client) => {
                client.call_async(self.method, self.args, self.cb)
            }

            #[cfg(unix)]
            &mut ClientConnection::UnixSocket(ref mut client) => {
                client.call_async(self.method, self.args, self.cb)
            }
        };
    }
}
