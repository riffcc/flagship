### **Production: Deploying a Flagship Instance with a Lens Node**

This guide will walk you through deploying a production-ready Flagship instance, including setting up a server, configuring DNS, securing it with SSL, and running the Lens Node in a persistent terminal session.

### **Prerequisites**

*   A server instance (e.g., an AWS EC2 or DigitalOcean Droplet) running a modern Linux distribution like Ubuntu 22.04.
    *   **Recommended Specs:** 2+ CPU Cores, 4GB+ RAM.
*   A registered domain name (e.g., `your-domain.com`).
*   Access to your domain's DNS management panel.
*   The following software installed on your server:
    *   [Node.js](https://nodejs.org/)
    *   [pnpm](https://pnpm.io/installation)
    *   [Git](https://git-scm.com/downloads)
    *   [Nginx](https://www.nginx.com/)
    *   [Screen](https://www.gnu.org/software/screen/) (`sudo apt install screen`)

---

### **Part 1: Server and Domain Configuration**

This section covers the initial setup of your server, DNS, and SSL certificates.

#### **Step 1: Point Your Domain to the Server**

In your DNS provider's control panel, create two `A` records pointing to your server's public IP address:

1.  An `A` record for your main domain:
    *   **Type:** `A`
    *   **Name/Host:** `@` (or `your-domain.com`)
    *   **Value/Points to:** `your-server-ip`

2.  An `A` record for the Lens Node subdomain:
    *   **Type:** `A`
    *   **Name/Host:** `lens`
    *   **Value/Points to:** `your-server-ip`

> **Note:** DNS changes can take some time to propagate.

#### **Step 2: Install Nginx and Obtain SSL Certificates**

1.  **Install Nginx and Certbot:**
    ```bash
    sudo apt update
    sudo apt install nginx certbot python3-certbot-nginx
    ```

2.  **Obtain SSL Certificates.** We will use Certbot to get certificates for both your domains. This command will temporarily stop Nginx to free up port 80 for validation.
    ```bash
    sudo systemctl stop nginx
    sudo certbot certonly --standalone -d your-domain.com -d lens.your-domain.com
    ```
    Follow the prompts to complete the process. Once finished, Certbot will have created your certificate files.

#### **Step 3: Configure Nginx**

Now we will create the Nginx configuration files to serve your Flagship site and proxy requests to the Lens Node.

1.  **Create the Nginx config for your main Flagship site:**
    ```bash
    sudo nano /etc/nginx/sites-available/your-domain.com
    ```
    Paste in the following, replacing `your-domain.com` with your actual domain:
    ```nginx
    server {
        listen 80;
        server_name your-domain.com;
        # Redirect all HTTP requests to HTTPS
        return 301 https://$host$request_uri;
    }

    server {
        listen 443 ssl;
        server_name your-domain.com;

        root /var/www/your-domain.com/html;
        index index.html;

        # SSL configuration
        ssl_certificate /etc/letsencrypt/live/your-domain.com/fullchain.pem;
        ssl_certificate_key /etc/letsencrypt/live/your-domain.com/privkey.pem;
        include /etc/letsencrypt/options-ssl-nginx.conf;
        ssl_dhparam /etc/letsencrypt/ssl-dhparams.pem;

        location / {
            try_files $uri $uri/ /index.html;
        }
    }
    ```

2.  **Create the web root directory for the Flagship site:**
    ```bash
    sudo mkdir -p /var/www/your-domain.com/html
    ```

3.  **Create the Nginx config for your Lens Node subdomain:**
    ```bash
    sudo nano /etc/nginx/sites-available/lens.your-domain.com
    ```
    Paste the entire configuration block below into the file.

    > **IMPORTANT:** You must replace every instance of **`$YOUR_SUBDOMAIN`** with your actual subdomain (e.g., `lens.your-domain.com`).

    ```nginx
    server_names_hash_bucket_size 128;

    server {
        server_name $YOUR_SUBDOMAIN;
        listen [::]:4002 ssl ipv6only=on;
        listen 4002 ssl;

        # Load the certificate files.
        ssl_certificate         /etc/letsencrypt/live/$YOUR_SUBDOMAIN/fullchain.pem;
        ssl_certificate_key     /etc/letsencrypt/live/$YOUR_SUBDOMAIN/privkey.pem;
        ssl_trusted_certificate /etc/letsencrypt/live/$YOUR_SUBDOMAIN/chain.pem;

        # Load the Diffie-Hellman parameter.
        ssl_dhparam /etc/letsencrypt/ssl-dhparams.pem;

        location / {
            proxy_pass http://127.0.0.1:8001;
            proxy_http_version 1.1;
            proxy_set_header Upgrade $http_upgrade;
            proxy_set_header Connection "upgrade";
        }
    }

    server {
        server_name $YOUR_SUBDOMAIN;
        listen [::]:4003 ssl ipv6only=on;
        listen 4003 ssl;

        # Load the certificate files.
        ssl_certificate         /etc/letsencrypt/live/$YOUR_SUBDOMAIN/fullchain.pem;
        ssl_certificate_key     /etc/letsencrypt/live/$YOUR_SUBDOMAIN/privkey.pem;
        ssl_trusted_certificate /etc/letsencrypt/live/$YOUR_SUBDOMAIN/chain.pem;

        # Load the Diffie-Hellman parameter.
        ssl_dhparam /etc/letsencrypt/ssl-dhparams.pem;

        location / {
            proxy_pass http://127.0.0.1:8002;
            proxy_http_version 1.1;
            proxy_set_header Upgrade $http_upgrade;
            proxy_set_header Connection "upgrade";
        }
    }

    server {
        server_name $YOUR_SUBDOMAIN;
        listen [::]:4004 ssl ipv6only=on;
        listen 4004 ssl;

        # Load the certificate files.
        ssl_certificate         /etc/letsencrypt/live/$YOUR_SUBDOMAIN/fullchain.pem;
        ssl_certificate_key     /etc/letsencrypt/live/$YOUR_SUBDOMAIN/privkey.pem;
        ssl_trusted_certificate /etc/letsencrypt/live/$YOUR_SUBDOMAIN/chain.pem;

        # Load the Diffie-Hellman parameter.
        ssl_dhparam /etc/letsencrypt/ssl-dhparams.pem;

        location / {
            proxy_pass http://127.0.0.1:8083;
            proxy_http_version 1.1;
            proxy_set_header Upgrade $http_upgrade;
            proxy_set_header Connection "upgrade";
        }
    }


    server {
        server_name $YOUR_SUBDOMAIN;
        listen [::]:4005 ssl ipv6only=on;
        listen 4005 ssl;

        # Load the certificate files.
        ssl_certificate         /etc/letsencrypt/live/$YOUR_SUBDOMAIN/fullchain.pem;
        ssl_certificate_key     /etc/letsencrypt/live/$YOUR_SUBDOMAIN/privkey.pem;
        ssl_trusted_certificate /etc/letsencrypt/live/$YOUR_SUBDOMAIN/chain.pem;

        # Load the Diffie-Hellman parameter.
        ssl_dhparam /etc/letsencrypt/ssl-dhparams.pem;

        location / {
            proxy_pass http://127.0.0.1:8084;
            proxy_http_version 1.1;
            proxy_set_header Upgrade $http_upgrade;
            proxy_set_header Connection "upgrade";
        }
    }

    server {
        server_name $YOUR_SUBDOMAIN; # managed by Certbot

        root /usr/share/nginx/html;
        index index.html index.htm;

        location / {
            try_files $uri $uri/ =404;
        }

        listen [::]:443 ssl; # managed by Certbot
        listen 443 ssl; # managed by Certbot

        # Load the certificate files.
        ssl_certificate         /etc/letsencrypt/live/$YOUR_SUBDOMAIN/fullchain.pem;
        ssl_certificate_key     /etc/letsencrypt/live/$YOUR_SUBDOMAIN/privkey.pem;
        ssl_trusted_certificate /etc/letsencrypt/live/$YOUR_SUBDOMAIN/chain.pem;

        # Load the Diffie-Hellman parameter.
        ssl_dhparam /etc/letsencrypt/ssl-dhparams.pem;
    }
    server {
        server_name $YOUR_SUBDOMAIN;
        listen [::]:9002 ssl ipv6only=on;
        listen 9002 ssl;

        # Load the certificate files.
        ssl_certificate         /etc/letsencrypt/live/$YOUR_SUBDOMAIN/fullchain.pem;
        ssl_certificate_key     /etc/letsencrypt/live/$YOUR_SUBDOMAIN/privkey.pem;
        ssl_trusted_certificate /etc/letsencrypt/live/$YOUR_SUBDOMAIN/chain.pem;

        # Load the Diffie-Hellman parameter.
        ssl_dhparam /etc/letsencrypt/ssl-dhparams.pem;

        location / {
            proxy_pass http://127.0.0.1:8082;
            proxy_http_version 1.1;
            proxy_set_header Upgrade $http_upgrade;
            proxy_set_header Connection "upgrade";
        }
    }

    server {
        listen 80;
        listen [::]:80 ;
        server_name $YOUR_SUBDOMAIN;

        # Anything requesting this particular URL should be served content from
        # Certbot's folder so the HTTP-01 ACME challenges can be completed for the
        # HTTPS certificates.
        location '/.well-known/acme-challenge' {
            default_type "text/plain";
            root /var/www/letsencrypt;
        }

        if ($host = $YOUR_SUBDOMAIN) {
            return 301 https://$host$request_uri;
        } # managed by Certbot

        return 404; # managed by Certbot
    }

    ```

4.  **Enable both sites by creating symbolic links:**
    ```bash
    sudo ln -s /etc/nginx/sites-available/your-domain.com /etc/nginx/sites-enabled/
    sudo ln -s /etc/nginx/sites-available/lens.your-domain.com /etc/nginx/sites-enabled/
    ```

5.  **Test your Nginx configuration and restart the service:**
    ```bash
    sudo nginx -t
    sudo systemctl restart nginx
    ```

---

### **Part 2: Set Up and Run the Lens Node**

We will use `screen` to run the Lens Node process in the background, allowing it to persist even after you disconnect from your SSH session.

1.  **Install the Lens Node globally:**
    ```bash
    pnpm install -g @riffcc/lens-node
    ```

2.  **Run the initial setup process:**
    ```bash
    lens-node setup
    ```

3.  **Start a new `screen` session:**
    ```bash
    screen -S lens-node
    ```
    Your terminal will clear, and you are now inside a new virtual session named `lens-node`.

4.  **Start the node inside the screen session.** Replace `lens.your-domain.com` with your actual subdomain.
    ```bash
    lens-node run --domain lens.your-domain.com --relay
    ```

5.  Once the node is running, you will see output similar to this. **Take note of your `Site Address` and the `Listening on` address that contains `/wss`**.

    ```bash
    Node Directory: /root/.lens-node
    Peer ID: 12D3KooWFzrhasG7pouxZSsSP3bHP45C3TCq2MYirTPC5oA9Lj2v
    Site Address: zb2rhoGguxQaNZy47DvRPLRmpT84mwvwuvekUhmCgF7nkhTCn
    Listening on: [
      "/dns4/lens.your-domain.com/tcp/4002/p2p/PEER_ID",
      "/dns4/lens.your-domain.com/tcp/4003/wss/p2p/PEER_ID"
    ]
    ```

6.  **Detach from the screen session**, leaving the node running. Press **`Ctrl+A`**, then press **`D`**.

You now have a Lens Node running successfully in the background.

> **To manage the node:**
> *   **Re-attach to the session** to view logs or stop the process: `screen -r lens-node`
> *   **To stop the node**, re-attach and press `Ctrl+C`.
> *   **Warning:** This method does not survive a server reboot. You will need to manually restart the node in a new screen session if the server restarts.

---

### **Part 3: Deploy the Flagship Instance**

Finally, configure, build, and deploy the Flagship frontend application.

1.  **Clone the repository and navigate into the directory:**
    ```bash
    git clone https://github.com/riffcc/flagship
    cd flagship
    ```

2.  **Create the environment file:**
    ```bash
    cp .env.example .env
    ```

3.  **Configure the environment variables.** Open the `.env` file and set the following variables using the values from your running Lens Node:

    ```dotenv
    # Paste the "Site Address" from your node's output here
    VITE_SITE_ADDRESS=zb2rhoGguxQaNZy47DvRPLRmpT84mwvwuvekUhmCgF7nkhTCn

    # Paste the secure "/wss" listening address here
    VITE_BOOTSTRAPPERS=/dns4/lens.your-domain.com/tcp/4003/wss/p2p/12D3KooWFzrhasG7pouxZSsSP3bHP45C3TCq2MYirTPC5oA9Lj2v
    ```

4.  **Install dependencies and build the application:**
    ```bash
    pnpm install
    pnpm compile:web
    ```

5.  **Deploy the built files to your Nginx web root.**
    ```bash
    sudo cp -r packages/renderer/dist/web/* /var/www/your-domain.com/html/
    ```

6.  **Set the correct ownership for the web files:**
    ```bash
    sudo chown -R www-data:www-data /var/www/your-domain.com/html
    ```

Your Flagship instance is now live! You can access it by navigating to `https://your-domain.com` in your browser.
