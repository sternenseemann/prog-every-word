# Program Every Word!

The paradigm shift has arrived. Word Oriented Programming now!

[Follow @progeveryword](https://twitter.com/progeveryword).

## Setup

In order to use the bot one needs to create a twitter account and a [twitter application](https://apps.twitter.com/).

Then it is necessary to setup `./state.json` (path relative to the binary) and enter the twitter API consumer key as well as secret for the application:

```json
{
  "consumer_token" : {
    "key" : "<consumer key>",
    "secret":"<consumer secret>"
  },
  "access_token" : null,
  "counter" : 0
}
```

Now the bot can be run for the first time! This run must be done manually since the twitter access token is retrieved interactively.

If the bot should tweet regularly the [supplied systemd service and timer](./system) can be used! These files might to be adjusted slightly though.

## Design

The beautiful color used in the [logo](./design) is `#fab81e`.
