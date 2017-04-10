#!/bin/bash -e

if [ "${TRAVIS_OS_NAME}" == "osx" ]; then
	echo "Configuring openssl libs for OSX..."
	#brew install openssl
	export OPENSSL_INCLUDE_DIR=`brew --prefix openssl`/include
	export OPENSSL_LIB_DIR=`brew --prefix openssl`/lib
	echo "...done!"

  SUFFIX="_darwin_x86_64"
  export PATH=$PATH:/Users/travis/Library/Python/2.7/bin
else 
  SUFFIX="_linux_x86_64"
fi

if [ "$1" == "--release" ]; then
  env RM_SUFFIX=${SUFFIX} release-manager --config ./.travis/release.yml --check-version --make-artifact --make-version --upload-artifact
else
	cargo build --verbose
	cargo test --verbose
fi
