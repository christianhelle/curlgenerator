#
# Request: POST /streams
# Description: subscribes a client to receive out-of-band data
#

# the location where data will be sent.  Must be network accessible
by the source server

callbackurl=""

curl -X POST "/streams?callbackUrl=${callbackurl}" \
  -H "Accept: application/json" \
  -H "Content-Type: application/json" \
