[![Contributors](https://img.shields.io/opencollective/all/riffcc?style=flat-square)](https://opencollective.com/riffcc)
[![Orbiter installable app](https://github.com/riffcc/orbiter/actions/workflows/ci.yml/badge.svg)](https://github.com/riffcc/orbiter/actions/workflows/ci.yml)

# Riff.CC "Orbiter"
Orbiter is the proof-of-concept version of Riff.CC, implemented using OrbitDB, Constellation, IPFS and other tools. This repository contains instructions on how to set up and install a Riff.CC instance through Orbiter, as well as the necessary code to do so. For more information about Orbiter as well as the Riff.CC project itself, [see the project's main page](https://riff.cc/riff-docs/).

Funding provided via [OpenCollective](https://opencollective.com/riffcc).

## What's Riff.CC?
Riff.CC is an in-development platform for watching, sharing, distributing and curating legally free media.

It's like if sites like Netflix, Spotify and YouTube were rolled into one platform where you are not the product.

The software we're making is open source and you can not only make your own things with it but federate with other sites using the Defederation Model.

## Getting started
Orbiter will work on any standard Linux distribution where standard utilities like a webserver and Node.JS are available, but **our instructions specifically cover the latest LTS releases of Ubuntu and Debian only**. Ask us (or submit a PR) if you want any others to be added.

### Dependencies
Before installing Orbiter, you must install the following dependencies:
```
sudo apt install libnss3-dev libgtk-3-0 libgbm1 libasound2-dev libatk1.0-0 node-pre-gyp git curl
```

Additionally, Orbiter utilizes **Node.JS** and **pnpm** to set up and install the platform, and as such it's required to install them as well.

#### **Base: Node.JS**
Required whether you use pnpm or yarn. We suggest using the [NodeSource installation guide](https://github.com/nodesource/distributions#installation-instructions), with $NODE_MAJOR set to 22:

1. Download and import the Nodesource GPG key

```sh
sudo apt-get update
sudo apt-get install -y ca-certificates curl gnupg
sudo mkdir -p /etc/apt/keyrings
curl -fsSL https://deb.nodesource.com/gpgkey/nodesource-repo.gpg.key | sudo gpg --dearmor -o /etc/apt/keyrings/nodesource.gpg
```

2. Create deb repository

```sh
NODE_MAJOR=22
echo "deb [signed-by=/etc/apt/keyrings/nodesource.gpg] https://deb.nodesource.com/node_$NODE_MAJOR.x nodistro main" | sudo tee /etc/apt/sources.list.d/nodesource.list
```

3. Run apt update and install the Node.JS package

```sh
sudo apt-get update
sudo apt-get install nodejs -y
```

4. Download the official pnpm installation script:

```sh
curl -fsSL https://get.pnpm.io/install.sh | sh -
```

2. Source your .bashrc file so your system recognizes pnpm's path properly:

```sh
source ~/.bashrc
```

### Installation

1. Clone Orbiter's GitHub repository:
```sh
git clone https://github.com/riffcc/orbiter.git
```

2. Move into Orbiter's directory and set up its installation dependencies:
    ```sh
    cd orbiter
    pnpm install
    ```

3. Run the application in development mode (with hot reloading):
    ```sh
    pnpm watch:web
    ```

4. Visit the application in your browser (for productio, in Electron), generate a moderation database, copy the parameters the application gives you, and copy the line beginning with `VITE_SITE_ID=/orbitdb/...` to `./.env.local` (or the whole texto to `./.env`).

5. Stop and re-run the application to apply the new moderation database.

## Deploying to production

Compile and minify the application:

```
pnpm build
```

Then deploy the bundle of HTML/CSS/JS that is produced.

## Customize configuration

See [Configuration Reference](https://vitejs.dev/config/).

## Credits
Authored by [@julienmalard](https://github.com/julienmalard), [@Zorlin](https://github.com/Zorlin), and the Riff.CC Project.

## Sponsors
Thank you to our sponsors, who have generously provided funding for the development of the Riff.CC Project:

* [ARData Tech](https://ardata.tech)
* [Money Every 3 Days](https://moneyevery3days.com/)
