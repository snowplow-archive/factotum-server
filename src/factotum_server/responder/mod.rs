// Copyright (c) 2017-2021 Snowplow Analytics Ltd. All rights reserved.
//
// This program is licensed to you under the Apache License Version 2.0, and
// you may not use this file except in compliance with the Apache License
// Version 2.0.  You may obtain a copy of the Apache License Version 2.0 at
// http://www.apache.org/licenses/LICENSE-2.0.
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the Apache License Version 2.0 is distributed on an "AS
// IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or
// implied.  See the Apache License Version 2.0 for the specific language
// governing permissions and limitations there under.
//

use std::collections::HashMap;
use std::error::Error;
use std::ops::{Deref, DerefMut};
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use iron::mime::*;
use iron::prelude::*;
use iron::status;
use iron::status::Status;
use url::Url;
use bodyparser;
use persistent::{Read, State};
use serde::Serialize;
use serde_json;

use factotum_server::{Paths, Server, Storage, Updates};
use factotum_server::command::Execution;
use factotum_server::dispatcher::{Dispatch, Query};
use factotum_server::persistence;
use factotum_server::persistence::{Persistence, JobState};
use factotum_server::server::{ServerManager, SettingsRequest, JobRequest, ValidationError};

#[cfg(test)]
mod tests;

// JSON Response Structs

#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct ResponseMessage {
    message: String
}

#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct FactotumServerStatus {
    version: String,
    server: ServerStatus,
    dispatcher: DispatcherStatus,
}

#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct ServerStatus {
    start_time: String,
    up_time: String,
    state: String
}

#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DispatcherStatus {
    pub workers: WorkerStatus,
    pub jobs: JobStatus,
}

#[derive(Debug,PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerStatus {
    pub total: usize,
    pub idle: usize,
    pub active: usize,
}

#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobStatus {
    pub max_queue_size: usize,
    pub in_queue: usize,
}

// Response handlers

pub fn api(request: &mut Request) -> IronResult<Response> {
    let url: Url = request.url.clone().into();
    let response = get_help_message();
    return_json(status::Ok, encode(&url, response))
}

pub fn status(request: &mut Request) -> IronResult<Response> {
    let url: Url = request.url.clone().into();
    let rwlock = match request.get::<State<Server>>() {
        Ok(lock) => lock,
        Err(e) => return return_json(status::ServiceUnavailable, encode(&url, e.to_string()))
    };
    let reader = match rwlock.read() {
        Ok(result) => result,
        Err(e) => return return_json(status::ServiceUnavailable, encode(&url, e.to_string()))
    };
    let server_manager = reader.deref();
    let mutex = match request.get::<Read<Updates>>() {
        Ok(lock) => lock,
        Err(e) => return return_json(status::ServiceUnavailable, encode(&url, e.to_string()))
    };
    let jobs_channel = match mutex.try_lock() {
        Ok(result) => result,
        Err(e) => return return_json(status::ServiceUnavailable, encode(&url, e.to_string()))
    };

    let response = get_server_status(server_manager, jobs_channel.clone());
    return_json(status::Ok, encode(&url, response))
}

pub fn settings(request: &mut Request) -> IronResult<Response> {
    let url: Url = request.url.clone().into();
    let request_body = request.get::<bodyparser::Struct<SettingsRequest>>();
    let rwlock = match request.get::<State<Server>>() {
        Ok(lock) => lock,
        Err(e) => return return_json(status::ServiceUnavailable, encode(&url, e.to_string()))
    };
    let mut server = match rwlock.write() {
        Ok(result) => result,
        Err(e) => return return_json(status::ServiceUnavailable, encode(&url, e.to_string()))
    };
    
    let (status, response) = process_settings(&url, request_body, server.deref_mut());
    return_json(status, response)
}

pub fn submit(request: &mut Request) -> IronResult<Response> {
    let url: Url = request.url.clone().into();
    let request_body = request.get::<bodyparser::Struct<JobRequest>>();
    let server_rwlock = match request.get::<State<Server>>() {
        Ok(lock) => lock,
        Err(e) => return return_json(status::ServiceUnavailable, encode(&url, e.to_string()))
    };
    let server = match server_rwlock.write() {
        Ok(result) => result,
        Err(e) => return return_json(status::ServiceUnavailable, encode(&url, e.to_string()))
    };
    let storage_rwlock = match request.get::<State<Storage>>() {
        Ok(lock) => lock,
        Err(e) => return return_json(status::ServiceUnavailable, encode(&url, e.to_string()))
    };
    let persistence = match storage_rwlock.write() {
        Ok(result) => result,
        Err(e) => return return_json(status::ServiceUnavailable, encode(&url, e.to_string()))
    };
    let command_store_rwlock = match request.get::<Read<Paths>>() {
        Ok(lock) => lock,
        Err(e) => return return_json(status::ServiceUnavailable, encode(&url, e.to_string()))
    };
    let command_store = match command_store_rwlock.read() {
        Ok(result) => result,
        Err(e) => return return_json(status::ServiceUnavailable, encode(&url, e.to_string()))
    };
    let sender_mutex = match request.get::<Read<Updates>>() {
        Ok(lock) => lock,
        Err(e) => return return_json(status::ServiceUnavailable, encode(&url, e.to_string()))
    };
    let jobs_channel = match sender_mutex.try_lock() {
        Ok(result) => result,
        Err(e) => return return_json(status::ServiceUnavailable, encode(&url, e.to_string()))
    };

    let (status, response) = process_submission(&url, request_body, server.deref(), persistence.deref(), command_store.deref(), jobs_channel.deref());
    return_json(status, response)
}

pub fn check(request: &mut Request) -> IronResult<Response> {
    let url: Url = request.url.clone().into();
    let storage_rwlock = match request.get::<State<Storage>>() {
        Ok(lock) => lock,
        Err(e) => return return_json(status::ServiceUnavailable, encode(&url, e.to_string()))
    };
    let persistence = match storage_rwlock.write() {
        Ok(result) => result,
        Err(e) => return return_json(status::ServiceUnavailable, encode(&url, e.to_string()))
    };

    let (status, response) = check_job_request(&url, persistence.deref());
    return_json(status, response)
}

// Helpers

fn get_help_message() -> serde_json::Value {
    json!(
        {
            "/help": {
                "function": "Returns this message!",
                "params": "pretty=1"
            },
            "/status": {
                "function": "Returns general information about the server and host system.",
                "params": "pretty=1"
            },
            "/settings": {
                "function": "Updates settings within the server.",
                "body": {
                    "state": "run|drain"
                },
                "params": "pretty=1"
            },
            "/submit": {
                "function": "Submits a job to the queue.",
                "body": {
                    "jobName": "com.acme-main",
                    "factfilePath": "/com.acme-main/factfile",
                    "factfileArgs": "[ --start step-2 ]"
                },
                "params": "pretty=1"
            },
            "/check": {
                "function": "Fetches the state of a job by the ID.",
                "params": "pretty=1, id=[id string]"
            }
        }
    )
}

fn get_server_status(server: &ServerManager, jobs_channel: Sender<Dispatch>) -> FactotumServerStatus {
    let (tx, rx) = mpsc::channel();
    jobs_channel.send(Dispatch::StatusUpdate(Query::new("status_query", tx))).expect("Job requests channel receiver has been deallocated");
    let dispatcher_status = rx.recv().expect("Server status senders have been disconnected");

    FactotumServerStatus {
        version: ::VERSION.to_string(),
        server: ServerStatus {
            start_time: server.get_start_time(),
            up_time: server.get_uptime(),
            state: server.state.to_string()
        },
        dispatcher: dispatcher_status,
    }
}

fn process_settings(url: &Url, request_body: Result<Option<SettingsRequest>, bodyparser::BodyError>, server: &mut ServerManager) -> (Status, String) {
    // get body
    let settings = match request_body {
        Ok(Some(decoded_settings)) => decoded_settings,
        Ok(None) => {
            return (status::BadRequest, create_warn_response(url, "Error: No body found in POST request"))
        },
        Err(e) => {
            return (status::BadRequest, create_warn_response(url, &format!("Error decoding JSON string: {}", e.cause().expect("Cause not found"))))
        }
    };

    // validate settings request
    let validated_settings = match SettingsRequest::validate(settings) {
        Ok(validated_settings) => validated_settings,
        Err(e) => {
            return (status::BadRequest, create_warn_response(url, &format!("{}", e)))
        }
    };

    // update server state
    server.state = validated_settings.state.to_string();
    (status::Ok, create_ok_response(url, &format!("Update acknowledged: [state: {}]", server.state)))
}

fn process_submission<T, U>(url: &Url, request_body: Result<Option<JobRequest>, bodyparser::BodyError>, server: &ServerManager, persistence: &T, command_store: &U, jobs_channel: &Sender<Dispatch>) -> (Status, String) where
    T: Persistence,
    U: Execution {
    process_valid_submission(url, request_body, server, persistence, command_store, jobs_channel, JobRequest::validate, is_requests_queue_full)
}

fn process_valid_submission<T, U, F, G>(url: &Url, request_body: Result<Option<JobRequest>, bodyparser::BodyError>, server: &ServerManager, persistence: &T, command_store: &U, jobs_channel: &Sender<Dispatch>, validate: F, is_requests_queue_full: G) -> (Status, String) where
    T: Persistence,
    U: Execution,
    F: Fn(JobRequest, &U) -> Result<JobRequest, ValidationError>,
    G: Fn(Sender<Dispatch>) -> bool {
    // get body
    let job_request = match request_body {
        Ok(Some(decoded_job_request)) => decoded_job_request,
        Ok(None) => {
            return (status::BadRequest, create_warn_response(url, "Error: No body found in POST request"))
        },
        Err(e) => {
            return (status::BadRequest, create_warn_response(url, &format!("Error decoding JSON string: {}", e.cause().expect("Cause not found"))))
        }
    };

    // check state
    if !server.is_running() {
        return (status::BadRequest, create_warn_response(url, &format!("Server in [{}] state - cannot submit job", server.state)))
    }

    // validate job request
    let mut validated_job_request = match validate(job_request, command_store) {
        Ok(validated_job_request) => validated_job_request,
        Err(e) => {
            return (status::BadRequest, create_warn_response(url, &format!("{}", e)))
        }
    };

    if is_job_queued_or_running(persistence, &mut validated_job_request) {
        return (status::BadRequest, create_warn_response(url, "Job is already being processed"))
    }

    // check queue size
    if is_requests_queue_full(jobs_channel.clone()) {
        return (status::BadRequest, create_warn_response(url, "Queue is full, cannot add job"))
    }

    // append args
    JobRequest::append_job_args(&server.deref(), &mut validated_job_request);
    let job_id = validated_job_request.job_id.clone();
    jobs_channel.send(Dispatch::NewRequest(validated_job_request)).expect("Job requests channel receiver has been deallocated");
    (status::Ok, create_ok_response(url, &format!("SUBMITTING JOB REQ jobId:[{}]", job_id)))
}

fn is_job_queued_or_running<T: Persistence>(persistence: &T, job_request: &mut JobRequest) -> bool {
    let mut is_running = false;
    match persistence::get_entry(persistence, &job_request.job_id) {
        Some(job_entry) => {
            debug!("Job entry id='{}' state='{}'", &job_entry.job_request.job_id, &job_entry.state);
            if job_entry.state != JobState::DONE {
                is_running = true;
            }
        },
        None => {
            debug!("No state found for job entry id='{}'", job_request.job_id);
        },
    };
    is_running
}

fn is_requests_queue_full(jobs_channel: Sender<Dispatch>) -> bool {
    let (tx, rx) = mpsc::channel();
    jobs_channel.send(Dispatch::CheckQueue(Query::new("queue_query", tx))).expect("Job requests channel receiver has been deallocated");
    rx.recv().expect("Queue query senders have been disconnected")
}

fn check_job_request<T: Persistence>(url: &Url, persistence: &T) -> (Status, String) {
    let query_map = get_query_map(&url);
    let job_request_id = match query_map.get("id") {
        Some(id) => id,
        None => return (status::BadRequest, create_warn_response(url, "Error: No 'id' found in URL query parameters"))
    };
    let response = match persistence::get_entry(persistence, &job_request_id) {
        Some(job_entry) => {
            debug!("Job entry id='{}' state='{}'", job_entry.job_request.job_id, job_entry.state);
            job_entry
        },
        None => {
            debug!("No job entry found for id='{}'", &job_request_id);
            return (status::BadRequest, create_warn_response(url, &format!("Error: No job entry found for id='{}'", &job_request_id)))
        },
    };
    debug!("{:?}", &response);
    (status::Ok, encode(&url, &response))
}

fn get_query_map(url: &Url) -> HashMap<String, String> {
    let parser = url.query_pairs().into_owned();
    parser.collect()
}

fn encode_compact<T: Serialize>(message: T) -> String {
    serde_json::to_string(&message).expect("JSON compact encode error")
}

fn encode_pretty<T: Serialize>(message: T) -> String {
    serde_json::to_string_pretty(&message).expect("JSON pretty encode error")
}

fn encode<T: Serialize>(url: &Url, message: T) -> String {
    let query_map = get_query_map(url);
    if let Some(pretty) = query_map.get("pretty") {
        if pretty == "1" {
            return encode_pretty(message)
        }
    }
    encode_compact(message)
}

fn create_response(url: &Url, message: &str) -> String {
    let response = ResponseMessage { message: message.to_string() };
    encode(&url, &response)
}

fn create_ok_response(url: &Url, message: &str) -> String {
    info!("{}", message);
    create_response(url, message)
}

fn create_warn_response(url: &Url, message: &str) -> String {
    warn!("{}", message);
    create_response(url, message)
}

fn return_json(code: Status, response: String) -> IronResult<Response> {
    let content_type = ::JSON_CONTENT_TYPE.parse::<Mime>().expect(&format!("Unable to parse Mime type for '{}'", ::JSON_CONTENT_TYPE));
    Ok(Response::with((content_type, code, response)))
}
