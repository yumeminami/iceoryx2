// Copyright (c) 2025 Contributors to the Eclipse Foundation
//
// See the NOTICE file(s) distributed with this work for additional
// information regarding copyright ownership.
//
// This program and the accompanying materials are made available under the
// terms of the Apache Software License 2.0 which is available at
// https://www.apache.org/licenses/LICENSE-2.0, or the MIT license
// which is available at https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::cli::HzOptions;
use crate::command::get_pubsub_service_types;
use anyhow::Result;
use iceoryx2::prelude::*;
use iceoryx2::service::builder::{CustomHeaderMarker, CustomPayloadMarker};
use std::collections::VecDeque;
use std::time::{Duration, Instant};

pub(crate) fn hz(options: HzOptions) -> Result<()> {
    let node = NodeBuilder::new()
        .name(&NodeName::new(&options.node_name)?)
        .create::<ipc::Service>()?;

    let service_name = ServiceName::new(&options.service)?;
    let service_types = get_pubsub_service_types(&service_name, &node)?;

    let service = unsafe {
        node.service_builder(&service_name)
            .publish_subscribe::<[CustomPayloadMarker]>()
            .user_header::<CustomHeaderMarker>()
            .__internal_set_payload_type_details(&service_types.payload)
            .__internal_set_user_header_type_details(&service_types.user_header)
            .open_or_create()?
    };

    let subscriber = service.subscriber_builder().create()?;
    let cycle_time = Duration::from_millis(10);

    let mut intervals: VecDeque<u128> = VecDeque::new();
    let mut last_msg_time: Option<Instant> = None;
    let mut last_print = Instant::now();
    let mut last_printed_msg_time: Option<Instant> = None;
    let start = Instant::now();

    while node.wait(cycle_time).is_ok() {
        if let Some(timeout) = options.timeout {
            if start.elapsed().as_secs() >= timeout {
                break;
            }
        }

        while let Some(_sample) = unsafe { subscriber.receive_custom_payload()? } {
            let now = Instant::now();
            if let Some(prev) = last_msg_time {
                let interval_ns = now.duration_since(prev).as_nanos();
                intervals.push_back(interval_ns);
                if intervals.len() > options.window {
                    intervals.pop_front();
                }
            }
            last_msg_time = Some(now);
        }

        if last_print.elapsed() >= Duration::from_secs(1) {
            last_print = Instant::now();
            if last_msg_time == last_printed_msg_time {
                continue;
            }
            last_printed_msg_time = last_msg_time;
            print_stats(&intervals, &options.service);
        }
    }

    Ok(())
}

fn print_stats(intervals: &VecDeque<u128>, service: &str) {
    let n = intervals.len();
    if n == 0 {
        println!("[{}] no messages received", service);
        return;
    }

    let mean_ns = intervals.iter().sum::<u128>() as f64 / n as f64;
    let rate = if mean_ns > 0.0 { 1e9 / mean_ns } else { 0.0 };

    let min_ns = *intervals.iter().min().unwrap() as f64;
    let max_ns = *intervals.iter().max().unwrap() as f64;

    let variance = intervals
        .iter()
        .map(|&x| {
            let diff = x as f64 - mean_ns;
            diff * diff
        })
        .sum::<f64>()
        / n as f64;
    let std_dev_ns = variance.sqrt();

    println!(
        "average rate: {:.3} Hz\n    min: {:.3}s  max: {:.3}s  std dev: {:.5}s  window: {}",
        rate,
        min_ns * 1e-9,
        max_ns * 1e-9,
        std_dev_ns * 1e-9,
        n
    );
}
