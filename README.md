# <p align="center">Juncto</p>

Juncto is a set of Open Source projects which empower users to use and deploy
video conferencing platforms with state-of-the-art video quality and features.

<hr />

<p align="center">
<img src="https://raw.githubusercontent.com/juncto/juncto/master/readme-img1.png" width="900" />
</p>

<hr />

Amongst others here are the main features Juncto offers:

* Support for all current browsers
* Mobile applications
* Web and native SDKs for integration
* HD audio and video
* Content sharing
* Raise hand and reactions
* Chat with private conversations
* Polls
* Virtual backgrounds

And many more!

## Using Juncto

Using Juncto is straightforward, as it's browser based. Head over to [meet.juncto.net](https://meet.juncto.net) and give it a try. It's scalable and free to use. All you need is a Google, Facebook or GitHub account in order to start a meeting. All browsers are supported!

Using mobile? No problem, you can either use your mobile web browser or our fully-featured
mobile apps:

| Android | Android (F-Droid) | iOS |
|:-:|:-:|:-:|
| [<img src="resources/img/google-play-badge.png" height="50">](https://play.google.com/store/apps/details?id=org.juncto.meet) | [<img src="resources/img/f-droid-badge.png" height="50">](https://f-droid.org/packages/org.juncto.meet/) | [<img src="resources/img/appstore-badge.png" height="50">](https://itunes.apple.com/us/app/juncto/id1165103905) |

If you are feeling adventurous and want to get an early scoop of the features as they are being
developed you can also sign up for our open beta testing here:

* [Android](https://play.google.com/apps/testing/org.juncto.meet)
* [iOS](https://testflight.apple.com/join/isy6ja7S)

## Running your own instance

The web client in this repository now lives in [`rust-app/`](rust-app/), a Leptos (WASM) frontend
backed by an Axum server. The previous React/Webpack implementation has been removed.

```sh
cd rust-app
bash build.sh        # builds the WASM frontend and copies static assets
cd backend && cargo run --release   # serves the app on :3000
```

Rust unit tests and the single consolidated Playwright suite:

```sh
cd rust-app && cargo test --workspace             # unit tests
cd rust-app/tests/e2e && npx playwright test      # end-to-end parity suite
```

Legacy guidance below applies to the removed legacy React implementation and is
kept for historical reference.

## Juncto as a Service

If you like the branding capabilities of running your own instance but you'd like
to avoid dealing with the complexity of monitoring, scaling and updates, JunctoService might be
for you.

[Juncto Juncto as a Service (JunctoService)](https://jaas.Juncto.vc) is an enterprise-ready video meeting platform that allows developers, organizations and businesses to easily build and deploy video solutions. With Juncto as a Service we now give you all the power of Juncto running on our global platform so you can focus on building secure and branded video experiences.

## Documentation

All the Juncto documentation is available in [the handbook](https://juncto.github.io/handbook/).

## Security

For a comprehensive description of all Juncto's security aspects, please check [this link](https://juncto.org/security).

For a detailed description of Juncto's End-to-End Encryption (E2EE) implementation,
please check [this link](https://juncto.org/e2ee-whitepaper/).

For information on reporting security vulnerabilities in Juncto, see [SECURITY.md](./SECURITY.md).

## Contributing

If you are looking to contribute to Juncto, first of all, thank you! Please
see our [guidelines for contributing](CONTRIBUTING.md).

<br />
<br />

<footer>
<p align="center" style="font-size: smaller;">
Built with ❤️ by the Juncto team at <a href="https://Juncto.com" target="_blank">Juncto</a> and our community.
</p>
</footer>
