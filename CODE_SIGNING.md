# Code signing policy

Free code signing provided by [SignPath.io](https://about.signpath.io/), certificate by [SignPath Foundation](https://signpath.org/).

The Windows compiler, `zam.exe`, is signed. Each release is built from this repository by the [release workflow](.github/workflows/release.yml) on GitHub Actions, and a maintainer approves every signing request by hand. The Linux and macOS builds aren't signed.

## Team roles

| Role | Members |
| --- | --- |
| Authors (can change the source code without another review) | [rayL-K](https://github.com/rayL-K) |
| Reviewers (review changes from other contributors) | [rayL-K](https://github.com/rayL-K) |
| Approvers (approve each signing request) | [rayL-K](https://github.com/rayL-K) |

Every member uses multi-factor authentication for GitHub and SignPath. Contributions from anyone else are merged only after a reviewer approves the pull request.

## Privacy

This program will not transfer any information to other networked systems unless specifically requested by the user. `zam` connects to the network only when you ask it to work with packages: `zam add`, `zam update`, `zam search`, and `zam publish` fetch from the Git repositories and the package index you name, and `zam build` fetches a project's Git dependencies. These hosts, such as GitHub, apply their own privacy policies.
