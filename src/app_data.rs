use std::{path::Path, time::Duration};

use smithay_client_toolkit::{
    compositor::CompositorState,
    output::OutputState,
    reexports::{
        calloop::{EventLoop, LoopHandle},
        calloop_wayland_source::WaylandSource,
        client::{
            globals::registry_queue_init, protocol::wl_keyboard::WlKeyboard, Connection,
            QueueHandle,
        },
    },
    registry::RegistryState,
    seat::SeatState,
    session_lock::SessionLockState,
    shm::Shm,
};

use crate::{conf::Config, lock_data::LockData, media::Media};

pub struct AppData {
    pub conn: Connection,
    pub loop_handle: LoopHandle<'static, Self>,
    pub compositor_state: CompositorState,
    pub registry_state: RegistryState,
    pub output_state: OutputState,
    pub shm: Shm,
    pub seat_state: SeatState,
    pub keyboard: Option<WlKeyboard>,
    pub lock_data: LockData,
    pub media: Media,
    pub config: Config,
    pub exit: bool,
}

impl AppData {
    pub fn connect(config: Config) {
        let conn = Connection::connect_to_env().unwrap();

        let (globals, event_queue) = registry_queue_init(&conn).unwrap();
        let qh: QueueHandle<AppData> = event_queue.handle();
        let mut event_loop: EventLoop<AppData> =
            EventLoop::try_new().expect("Failed to initialize the event loop!");
        let mut app_data = AppData {
            loop_handle: event_loop.handle(),
            conn: conn.clone(),
            compositor_state: CompositorState::bind(&globals, &qh).unwrap(),
            output_state: OutputState::new(&globals, &qh),
            registry_state: RegistryState::new(&globals),
            lock_data: LockData::from_state(SessionLockState::new(&globals, &qh)),
            seat_state: SeatState::new(&globals, &qh),
            shm: Shm::bind(&globals, &qh).unwrap(),
            keyboard: None,
            media: Media::from_config(&config),
            config,
            exit: false,
        };

        app_data.lock_data.lock(&qh);

        WaylandSource::new(conn, event_queue)
            .insert(event_loop.handle())
            .unwrap();

        loop {
            event_loop
                .dispatch(Duration::from_millis(50), &mut app_data)
                .unwrap();

            if app_data.exit {
                break;
            }
        }
    }
}

smithay_client_toolkit::reexports::client::delegate_noop!(AppData: ignore smithay_client_toolkit::reexports::client::protocol::wl_buffer::WlBuffer);
