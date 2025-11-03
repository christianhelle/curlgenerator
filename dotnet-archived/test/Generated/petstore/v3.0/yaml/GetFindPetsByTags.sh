#
# Request: GET /pet/findByTags
# Summary: Finds Pets by tags
# Description: Multiple tags can be provided with comma separated strings. Use tag1, tag2, tag3 for testing.
#

# Tags to filter by
tags=""

curl -X GET "/pet/findByTags?tags=${tags}" \
  -H "Accept: application/json" \
  -H "Content-Type: application/json" \
