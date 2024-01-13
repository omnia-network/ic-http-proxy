#!/bin/bash

set -e

./scripts/unit-test.sh

./scripts/integration-test.sh
