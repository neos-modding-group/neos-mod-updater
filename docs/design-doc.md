# Automatic Mod Management

## The Problem

The problem is outdated mods, and the work involved in keeping them updated. Why it's a problem:

- Outdated versions may be broken
- It's not obvious when you're running an outdated version
- It's a decent amount of manual work to update a single mod. Multiply that by every mod you use when you return after taking a break from the game.
- Mods shouldn't be rolling their own auto-updaters
  - Security concerns
  - Non-uniform experience
  - Doing it all from one mod manager simply makes more sense

## The Solution

We make a mod manager that can automatically install and update mods. Easier said than done, though. This will definitely take time to build well, because the requirements to make this are a bit more than what it would initially seem. The VRChat Modding Group has a great example of what this could look like, and for them it was years and tens of thousands of users before they had it put together.

My proposed design:
1. A single-endpoint "API" that provides a JSON manifest of known good mods
   - This will simply be a JSON file hosted in a github repo
   - Mod devs can PR in metadata about their new mod releases
   - Trusted members of the community will be able to review and approve PRs
   - Only PRs with a certain number of approvals will be mergable. Ideally, I want at least 2 approvals, but if the community is sleepy maybe 1 will suffice.
   - Approvers are forbidden from self-approving their own mods
   - JSON schema details below
2. An auto-updater GUI client
   - Fetches the manifest and uses it to install and update mods
   - Validates mod integrity clientside
   - Implementation details below

## Security

The security concerns about auto-updaters are huge.

- Mods are coming from dozens of different users, who are relatively unknown and don't have any sort of trust built up. This presents a pretty large risk:
  - Any one of these modders might have their credentials compromised
  - A malicious actor could hide in the crowd of non-malicious modders
  - A malicious actor could remain anonymous, which wouldn't be unusual for a modder
- Having a mod review board shifts the trust issue from the mod devs to the review board
  - Liability issues?
  - Corruption issues? Solved via transparency?
    - We must publicily record ALL mod reviews, who specifically approved/denied the mod, and rejection reasons if applicable
    - I wonder if we can abuse github PRs for this purpose? That would remove the need to cert pin as we can trust that github.io/github.com won't be taken over
- An auto-updater further increases risk, because now you're automatically grabbing and executing binaries
  - The auto-updater itself has an attack surface.
    - If it doesn't use TLS it's vulnerable to MITM
    - If it doesn't cert pin it's vulnerable to a domain takeover
    - If it downloads binaries from an untrusted source there's all sorts of nefarious things that could happen
    - If it loads assemblies to inspect them it might accidentally execute them
  - If we *do* cert pin, we need to use a self-signed cert and be absolutely sure the private key doesn't get compromised
  - As the auto-updater has potential security pitfalls and attack surfaces, you want to only have one. If every mod has its own auto-updater that's just an auditing nightmare.
  - The auto updater will need an API, and an object storage. These can be separate applications.
  - The object storage can be whatever, as we'll be validating artifacts clientside for reasons:
    - Github artifacts are NOT immutable, meaning a fixed URL does not guarantee a fixed file. Therefore we MUST verify artifacts clientside.
    - If we do clientside verification we can actually host artifacts wherever we want to, which frees mod devs from github if they hate it
    - Checking against a SHA-256 hash in the manifest should be a secure way of doing this

At minimum, we have to trust:
- TLS (duh)
- The auto-updater
  - Client
    - Maintainer
  - API
    - Maintainer
    - Host (github!)
  - artifact hashing algorithm (SHA-256)
- Mod review board
  - The admin
  - The members

## Mod Review
1. Start with the manifest PR
2. Download the mod from the URL
3. Verify the file hash in the PR matches the file hash you downloaded
4. Decompile the mod
5. Verify the mod isn't malicious
6. Optionally, make sure the mod isn't doing anything stupid that will impact Neos performance/stability
7. Approve the PR
8. If the PR has enough approvals, merge the PR

## API Schema
[Example manifest](example-manifest.json)

Top-level object:
- Schema version
- Content version? Maybe just use an etag for this.
- Mod Map
  - Key: Mod id (the mod's official filename)
  - Value: Mod
    - Name
    - Description
    - Author
    - Author URL
    - Source Location
    - Website
    - tag list (list of strings) used for search?
    - category (one string)
    - flag list (list of strings) special meaning
    - conflicts (list of mod ids)
    - dependencies
      - dependency map
        - key: mod id
        - value: dependency
          - version specifier
    - Version Map (this might be in a separate json object)
      - Key: version number
      - Value: version
        - changelog
        - releaseUrl
        - Neos version compatibility? (NOT semver `2022.1.28.1310` but `<` and `>` rules will work fine)
        - Modloader version compatibility? (semver)
        - flag list (list of strings) special meaning, inherits from mod
        - conflicts (list of mod ids), inherits from mod
        - Mod dependencies? (circular dependencies are actually okay), list of mod ids + version specifiers?. NML dependency is implied by default, inherits from mod
        - Artifact list
          - Artifact
            - download URL
            - file hash
            - install location, defaults to `/nml_mods`

Flags:
- Mod Flags
  - `deprecated` Deprecated (maintainer is gone, users need to migrate)
  - `plugin` it needs a -LoadAssembly argument to work and it does not depend on NML
  - `file` it does not depend on NML
- Version Flags (mod flags are inherited)
  - Security Vulnerability
    - `vulnerablity:low` Low
    - `vulnerablity:medium` Medium
    - `vulnerablity:high` High
    - `vulnerablity:critical` Critical
  - `broken` Broken (different from incompatible, means the version itself is broken by design)
    - `broken:linux-native` Doesn't work on linux native
    - `broken:linux-wine` Doesn't work on linux wine/proton
    - `broken:windows` Doesn't work on windows
  - `prerelease` Mod dev wants to limit distribution

## Auto updater components
1. Get manifest, save to cache. Fall back to cached manifest if no internet.
2. post-process manifest into internal data structure
3. Get NML version
   - this is doable with pelite or hash lookup
4. Get Neos version
   - requires parsing of PE32 (crate) and CLI layout (no crate)
   - how to determine if it's the tokens build or not
     - parse FrooxEngine.Engine::get_TokensSupported method body?
     - guess by install folder? `root/steamapps/common/GameDir` where `root` should contain a `libraryfolder.vdf` file and `root/steamapps` should contain a `appmanifest_740250.acf`
   - how to determine if it's WINE or native linux
     - `GameDir` will contain a `Neos.x86_64` ELF instead of a Neos.exe PE32
   - how to determine if headless
     - we might be able to figure out a bunch of stuff by parsing betakey and InstalledDepots from `appmanifest_740250.acf`, which is NOT a json file.
5. Get installed mods, and their versions
   - doable by hash lookup
6. Get cached mods
   - hash lookup?
   - storing them in some sort of fancy-schmancy way?
7. Determine what platform we're on
8. Determine what mods to hide
9. Render GUI
10. Download mods
11. Verify mod hashes
12. Install mods
13. Disabled mods should be moved to cache instead of deleted
14. Launch Neos

### Platforms
- `windows`
- `windows:headless`
- `windows:standalone`
- `linux:native`
- `linux:wine`... is this just running the windows install?
- `linux:headless`

### What mods to hide?
- `broken` or broken for your platform
- no longer compatible
- has an unresolvable dependency
- has a vulnerability higher than a configured threshold
- `prerelease` unless overridden in config

### GUI
- List of mods
  - Unavailable mods should be visually distinct or hidden
  - Installed but outdated mods should be visually distinct
  - Installed but unknown mods should be detected and made visually distinct
- Select mods to install
  - Ability to select old versions?
    - changelogs
- Mod details
  - Changelogs
  - Description
  - Links
  - Compatibility info
