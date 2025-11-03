#
# Request: POST /pet/{petId}/uploadImage
# Summary: uploads an image
#

# ID of pet to update
petid=""
additionalMetadata=""
file=""

curl -X POST "https://petstore.swagger.io/v2/pet/$petId/uploadImage" \
  -H "Accept: application/json" \
  -H "Content-Type: multipart/form-data" \
-F "additionalMetadata=${additionalMetadata}" \
-F "file=${file}" \

