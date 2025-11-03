#
# Request: GET /2.0/repositories/{username}/{slug}
#

# path parameter: username
username=""
# path parameter: slug
slug=""

curl -X GET "/2.0/repositories/$username/$slug" \
  -H "Accept: application/json" \
  -H "Content-Type: application/json" \
