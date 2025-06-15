#!/bin/bash
set -eou pipefail

export TZ=America/Los_Angeles
DESTINATION_DIR=$(dirname "$(realpath $0)")/data/$(date '+%Y-%m-%d')
mkdir -p $DESTINATION_DIR
DESTINATION=$DESTINATION_DIR/$(date '+%H%M').proto

curl "http://developer.trimet.org/ws/V1/VehiclePositions?appid=$TRIMET_APP_ID" -o $DESTINATION
