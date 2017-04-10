# Factotum Server

[![Build Status](https://travis-ci.org/snowplow/factotum-server.svg?branch=master)](https://travis-ci.org/snowplow/factotum-server) [![Release 0.1.0](http://img.shields.io/badge/release-0.1.0-blue.svg?style=flat)](https://github.com/snowplow/factotum-server/releases) [![Apache License 2.0](http://img.shields.io/badge/license-Apache--2-blue.svg?style=flat)](http://www.apache.org/licenses/LICENSE-2.0)

A dag running tool designed for efficiently running complex jobs with non-trivial dependency trees. 

## The zen of Factotum Server

1. A Turing-complete job is not a job, it's a program
2. A job must be composable from other jobs
3. A job exists independently of any job schedule

## User quickstart

Assuming you're running **64 bit Linux**: 

```{bash}
wget https://bintray.com/artifact/download/snowplow/snowplow-generic/factotum-server_0.1.0_linux_x86_64.zip
unzip factotum-server_0.1.0_linux_x86_64.zip
./factotum-server --version
```

## Developer quickstart

Factotum Server is written in **[Rust](https://www.rust-lang.org/)**.

### Using Vagrant

* Clone this repository - `git clone git@github.com:snowplow/factotum-server.git`
* `cd factotum-server`
* Set up a Vagrant box and ssh into it - `vagrant up && vagrant ssh`
   * This will take a few minutes
* `cd /vagrant`
* Compile and run a demo - `cargo run -- --factotum-bin=/vagrant/target/debug/factotum` 
   * Note: You will need to have downloaded the Factotum binary to the path above.

### Using stable Rust without Vagrant 

* **[Install Rust](https://www.rust-lang.org/downloads.html)**
   * on Linux/Mac - `curl -sSf https://static.rust-lang.org/rustup.sh | sh`
* Clone this repository - `git clone git@github.com:snowplow/factotum-server.git`
* `cd factotum-server`
* Compile and run the server - `cargo run -- --factotum-bin=/vagrant/target/debug/factotum` 
   * Note: You will need to have downloaded the Factotum binary to the path above.

## Copyright and license

Snowplow is copyright 2017 Snowplow Analytics Ltd.

Licensed under the **[Apache License, Version 2.0][license]** (the "License");
you may not use this software except in compliance with the License.

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.

[license]: http://www.apache.org/licenses/LICENSE-2.0
