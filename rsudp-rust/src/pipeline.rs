use std::collections::HashMap;
use crate::trigger::RecursiveStaLta;
use crate::parser::{TraceSegment, mseed::parse_mseed_record};
use tokio::sync::mpsc;
use tracing::{info, error, warn};
use chrono::{DateTime, Utc};

pub struct FilterManager {
    filters: HashMap<String, RecursiveStaLta>,
    last_times: HashMap<String, DateTime<Utc>>,
    nsta: usize,
    nlta: usize,
}

impl FilterManager {
    pub fn new(nsta: usize, nlta: usize) -> Self {
        Self {
            filters: HashMap::new(),
            last_times: HashMap::new(),
            nsta,
            nlta,
        }
    }

    pub fn get_or_create(&mut self, nslc: &str) -> &mut RecursiveStaLta {
        self.filters.entry(nslc.to_string())
            .or_insert_with(|| RecursiveStaLta::new(self.nsta, self.nlta))
    }

    pub fn process_segment(&mut self, segment: TraceSegment) {
        let nslc = segment.nslc();
        
        // T014: Gap detection
        if let Some(last_time) = self.last_times.get(&nslc) {
            let gap = segment.starttime.signed_duration_since(*last_time);
            if gap.num_seconds().abs() > 10 {
                warn!("Gap detected for {}: {}s. Resetting filter.", nslc, gap.num_seconds());
                self.filters.insert(nslc.clone(), RecursiveStaLta::new(self.nsta, self.nlta));
            }
        }
        
        let filter = self.get_or_create(&nslc);
        for sample in segment.samples {
            let ratio = filter.process(sample);
            if ratio > 3.0 {
                info!("Event detected! NSLC: {}, Ratio: {:.2}", nslc, ratio);
            }
        }
        
        self.last_times.insert(nslc, segment.starttime);
    }
}

pub async fn run_pipeline(
    mut input_rx: mpsc::Receiver<Vec<u8>>,
    mut manager: FilterManager,
) {
    info!("Pipeline started");
    while let Some(data) = input_rx.recv().await {
        match parse_mseed_record(&data) {
            Ok(segments) => {
                for segment in segments {
                    manager.process_segment(segment);
                }
            }
            Err(e) => {
                error!("Parser error: {}", e);
            }
        }
    }
}