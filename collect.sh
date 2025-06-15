#!/bin/bash
set -eou pipefail

export TZ=America/Los_Angeles
DESTINATION_DIR=$(dirname "$(realpath $0)")/data/$(date '+%Y-%m-%d')
mkdir -p $DESTINATION_DIR
DESTINATION=$DESTINATION_DIR/$(date '+%H%M').json

curl "https://developer.trimet.org/ws/v2/vehicles?showNonRevenue=true&onRouteOnly=true&appid=$TRIMET_APP_ID" -o $DESTINATION
