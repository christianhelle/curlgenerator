# NuGet Trusted Publishing Setup Guide

This repository has been migrated to use NuGet Trusted Publishing for enhanced security. This document explains the configuration required on NuGet.org to enable automated package publishing.

## What is Trusted Publishing?

Trusted Publishing is a security feature that eliminates the need for long-lived API keys. Instead, it uses OpenID Connect (OIDC) tokens issued by GitHub Actions to obtain short-lived credentials (valid for 1 hour) for publishing packages.

### Benefits
- **Enhanced Security**: No long-lived secrets to manage or rotate
- **Reduced Risk**: Temporary credentials reduce the impact of potential leaks
- **Simplified Management**: No manual API key rotation required
- **Better Audit Trail**: All publishing actions are tied to specific workflow runs

## Configuration Steps

### Prerequisites
- You must be the owner or maintainer of the `curlgenerator` package on NuGet.org
- The package must already exist on NuGet.org (Trusted Publishing cannot be used for initial package creation)

### Setup on NuGet.org

1. **Sign in to NuGet.org**
   - Navigate to https://www.nuget.org
   - Sign in with your account that has access to the `curlgenerator` package

2. **Access Trusted Publishing Settings**
   - Go to your account settings: https://www.nuget.org/account/apikeys
   - Scroll down to the **Trusted Publishing** section

3. **Create a New Trusted Publishing Policy**
   - Click on "Add trusted publishing policy" or "Create"
   - Fill in the following information:

   | Field | Value | Description |
   |-------|-------|-------------|
   | **Package Owner** | `christianhelle` | The NuGet.org account/organization that owns the package |
   | **Package Name(s)** | `curlgenerator` or `*` | Specific package name or `*` for all packages owned by the account |
   | **Repository Owner** | `christianhelle` | The GitHub account/organization that owns the repository |
   | **Repository Name** | `curlgenerator` | The name of the GitHub repository |
   | **Workflow File** | `release-template.yml` | The workflow file that will publish packages |
   | **Environment** | _(optional)_ | Leave blank unless using GitHub Environments |

4. **Save the Policy**
   - Review the information
   - Click "Create" or "Save" to activate the policy

### Verification

Once configured, the workflow will automatically:
1. Request an OIDC token from GitHub Actions
2. Exchange it for a temporary NuGet API key (valid for 1 hour)
3. Use that key to publish the package
4. The temporary key expires automatically

### Removing the Old API Key (Optional)

After verifying that Trusted Publishing works:
1. Navigate to https://www.nuget.org/account/apikeys
2. Find the old API key used for publishing (if any)
3. Delete or expire the key to complete the security migration

**Note**: Keep the old key temporarily until you've verified that publishing works with Trusted Publishing.

## Workflow Changes

The following changes were made to the release workflow:

### Added Permissions
```yaml
permissions:
  contents: write
  id-token: write  # Required for OIDC token issuance
```

### Added Authentication Step
```yaml
- name: Authenticate to NuGet with Trusted Publishing
  id: nuget-login
  uses: NuGet/login@v1
  with:
    user: christianhelle
```

### Updated Push Command
```yaml
- name: Push packages to NuGet
  run: dotnet nuget push **/*.nupkg --api-key ${{ steps.nuget-login.outputs.NUGET_API_KEY }} --source ${{ env.NUGET_REPO_URL }} --no-symbols
```

## Troubleshooting

### Error: "Trusted publishing authentication failed"
- Verify the policy is correctly configured on NuGet.org
- Check that the repository owner, name, and workflow file match exactly
- Ensure the package already exists (Trusted Publishing doesn't work for first-time publishes)

### Error: "Ensure GITHUB_TOKEN has permission 'id-token: write'"
- Verify the `permissions` block is present in the workflow
- Check that `id-token: write` is included in the permissions

### Package Still Requires API Key
- The package must exist on NuGet.org before Trusted Publishing can be used
- For initial package creation, use a traditional API key with push permissions
- After the first publish, configure Trusted Publishing for subsequent releases

## References

- [Official Microsoft Documentation](https://learn.microsoft.com/en-us/nuget/nuget-org/trusted-publishing)
- [NuGet Blog Post](https://devblogs.microsoft.com/dotnet/enhanced-security-is-here-with-the-new-trust-publishing-on-nuget-org/)
- [GitHub OIDC Documentation](https://docs.github.com/en/actions/security-for-github-actions/security-hardening-your-deployments/about-security-hardening-with-openid-connect)

## Support

For issues related to:
- **Trusted Publishing setup**: Contact NuGet.org support
- **Workflow configuration**: Open an issue in this repository
- **GitHub OIDC**: Consult GitHub Actions documentation
