# Qpackt installation process

At the moment Qpackt is distributed only from source. You need Npm (https://nodejs.org/en/download) and Rust (https://www.rust-lang.org/tools/install)
on your system.

Steps:
1. Create qpackt user
   ```bash
     sudo adduser --system qpackt
   ```
2. Clone this repository
   ```bash
    git clone https://github.com/qpackt/qpackt.git
    cd qpackt
    ```
3. Build frontend and move files to be served as admin panel
    ```bash
        sudo mkdir -p /usr/share/qpackt/html
        sudo chown qpackt /usr/share/qpackt/html
        sudo chmod 700 /usr/share/qpackt/html
        cd qpackt-frontend
        npm install
        npm run build
        sudo cp -a dist/* /usr/share/qpackt/html
        cd ..
    ```
4. Build backend. On Ubuntu you need pkg-config and libssl-dev installed.
    ```bash
        cd qpackt-backend
        cargo build --release
        cd ..
        sudo cp ./target/release/qpackt-backend /usr/bin/qpackt
    ```

5. Run for the first time to build configuration:
    ```bash
      /usr/bin/qpackt
   ```
   Qpackt needs to ask for a few things:
    * Domain: your site domain. Needed mainly for fetching SSL certificate.
    * Ip/port for HTTP traffic (default 0.0.0.0:8080): port for http (not secure) traffic. You don't want port below 1024 as binding there requires root's privileges.
    * Ip/port for HTTPS traffic (leave empty for no HTTPS): leave empty if you don't want https (secure) server. Otherwise, put something like 0.0.0.0:8443.
    * Administrator's password: choose some safe password for administrator's panel.
    * Run directory (default /usr/share/qpackt/run): Qpackt's storage directory (database, sites, certificate). Must be writeable for qpackt's user.
   Config file qpackt.yaml will be created in current directory. Change file's permission and owner:
   ```bash
    sudo mkdir -p /usr/share/qpackt/run
    sudo chown qpackt /usr/share/qpackt/run
    sudo chmod 700 /usr/share/qpackt/run
    sudo chmod 600 qpackt.yaml
    sudo chown qpackt qpackt.yaml
   ```
   Copy the file to some general config directory, like '/etc/'
   ```bash
    sudo mv qpackt.yaml /etc
    ```
6. Setup firewall with redirects
   To run Qpackt as non-root you need to bind it to ports higher than 1023. Browsers however, try ports 80/443.
   To make sure all packets get to the right service, you need to redirect them (use the same ports as in configuration above):
   ```bash
     sudo iptables -t nat -A PREROUTING -p tcp --dport 80 -j REDIRECT --to-port 8080
     sudo iptables -t nat -A PREROUTING -p tcp --dport 443 -j REDIRECT --to-port 8443
     sudo sysctl -w net.ipv4.ip_forward=1
   ```
   You also need access to the panel and files served via localhost:
    ```bash
      sudo iptables -A INPUT -p tcp --dport 9443 -j ACCEPT # for https when configured
      sudo iptables -A INPUT -p tcp --dport 9080 -j ACCEPT # for http
      sudo iptables -A INPUT -i lo -j ACCEPT
   ```
   Use whatever mechanism is available on your system to make those redirects survive reboots. On Debian based systems:
    ```bash
      sudo apt install iptables-persistent
      sudo iptables-save | sudo tee /etc/iptables/rules.v4
   ```

7. Setup systemd files (with `sudo vi /etc/systemd/system/qpackt.service`):
    ```text
   [Unit]
   Description=Qpackt Web Server
   After=network.target
   StartLimitIntervalSec=0
   [Service]
   Type=simple
   Restart=always
   RestartSec=1
   User=qpackt
   ExecStart=/usr/bin/qpackt /etc/qpackt.yaml

   [Install]
   WantedBy=multi-user.target
   ```

   ```bash
    sudo systemctl enable qpackt
    sudo systemctl start qpackt
   ```
   If you configured HTTPS then Qpackt will get an SSL certificate at the startup.
8. Connect to admin panel (http://domain:9080 or https://domain:9443) and upload your site.