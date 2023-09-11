// SPDX-License-Identifier: Apache-2.0

//! Example 1

use crate::meter::{HttpMultivariateMetricProvider, HttpUnivariateMetricProvider};

mod tracer;
mod meter;
mod logger;


fn main() {
    let mp = HttpUnivariateMetricProvider::default();
    mp.server_request_duration("localhost", 8080, "GET", 200, "http", 100, None);

    let mmp = HttpMultivariateMetricProvider::default();
    mmp.report("localhost", 8080, "GET", 200, "http", 100, 100, 100, 100, None);
}