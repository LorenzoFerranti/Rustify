use std::process::exit;
use std::thread;

use crate::backend::music_dir::MusicDir;
use crate::backend::{loader_loop, loader_messages, player_loop, player_messages};

use crate::settings::Settings;
use crate::{messages, settings};
use crossbeam_channel::{select, unbounded, Receiver, RecvError, Sender};
use eframe::egui::Context;
use crate::messages::Event;

const TRACK_QUEUE_FILL_UNTIL: u8 = 3;

struct ThreadData {
    settings: Settings,
    root_music_dir: Option<MusicDir>,
    queued_tracks: u8,
    loading_tracks: u8,
    waiting_jump_response: bool,
    ctx: Option<Context>,
    event_sender: Sender<messages::Event>,
    player_req_sender: Sender<player_messages::Request>,
    load_req_sender: Sender<loader_messages::Request>,
}

impl ThreadData {
    fn new(
        settings: Settings,
        event_sender: Sender<messages::Event>,
        player_req_sender: Sender<player_messages::Request>,
        load_req_sender: Sender<loader_messages::Request>,
    ) -> Self {
        Self {
            settings,
            root_music_dir: None,
            queued_tracks: 0,
            loading_tracks: 0,
            waiting_jump_response: false,
            ctx: None,
            event_sender,
            player_req_sender,
            load_req_sender,
        }
    }
}

pub fn run(request_receiver: Receiver<messages::Request>, event_sender: Sender<messages::Event>) {
    // player thread
    let (player_req_sender, player_req_receiver) = unbounded::<player_messages::Request>();
    let (player_event_sender, player_event_receiver) = unbounded::<player_messages::Event>();

    // loader thread
    let (load_req_sender, load_req_receiver) = unbounded::<loader_messages::Request>();
    let (load_resp_sender, load_resp_receiver) = unbounded::<loader_messages::Response>();

    // read settings and send them to frontend
    let settings = settings::read();
    event_sender.send(Event::NewSettings(settings.clone())).expect("Error in send");

    // data
    let mut data = ThreadData::new(settings, event_sender, player_req_sender, load_req_sender);


    thread::spawn(move || loader_loop::run(load_req_receiver, load_resp_sender));
    thread::spawn(move || player_loop::run(player_req_receiver, player_event_sender));

    loop {
        select! {
            recv(request_receiver) -> res => handle_request(
                res,
                &mut data
            ),
            recv(load_resp_receiver) -> res => handle_load_response(
                res,
                &mut data
            ),
            recv(player_event_receiver) -> res => handle_player_event(
                res,
                &mut data
            )
        }
    }
}

fn handle_request(res: Result<messages::Request, RecvError>, data: &mut ThreadData) {
    match res {
        Ok(req) => match req {
            messages::Request::ChangeRoot(path) => {
                // send clear
                data.player_req_sender
                    .send(player_messages::Request::Clear)
                    .unwrap();
                // new music dir and load tracks
                data.root_music_dir = Some(MusicDir::new(path.clone()));
                load_random_tracks(TRACK_QUEUE_FILL_UNTIL, data);

                // update settings
                data.settings.root_music_path = path.into_os_string().into_string().unwrap();
                settings::write(&data.settings);

                // Send play just to be sure
                data.player_req_sender
                    .send(player_messages::Request::Play)
                    .unwrap();
            }
            messages::Request::Play => {
                println!("Backend Main: Play Sent");
                data.player_req_sender
                    .send(player_messages::Request::Play)
                    .unwrap();
            }
            messages::Request::Pause => {
                println!("Backend Main: pause Sent");
                data.player_req_sender
                    .send(player_messages::Request::Pause)
                    .unwrap();
            }
            messages::Request::JumpToFraction(f) => {
                data.waiting_jump_response = true;
                data.player_req_sender
                    .send(player_messages::Request::JumpToFraction(f))
                    .unwrap();
            }
            messages::Request::Skip => {
                data.player_req_sender
                    .send(player_messages::Request::Skip)
                    .unwrap();
            }
            messages::Request::SetVolume(v) => {
                data.player_req_sender
                    .send(player_messages::Request::SetVolume(v))
                    .unwrap();
                // update settings
                data.settings.volume = v;
                // TODO: dont write every time the volume changes!
                settings::write(&data.settings);
            }
            messages::Request::ProvideContext(c) => {
                data.ctx = Some(c);
            }
        },
        // TODO: handle this
        Err(e) => {
            println!("Error in handle request: {e:?}");
            exit(1);
        }
    }
}

fn handle_load_response(res: Result<loader_messages::Response, RecvError>, data: &mut ThreadData) {
    match res {
        Ok(response) => {
            match response {
                loader_messages::Response::Track(source, metadata) => {
                    data.player_req_sender
                        .send(player_messages::Request::Enqueue(source, metadata))
                        .unwrap();
                    data.queued_tracks += 1;
                    data.loading_tracks -= 1
                }
                // TODO: handle this
                loader_messages::Response::NotFound(path) => {
                    println!("{path:?} not found!!!!!");
                    exit(1);
                }
            }
        }
        Err(e) => {
            println!("Error in handle load response: {e:?}");
            exit(1);
        }
    }
}

fn handle_player_event(res: Result<player_messages::Event, RecvError>, data: &mut ThreadData) {
    match res {
        Ok(event) => {
            match event {
                player_messages::Event::ProgressUpdate(d) => {
                    data.event_sender
                        .send(messages::Event::ProgressUpdate(d))
                        .unwrap();
                }
                player_messages::Event::NewTrackPlaying(metadata) => {
                    println!(
                        "Queued track var before subtraction: {0}",
                        data.queued_tracks
                    );
                    println!("New Track Playing Event: {metadata:?}");
                    data.event_sender
                        .send(messages::Event::NewTrackPlaying(metadata))
                        .unwrap();
                    let tracks_to_load = (TRACK_QUEUE_FILL_UNTIL as i16)
                        - ((data.queued_tracks + data.loading_tracks) as i16);
                    println!("Tracks to load: {tracks_to_load}");
                    if tracks_to_load > 0 {
                        load_random_tracks(tracks_to_load as u8, data);
                    }
                }
                player_messages::Event::TrackFinished => {
                    data.queued_tracks -= 1; // panics if underflow
                }
                player_messages::Event::JumpedTo(d) => {
                    data.event_sender
                        .send(messages::Event::JumpedTo(d))
                        .unwrap();
                }
                player_messages::Event::NowPlaying => {
                    data.event_sender.send(messages::Event::NowPlaying).unwrap();
                }
                player_messages::Event::NowPaused => {
                    data.event_sender.send(messages::Event::NowPaused).unwrap();
                }
            }
            if let Some(c) = &data.ctx {
                c.request_repaint();
            }
        }
        Err(e) => {
            println!("Error in handle player event: {e:?}");
            exit(1);
        }
    }
}

fn load_random_tracks(amount: u8, data: &mut ThreadData) {
    for i in 0..amount {
        // println!("Loading {i} / {amount}");
        let random_path = data
            .root_music_dir
            .as_ref()
            .expect("Error")
            .get_random_track_path()
            .unwrap();
        data.load_req_sender
            .send(loader_messages::Request::Track(random_path))
            .unwrap();
    }
    data.loading_tracks += amount;
}
