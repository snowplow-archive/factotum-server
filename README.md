# Factotum Server

[![Build Status][travis-image]][travis] [![Release][release-image]][releases] [![License][license-image]][license]

## Overview

Job server as a web service. Enables the scheduling and concurrent execution of Factotum jobs.

| **[User Guide][user-guide]**     | 
|:--------------------------------------:|
| [![i1][user-image]][user-guide] |

## User quickstart

Assuming you're running **64 bit Linux**:

```{bash}
$ wget http://dl.bintray.com/snowplow/snowplow-generic/factotum_server_0.1.0_linux_x86_64.zip
$ unzip factotum_server_0.1.0_linux_x86_64.zip
$ ./factotum-server --help
```

[Factotum][factotum-repo] needs to be available locally. You can then run Factotum Server with preset defaults and the Factotum path specified in the following way:

```{bash}
$ ./factotum-server --factotum-bin=<PATH>
```

If Factotum is **not** already available:

```{bash}
$ wget http://dl.bintray.com/snowplow/snowplow-generic/factotum_0.4.1_linux_x86_64.zip
$ unzip factotum_0.4.1_linux_x86_64.zip
$ wget https://raw.githubusercontent.com/snowplow/factotum/master/samples/echo.factfile
```

These commands will download the 0.4.1 Factotum release, unzip it in your current working directory, and download a sample job for you to run.

Consul is an operational dependency - please see HashiCorp's [getting started guide for Consul][consul-install].

See the [wiki][wiki-home] for further guides and information.

## Developer quickstart

Factotum Server is written in **[Rust][rust-lang]**.

### Using Vagrant

* Clone this repository - `git clone git@github.com:snowplow/factotum-server.git`
* `cd factotum-server`
* Set up a Vagrant box and ssh into it - `vagrant up && vagrant ssh`
   * This will take a few minutes
* `cd /vagrant`
* Compile and run a demo - `cargo run -- --factotum-bin=/vagrant/factotum`
   * Note: You will need to have downloaded the Factotum binary to the path above

### Using stable Rust without Vagrant 

* **[Install Rust][rust-install]**
   * on Linux/Mac - `curl -sSf https://static.rust-lang.org/rustup.sh | sh`
* Clone this repository - `git clone git@github.com:snowplow/factotum-server.git`
* `cd factotum-server`
* Compile and run the server - `cargo run -- --factotum-bin=/vagrant/factotum`
   * Note: You will need to have downloaded the Factotum binary to the path above

## Copyright and license

Factotum Server is copyright 2017-2018 Snowplow Analytics Ltd.

Licensed under the **[Apache License, Version 2.0][license]** (the "License");
you may not use this software except in compliance with the License.

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.

[license]: http://www.apache.org/licenses/LICENSE-2.0
[license-image]: http://img.shields.io/badge/license-Apache--2-blue.svg?style=flat

[travis]: https://travis-ci.org/snowplow/factotum-server
[travis-image]: https://travis-ci.org/snowplow/factotum-server.svg?branch=master

[releases]: https://github.com/snowplow/factotum-server/releases
[release-image]: http://img.shields.io/badge/release-0.1.0-6ad7e5.svg?style=flat

[user-image]: http://sauna-github-static.s3-website-us-east-1.amazonaws.com/analyst.svg
[devops-image]:  http://sauna-github-static.s3-website-us-east-1.amazonaws.com/devops.svg
[developer-image]:  http://sauna-github-static.s3-website-us-east-1.amazonaws.com/developer.svg

[factotum-repo]: https://github.com/snowplow/factotum
[wiki-home]: https://github.com/snowplow/factotum/wiki/Factotum-Server
[user-guide]: https://github.com/snowplow/factotum/wiki/Factotum-Server-User-Guide

[rust-lang]: https://www.rust-lang.org/
[rust-install]: https://www.rust-lang.org/downloads.html
[consul-install]: https://www.consul.io/intro/getting-started/install.html
