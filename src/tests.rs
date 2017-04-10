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

#[test]
fn check_factotum_bin_arg_fail() {
    let expected = Err("Invalid path for Factotum binary at: '/fake/path'".to_string());
    let actual = check_factotum_bin_arg("/fake/path");
    assert_eq!(expected, actual);
}

#[test]
fn check_factotum_bin_arg_success() {
    let expected = Ok(());
    let actual = check_factotum_bin_arg(".");
    assert_eq!(expected, actual);
}

#[test]
fn check_ip_arg_fail() {
    let expected = Err("Invalid IP address: [NOT.AN.IP] - Regex mismatch".to_string());
    let actual = check_ip_arg(&Some("NOT.AN.IP".to_string()));
    assert_eq!(expected, actual);
}

#[test]
fn check_ip_arg_success_with_valid_ip() {
    let expected = Ok(());
    let actual = check_ip_arg(&Some("255.255.255.255".to_string()));
    assert_eq!(expected, actual);
}

#[test]
fn check_ip_arg_success_with_none() {
    let expected = Ok(());
    let actual = check_ip_arg(&None);
    assert_eq!(expected, actual);
}

#[test]
fn is_a_valid_ip_fail() {
    let result = is_a_valid_ip("NOT.AN.IP");
    assert!(result == false);
}

#[test]
fn is_a_valid_ip_success() {
    let result = is_a_valid_ip("255.255.255.255");
    assert!(result == true);
}

#[test]
fn get_log_level_fail() {
    let expected = Err("Unknown log level: 'NOT A LOG LEVEL'\nPlease select a valid log level.".to_string());
    let actual = get_log_level(&Some("NOT A LOG LEVEL".to_string()));
    assert_eq!(expected, actual);
}

#[test]
fn get_log_level_success_valid_level() {
    let expected = Ok(LogLevelFilter::Debug);
    let actual = get_log_level(&Some("DEBUG".to_string()));
    assert_eq!(expected, actual);
}

#[test]
fn get_log_level_success_with_none() {
    let expected = Ok(LogLevelFilter::Warn);
    let actual = get_log_level(&None);
    assert_eq!(expected, actual);
}