use std::collections::VecDeque;
use std::process::exit;
use std::sync::Arc;
use std::time::Duration;

use crossbeam_channel::{select, unbounded, Receiver, RecvError, Sender};
use rodio::source::EmptyCallback;
use rodio::{Decoder, Sink};

use crate::backend::player_messages::{Event, Request};
use crate::track_metadata::TrackMetaData;

pub fn run(request_receiver: Receiver<Request>, event_sender: Sender<Event>) {
    // track finished message
    let (track_finished_sender, track_finished_receiver) = unbounded::<()>();

    // track metadata queue
    let mut track_metadata_queue: VecDeque<Arc<TrackMetaData>> = VecDeque::new();

    let stream_handle =
        rodio::OutputStreamBuilder::open_default_stream().expect("open default audio stream");
    let sink = Sink::connect_new(&stream_handle.mixer());

    loop {
        select! {
            recv(request_receiver) -> res => handle_request(
                res,
                &sink,
                &track_finished_sender,
                &event_sender,
                &mut track_metadata_queue,
            ),
            recv(track_finished_receiver) -> _ => handle_track_finished(
                &mut track_metadata_queue,
                &event_sender,
            ),
            default(Duration::from_millis(100)) => {},
        }
        if !sink.empty() {
            event_sender
                .send(Event::ProgressUpdate(sink.get_pos()))
                .unwrap()
        }
    }
}

fn handle_request(
    res: Result<Request, RecvError>,
    sink: &Sink,
    track_finished_sender: &Sender<()>,
    event_sender: &Sender<Event>,
    track_metadata_queue: &mut VecDeque<Arc<TrackMetaData>>,
) {
    match res {
        Ok(req) => match req {
            Request::Enqueue(track, metadata) => {
                let source = Decoder::try_from(track).unwrap();
                sink.append(source);

                // append empty callback to send track finished signal
                let sender = track_finished_sender.clone();
                let ec: EmptyCallback = EmptyCallback::new(Box::new(move || {
                    sender.send(()).unwrap();
                }));
                sink.append(ec);

                track_metadata_queue.push_back(metadata);

                if track_metadata_queue.len() == 1 {
                    // safe unwrap
                    let arc_clone = Arc::clone(track_metadata_queue.front().unwrap());
                    event_sender
                        .send(Event::NewTrackPlaying(Some(arc_clone)))
                        .unwrap()
                }
            }
            Request::Play => {
                println!("Player thread: received play");
                println!("Sink is paused: {0}", sink.is_paused());
                sink.play();
                println!("Sink is paused: {0}", sink.is_paused());

                event_sender.send(Event::NowPlaying).unwrap();
            }
            Request::Pause => {
                println!("Player thread: received pause");
                println!("Sink is paused: {0}", sink.is_paused());
                sink.pause();
                println!("Sink is paused: {0}", sink.is_paused());

                event_sender.send(Event::NowPaused).unwrap();
            }
            Request::JumpToFraction(f) => match track_metadata_queue.front().unwrap().duration {
                None => {
                    unreachable!();
                }
                Some(d) => {
                    let progress_seconds = d.mul_f32(f);
                    println!("JUMP TO {progress_seconds:?}");
                    match sink.try_seek(progress_seconds) {
                        Ok(_) => {
                            event_sender
                                .send(Event::JumpedTo(progress_seconds))
                                .unwrap();
                        }
                        Err(e) => {
                            println!("ERROR IN SEEK!");
                            println!("{e}");
                            exit(1)
                        }
                    }
                }
            },
            Request::Skip => {
                sink.skip_one();
            }
            Request::Clear => {
                sink.clear();
                track_metadata_queue.clear();
                event_sender.send(Event::NewTrackPlaying(None)).unwrap()
            }
            Request::SetVolume(v) => {
                sink.set_volume(v * v); // adjust volume curve
            }
        },
        // TODO: handle this
        Err(e) => {
            eprintln!("Error in handle request: {e:?}");
            exit(1);
        }
    }
}

fn handle_track_finished(
    track_metadata_queue: &mut VecDeque<Arc<TrackMetaData>>,
    event_sender: &Sender<Event>,
) {
    event_sender.send(Event::TrackFinished).unwrap();
    track_metadata_queue.pop_front();
    let front = track_metadata_queue.front();
    match front {
        None => event_sender.send(Event::NewTrackPlaying(None)).unwrap(),
        Some(metadata) => event_sender
            .send(Event::NewTrackPlaying(Some(Arc::clone(metadata))))
            .unwrap(),
    }
}
