<#
  Request: GET /
  Summary: List available data sets
#>

curl -X GET {scheme}://developer.uspto.gov/ds-api/ `
  -H 'Accept: application/json' `
  -H 'Content-Type: application/json' `

