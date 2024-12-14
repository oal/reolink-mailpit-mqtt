# Reolink - Mailpit - Home Assistant bridge

Battery powered [Reolink](https://reolink.com/)
cameras [require a hub to be used with Home Assistant](https://www.home-assistant.io/integrations/reolink#tested-battery-powered-models).

This project relies on the camera's ability to send emails with image attachments when motion is detected.

By running a local SMTP server ([Mailpit](https://github.com/axllent/mailpit) in this case), and using
Mailpit's [webhook](https://mailpit.axllent.org/docs/integration/webhook/) feature, we can forward these images to Home
Assistant.

Images are sent to Home Assistant using the [MQTT image](https://www.home-assistant.io/integrations/image.mqtt/)
platform.

## Setup

### Download and set up `reolink-mailpit-mqtt`

Clone and build the project, and put the binary where you want the service to run.

You need the Rust compiler installed. Then it can be built with:

```sh
cargo build --release
```

Next, create a `.env` file in the same directory as the binary. See `.env.example` for available config variables.
Example:

```
MAILPIT_URL=http://192.168.1.123:8025
MQTT_HOST=192.168.1.123
MQTT_PORT=1883
```

Run the binary. It will listen on port 8026. It's recommended to run it as a service, for example with systemd. Here's an example service file:

```ini
[Unit]
Description=Reolink - Mailpit - Home Assistant bridge
After=network.target

[Service]
Type=simple
User=your-user
WorkingDirectory=/path/to/reolink-mailpit-mqtt
ExecStart=/path/to/reolink-mailpit-mqtt
Restart=always

[Install]
WantedBy=multi-user.target
```

Put this in `/etc/systemd/system/reolink-mailpit-mqtt.service`, and replace the paths and user with your own.

### Set up Mailpit

An easy way to do this, is with Docker. Here's an example `docker-compose.yml`:

```yaml
services:
  mailpit:
    image: axllent/mailpit
    container_name: mailpit
    restart: unless-stopped
    volumes:
      - ./data:/data
    ports:
      - 8025:8025
      - 1025:1025
    environment:
      MP_MAX_MESSAGES: 1000
      MP_DATABASE: /data/mailpit.db
      MP_SMTP_AUTH_ACCEPT_ANY: 1
      MP_SMTP_AUTH_ALLOW_INSECURE: 1
      MP_WEBHOOK_URL: "http://your-ip:8026/email-webhook"
```

Replace the webhook URL to match your setup. This should point to where `reolink-mailpit-mqtt` is running.

### Configure your Reolink camera

Using the Reolink app, enable SMTP, and set use the IP and SMTP port of your Mailpit server. Example:
`192.168.1.123:1025`.

### Launch or enable the service

Start the service, and you should now receive images in Home Assistant when motion is detected by your camera.

With `systemd`, you can enable and start the service with:

```sh
systemctl enable reolink-mailpit-mqtt
systemctl start reolink-mailpit-mqtt
```

Note that only the most recent image will be available in Home Assistant. However, you can use the Mailpit web interface
to view all images received (using the docker compose file above, the 1000 most recent messages will be retained).
