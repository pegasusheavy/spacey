// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// Copyright (c) 2025 Pegasus Heavy Industries, LLC

//! Core runtime implementation

mod event_loop;
mod node_runtime;

pub use event_loop::{EventLoop, Timer, TimerId};
pub use node_runtime::NodeRuntime;



