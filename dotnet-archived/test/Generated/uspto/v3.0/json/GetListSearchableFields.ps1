<#
  Request: GET /{dataset}/{version}/fields
  Summary: Provides the general information about the API and the list of fields that can be used to query the dataset.
  Description: This GET API returns the list of all the searchable field names that are in the oa_citations. Please see the 'fields' attribute which returns an array of field names. Each field or a combination of fields can be searched using the syntax options shown below.
#>
param(
   <# Name of the dataset. #>
   [Parameter(Mandatory=$True)]
   [String] $dataset,

   <# Version of the dataset. #>
   [Parameter(Mandatory=$True)]
   [String] $version
)

curl -X GET {scheme}://developer.uspto.gov/ds-api/$dataset/$version/fields?dataset=$dataset&version=$version `
  -H 'Accept: application/json' `
  -H 'Content-Type: application/json' `

