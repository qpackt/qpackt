# Qpackt :: Web Server with Analytics and A/B testing

## About

Qpackt is a web server that makes life a bit easier:

* Ability to serve multiple versions of your website (why you need this: https://qpackt.com/ab-testing.html)
* Basic, GDPR-compliant analytics
* Auto fetch and renew SSL certificate
* GUI configuration

Qpackt is not yet production ready but is able to serve itself at https://qpackt.com/

If you're interested about future development, follow me on X (https://twitter.com/QPackt)

## Installation

Currently, Qpackt is only available as source code. Installation isn't too difficult though and should take less than 10
minutes. Follow instructions in [Installation.md](./Installation.md)

## Usage

Use case scenarios are explained [here](https://qpackt.com/ab-testing.html).

### Serve multiple versions of your website

With Qpackt you can serve multiple versions of your website. This allows for:

- A/B testing. You can split traffic proportionally to arbitrary weight or url parameter.
- Gently rolling new version of your website to prevent broken link errors for existing visitors.

### Automatically fetch SSL certificate

Qpackt fetches SSL certificate when started for the first time. No more manually running certbot or anything.

### Basic analytics without tracking cookies

Qpackt tries to collect visitors' stats. This is done without tracking cookies so no consent popup is necessary.