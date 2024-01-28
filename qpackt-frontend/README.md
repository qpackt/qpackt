# Vue 3 + Vite

This template should help get you started developing with Vue 3 in Vite. The template uses Vue 3 `<script setup>` SFCs, check out the [script setup docs](https://v3.vuejs.org/api/sfc-script-setup.html#sfc-script-setup) to learn more.

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Volar](https://marketplace.visualstudio.com/items?itemName=Vue.volar) (and disable Vetur) + [TypeScript Vue Plugin (Volar)](https://marketplace.visualstudio.com/items?itemName=Vue.vscode-typescript-vue-plugin).

## Build for production

```bash
npm run build
```

## Serve

```bash
npm run dev
```

For debian install pkg-config
For ubuntu install libssl-dev
Forwarding:
sudo iptables -t nat -A PREROUTING -p tcp --dport 80 -j REDIRECT --to-port 8080
sudo iptables -t nat -A PREROUTING -p tcp --dport 443 -j REDIRECT --to-port 8443
sudo sysctl -w net.ipv4.ip_forward=1

Use whatever you're using to make these rules persistent after restart.

serving files via localhost
sudo iptables -A INPUT -i lo -j ACCEPT

panel:
sudo iptables -A INPUT -p tcp --dport 8444 -j ACCEPT
