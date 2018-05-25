// Copyright (c) 2017-2018 Snowplow Analytics Ltd. All rights reserved.
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

use super::*;
use factotum_server::persistence;
use factotum_server::persistence::{ConsulPersistence, JobEntry, JobOutcome};
use factotum_server::command::Execution;
use std::time::Duration;
use std::thread::Result as ThreadResult;
use std::cell::RefCell;
use std::collections::HashMap;

#[derive(Debug)]
struct GoodPersistenceMock {
    id: String,
    ref_map: RefCell<HashMap<String, String>>,
}

impl GoodPersistenceMock {
    fn new(id: &str) -> Self {
        GoodPersistenceMock {
            id: id.to_owned(),
            ref_map: RefCell::new(HashMap::new()),
        }
    }
}

impl Persistence for GoodPersistenceMock {
    fn id(&self) -> &str {
        &self.id
    }

    fn set_key(&self, key: &str, value: &str) -> ThreadResult<()> {
        let mut map = self.ref_map.borrow_mut();
        map.insert(key.to_owned(), value.to_owned());
        Ok(())
    }

    fn get_key(&self, key: &str) -> ThreadResult<Option<String>> {
        let map = self.ref_map.borrow();
        let value = map.get(key);
        Ok(value.map(|s| s.to_owned()))
    }

    fn prepend_namespace(&self, key: &str) -> String {
        persistence::apply_namespace_if_absent("com.test/namespace", key)
    }
}

#[test]
fn process_settings_fail_no_body() {
    let url = Url::parse("http://not.a.real.address/").unwrap();
    let request_body = Ok(None);
    let mut server_manager = ServerManager::new(Some("0.0.0.0".to_string()), 8080, "http://dummy.test/".to_string(), false, 10_000);

    let (status, response) = process_settings(&url, request_body, &mut server_manager);

    assert_eq!(status::BadRequest, status);
    assert_eq!(r#"{"message":"Error: No body found in POST request"}"#, response);
}

#[test]
fn process_settings_fail_invalid_json() {
    let url = Url::parse("http://not.a.real.address/").unwrap();
    let request_body = Err(bodyparser::BodyError{
        detail: "dummy error".to_string(),
        cause: bodyparser::BodyErrorCause::IoError(::std::io::Error::new(::std::io::ErrorKind::Other, "bad stuff")),
    });
    let mut server_manager = ServerManager::new(Some("0.0.0.0".to_string()), 8080, "http://dummy.test/".to_string(), false, 10_000);

    let (status, response) = process_settings(&url, request_body, &mut server_manager);

    assert_eq!(status::BadRequest, status);
    assert_eq!(r#"{"message":"Error decoding JSON string: bad stuff"}"#, response);
}

#[test]
fn process_settings_fail_invalid_settings_request() {
    let url = Url::parse("http://not.a.real.address/").unwrap();
    let request_body = Ok(Some(SettingsRequest::new("INVALID")));
    let mut server_manager = ServerManager::new(Some("0.0.0.0".to_string()), 8080, "http://dummy.test/".to_string(), false, 10_000);

    let (status, response) = process_settings(&url, request_body, &mut server_manager);

    assert_eq!(status::BadRequest, status);
    assert_eq!(r#"{"message":"Validation Error: Invalid 'state', must be one of (run|drain)"}"#, response);
}

#[test]
fn process_settings_success() {
    let url = Url::parse("http://not.a.real.address/").unwrap();
    let request_body = Ok(Some(SettingsRequest::new("drain")));
    let mut server_manager = ServerManager::new(Some("0.0.0.0".to_string()), 8080, "http://dummy.test/".to_string(), false, 10_000);

    assert_eq!(::SERVER_STATE_RUN, server_manager.state);

    let (status, response) = process_settings(&url, request_body, &mut server_manager);

    assert_eq!(::SERVER_STATE_DRAIN, server_manager.state);
    assert_eq!(status::Ok, status);
    assert_eq!(r#"{"message":"Update acknowledged: [state: drain]"}"#, response);
}

#[test]
fn process_submission_fail_no_body() {
    let url = Url::parse("http://not.a.real.address/").unwrap();
    let request_body = Ok(None);
    let server_manager = ServerManager::new(Some("0.0.0.0".to_string()), 8080, "http://dummy.test/".to_string(), false, 10_000);
    let persistence = ConsulPersistence::new(None, None, None, None);
    let command_store = commands![::FACTOTUM.to_string() => "/tmp/fake_command".to_string()];
    let (tx, _) = mpsc::channel();

    let (status, response) = process_submission(&url, request_body, &server_manager, &persistence, &command_store, &tx);

    assert_eq!(status::BadRequest, status);
    assert_eq!(r#"{"message":"Error: No body found in POST request"}"#, response);
}

#[test]
fn process_submission_fail_invalid_json() {
    let url = Url::parse("http://not.a.real.address/").unwrap();
    let request_body = Err(bodyparser::BodyError{
        detail: "dummy error".to_string(),
        cause: bodyparser::BodyErrorCause::IoError(::std::io::Error::new(::std::io::ErrorKind::Other, "bad stuff")),
    });
    let server_manager = ServerManager::new(Some("0.0.0.0".to_string()), 8080, "http://dummy.test/".to_string(), false, 10_000);
    let persistence = ConsulPersistence::new(None, None, None, None);
    let command_store = commands![::FACTOTUM.to_string() => "/tmp/fake_command".to_string()];
    let (tx, _) = mpsc::channel();

    let (status, response) = process_submission(&url, request_body, &server_manager, &persistence, &command_store, &tx);

    assert_eq!(status::BadRequest, status);
    assert_eq!(r#"{"message":"Error decoding JSON string: bad stuff"}"#, response);
}

#[test]
fn process_submission_fail_server_in_drain_state() {
    let url = Url::parse("http://not.a.real.address/").unwrap();
    let request_body = Ok(Some(JobRequest::new("1", "dummy", "/tmp/somewhere", vec!["--first-arg".to_string()])));
    let mut server_manager = ServerManager::new(Some("0.0.0.0".to_string()), 8080, "http://dummy.test/".to_string(), false, 10_000);
    let persistence = ConsulPersistence::new(None, None, None, None);
    let command_store = commands![::FACTOTUM.to_string() => "/tmp/fake_command".to_string()];
    let (tx, _) = mpsc::channel();

    server_manager.state = ::SERVER_STATE_DRAIN.to_string();
    let (status, response) = process_submission(&url, request_body, &server_manager, &persistence, &command_store, &tx);

    assert_eq!(status::BadRequest, status);
    assert_eq!(r#"{"message":"Server in [drain] state - cannot submit job"}"#, response);
}

#[test]
fn process_submission_fail_invalid_job_request() {
    let url = Url::parse("http://not.a.real.address/").unwrap();
    let request_body = Ok(Some(JobRequest::new("1", "", "/tmp/somewhere", vec!["--first-arg".to_string()])));
    let server_manager = ServerManager::new(Some("0.0.0.0".to_string()), 8080, "http://dummy.test/".to_string(), false, 10_000);
    let persistence = ConsulPersistence::new(None, None, None, None);
    let command_store = commands![::FACTOTUM.to_string() => "/tmp/fake_command".to_string()];
    let (tx, _) = mpsc::channel();

    let (status, response) = process_submission(&url, request_body, &server_manager, &persistence, &command_store, &tx);

    assert_eq!(status::BadRequest, status);
    assert_eq!(r#"{"message":"Validation Error: No valid value found: field 'jobName' cannot be empty"}"#, response);
}

#[derive(Debug)]
struct NoopCommandMock;

impl Execution for NoopCommandMock {
    fn get_command(&self, _: &str) -> Result<String, String> {
        Ok("/noop/command".to_string())
    }

    fn execute(&self, _: String, _: Vec<String>) -> Result<String, String> {
        Ok("NOOP command".to_string())
    }
}

fn validate_ok_mock<U: Execution>(request: JobRequest, _: &U) -> Result<JobRequest, ValidationError> {
    Ok(request)
}

fn queue_is_full(_: Sender<Dispatch>) -> bool {
    true
}

fn queue_is_not_full(_: Sender<Dispatch>) -> bool {
    false
}

#[test]
fn process_valid_submission_fail_job_already_run() {
    use base64::encode as base64_encode;

    let url = Url::parse("http://not.a.real.address/").unwrap();
    let server_manager = ServerManager::new(Some("0.0.0.0".to_string()), 8080, "http://dummy.test/".to_string(), false, 10_000);
    let persistence = GoodPersistenceMock::new("test_submission_fail");
    let request = JobRequest::new("dummy_id_1", "dummy", "/tmp", vec!["--no-colour".to_string()]);
    let job_entry = JobEntry::new(&JobState::QUEUED, &request, &persistence.id(), &JobOutcome::WAITING);
    let job_entry_json = serde_json::to_string(&job_entry).expect("JSON compact encode error");
    let encoded_entry = base64_encode(job_entry_json.as_bytes());
    {
        let mut map = persistence.ref_map.borrow_mut();
        map.insert("com.test/namespace/dummy_id_1".to_string(), encoded_entry);
    }
    let noop_command = NoopCommandMock;
    let request_body = Ok(Some(request));
    let (tx, _) = mpsc::channel();

    let (status, response) = process_valid_submission(&url, request_body, &server_manager, &persistence, &noop_command, &tx, validate_ok_mock, queue_is_not_full);

    assert_eq!(status::BadRequest, status);
    assert_eq!(r#"{"message":"Job is already being processed"}"#, response);
}

#[test]
fn process_valid_submission_fail_queue_is_full() {
    let url = Url::parse("http://not.a.real.address/").unwrap();
    let server_manager = ServerManager::new(Some("0.0.0.0".to_string()), 8080, "http://dummy.test/".to_string(), false, 10_000);
    let persistence = GoodPersistenceMock::new("test_submission_fail");
    let request = JobRequest::new("dummy_id_1", "dummy", "/tmp", vec!["--no-colour".to_string()]);
    let noop_command = NoopCommandMock;
    let request_body = Ok(Some(request));
    let (tx, _) = mpsc::channel();

    let (status, response) = process_valid_submission(&url, request_body, &server_manager, &persistence, &noop_command, &tx, validate_ok_mock, queue_is_full);

    assert_eq!(status::BadRequest, status);
    assert_eq!(r#"{"message":"Queue is full, cannot add job"}"#, response);
}

#[test]
fn process_valid_submission_success() {
    let url = Url::parse("http://not.a.real.address/").unwrap();
    let server_manager = ServerManager::new(Some("0.0.0.0".to_string()), 8080, String::new(), false, 10_000);
    let persistence = GoodPersistenceMock::new("test_submission_success");
    let request = JobRequest::new("dummy_id_1", "dummy", "/tmp", vec!["--no-colour".to_string()]);
    let noop_command = NoopCommandMock;
    let request_body = Ok(Some(request.clone()));
    let (tx, rx) = mpsc::channel();

    let (status, response) = process_valid_submission(&url, request_body, &server_manager, &persistence, &noop_command, &tx, validate_ok_mock, queue_is_not_full);

    let result = rx.recv_timeout(Duration::from_millis(1000)).unwrap();
    assert_eq!(Dispatch::NewRequest(request), result);

    assert_eq!(status::Ok, status);
    assert_eq!(r#"{"message":"SUBMITTING JOB REQ jobId:[dummy_id_1]"}"#, response);
}