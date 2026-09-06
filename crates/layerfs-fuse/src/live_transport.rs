use crate::live_runtime::{LiveRuntime, Scheduler};
use crate::live_wire::{invalid, MAX_FRAME};
use crate::{PortError, PortResult};
use std::io;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{watch, Mutex, Semaphore};

pub struct BackingServer {
    port: u16,
    capability: [u8; 32],
    stop: watch::Sender<bool>,
    listener: tokio::task::JoinHandle<()>,
    failed: Arc<AtomicBool>,
    control: Arc<Mutex<Option<TcpStream>>>,
    connected: Arc<tokio::sync::Notify>,
}

impl BackingServer {
    pub fn start(
        handler: impl Fn(&[u8]) -> PortResult<Vec<u8>> + Send + Sync + 'static,
    ) -> io::Result<Self> {
        let runtime = LiveRuntime::shared()?;
        let scheduler = runtime.scheduler();
        let listener = runtime.block_on(TcpListener::bind((std::net::Ipv4Addr::UNSPECIFIED, 0)))?;
        let port = listener.local_addr()?.port();
        let capability = crate::proxy_host::capability()?;
        let handler = Arc::new(handler);
        let failed = Arc::new(AtomicBool::new(false));
        let (stop, mut stopping) = watch::channel(false);
        let fail = failed.clone();
        let connections = Arc::new(Semaphore::new(2));
        let control = Arc::new(Mutex::new(None));
        let connected = Arc::new(tokio::sync::Notify::new());
        let control_slot = control.clone();
        let control_ready = connected.clone();
        let task = scheduler.handle.spawn({let scheduler=scheduler.clone();async move {
            loop {
                let slot = tokio::select! {
                    _ = stopping.changed() => break,
                    slot = connections.clone().acquire_owned() => match slot {Ok(slot)=>slot,Err(_)=>break},
                };
                let (stream,_) = tokio::select! {
                    _ = stopping.changed() => break,
                    accepted = listener.accept() => match accepted {Ok(value)=>value,Err(_)=>{fail.store(true,Ordering::Release);break}},
                };
                let mut closed = stopping.clone();
                let handler = handler.clone(); let scheduler=scheduler.clone(); let fail=fail.clone();
                let control=control_slot.clone(); let connected=control_ready.clone();
                scheduler.handle.clone().spawn(async move {
                    let _slot=slot;
                    tokio::select! {
                        _ = closed.changed() => {},
                        result = serve(stream,capability,scheduler,handler,control,connected) => {
                            if result.is_err() {fail.store(true,Ordering::Release);}
                        }
                    }
                });
            }
        }});
        Ok(Self {
            port,
            capability,
            stop,
            listener: task,
            failed,
            control,
            connected,
        })
    }
    pub fn port(&self) -> u16 {
        self.port
    }
    pub fn capability(&self) -> [u8; 32] {
        self.capability
    }
    pub fn healthy(&self) -> bool {
        !self.failed.load(Ordering::Acquire)
    }

    pub fn request(&self, bytes: &[u8]) -> PortResult<Vec<u8>> {
        LiveRuntime::shared()
            .map_err(|_| PortError::Io)?
            .block_on(async {
                tokio::time::timeout(std::time::Duration::from_secs(120), async {
                    let ready = self.connected.notified();
                    if self.control.lock().await.is_none() {
                        ready.await;
                    }
                    let mut held = self.control.lock().await;
                    let mut stream = held.take().ok_or(PortError::Io)?;
                    let response = exchange(&mut stream, bytes)
                        .await
                        .map_err(|_| PortError::Io)?;
                    *held = Some(stream);
                    response
                })
                .await
                .map_err(|_| PortError::Io)?
            })
    }

    pub fn control(&self, command: &str) -> PortResult<()> {
        let opcode = match command {
            "pause" => crate::live_wire::FREEZE,
            "resume" => crate::live_wire::RESUME,
            "shutdown" => crate::live_wire::SHUTDOWN,
            _ => return Err(PortError::Invalid),
        };
        self.request(&[opcode]).map(drop)
    }

    pub fn failure(&self) -> Option<(&'static str, PortError)> {
        (!self.healthy()).then_some(("live backing", PortError::Io))
    }

    pub fn take_write_metrics(&self) -> PortResult<crate::FuseWriteMetrics> {
        let bytes = self.request(&[crate::live_wire::WRITE_METRICS])?;
        crate::FuseWriteMetrics::read_from(&mut bytes.as_slice()).map_err(|_| PortError::Io)
    }

    pub fn take_read_metrics(&self) -> PortResult<crate::FuseReadMetrics> {
        let bytes = self.request(&[crate::live_wire::READ_METRICS])?;
        crate::FuseReadMetrics::read_from(&mut bytes.as_slice()).map_err(|_| PortError::Io)
    }

    pub fn invalidate_file(&self, node: crate::NodeId) -> PortResult<()> {
        let mut request = vec![crate::live_wire::INVALIDATE];
        crate::live_wire::u64_out(&mut request, node.0);
        self.request(&request).map(drop)
    }
}
impl Drop for BackingServer {
    fn drop(&mut self) {
        let _ = self.stop.send(true);
        self.listener.abort();
    }
}

async fn serve(
    mut stream: TcpStream,
    capability: [u8; 32],
    scheduler: Scheduler,
    handler: Arc<impl Fn(&[u8]) -> PortResult<Vec<u8>> + Send + Sync + 'static>,
    control: Arc<Mutex<Option<TcpStream>>>,
    connected: Arc<tokio::sync::Notify>,
) -> io::Result<()> {
    stream.set_nodelay(true)?;
    let mut presented = [0; 32];
    tokio::time::timeout(
        std::time::Duration::from_secs(10),
        stream.read_exact(&mut presented),
    )
    .await??;
    if presented
        .iter()
        .zip(capability)
        .fold(0u8, |different, (a, b)| different | (a ^ b))
        != 0
    {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "live backing capability",
        ));
    }
    let role = stream.read_u8().await?;
    if role == b'c' {
        let mut slot = control.lock().await;
        if slot.is_some() {
            return Err(invalid());
        }
        stream.write_u8(1).await?;
        *slot = Some(stream);
        connected.notify_one();
        return Ok(());
    }
    if role != b'd' {
        return Err(invalid());
    }
    stream.write_u8(1).await?;
    loop {
        let length = match stream.read_u32().await {
            Ok(length) => length as usize,
            Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => return Ok(()),
            Err(error) => return Err(error),
        };
        if length == 0 || length > MAX_FRAME {
            return Err(invalid());
        }
        // Input, decoded temporary state and a bounded response are admitted
        // before retaining any frame body or entering the physical queue.
        let admitted = scheduler.admit(length + 2 * MAX_FRAME).await?;
        let mut bytes = vec![0; length];
        stream.read_exact(&mut bytes).await?;
        let handler = handler.clone();
        let response = scheduler.physical(move || Ok(handler(&bytes))).await?;
        match response {
            Ok(bytes) => {
                if bytes.len() > MAX_FRAME {
                    return Err(invalid());
                }
                stream.write_u32((bytes.len() + 1) as u32).await?;
                stream.write_u8(0).await?;
                stream.write_all(&bytes).await?;
            }
            Err(error) => {
                stream.write_u32(2).await?;
                stream.write_u8(1).await?;
                stream.write_u8(crate::protocol::error_code(error)).await?;
            }
        }
        drop(admitted);
    }
}

pub struct BackingConnection {
    // A cancelled/failed exchange drops the taken stream. It cannot reuse a
    // partial frame or silently repeat an append after an uncertain reply.
    stream: Mutex<Option<TcpStream>>,
}
impl BackingConnection {
    pub async fn connect(
        endpoint: String,
        capability: [u8; 32],
        scheduler: &Scheduler,
    ) -> io::Result<Self> {
        use std::net::ToSocketAddrs;
        let address = scheduler
            .physical(move || endpoint.to_socket_addrs()?.next().ok_or_else(invalid))
            .await?;
        let mut stream = TcpStream::connect(address).await?;
        stream.set_nodelay(true)?;
        stream.write_all(&capability).await?;
        stream.write_u8(b'd').await?;
        if stream.read_u8().await? != 1 {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "live backing capability",
            ));
        }
        Ok(Self {
            stream: Mutex::new(Some(stream)),
        })
    }
    pub async fn call(&self, bytes: &[u8]) -> PortResult<Vec<u8>> {
        if bytes.is_empty() || bytes.len() > MAX_FRAME {
            return Err(PortError::Invalid);
        }
        let mut held = self.stream.lock().await;
        let mut stream = held.take().ok_or(PortError::Io)?;
        let result = exchange(&mut stream, bytes).await;
        match result {
            Ok(result) => {
                *held = Some(stream);
                result
            }
            Err(_) => Err(PortError::Io),
        }
    }
}

async fn exchange(stream: &mut TcpStream, bytes: &[u8]) -> io::Result<PortResult<Vec<u8>>> {
    if bytes.is_empty() || bytes.len() > MAX_FRAME {
        return Err(invalid());
    }
    stream.write_u32(bytes.len() as u32).await?;
    stream.write_all(bytes).await?;
    let length = stream.read_u32().await? as usize;
    if length == 0 || length > MAX_FRAME + 1 {
        return Err(invalid());
    }
    let status = stream.read_u8().await?;
    match status {
        0 => {
            let mut bytes = vec![0; length - 1];
            stream.read_exact(&mut bytes).await?;
            Ok(Ok(bytes))
        }
        1 if length == 2 => Ok(Err(crate::protocol::port_error(stream.read_u8().await?)?)),
        _ => Err(invalid()),
    }
}
