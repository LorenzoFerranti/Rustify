use rodio::{OutputStream, OutputStreamHandle, Sample, Sink, Source};
use std::collections::VecDeque;
use std::time::Duration;
use rodio::cpal::FromSample;

pub struct SinkWrapper {
    output_stream: OutputStream,
    output_stream_handle: OutputStreamHandle,
    sink: Sink,
    queue_durations: VecDeque<Duration>,
}

impl SinkWrapper {
    pub fn new() -> Self {
        let (output_stream, output_stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&output_stream_handle).unwrap();
        Self {
            output_stream,
            output_stream_handle,
            sink,
            queue_durations: VecDeque::new(),
        }
    }

    pub fn clear(&mut self) {
        self.sink.clear();
        self.queue_durations.clear();
    }

    pub fn play(&self) {
        self.sink.play()
    }

    pub fn append<S>(&mut self, source: S)
    where
        S: Source + Send + 'static,
        f32: FromSample<S::Item>,
        S::Item: Sample + Send
    {
        self.queue_durations.push_back(source.total_duration().unwrap());
        self.sink.append(source);
    }

    pub fn set_volume(&self, value: f32) {
        self.sink.set_volume(value);
    }

    pub fn set_paused(&mut self, b: bool) {
        if b {
            self.sink.pause();
        } else {
            self.sink.play();
        }
    }

}
