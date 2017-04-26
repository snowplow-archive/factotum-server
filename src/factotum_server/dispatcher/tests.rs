// Copyright (c) 2017 Snowplow Analytics Ltd. All rights reserved.
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
use std::sync::mpsc;

#[test]
fn create_new_dispatcher() {
    let dispatcher = Dispatcher::new(10, 2);

    assert_eq!(dispatcher.max_jobs, 10);
    assert_eq!(dispatcher.max_workers, 2);
    assert!(dispatcher.requests_queue.is_empty());
}

#[test]
fn create_new_queries_same_name_different_sender_still_equal() {
    let (tx1, _) = mpsc::channel();
    let (tx2, _) = mpsc::channel();
    let query_one: Query<String> = Query::new("dummy_query", tx1);
    let query_two: Query<String> = Query::new("dummy_query", tx2);
    assert!(query_one == query_two);
}

#[test]
fn create_new_queries_same_name_same_sender_still_equal() {
    let (tx, _) = mpsc::channel();
    let query_one: Query<String> = Query::new("dummy_query", tx.clone());
    let query_two: Query<String> = Query::new("dummy_query", tx.clone());
    assert!(query_one == query_two);
}

#[test]
fn create_new_queries_different_name_different_sender_is_not_equal() {
    let (tx1, _) = mpsc::channel();
    let (tx2, _) = mpsc::channel();
    let query_one: Query<String> = Query::new("dummy_query_one", tx1);
    let query_two: Query<String> = Query::new("dummy_query_two", tx2);
    assert!(query_one != query_two);
}

#[test]
fn create_new_queries_different_name_same_sender_is_not_equal() {
    let (tx, _) = mpsc::channel();
    let query_one: Query<String> = Query::new("dummy_query_one", tx.clone());
    let query_two: Query<String> = Query::new("dummy_query_two", tx.clone());
    assert!(query_one != query_two);
}
