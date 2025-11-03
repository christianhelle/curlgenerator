#
# Request: GET /2.0/repositories/{username}
#

# path parameter: username
username=""

curl -X GET "/2.0/repositories/$username" \
  -H "Accept: application/json" \
  -H "Content-Type: application/json" \
