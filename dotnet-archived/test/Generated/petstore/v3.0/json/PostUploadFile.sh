#
# Request: POST /pet/{petId}/uploadImage
# Summary: uploads an image
#

# ID of pet to update
petid=""
# Additional Metadata
additionalmetadata=""

curl -X POST "/api/v3/pet/$petId/uploadImage?additionalMetadata=${additionalmetadata}" \
  -H "Accept: application/json" \
  -H "Content-Type: application/octet-stream" \
  --data-binary '@filename'

