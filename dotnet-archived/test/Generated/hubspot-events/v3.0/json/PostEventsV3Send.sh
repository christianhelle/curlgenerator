#
# Request: POST /events/v3/send
# Summary: Sends Custom Behavioral Event
# Description: Endpoint to send an instance of a behavioral event
#

curl -X POST "https://api.hubapi.com//events/v3/send" \
  -H "Accept: application/json" \
  -H "Content-Type: application/json" \
  -d '{
  "utk": "string",
  "email": "string",
  "eventName": "string",
  "properties": {},
  "occurredAt": "2025-09-25T10.30.57Z",
  "objectId": "string"
}'

