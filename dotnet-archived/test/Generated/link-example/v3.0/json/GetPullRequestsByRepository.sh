#
# Request: GET /2.0/repositories/{username}/{slug}/pullrequests
#

# path parameter: username
username=""
# path parameter: slug
slug=""
# query parameter: state
state=""

curl -X GET "/2.0/repositories/$username/$slug/pullrequests?state=${state}" \
  -H "Accept: application/json" \
  -H "Content-Type: application/json" \
